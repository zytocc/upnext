# AI backends

`llm_backend` chooses where picks, Help me decide, review summaries and series guides go. `GET /api/settings` lists the choices as `llm_backends`:

| `llm_backend` | What | Details |
|---|---|---|
| `claude-cli` (the default) | a Claude subscription, through the Claude Code CLI | [Claude models](#claude-models) |
| `codex-cli` | a ChatGPT subscription, through the Codex CLI | [Codex CLI](ai-providers.md#codex-cli) |
| `copilot-cli` | a GitHub Copilot plan, through the Copilot CLI | [Copilot CLI](ai-providers.md#copilot-cli) |
| `api-key` | an Anthropic API key, billed per call | [Claude models](#claude-models) |
| `openai-key` | an OpenAI API key, billed per call | [OpenAI API key](ai-providers.md#openai-api-key) |
| `openai` | any OpenAI-compatible server | [Another provider](ai-providers.md#another-openai-compatible-provider) |

A change applies to the next request without a restart. Clearing a setting (`null` or `""`) falls back to `.env`. `GET /api/config` gives `llm_backend`, `claude_available` (whether the AI in use can run) and `llm_error` (why not). Each player can have AI accounts of their own ([Accounts](accounts.md#ai-accounts)).

## Settings every backend has

- `<backend>_model` is for picks, series guides and Help me decide, and `<backend>_digest_model` for review summaries and review tags. The Claude backends use `model` and `digest_model`.
- A backend with an effort setting has `<backend>_effort`: one of the `<backend>_efforts` that `GET /api/settings` lists, or null for the provider's default. Anything else is a `400`.
- A family id such as `luna` means the newest model of that family, worked out on each call.
- A key is write-only. `GET` gives `<key>_set` and `<key>_source`, and the key is masked in `config list` and left out of exports and copies without keys.
- Each setting has an `UPNEXT_…` environment twin, listed with its backend in [AI providers](ai-providers.md).
- Calls are recorded with the backend and the model they resolved to ([AI usage](ai-usage.md)).
- A rejected sign-in or key, a refusal, a model the account can't use and spent credit fail every call the same way, so a batch job stops at the first.

## Claude models

`model` is for picks, series guides and Help me decide, and `digest_model` for review summaries and review tags. On the Claude CLI both are `sonnet` unless set; `opus` and `default` (the CLI's own default) are the other choices, as `models` and `digest_models` list them.

With the API key (`anthropic_api_key`, starting `sk-ant-`), `sonnet`, `opus` and `haiku` mean `claude-sonnet-5-5`, `claude-opus-5-5` and `claude-haiku-5-5`. Picks, series guides and Help me decide default to Sonnet, and review summaries and tags to Haiku. While a setting was never chosen or is `default`, `GET /api/settings` shows `default`. `models` offers `haiku` too.

## A separate AI for summaries

Review summaries and tags, Second Chance's "Now" and hidden gem ratings can go to another backend, such as a Claude subscription for picks and a model at home for this bulk work. Picks, series guides, Help me decide and "It's too ___" stay on `llm_backend`.

- `digest_backend` (`UPNEXT_DIGEST_LLM`) is `same` (the default) or one of `llm_backends`. `GET` lists `digest_backends` and `digest_models`, the Claude models that backend offers.
- The chosen backend uses its own settings, with its summaries model: `digest_model`, `codex_digest_model`, `copilot_digest_model`, `openai_key_digest_model`, or `openai_digest_model` falling back to `openai_model`. Two different OpenAI-compatible servers aren't possible; make one of them the OpenAI API key backend.
- `ai` in `GET /api/gems` and `GET /api/second-chances` is the summaries backend's state, and `GET /api/ai/estimates` gives each feature its `backend`. `GET /api/ai/plan` reads the picks' subscription, or the summaries' when only summaries run on one.

## POST /api/llm/test

A tiny JSON round trip through the backend, always `200`: `{"ok": true, "backend", "model", "duration_ms", "usage", "cost_usd", "api_usd"}` or `{"ok": false, "backend", "error"}`. This is Settings' "Test" button, recorded like any call (kind `test`). `?purpose=summaries` tries the summaries backend and model.
