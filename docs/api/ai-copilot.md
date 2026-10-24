# GitHub Copilot through the Copilot CLI

[API index](../API.md) · [AI backends](ai-backends.md) · [Prices and plans](ai-prices.md)

`llm_backend` `copilot-cli` runs picks, Help me decide, review summaries and series guides on a GitHub Copilot plan through GitHub's Copilot CLI. You sign in with `copilot login`; UpNext only runs the program and never reads Copilot's credentials.

- `copilot_model` (picks, series guides, Help me decide) and `copilot_digest_model` (review summaries): a model by Copilot's name, `claude-sonnet-5.5` and `claude-haiku-5.5` unless set. `auto` lets Copilot choose, and is the only model Copilot Free allows. A value with a space is a `400`.
- Env `UPNEXT_COPILOT_MODEL` and `UPNEXT_COPILOT_DIGEST_MODEL`, and `UPNEXT_COPILOT_BIN` (env only, `copilot` unless set).

Each call checks `copilot --version` first, then sends the prompt on stdin with `--output-format json`. The CLI has no system prompt or output schema option, so UpNext's instructions and the JSON Schema come first in the prompt; a reply that isn't JSON is asked for once more, and both calls' tokens count. Every tool, the built-in MCP servers, custom instructions (`AGENTS.md` and the like), questions to the user, updates and the CLI's own telemetry are turned off. The tokens come from the CLI's `--usage-output-file`.

A signed-out CLI fails with "Copilot isn't signed in, or its sign-in has expired: run `copilot login` in a terminal, then try again". Copilot's `rate_limit` and `quota` errors stop a batch job ("Copilot said: …"), and `authorization` and `policy` ones (a model the plan or an organization doesn't allow) fail every call the same way. Calls are recorded with backend `copilot-cli` and the model Copilot names in its answer. They aren't priced: `cost_usd` and `api_usd` are null, since a Copilot plan isn't charged per call and one plan runs models from several providers.

Copilot keeps each session in its home folder. Each call gets a session id of its own, and UpNext removes that session's folder when the call ends. Copilot's search index of past sessions (`session-store.db` in the same folder) still lists the calls; UpNext doesn't write to it.

## GET /api/llm/copilot

The Copilot CLI, for Settings, from `copilot --version`, which answers locally:

```json
{"installed": true, "version": "GitHub Copilot CLI 1.0.103", "error": null,
 "models": [{"id": "auto", "name": "Auto", "description": "Copilot chooses the model for each call. The only choice on Copilot Free."},
            {"id": "claude-sonnet-5.5", "name": "Claude Sonnet 5.5", "description": "UpNext's default for picks."}]}
```

- `version` is null and `error` says why when `copilot` can't be run (not installed, or an npm install without Node.js).
- `models` is a list built into UpNext: the CLI lists a plan's models only inside a session. A model the plan doesn't include fails on `POST /api/llm/test` with Copilot's reason.
- Whether the CLI is signed in shows only on a call, such as `POST /api/llm/test`.

## AI accounts

On an AI account other than this computer's ([AI accounts](ai-accounts.md)), the CLI runs with `COPILOT_HOME` set to `<data folder>/copilot/<id>`, so the account signs in there: `COPILOT_HOME=<data folder>/copilot/<id> copilot login`. This computer's `COPILOT_GITHUB_TOKEN`, `GH_TOKEN` and `GITHUB_TOKEN` are taken out of its environment, and `GH_CONFIG_DIR` points into that folder, so it can't fall back to this computer's GitHub CLI sign-in.
