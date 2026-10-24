//! `copilot` subprocess backend: the user's GitHub Copilot plan through the
//! official Copilot CLI. UpNext never reads Copilot's credentials; it runs
//! the binary the user signed in with (`copilot login`), the way
//! `claude_cli` runs `claude` and `codex_cli` runs `codex`.
//!
//! The CLI has no system prompt option and no output schema, so UpNext's
//! instructions and the JSON Schema go at the top of the prompt, which comes
//! on stdin (a candidate list is longer than a Windows command line takes).
//! A reply that isn't JSON is asked for once more. Copilot is an agent, so
//! each call turns off what UpNext doesn't need: every tool (with them gone
//! its own prompt drops from about 22k characters to 8k on Copilot CLI
//! 1.0.103), the built-in MCP servers, custom instructions, questions to
//! the user, updates and the CLI's own telemetry.
//!
//! Copilot keeps every session in its home folder. Each call is given a
//! session id of its own, and that session's folder is removed after the
//! call, so the prompts (player reviews among them) don't pile up there.
//! The CLI's search index of past sessions (`session-store.db`) still lists
//! them; UpNext doesn't write to Copilot's database.

use super::{LlmBackend, LlmError, LlmRequest, LlmResponse, Usage, truncate};
use crate::process::{FeedError, Launch, RunError};
use async_trait::async_trait;
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::{Duration, Instant};
use tokio::process::Command;

/// The model when none is set: Auto, which every Copilot plan can use,
/// Copilot Free among them. Settings always sets one (`copilot_model`).
const DEFAULT_MODEL: &str = "auto";

pub struct CopilotCliBackend {
    bin: String,
    timeout: Duration,
    /// An AI account's own `COPILOT_HOME`, where its own `copilot login` is
    /// kept; None for this computer's.
    home: Option<PathBuf>,
}

impl CopilotCliBackend {
    pub fn new(bin: String) -> Self {
        Self { bin, timeout: Duration::from_secs(300), home: None }
    }

    /// Run Copilot with its own home, an AI account's.
    pub fn with_home(mut self, home: Option<PathBuf>) -> Self {
        self.home = home;
        self
    }
}

/// The environment variables Copilot takes a sign-in from, ahead of its own.
const TOKEN_VARS: [&str; 3] = ["COPILOT_GITHUB_TOKEN", "GH_TOKEN", "GITHUB_TOKEN"];

/// `copilot`, run as `process::Launch` says: on Windows an npm install's
/// `copilot.cmd` is replaced by the node script it starts. An AI account's
/// own `home` keeps it to that account's sign-in: this computer's tokens in
/// the environment are taken away, and the GitHub CLI's sign-in, which
/// Copilot falls back to, is looked for in the account's folder, where
/// there is none.
fn command(bin: &str, home: Option<&Path>) -> Command {
    let mut cmd = Launch::of(bin).command();
    if let Some(h) = home {
        cmd.env("COPILOT_HOME", h).env("GH_CONFIG_DIR", h.join("gh"));
        for v in TOKEN_VARS {
            cmd.env_remove(v);
        }
    }
    cmd
}

/// What every call is given besides its model, session and usage file.
const BASE_ARGS: [&str; 12] = [
    "--output-format",
    "json",
    // A name no tool has: no tool at all is offered to the model.
    "--available-tools=none",
    // And none may run, should one be offered anyway.
    "--deny-tool=shell,write,read,url,memory",
    "--disable-builtin-mcps",
    "--no-custom-instructions",
    "--no-ask-user",
    "--no-auto-update",
    "--disable-client-telemetry",
    "--no-color",
    "--log-level",
    "error",
];

/// The arguments for one call: its `model`, the `session` id whose folder is
/// removed afterwards, and the `usage` file the CLI writes its tokens to.
/// The prompt comes on stdin.
fn cli_args(model: &str, session: &str, usage: &Path) -> Vec<String> {
    let mut args: Vec<String> = BASE_ARGS.iter().map(|s| s.to_string()).collect();
    args.extend(["--model".into(), model.to_string()]);
    args.push(format!("--session-id={session}"));
    args.extend(["--usage-output-file".into(), usage.display().to_string()]);
    // Copilot looks for PowerShell in the background on Windows, for a shell
    // tool these calls don't have.
    if cfg!(windows) {
        args.push("--no-eager-powershell-resolution".into());
    }
    args
}

