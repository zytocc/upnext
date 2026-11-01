# Checks

CI (`.github/workflows/ci.yml`) runs all of these. Run them before you push.

```sh
cargo fmt --check
cargo clippy --all-targets --locked -- -D warnings
cargo clippy --all-targets --locked --no-default-features -- -D warnings
cargo test --locked
cargo deny --locked check                # licences, advisories, bans, sources (deny.toml)

cd web
npm run licenses                         # every npm package's licence is on the allowed list
npm run gen:api && git diff --exit-code src/generated/api.ts
npx tsc -b
npm run lint                             # oxlint
npm test                                 # Vitest
npx vite build
node scripts/demo-build.mjs              # the online demo

cd ../services/verdicts                  # the backlog verdicts service
npm ci
npm run licenses && npm run typecheck && npm run lint && npm test && npm run build
```

`cargo deny` comes from `cargo install --locked cargo-deny`. The default build links WebKitGTK and GTK 3 (`libwebkit2gtk-4.1-dev libgtk-3-dev pkg-config` on Debian and Ubuntu); `--no-default-features` builds the server alone.

## Third-party notices

After a change to `Cargo.lock`, `web/package-lock.json`, `android/gradle/libs.versions.toml` or `web/src/lib/credits.ts`, regenerate the third-party notices and commit `THIRD_PARTY_NOTICES.txt` and the Android app's copy with the change. The `notices` workflow fails until they match.

```sh
cargo install --locked cargo-about --version 0.9.2 --features cli   # once
cargo fetch --locked && (cd web && npm ci)
node contrib/notices/generate.mjs           # --check only compares
```

## End-to-end test

`web/e2e/` drives the real server on the mock library, with the fake model, in Chromium:

```sh
cargo build --locked --no-default-features    # or set UPNEXT_BIN to a binary
cd web
npx vite build
npx playwright install chromium               # once
npm run e2e
```

`web/e2e/serve.sh` starts the server in a throwaway folder with its own `HOME`, so nothing on your machine is read.

`npm run demo:check` in `web/` also needs the built server. It checks that [the online demo](demo.md)'s recording still matches the API.

`UPNEXT_README_SHOT=1 npx playwright test readme-shot` remakes the README's screenshot (`docs/images/pick.png`) and the big-screen one (`docs/images/big-screen.png`) on the mock library. It needs a machine that can reach Steam's image servers.

## Windows

The `windows` workflow builds and tests every Rust change on Windows. Check code that touches paths, processes or the registry there: the unit tests that run shell scripts as stand-in CLIs are `#[cfg(unix)]`.
