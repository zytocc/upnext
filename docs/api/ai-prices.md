# Prices and plans

## Prices

Each call is priced when it comes back, from the list prices of the provider its backend uses:

| Backend | Provider | Charged |
|---|---|---|
| Claude subscription (`claude-cli`) | Anthropic | no: `api_usd` is what it would cost on the API |
| ChatGPT subscription (`codex-cli`) | OpenAI | no: `api_usd` is what it would cost on the API |
| GitHub Copilot plan (`copilot-cli`) | none | no figure: one plan runs several providers' models |
| Anthropic API key (`api-key`) | Anthropic | yes |
| OpenAI API key (`openai-key`) | OpenAI | yes |
| Another provider at `api.openai.com` | OpenAI | yes |
| Another provider at `openrouter.ai` | OpenRouter | yes |
| Any other server (local models, Groq, Gemini…) | none | no figure |

UpNext reads the prices from the provider's own pricing page, and from OpenRouter's models API for OpenRouter. Until a read works, the last prices stay in use. Anthropic's fall back to a list built into UpNext; the others have none, so their calls have no figure until a read works.

### GET /api/ai/prices

The prices the AI in use is priced at:

```json
{"backend": "claude-cli", "charged": false,
 "prices": {"provider": "anthropic", "label": "Anthropic", "page_url": "https://platform.claude.com/docs/en/about-claude/pricing",
   "fetched_at": 1759280000, "built_in_checked": null, "error": null, "tried_at": 1759280000,
   "models": {"<model>": {"input": 2.0, "output": 10.0, "cache_read": 0.1, "cache_write_5m": 2.5, "cache_write_1h": 4.0}}}}
```

Prices are US dollars per million tokens, and `page_url` is the official page, to link wherever a price shows. `fetched_at` is null, with `built_in_checked` the date the built-in list was checked, while that list is in use. `error` is why the last read failed, at `tried_at`. `prices` is null for a local model or a provider whose prices UpNext can't read. With a separate summaries backend, the answer describes the picks' backend. `prices.flex` gives OpenAI's Flex prices by model when the page lists them.

`POST /api/ai/prices/refresh` reads them again now, in the same shape. A failure is in `prices.error` and the prices in use stay.

## GET /api/ai/plan

On a subscription, the plan's usage bars and what UpNext's AI takes of them. It reads the bars now, unless a reading is under 30 seconds old.

```json
{"backend": "claude-cli", "subscription": true, "plan": "max",
 "reading": {"at": 1759280000, "windows": [
   {"id": "session", "label": "Current session", "used": 20.0, "resets": "Sep 30, 6:30pm (America/New_York)", "resets_at": null},
   {"id": "week", "label": "Current week (all models)", "used": 40.0, "resets": "Oct 4, 3pm (America/New_York)", "resets_at": null}]},
 "error": null,
 "measured": {"measure": "api_usd", "spans": 3,
   "windows": [{"id": "session", "label": "Current session", "per_point": 0.36, "spans": 3, "from_session": null},
               {"id": "week", "label": "Current week (all models)", "per_point": 1.98, "spans": 0, "from_session": 5.5}],
   "each": [{"kind": "pick", "items": 1, "percent": {"session": 0.33, "week": 0.06}}],
   "one_time": {"kind": "review_summaries", "items": 200, "percent": {"session": 41.0, "week": 7.5}}}}
```

- `subscription` is true on the Claude CLI and the Codex CLI. For any other backend it's false and the rest is null.
- `plan` is the plan's name (`max` and `pro` for Claude; `plus`, `pro` and `team` for ChatGPT). UpNext doesn't know any plan's limits.
- `reading.windows` has the limits the plan has. A limit it doesn't have is absent, not 0 or 100. `used` is always the percent used.
- Claude windows have `resets` (the CLI's own wording) and a null `resets_at`. Codex windows have `resets_at` in Unix seconds and a null `resets`.
- A Codex window's `id` comes from its length: `session` (5 hours), `week`, or `<minutes>m` for any other, such as `1440m`. A limit for one model adds its name, `week:<model>`.
- `error` says why the bars couldn't be read, with `reading` null. Signed out, Codex says "codex account authentication required to read rate limits".
- Reading the bars spends nothing. UpNext reads them around its own calls and keeps the readings for a year, per AI account.
- `measured` is null until UpNext has a usable measurement. `per_point` is what one point of a window cost, in `measure` (`api_usd` or `tokens`). `each` and `one_time` turn it into percent of each window per pick, decide, summary or guide, and for the summaries still to write.
- A change of plan starts the readings over.

A Claude Code CLI account other than this computer's with no token of its own has no bars to read (`subscription: false`). See [Accounts](accounts.md#ai-accounts).