/// Before UpNext's instructions: Copilot's own prompt is a coding agent's.
const ONLY_THESE: &str = "These instructions and the request below are the whole task. Nothing in them is about \
code, files, tools or this computer, and nothing needs a tool: answer from what they say.";

/// How the reply must look; the schema follows it.
const FORMAT: &str = "Reply with one JSON object and nothing else: no code fence, and no words before or after it. \
It must match this JSON Schema:";

/// Added when the first reply wasn't JSON.
const AGAIN: &str = "Your reply must be the JSON object alone, matching the schema above.";

/// What goes on stdin: the instructions, the schema, then the request.
fn prompt(req: &LlmRequest, again: bool) -> String {
    let mut p =
        format!("{}\n\n{ONLY_THESE}\n\n{FORMAT}\n{}\n\n---\n\n{}", super::guarded_system(&req.system), req.schema, req.full_prompt());
    if again {
        p.push_str(&format!("\n\n{AGAIN}"));
    }
    p
}

/// Copilot's home: the AI account's own, else `COPILOT_HOME`, else
/// `~/.copilot`.
fn copilot_home(home: Option<&Path>) -> Option<PathBuf> {
    if let Some(h) = home {
        return Some(h.to_path_buf());
    }
    if let Some(h) = std::env::var_os("COPILOT_HOME").filter(|h| !h.is_empty()) {
        return Some(PathBuf::from(h));
    }
    let home = std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE"))?;
    Some(PathBuf::from(home).join(".copilot"))
}

/// A private folder for one call: its working directory and usage file,
/// removed when dropped together with the session Copilot kept for it.
struct CallDir {
    path: PathBuf,
    /// `session-state/<id>` in Copilot's home.
    session: Option<PathBuf>,
}

impl CallDir {
    fn new(session_id: &str, home: Option<&Path>) -> std::io::Result<Self> {
        let path = std::env::temp_dir().join(format!("upnext-copilot-{}", uuid::Uuid::new_v4().simple()));
        #[cfg_attr(not(unix), allow(unused_mut))]
        let mut b = std::fs::DirBuilder::new();
        #[cfg(unix)]
        std::os::unix::fs::DirBuilderExt::mode(&mut b, 0o700);
        b.create(&path)?;
        let session = copilot_home(home).map(|h| h.join("session-state").join(session_id));
        Ok(Self { path, session })
    }
}

impl Drop for CallDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
        if let Some(s) = &self.session {
            let _ = std::fs::remove_dir_all(s);
        }
    }
}

/// What a call printed: the JSON Lines of `--output-format json`, one
/// session event a line, ending with a `result`.
#[derive(Debug, Default)]
struct Events {
    /// The session started (the prompt was taken). Without it, Copilot
    /// failed before it got that far: signed out, an option it doesn't
    /// know, a model it refused.
    started: bool,
    /// The last message of the main agent (not a sub-agent's).
    answer: Option<String>,
    /// The model that wrote it.
    model: Option<String>,
    /// Why the session failed: its `errorType`, message and HTTP status.
    error: Option<(String, String, Option<u64>)>,
}

fn read_events(stdout: &str) -> Events {
    let mut ev = Events::default();
    for line in stdout.lines() {
        let Ok(v) = serde_json::from_str::<Value>(line) else { continue };
        let text = |p: &str| v.pointer(p).and_then(Value::as_str).map(str::to_string);
        match v.get("type").and_then(Value::as_str) {
            Some("user.message") => ev.started = true,
            Some("assistant.message") if v.get("agentId").is_none() => {
                ev.answer = text("/data/content");
                ev.model = text("/data/model").or(ev.model.take());
            }
            Some("session.error") => {
                ev.error = Some((
                    text("/data/errorType").unwrap_or_default(),
                    text("/data/message").unwrap_or_default(),
                    v.pointer("/data/statusCode").and_then(Value::as_u64),
                ));
            }
            _ => {}
        }
    }
    ev
}

