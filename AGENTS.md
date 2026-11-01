# UpNext

A local game-backlog picker: a Rust server (axum, rusqlite, clap) in `src/` and a React, Vite and Tailwind UI in `web/`. README.md says what it does; CONTRIBUTING.md has setup. This file is the short form for AI coding agents.

## Checks before a commit

```sh
cargo fmt --check
cargo clippy --all-targets --locked -- -D warnings
cargo clippy --all-targets --locked --no-default-features -- -D warnings
cargo test --locked
cargo deny --locked check
node contrib/notices/generate.mjs --check
(cd web && npm run licenses && npm run gen:api && git diff --exit-code src/generated/api.ts && npx tsc -b && npm run lint && npm test && npx vite build)
(cd services/verdicts && npm ci && npm run licenses && npm run typecheck && npm run lint && npm test && npm run build)
```

CI runs the same list (`.github/workflows/ci.yml`). [docs/contributing/checks.md](docs/contributing/checks.md) explains each step, the end-to-end tests and the Linux build packages.

## Rules

- Never use a real library or real keys, and never commit a database, `.env`, token or data cut from a real library. Use `upnext mock-library` with a scratch `UPNEXT_DATA_DIR`, `upnext prompt` (prints, sends nothing) or the fake model in `web/e2e/fake-llm.mjs`. When real AI calls are needed, use the API key backend's defaults, `sonnet` for picks and `haiku` for summaries.
- Tests never call real services: clients take a base URL, and tests point it at `wiremock`.
- Refactors change structure only; a behaviour change goes in its own commit with a test.
- An API change, a schema change and a new data source each have a recipe in [docs/contributing/code-map.md](docs/contributing/code-map.md). Each lands in one commit with its docs, its OpenAPI update and its tests.
- A dependency change or a new credit in `web/src/lib/credits.ts` means regenerating the third-party notices ([checks](docs/contributing/checks.md#third-party-notices)).
- HowLongToBeat is scraped and stays off unless the user turns it on (its job's schedule, or `--with hltb`).
- Comments are plain, full sentences that say why, matching the code around them.
- Docs say what it does and what to type. Give a reason only when ignoring it breaks something. One fact per sentence, at most one parenthesis, in plain order: say what happens and what it causes, not "The one X is Y" or "What X is, is Y". No "never" or "isn't" unless a reader would assume the opposite. Leave out internals the reader can't type, click or see (tables, file formats, prompt text, scoring weights, retry timings), and history, decision dates or "yet". Each fact has one home page; other pages link to it. API pages give the route, what it means and the rules a client must handle; `docs/openapi.json` holds the fields and status codes. Pages under `docs/` stay under 900 words. A changelog entry is one line: what a user notices.

`docs/dev/` holds the maintainer's runbooks; contributors don't need them.
