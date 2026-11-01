# AI providers

Settings every backend shares are on [AI backends](ai-backends.md).

## Codex CLI

`codex-cli` runs on a ChatGPT plan through `codex exec`. Sign in with `codex login`. A sign-in with an API key is refused (use [OpenAI API key](#openai-api-key)), and a signed-out Codex fails at once with "Codex isn't signed in: run `codex login` in a terminal, then try again".

`codex_model` and `codex_digest_model` are `luna` unless set. `codex_effort` is one of `codex_efforts` (`low`, `medium`, `high`, `xhigh`, `max`, `ultra`; a model takes some of them) or null. Env: `UPNEXT_CODEX_MODEL`, `UPNEXT_CODEX_DIGEST_MODEL`, `UPNEXT_CODEX_EFFORT` and `UPNEXT_CODEX_BIN` (env only, `codex` unless set).

`GET /api/llm/codex` describes the CLI for Settings:

```json
{"installed": true, "version": "codex-cli 0.159.2",
 "login": {"signed_in": true, "status": "Logged in using ChatGPT", "per_call": false},
 "error": null,
 "models": [{"id": "<model>", "name": "<model>", "efforts": ["low", "medium"], "default_effort": "medium"}],
 "families": [{"id": "luna", "model": "<model>"}],
 "models_from": "codex", "models_error": null}
```

`login` is null and `error` says why when `codex` can't run. `per_call` is true for a sign-in that bills each call. When Codex's model list can't be read, `models_from` is `"built_in"` and `models_error` says why. The plan's usage bars are on [Prices and plans](ai-prices.md#get-apiaiplan).

## Copilot CLI

`copilot-cli` runs on a GitHub Copilot plan through GitHub's Copilot CLI. Sign in with `copilot login`. A signed-out CLI fails with "Copilot isn't signed in, or its sign-in has expired: run `copilot login` in a terminal, then try again".

`copilot_model` and `copilot_digest_model` are `claude-sonnet-5.5` and `claude-haiku-5.5` unless set. `auto` lets Copilot choose and is the only model Copilot Free allows. A value with a space is a `400`. There is no effort setting. Env: `UPNEXT_COPILOT_MODEL`, `UPNEXT_COPILOT_DIGEST_MODEL` and `UPNEXT_COPILOT_BIN` (env only, `copilot` unless set).

Copilot's `rate_limit` and `quota` errors stop a batch job ("Copilot said: …"), and `authorization` and `policy` ones (a model the plan or organization doesn't allow) fail every call. Calls have no price figure.

`GET /api/llm/copilot`:

```json
{"installed": true, "version": "GitHub Copilot CLI 1.0.103", "error": null,
 "models": [{"id": "auto", "name": "Auto"}]}
```

`version` is null and `error` says why when `copilot` can't run. `models` is a list built into UpNext, and a model the plan lacks fails `POST /api/llm/test` with Copilot's reason. Whether the CLI is signed in shows only on a call.

## OpenAI API key

`openai-key` runs on the OpenAI API with your own key, billed per call.

`openai_key` is the key, write-only, starting `sk-` (an Anthropic `sk-ant-` key is a `400`). `GET` gives `openai_key_set` and `openai_key_source`. `openai_key_model` and `openai_key_digest_model` are `luna` unless set. `openai_key_effort` is one of `openai_key_efforts` (`low`, `medium`, `high`, `xhigh`, `max`) or null. Env: `OPENAI_API_KEY`, `UPNEXT_OPENAI_KEY_MODEL`, `UPNEXT_OPENAI_KEY_DIGEST_MODEL`, `UPNEXT_OPENAI_KEY_EFFORT` and `UPNEXT_OPENAI_URL` (env only, `https://api.openai.com/v1` unless set, for a mock).

Review summaries and series guides use OpenAI's cheaper, slower Flex tier, and go at the standard price when Flex is unavailable. A spend limit stops a batch with "the OpenAI key's credit has run out" or "the OpenAI key reached its spend limit". Calls are charged at OpenAI's list prices, so `cost_usd` and `api_usd` are both set.

`GET /api/llm/openai` lists what the key can use:

```json
{"key_set": true, "families": [{"id": "luna", "model": "<model>"}],
 "models": ["<model>"], "models_from": "openai", "models_error": null}
```

`models` are newest first. Without a key, or when OpenAI can't be reached or rejects the key, `models_from` is `"built_in"` and `models_error` says why.

## Another OpenAI-compatible provider

`openai` is any server that answers `POST {base}/chat/completions`: Ollama, LM Studio, llama.cpp, vLLM, LocalAI or a hosted API. Settings calls it "Another provider".

- `openai_base_url` (`http://localhost:11434/v1` for Ollama), `openai_model` (required), `openai_api_key` (optional, write-only; `GET` gives `openai_api_key_set`) and `openai_digest_model`, which falls back to `openai_model`. Env: `UPNEXT_OPENAI_BASE_URL`, `UPNEXT_OPENAI_MODEL` and `UPNEXT_OPENAI_API_KEY`.
- From another device, a `PUT` that changes `openai_base_url` clears the saved key unless it sends a new one. With the key in the environment, such a change is `403` unless it brings its own key.
- `openai_thinking` (`UPNEXT_OPENAI_THINKING`) is `auto` (the default), `off` or `on`. `off` asks the server not to think first, for models that reason before answering. `auto` means `off` for a server on this computer or network and `on` elsewhere.
- UpNext asks for JSON that fits a schema and falls back to plain JSON when the server refuses.

`GET /api/llm/models` lists the server's models for Settings: `{"ok": true, "models": ["…"]}` or `{"ok": false, "models": [], "error": "…"}`.

Calls to `api.openai.com` and `openrouter.ai` are priced and charged at their list prices. Any other server has no price figure.