/// The tokens a call used, from the file `--usage-output-file` wrote: every
/// model's, added up. Like the OpenAI API's, `inputTokens` counts the cached
/// ones too.
fn read_usage(file: &Value) -> Option<Usage> {
    let models = file.get("modelMetrics")?.as_object()?;
    let mut total = Usage::default();
    for m in models.values() {
        let n = |k: &str| m.pointer(&format!("/usage/{k}")).and_then(Value::as_u64).unwrap_or(0);
        total += Usage {
            input_tokens: n("inputTokens"),
            output_tokens: n("outputTokens"),
            cache_read_tokens: n("cacheReadTokens"),
            cache_write_tokens: n("cacheWriteTokens"),
            cache_write_1h_tokens: 0,
        };
    }
    (!models.is_empty()).then_some(total)
}

/// Why Copilot can't run until it is signed in.
const SIGN_IN: &str = "Copilot isn't signed in, or its sign-in has expired: run `copilot login` in a terminal, then try again";

/// What a call came to, from its events, exit status and stderr; `usage`
/// counts for an answer that wasn't JSON.
fn outcome(ev: Events, exit_ok: bool, stderr: &str, usage: Option<Usage>) -> Result<Value, LlmError> {
    if let Some((kind, msg, status)) = &ev.error {
        return Err(failure(kind, msg, *status));
    }
    if !exit_ok {
        let msg = truncate(stderr.trim(), 600);
        if !ev.started {
            // Before the session starts, every call fails the same way.
            if stderr.contains("No authentication information found") {
                return Err(LlmError::NotAvailable(SIGN_IN.into()));
            }
            return Err(LlmError::NotAvailable(format!("copilot couldn't start: {msg}")));
        }
        return Err(LlmError::Failed(format!("copilot exited with an error: {msg}")));
    }
    let Some(text) = ev.answer.filter(|t| !t.trim().is_empty()) else {
        return Err(LlmError::BadOutput("Copilot finished without an answer".into(), usage));
    };
    super::openai_compat::extract_json(&text)
        .filter(Value::is_object)
        .ok_or_else(|| LlmError::BadOutput(format!("Copilot's answer is not JSON: {}", truncate(&text, 400)), usage))
}

/// Why the session failed, as an `LlmError`, from Copilot's `errorType`
/// ("authentication", "authorization", "policy", "quota", "rate_limit",
/// "query" …) and the HTTP status.
fn failure(kind: &str, msg: &str, status: Option<u64>) -> LlmError {
    let said = format!("Copilot said: {}", truncate(msg.trim(), 400));
    match (kind, status) {
        ("authentication", _) | (_, Some(401)) => LlmError::NotAvailable(format!("{SIGN_IN}. ({said})")),
        ("quota" | "rate_limit", _) | (_, Some(429)) => LlmError::LimitReached(said),
        // The plan, or an organization's policy, doesn't allow it: every call
        // would fail the same way.
        ("authorization" | "policy", _) | (_, Some(403 | 404)) => LlmError::NotAvailable(said),
        _ => LlmError::Failed(said),
    }
}

/// A model Settings offers for Copilot.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct CopilotModel {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
}

/// The models Settings offers, by the names Copilot CLI 1.0.103 knows them
/// by. Copilot lists a plan's models only inside a session, so this list is
/// UpNext's; a model a plan doesn't include fails on Test with Copilot's
/// reason, and the environment (`UPNEXT_COPILOT_MODEL`) can name any other.
const MODELS: [CopilotModel; 6] = [
    CopilotModel { id: "auto", name: "Auto", description: "Copilot chooses the model for each call. The only choice on Copilot Free." },
    CopilotModel { id: "claude-sonnet-5.5", name: "Claude Sonnet 5.5", description: "UpNext's default for picks." },
    CopilotModel {
        id: "claude-haiku-5.5",
        name: "Claude Haiku 5.5",
        description: "UpNext's default for summaries: fast, and uses less of the plan.",
    },
    CopilotModel { id: "claude-opus-5.5", name: "Claude Opus 5.5", description: "The largest Claude model; uses more of the plan." },
    CopilotModel { id: "gpt-6.1-sol", name: "GPT-6.1-Sol", description: "OpenAI's workhorse model." },
    CopilotModel { id: "gpt-6-luna", name: "GPT-6-Luna", description: "OpenAI's fast, small model." },
];

