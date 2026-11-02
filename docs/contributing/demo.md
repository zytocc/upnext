# The online demo

[demo.upnext.games](https://demo.upnext.games) is the web UI with no server behind it. It shows the mock library's first player (`upnext mock-library --one-player`) and answers the UI's API calls inside the browser tab, from a recording.

## What a visitor gets

- Every page reads from `web/src/demo/recorded.json`. Its times move forward to today, so "last played 3 days ago" stays 3 days ago.
- A visitor's changes stay in memory in that tab: statuses, ratings, Play later, hiding, routines, the theme, the tour and lineups with their actions. A reload starts over from the recording.
- Lineups are written ahead of time: one for each routine, each "Or try" suggestion and three typed requests. A typed request gets the lineup it shares the most words with. The dice draw from the games not started yet. No AI is asked.
- Calls that need a real server answer 403 with a message that says so: syncs and other jobs, store and AI sign-ins, sending a backlog verdict, backups, pairing, players and the taste profile's edits.
- Game art loads from Steam's image servers. Nothing else leaves the browser, and `web/demo/_headers` sets a Content-Security-Policy that allows nothing else. The build adds the hashes of `index.html`'s inline scripts to it.

## Files

| File | What it holds |
|---|---|
| `web/src/demo/server.ts` | The stand-in server: what each call answers, and the visitor's changes |
| `web/src/demo/install.ts` | Sends the UI's `/api/` calls to the stand-in |
| `web/src/demo/DemoBanner.tsx` | The line across the top |
| `web/src/demo/recorded.json` | The recording |
| `web/scripts/demo-record.mjs` | Makes the recording. `LINEUPS` and `REASONS` hold the written lineups |
| `web/scripts/demo-build.mjs` | Builds the demo into `web/dist-demo/` |
| `web/demo/` | Cloudflare Pages' `_headers` and `_redirects` for the demo |

Only a build with `--mode demo` loads `web/src/demo/`. The normal build leaves it out.

## Build and try it

```sh
cd web
npm ci
npm run build:demo
npx vite preview --outDir dist-demo
```

## Recording again

Record again after a change to the API, to the mock library or to the written lineups:

```sh
cargo build --locked --no-default-features    # or set UPNEXT_BIN to a binary
cd web
npm run demo:record
```

It starts the server on a throwaway copy of the mock library, on ports 8796 and 8795, and reads every page's calls. A stand-in model in the script picks the written lineups. Commit `src/demo/recorded.json` with the change.

`npm run demo:check` records again and compares the keys of every answer with the committed recording. CI's end-to-end job runs it, and it fails when a field was added or removed. Values aren't compared.

A new read in `web/src/api.ts` goes in `READS` or `GAME_READS` in `demo-record.mjs`. A new write that the demo should keep goes in `server.ts`. Any other write answers 403.

## Hosting

The demo is a Cloudflare Pages project of its own, built from `main`:

| Setting | Value |
|---|---|
| Build command | `cd web && npm ci && npm run build:demo` |
| Build output directory | `web/dist-demo` |
| Environment variable | `NODE_VERSION` = `24` |
