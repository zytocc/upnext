# AI usage and estimates

Every AI call is recorded when it comes back, whether it worked or not: when, what for, the backend and model, the time it took and the tokens its backend reported. Records are kept for a year.

| `kind` | What | Counted per |
|---|---|---|
| `pick` | Picks, a Curate again, and new spares | pick |
| `too` | "It's too ___" | call |
| `decide` | Help me decide | call |
| `review_summaries` | People say, Vibes and review tags | title |
| `series_guides` | Series guides | series |
| `second_chances` | Second Chance's "Now" | game |
| `hidden_gems` | Find hidden gems' ratings | game |
| `test` | Settings' "Test" button | call |

A lineup's `usage` adds up every call made with it: the pick, its retry, "It's too ___" and new spares.

## Usage and totals

`Usage` is the tokens of a call or a sum of calls: `input_tokens` (all of the input), `output_tokens` (a reasoning model's thinking included), `cache_read_tokens`, `cache_write_tokens` and `cache_write_1h_tokens`. The cache fields are the parts of the input read from or written to the provider's prompt cache. OpenAI-style servers report cached input only when they cache, and no writes.

`AiTotals` adds calls up:

```json
{"calls": 12, "failed": 0, "input_tokens": 196691, "output_tokens": 15389, "cache_read_tokens": 40302, "cache_write_tokens": 156361,
 "cache_write_1h_tokens": 156361, "cost_usd": null, "api_usd": 0.79, "items": 40, "duration_ms": 120059}
```

- `failed` is how many of `calls` failed. A failed call counts the tokens of a reply that came back unusable, and none otherwise.
- `cost_usd` is money charged, at list prices: an API key, or a hosted provider whose prices UpNext reads. Null when nothing was charged.
- `api_usd` is every call at the API's list prices, charged or not. On a subscription it's what the same calls would cost on the API, not money spent. Null when no call had a known price, as with a local model.
- `items` is the titles or series the calls that worked covered.

## GET /api/ai/usage?days=30

The last `days` days (1 to 365, 30 unless asked), today included, in this computer's local days:

```json
{"days": 30, "backend": "claude-cli", "priced": false,
 "today": AiTotals, "last_7_days": AiTotals, "total": AiTotals,
 "by_day": [{"day": "2026-09-30", "...AiTotals": 0, "by_kind": [{"kind": "pick", "...AiTotals": 0}]}],
 "by_kind": [{"kind": "pick", "...AiTotals": 0}],
 "by_model": [{"backend": "claude-cli", "model": "<model>", "...AiTotals": 0}],
 "by_player": [], "by_account": []}
```

- `backend` is the one in use. `priced` is true when a backend in use is charged per call at prices UpNext reads. Money shows only with "Show costs in US dollars" on.
- `today` and `last_7_days` are there whatever `days` is; `total` covers the `days`.
- `by_day` lists every day of the period, oldest first, days without calls included. `by_kind` is in the table's order, and `by_model` has the most tokens first.
- `by_player` and `by_account` break it out by who asked and by AI account ([Players](players.md)).

## GET /api/ai/estimates

What each AI feature would take on this library, for the wizard's AI step and a job's estimate:

```json
{"backend": "claude-cli", "priced": false, "features": [
  {"kind": "review_summaries", "backend": "claude-cli", "model": "sonnet",
   "rate": {"input_tokens": 4060, "output_tokens": 520, "items_per_call": 6.0, "seconds_per_call": 21.4, "api_usd": null, "basis": "built_in", "calls": 0},
   "one_time": {"items": 1600, "calls": 267, "input_tokens": 6496000, "output_tokens": 832000, "cost_usd": null, "minutes": 31.8, "basis": "library"},
   "monthly": {"items": 16, "calls": 3, "input_tokens": 64960, "output_tokens": 8320, "cost_usd": null, "minutes": 0.4, "basis": "assumed"}}]}
```

- `rate` is per title (`review_summaries`), per series (`series_guides`) or per call (`pick`, `decide`, `too`). Its `basis` is `history` once this install has 3 calls of that kind that worked through the backend in use in the last 90 days, else `built_in`. `seconds_per_call` is null for an OpenAI-style server until it has been used.
- `one_time` is the run for titles with reviews and no summary, and series with no guide. Null for the features that run only when asked.
- `monthly` is a typical month from recent use. It is `assumed` until the library is 14 days old.
- `cost_usd` is set only for a backend charged per call.