pub fn models() -> &'static [CopilotModel] {
    &MODELS
}

/// How long `--version` may take. It answers locally in well under a second.
const READ_TIMEOUT: Duration = Duration::from_secs(30);

/// Why `copilot` didn't start (`e`): Node.js missing for an npm install, or
/// Copilot itself.
fn cannot_run(bin: &str, e: &std::io::Error) -> String {
    crate::process::node_missing(bin).unwrap_or_else(|| {
        format!(
            "cannot run `{bin}` ({e}): install the GitHub Copilot CLI (`winget install GitHub.Copilot` on Windows, \
             `npm install -g @github/copilot` anywhere) and sign in with `copilot login`"
        )
    })
}

/// `copilot --version` ("GitHub Copilot CLI 1.0.103."), its first line
/// without the full stop; or why it couldn't run, within `limit`.
async fn version_in(bin: &str, home: Option<&Path>, limit: Duration) -> Result<String, String> {
    let mut cmd = command(bin, home);
    cmd.arg("--version");
    match crate::process::output(cmd, limit).await {
        Ok(o) if o.status.success() => {
            let out = String::from_utf8_lossy(&o.stdout);
            Ok(out.lines().next().unwrap_or("").trim().trim_end_matches('.').to_string())
        }
        Ok(o) => Err(format!("`{bin} --version` failed: {}", truncate(String::from_utf8_lossy(&o.stderr).trim(), 300))),
        Err(RunError::Start(e)) => Err(cannot_run(bin, &e)),
        Err(RunError::TimedOut) => Err(format!("`{bin} --version` didn't answer within {} s", limit.as_secs())),
    }
}

/// The Copilot CLI's version, or why it can't be run: for Settings. Whether
/// it's signed in shows only on a call (Settings' Test): the CLI has no
/// command that says, and UpNext doesn't read its credentials.
pub async fn version(bin: &str, home: Option<&Path>) -> Result<String, String> {
    version_in(bin, home, READ_TIMEOUT).await
}

#[async_trait]
impl LlmBackend for CopilotCliBackend {
    fn name(&self) -> &str {
        "copilot-cli"
    }

    // No `pricing`: a Copilot plan isn't charged per call, and one plan runs
    // models from several providers, whose list prices UpNext can't tell apart
    // per call here.

    async fn available(&self) -> Result<(), LlmError> {
        // Every pick asks this first, so a CLI that hangs must not hold the
        // pick: it gets no longer than a status read, nor than the call itself.
        version_in(&self.bin, self.home.as_deref(), READ_TIMEOUT.min(self.timeout)).await.map(|_| ()).map_err(LlmError::NotAvailable)
    }

    async fn complete_json(&self, req: LlmRequest) -> Result<LlmResponse, LlmError> {
        self.available().await?;
        let model = req.model.clone().filter(|m| !m.is_empty()).unwrap_or_else(|| DEFAULT_MODEL.to_string());
        let started = Instant::now();
        let first = self.attempt(&req, &model, false).await;
        let (json, answered_by, usage) = match first {
            Err(LlmError::BadOutput(why, used)) => {
                tracing::info!("Copilot's answer wasn't the JSON asked for ({why}); asking once more");
                match self.attempt(&req, &model, true).await {
                    Ok((json, by, more)) => (json, by, add(used, more)),
                    Err(LlmError::BadOutput(why, more)) => return Err(LlmError::BadOutput(why, add(used, more))),
                    Err(e) => return Err(e),
                }
            }
            other => other?,
        };
        Ok(LlmResponse {
            json,
            model: answered_by.or(Some(model)),
            usage,
            // A Copilot plan isn't charged per call.
            cost_usd: None,
            api_usd: None,
            would_cost_usd: None,
            duration_ms: started.elapsed().as_millis() as u64,
            flex: false,
        })
    }
}

