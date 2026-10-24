# Third-party services: AI

[Third-party services](../THIRD_PARTY.md) › AI. Also: [stores](stores.md), [game data](data.md), [publishing](publishing.md), [what the project uses](project.md).

None of these is a data source, so none has a credit in Settings › About.

| Service | What UpNext calls | Terms |
|---|---|---|
| Claude, through your Claude Code CLI | `claude -p --output-format json` as a subprocess. UpNext never reads the CLI's credentials | [Claude Code legal and compliance](https://code.claude.com/docs/en/legal-and-compliance), [Consumer Terms](https://www.anthropic.com/legal/consumer-terms), [Usage Policy](https://www.anthropic.com/legal/aup), [Agent SDK with your Claude plan](https://support.claude.com/en/articles/15036540-use-the-claude-agent-sdk-with-your-claude-plan) |
| Claude Code's release site | `downloads.claude.ai/claude-code-releases`: the manifest and the unmodified `claude` binary, only when you ask (`upnext install-claude`, or `UPNEXT_INSTALL_CLAUDE=1` in UpNext's container) | Claude Code's licence (below) |
| Anthropic API | The Messages API, with your key | [Commercial Terms](https://www.anthropic.com/legal/commercial-terms), [Usage Policy](https://www.anthropic.com/legal/aup) |
| ChatGPT, through your Codex CLI | `codex exec` as a subprocess, on your ChatGPT sign-in. UpNext never reads its credentials | [OpenAI Terms of Use](https://openai.com/policies/terms-of-use/), [Codex authentication](https://learn.chatgpt.com/docs/auth) |
| GitHub Copilot, through your Copilot CLI | `copilot` with `--output-format json` as a subprocess, on your GitHub sign-in. UpNext never reads its credentials | [GitHub Terms of Service](https://docs.github.com/en/site-policy/github-terms/github-terms-of-service) (section J, AI features), [Acceptable Use Policies](https://docs.github.com/en/site-policy/acceptable-use-policies/github-acceptable-use-policies), [Copilot CLI programmatic use](https://docs.github.com/en/copilot/reference/copilot-cli-reference/cli-programmatic-reference) |
| OpenAI API | Chat Completions, with your key | [Services Agreement](https://openai.com/policies/services-agreement/), [Usage policies](https://openai.com/policies/usage-policies/) |
| OpenAI-compatible servers | Whatever you point UpNext at: Ollama, LM Studio, llama.cpp, vLLM, OpenRouter, and Settings' presets for Google Gemini, Groq, Mistral, DeepSeek and xAI | The server's and the model's own licences |

With an API key, the key owner's terms and usage policy apply. With the Codex CLI, your credentials stay your own. OpenAI recommends API keys for automated workflows and has opened its plans to third-party tools with "Sign in with ChatGPT"; nothing was found against running your own signed-in CLI.

## GitHub Copilot CLI

GitHub documents running the Copilot CLI from scripts without a person at the terminal ("programmatic" use, with `--output-format json` for machine-readable output), and publishes an SDK for building the same runtime into other applications on the user's own Copilot subscription. The CLI's licence (its `LICENSE.md`) lets an application go further and ship unmodified copies of it, "as part of an application or service that provides material functionality beyond the Software itself". UpNext doesn't ship it: it runs the `copilot` you installed and signed in to, as a subprocess.

For individual plans, section J of GitHub's Terms of Service (AI features) governs Copilot, and the Acceptable Use Policies forbid "excessive automated bulk activity". UpNext's calls are ones you start or schedule for your own library, a few for a lineup and one for each batch of review summaries, and every one counts against your plan's allowance like any other Copilot request. Copilot Business and Enterprise users are under their organization's agreement, and an organization can turn the CLI off. GitHub's name is used only to say which program runs.

## Claude Code

Anthropic's Claude Code legal page says:

- Subscription sign-in "is intended exclusively for purchasers of Claude Free, Pro, Max, Team, and Enterprise subscription plans and is designed to support ordinary use of Claude Code and other native Anthropic applications."
- "Developers building products or services that interact with Claude's capabilities, including those using the Agent SDK, should use API key authentication."
- "Anthropic does not permit third-party developers to offer Claude.ai login into their own applications, or to route requests through Free, Pro, or Max plan credentials on behalf of their users", and developers "may not collect, store, or intermediate Claude.ai credentials or session tokens".
- None of that prevents "an end user from signing in to the unmodified Claude Code binary with their own Claude subscription".
- "Advertised usage limits for Pro and Max plans assume ordinary, individual usage of Claude Code and the Agent SDK."

UpNext runs the `claude` binary you installed and signed in to through Anthropic's own flow, unmodified, as a subprocess with `-p`, the pattern the Agent SDK docs describe for other languages. It never sees, stores or forwards the credentials, offers no Claude login, and pays for and resells nothing. That's the end-user case the page allows. Anthropic's help article says `claude -p` and third-party app usage "still draw from your subscription's usage limits". The feature isn't called "Claude Code" in the app, and no Anthropic logos are used. See [decision 5](../THIRD_PARTY.md#decisions).

### In UpNext's container

Claude Code's licence (its `LICENSE.md`) is "© Anthropic PBC. All rights reserved. Use is subject to Anthropic's Commercial Terms of Service". Nothing grants the right to pass the binary on, so UpNext's published image doesn't carry it.

A container installs it only when you ask (`UPNEXT_INSTALL_CLAUDE=1`, or `upnext install-claude`). It downloads the unmodified binary from Anthropic's own release site, checks it against Anthropic's manifest, and runs the binary's own `claude install`, as Anthropic's `install.sh` does, so the copy comes from Anthropic to you.

You sign it in with `claude setup-token`, Anthropic's flow for machines without a browser, and pass the token to your own container as `CLAUDE_CODE_OAUTH_TOKEN`. UpNext reads only whether the variable is set and leaves it in the environment the CLI inherits. It never reads, stores or sends its value.
