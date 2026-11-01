# Contributing to UpNext

Thanks for looking. UpNext is a Rust server (axum, rusqlite, clap) in `src/` with a React, Vite and Tailwind UI in `web/`. [Checks](docs/contributing/checks.md) has the tests CI runs, [Where things go](docs/contributing/code-map.md) is a map of the code, and `AGENTS.md` is the short form for coding agents.

UpNext itself is developed with AI coding assistants, mostly Claude Code, as the README says. A pull request written with an assistant's help is welcome on the same terms as any other: it passes the checks, and you stand behind every line of it.

## Setting up

You need:

- Rust 1.90 or newer (`rust-version` in Cargo.toml; CI uses current stable), from [rustup](https://rustup.rs).
- Node 22.18 or newer on an even-numbered release (`engines` in web/package.json; CI uses 24), and npm.
- On Linux, the WebKitGTK and GTK 3 headers that [Building from source](docs/install/source.md) lists. Without them, build with `--no-default-features`: the server alone, which is all most changes need.

```sh
git clone https://github.com/zytocc/upnext && cd upnext
npm ci --prefix web                    # a release build embeds web/dist
npm run build --prefix web
cargo build
```

### A library to work on

Don't develop against your real library. `upnext mock-library` writes a made-up library and taste profile into an empty database:

```sh
export UPNEXT_DATA_DIR=$PWD/data/dev    # its own database and backups
cargo run -- mock-library
cargo run -- serve                       # http://127.0.0.1:8787
```

It has two players, so the UI asks who's playing; `mock-library --one-player` makes only the first. `data/` is in `.gitignore`.

A second copy of the desktop app beside an installed one, such as a scratch build on Windows, needs its own `UPNEXT_DATA_DIR` and `UPNEXT_CACHE_DIR`. Set them in the environment, not in a `.env`.

### The UI with hot reload

Run the server as above, then in another terminal:

```sh
cd web && npm run dev
```

Vite serves the UI on its own port and proxies `/api` to `127.0.0.1:8787` (`web/vite.config.ts`). A debug build (`cargo run`) serves `web/dist` from disk, so `npm run build` shows up without restarting the server.

### The AI without spending anything

Picks and the other AI features call a model. For development you don't need one:

- `upnext prompt "a short chill game"` prints the exact prompt a pick would send, without sending it.
- Point UpNext at any OpenAI-compatible server: `UPNEXT_LLM=openai UPNEXT_OPENAI_BASE_URL=http://127.0.0.1:11434/v1 UPNEXT_OPENAI_MODEL=<model>` for Ollama, or the fake model the end-to-end test uses (`node web/e2e/fake-llm.mjs`, which answers on port 8799).
- To make real calls, use your own Claude Code CLI, Codex CLI, Copilot CLI or API key, as [AI](docs/ai.md) describes.

### The container image

`docker build -t upnext .` at the repository root builds it; use `image: upnext` in the compose file to run it. `docker buildx build --platform linux/arm64 -t upnext:arm64 --load .` builds the arm64 image on an amd64 machine, and `contrib/docker/smoke-test.sh upnext` checks an image the way CI does.

## Conventions

- Refactors change structure only. A change in behaviour goes in its own commit, with a test.
- Never commit a database, a token, a `.env`, or fixtures cut from a real library. Fixtures in `tests/fixtures/` are made up or trimmed public responses.
- Keep the README, the guides in `docs/` and the API guide (`docs/API.md` and its pages in `docs/api/`) true to the code in the same change.
- Write docs plainly: say what it does and what to type, and give a reason only when ignoring it breaks something. Write sentences in plain order, without reveals such as "The one cause is …". Leave out internals a reader can't act on, and history. Each fact has one home page; other pages link to it. Pages stay under 900 words.
- A changelog entry is one line: what a user notices, without routes or constants.

## Pull requests

Open an issue first for anything large, so we can agree on the shape before you write it. Keep a pull request to one change, fill in the template, and make sure [the checks](docs/contributing/checks.md) pass. By contributing, you agree that your contribution is licensed as the project is, under MIT OR Apache-2.0.