/// Two calls' tokens together, when either says.
fn add(a: Option<Usage>, b: Option<Usage>) -> Option<Usage> {
    match (a, b) {
        (Some(mut a), Some(b)) => {
            a += b;
            Some(a)
        }
        (a, b) => a.or(b),
    }
}

impl CopilotCliBackend {
    /// One run of the CLI: the JSON, the model that wrote it, and the tokens.
    async fn attempt(&self, req: &LlmRequest, model: &str, again: bool) -> Result<(Value, Option<String>, Option<Usage>), LlmError> {
        let session = uuid::Uuid::new_v4().to_string();
        let dir =
            CallDir::new(&session, self.home.as_deref()).map_err(|e| LlmError::Failed(format!("making a folder for the call: {e}")))?;
        let usage_file = dir.path.join("usage.json");
        let mut cmd = command(&self.bin, self.home.as_deref());
        cmd.current_dir(&dir.path).args(cli_args(model, &session, &usage_file));
        cmd.stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped());
        let child = cmd.spawn().map_err(|e| LlmError::NotAvailable(cannot_run(&self.bin, &e)))?;
        let output = match crate::process::feed(child, prompt(req, again).as_bytes(), self.timeout).await {
            Ok(o) => o,
            Err(FeedError::Write(e)) => return Err(LlmError::Failed(format!("writing prompt: {e}"))),
            Err(FeedError::Wait(e)) => return Err(LlmError::Failed(format!("waiting for copilot: {e}"))),
            Err(FeedError::TimedOut) => return Err(LlmError::Failed(format!("copilot timed out after {}s", self.timeout.as_secs()))),
        };
        let usage = std::fs::read_to_string(&usage_file).ok().and_then(|t| serde_json::from_str(&t).ok()).as_ref().and_then(read_usage);
        let ev = read_events(&String::from_utf8_lossy(&output.stdout));
        let answered_by = ev.model.clone();
        let json = outcome(ev, output.status.success(), &String::from_utf8_lossy(&output.stderr), usage)?;
        Ok((json, answered_by, usage))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn events(fixture: &str) -> Events {
        read_events(&crate::testutil::fixture(fixture))
    }

    fn request(prompt: &str) -> LlmRequest {
        LlmRequest {
            system: "Write JSON.".into(),
            cached_prefix: None,
            prompt: prompt.into(),
            schema: json!({"type": "object", "properties": {"answer": {"type": "string"}}, "required": ["answer"]}),
            model: Some("claude-haiku-5.5".into()),
            background: true,
            effort: None,
        }
    }

    /// Real output (Copilot CLI 1.0.103, the flags `cli_args` gives, a
    /// stand-in model): the answer is the main agent's message, and the
    /// tokens come from the usage file.
    #[test]
    fn a_success_reports_its_answer_model_and_tokens() {
        let ev = events("llm/copilot_cli_success.jsonl");
        assert!(ev.started);
        assert_eq!(ev.model.as_deref(), Some("fake-model"));
        let usage = read_usage(&crate::testutil::fixture_json("llm/copilot_cli_usage.json"));
        assert_eq!(
            usage,
            Some(Usage { input_tokens: 1234, output_tokens: 56, cache_read_tokens: 1000, cache_write_tokens: 0, cache_write_1h_tokens: 0 })
        );
        assert_eq!(outcome(ev, true, "", usage).unwrap(), json!({"answer": "ready"}));

        let mut ev = events("llm/copilot_cli_success.jsonl");
        ev.answer = Some("Sure! Here is the answer you asked for.".into());
        let err = outcome(ev, true, "", usage).unwrap_err();
        assert!(matches!(err, LlmError::BadOutput(..)), "{err:?}");
        assert_eq!(err.usage(), usage, "an unusable answer's tokens still count");

        // A fence or a line before the object is forgiven.
        let mut ev = events("llm/copilot_cli_success.jsonl");
        ev.answer = Some("Here it is:\n```json\n{\"answer\": \"ready\"}\n```".into());
        assert_eq!(outcome(ev, true, "", None).unwrap(), json!({"answer": "ready"}));

        // A sub-agent's message isn't the answer.
        let ev = read_events(
            r#"{"type":"user.message","data":{"content":"x"}}
{"type":"assistant.message","data":{"content":"{\"answer\":\"main\"}","model":"m"}}
{"type":"assistant.message","agentId":"a1","data":{"content":"{\"answer\":\"sub\"}","model":"s"}}"#,
        );
        assert_eq!((ev.answer.as_deref(), ev.model.as_deref()), (Some(r#"{"answer":"main"}"#), Some("m")));
        assert_eq!(read_usage(&json!({"modelMetrics": {}})), None);
    }

    /// Real failures, from a stand-in model answering 429 (after the CLI's
    /// own retries) and 400, and from a Copilot that isn't signed in.
    #[test]
    fn a_failed_call_says_why() {
        let err = outcome(events("llm/copilot_cli_rate_limit.jsonl"), false, "", None).unwrap_err();
        let LlmError::LimitReached(msg) = &err else { panic!("{err:?}") };
        assert!(msg.starts_with("Copilot said: Failed to get response from the AI model; retried 5 times"), "{msg}");
        assert!(crate::llm::gave_up(&anyhow::Error::new(err).context("digest generation")).is_some(), "a batch stops at the limit");

        let err = outcome(events("llm/copilot_cli_bad_request.jsonl"), false, "", None).unwrap_err();
        assert!(matches!(&err, LlmError::Failed(m) if m == "Copilot said: 400 fake failure"), "{err:?}");

        let signed_out = crate::testutil::fixture("llm/copilot_cli_signed_out.txt");
        let err = outcome(Events::default(), false, &signed_out, None).unwrap_err();
        assert!(matches!(&err, LlmError::NotAvailable(m) if m.contains("run `copilot login`")), "{err:?}");

        // An option this CLI doesn't know: nothing else will work either.
        let err = outcome(Events::default(), false, "error: unexpected argument '--no-color' found\n", None).unwrap_err();
        assert!(matches!(&err, LlmError::NotAvailable(m) if m.ends_with("unexpected argument '--no-color' found")), "{err:?}");
        let started = Events { started: true, ..Events::default() };
        assert!(matches!(outcome(started, false, "boom", None).unwrap_err(), LlmError::Failed(_)));

        assert!(matches!(failure("authentication", "Bad credentials", None), LlmError::NotAvailable(m) if m.contains("copilot login")));
        assert!(matches!(failure("quota", "You have no AI credits left", Some(402)), LlmError::LimitReached(_)));
        assert!(matches!(failure("policy", "Model not enabled", Some(403)), LlmError::NotAvailable(_)));
    }

    #[test]
    fn calls_turn_off_what_upnext_doesnt_need() {
        let usage = Path::new("/tmp/upnext-copilot-x/usage.json");
        let args = cli_args("claude-sonnet-5.5", "11111111-2222-4333-8444-555555555555", usage);
        assert_eq!(&args[..BASE_ARGS.len()], BASE_ARGS);
        assert!(args.windows(2).any(|w| w == ["--output-format", "json"]));
        assert!(args.windows(2).any(|w| w == ["--model", "claude-sonnet-5.5"]));
        assert!(args.contains(&"--session-id=11111111-2222-4333-8444-555555555555".to_string()));
        assert!(args.windows(2).any(|w| w[0] == "--usage-output-file" && w[1] == usage.display().to_string()));
        assert!(!args.iter().any(|a| a == "-p" || a == "--prompt"), "the prompt comes on stdin");
        assert!(!args.iter().any(|a| a.starts_with("--allow")), "nothing is allowed to run");
        assert_eq!(args.iter().any(|a| a == "--no-eager-powershell-resolution"), cfg!(windows));
    }

    /// UpNext's instructions and the schema come before the request, which
    /// holds the untrusted text.
    #[test]
    fn the_prompt_carries_the_instructions_and_schema() {
        let req = LlmRequest { cached_prefix: Some("#1 Hades".into()), ..request("Pick one.") };
        let p = prompt(&req, false);
        let at = |s: &str| p.find(s).unwrap_or_else(|| panic!("{s:?} missing from {p}"));
        assert!(at("Write JSON.") < at(ONLY_THESE));
        assert!(at(FORMAT) < at(r#""required":["answer"]"#));
        assert!(at(r#""required":["answer"]"#) < at("#1 Hades") && at("#1 Hades") < at("Pick one."));
        assert!(p.contains(super::super::UNTRUSTED_NOTE));
        assert!(!p.contains(AGAIN));
        assert!(prompt(&req, true).ends_with(AGAIN));
    }

    #[test]
    fn an_accounts_own_home_keeps_its_sessions() {
        let home = Path::new("/srv/upnext/copilot/2");
        assert_eq!(copilot_home(Some(home)), Some(home.to_path_buf()));
        let dir = CallDir::new("abc", Some(home)).unwrap();
        assert_eq!(dir.session, Some(home.join("session-state").join("abc")));
        let path = dir.path.clone();
        assert!(path.is_dir());
        drop(dir);
        assert!(!path.exists());
    }

    /// A stand-in `copilot`: a shell script that answers `--version`, reads
    /// the prompt, writes `reply` lines and a usage file, and keeps a session
    /// folder in its home the way Copilot does.
    #[cfg(unix)]
    fn fake(dir: &Path, name: &str, body: &str) -> String {
        use std::os::unix::fs::PermissionsExt;
        let bin = dir.join(name);
        let script = format!(
            "#!/bin/sh\n[ \"$1\" = --version ] && {{ echo 'GitHub Copilot CLI 1.0.103.'; exit 0; }}\n\
             for a in \"$@\"; do case \"$a\" in --session-id=*) sid=${{a#--session-id=}} ;; esac; done\n\
             prev=; for a in \"$@\"; do [ \"$prev\" = --usage-output-file ] && usage=$a; prev=$a; done\n\
             mkdir -p \"$COPILOT_HOME/session-state/$sid\"\n\
             cat > \"$COPILOT_HOME/session-state/$sid/prompt.txt\"\n\
             echo '{{\"modelMetrics\":{{\"m\":{{\"usage\":{{\"inputTokens\":100,\"outputTokens\":10}}}}}}}}' > \"$usage\"\n\
             echo x >> \"$COPILOT_HOME/calls\"\n{body}\n"
        );
        std::fs::write(&bin, script).unwrap();
        std::fs::set_permissions(&bin, std::fs::Permissions::from_mode(0o755)).unwrap();
        bin.display().to_string()
    }

    /// The whole call against a stand-in CLI: the prompt reaches it, the
    /// answer and tokens come back, a reply that isn't JSON is asked for
    /// once more, and no session is left in Copilot's home.
    #[cfg(unix)] // The stand-in CLI is a shell script.
    #[tokio::test]
    async fn a_call_runs_the_cli_and_cleans_up() {
        let dir = tempfile::tempdir().unwrap();
        let home = dir.path().join("home");
        let says = |text: &str| {
            format!(
                "echo '{{\"type\":\"user.message\",\"data\":{{\"content\":\"x\"}}}}'\n\
                 echo '{{\"type\":\"assistant.message\",\"data\":{{\"model\":\"claude-haiku-5.5\",\"content\":{text}}}}}'\n\
                 echo '{{\"type\":\"result\",\"exitCode\":0}}'"
            )
        };
        let ok = fake(dir.path(), "ok", &says(r#""{\"answer\":\"ready\"}""#));
        let backend = CopilotCliBackend::new(ok).with_home(Some(home.clone()));
        let resp = backend.complete_json(request("Is it ready?")).await.unwrap();
        assert_eq!(resp.json, json!({"answer": "ready"}));
        assert_eq!(resp.model.as_deref(), Some("claude-haiku-5.5"));
        assert_eq!(resp.usage.map(|u| (u.input_tokens, u.output_tokens)), Some((100, 10)));
        let sessions = home.join("session-state");
        assert_eq!(std::fs::read_dir(&sessions).unwrap().count(), 0, "the call's session folder is removed");

        // Prose the first time, JSON the second.
        let flaky = fake(
            dir.path(),
            "flaky",
            &format!(
                "if [ \"$(wc -l < \"$COPILOT_HOME/calls\")\" -lt 2 ]; then\n{}\nelse\n{}\nfi",
                says(r#""I think it is ready.""#),
                says(r#""{\"answer\":\"ready\"}""#)
            ),
        );
        std::fs::remove_file(home.join("calls")).unwrap();
        let resp = CopilotCliBackend::new(flaky).with_home(Some(home.clone())).complete_json(request("Is it ready?")).await.unwrap();
        assert_eq!(resp.json, json!({"answer": "ready"}));
        assert_eq!(resp.usage.map(|u| u.input_tokens), Some(200), "both calls' tokens count");

        // Prose both times: given up after the second.
        std::fs::remove_file(home.join("calls")).unwrap();
        let chatty = fake(dir.path(), "chatty", &says(r#""I think it is ready.""#));
        let err = CopilotCliBackend::new(chatty).with_home(Some(home.clone())).complete_json(request("x")).await.unwrap_err();
        assert!(matches!(&err, LlmError::BadOutput(_, Some(u)) if u.input_tokens == 200), "{err:?}");
        assert_eq!(std::fs::read_to_string(home.join("calls")).unwrap().lines().count(), 2, "asked twice, not more");

        let missing = CopilotCliBackend::new(dir.path().join("missing").display().to_string());
        let err = missing.available().await.unwrap_err();
        assert!(matches!(&err, LlmError::NotAvailable(m) if m.contains("winget install GitHub.Copilot")), "{err:?}");
        assert_eq!(version(&fake(dir.path(), "v", ""), None).await.unwrap(), "GitHub Copilot CLI 1.0.103");
    }

    /// An AI account's own home: this computer's tokens and GitHub CLI
    /// sign-in don't reach the CLI; this computer's own calls keep them.
    #[test]
    fn an_accounts_call_gets_no_sign_in_of_this_computers() {
        let home = Path::new("/srv/upnext/copilot/2");
        let envs = |cmd: &Command| -> Vec<(String, Option<String>)> {
            cmd.as_std().get_envs().map(|(k, v)| (k.to_string_lossy().into_owned(), v.map(|v| v.to_string_lossy().into_owned()))).collect()
        };
        let set = envs(&command("copilot", Some(home)));
        for v in TOKEN_VARS {
            assert!(set.contains(&(v.to_string(), None)), "{v} is taken away: {set:?}");
        }
        assert!(set.contains(&("COPILOT_HOME".into(), Some(home.display().to_string()))));
        assert!(set.contains(&("GH_CONFIG_DIR".into(), Some(home.join("gh").display().to_string()))));
        assert!(envs(&command("copilot", None)).iter().all(|(k, _)| !TOKEN_VARS.contains(&k.as_str()) && k != "COPILOT_HOME"));
    }

    /// A Copilot that hangs without reading its input, given a prompt larger
    /// than the pipe holds: the call fails at its timeout instead of holding
    /// the job for ever.
    #[cfg(unix)] // The stand-in CLI is a shell script.
    #[tokio::test]
    async fn a_cli_that_never_reads_the_prompt_times_out() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().unwrap();
        let bin = dir.path().join("copilot");
        std::fs::write(&bin, "#!/bin/sh\n[ \"$1\" = --version ] && { echo 'GitHub Copilot CLI 1.0.103.'; exit 0; }\nexec sleep 60\n")
            .unwrap();
        std::fs::set_permissions(&bin, std::fs::Permissions::from_mode(0o755)).unwrap();
        let backend =
            CopilotCliBackend { bin: bin.display().to_string(), timeout: Duration::from_millis(500), home: Some(dir.path().join("home")) };
        let req = request(&"Отзыв игрока о сюжете. ".repeat(4000));
        assert!(req.full_prompt().len() > 64 * 1024);
        let started = Instant::now();
        let call = tokio::time::timeout(Duration::from_secs(10), backend.complete_json(req));
        let err = call.await.expect("the call ended on its own").unwrap_err();
        assert!(matches!(&err, LlmError::Failed(m) if m.contains("timed out")), "{err:?}");
        assert!(started.elapsed() < Duration::from_secs(5), "{:?}", started.elapsed());
    }
}
