# Shared game data

UpNext's shared data is People say, Vibes and review tags for popular games, written once by UpNext contributors and published under CC0 at `https://data.upnext.games`. It is rebuilt now and then, not with each release of the app. Applying it gives matching titles their summaries with no review fetch and no AI call.

- Each title is matched by its Steam appid, then by the GOG id of any of its GOG copies.
- A title keeps its own summary when that was written with the same or a newer prompt version. An older own summary gives way.
- Applied data is kept beside the database, so games that join the library later get their summaries after each sync, before the summaries job asks the AI.
- The summaries job and the review fetch skip titles with shared data unless an enrich names them (`--titles`). Their player reviews aren't fetched until the game page asks ([`POST /api/games/{id}/reviews`](details.md#post-apigamesidreviews)).
- `GameView.seed_date` is the date (YYYY-MM-DD) of the shared data the title's summary, Vibes and review tags came from, or null.

The data is a gzipped JSON-lines file with a manifest (`upnext-seed.json`) beside it. A damaged file, a checksum or count that doesn't match, a newer format or another licence refuses the file before anything is written.

## GET /api/seed

```json
{"seeded": 812, "seed_date": "2026-10-01", "kept": "upnext-seed-2026-10-01.jsonl.gz", "applies": [SeedApply],
 "default_url": "https://data.upnext.games/upnext-seed.json"}
```

- `seeded`: titles whose summary came from shared data; `seed_date` the newest data's date among them.
- `kept`: the data file applied again after each sync, or null.
- `applies`: the latest five applies, newest first. `source` is the address, the uploaded file's name, or "the kept shared data, after a sync".
- `default_url`: the newest published data's manifest.

## POST /api/seed

`{"url": "https://…/upnext-seed.json"}` (a manifest or a data file), or nothing for `default_url`. Downloads it (up to 64 MB), checks it and applies it. Answers `SeedApplied`: `{"date", "entries", "matched", "written", "kept_own", "already"}`, where `already` counts titles that had this data's summary or a newer one's.

`400` for an address that isn't http(s), an answer that isn't 2xx, or a file UpNext won't apply, with why. `502` when the download fails. One download or upload is applied at a time. Applying replaces this install's own summaries written with an older prompt, so removing the data later leaves those titles without one until the summaries job writes them again.

## POST /api/seed/offer

`{"url": "https://…/upnext-seed.json"}`, or nothing for `default_url`. Reads the manifest without downloading the data and answers `SeedManifest` for the wizard's "Use UpNext's game data" option. `400` and `502` as above. It's a `POST` that changes nothing, and it needs a `full` token (or this computer). When the offer fails, the wizard leaves the option out.

## POST /api/seed/upload?name=upnext-seed-2026-10-01.jsonl.gz

The data file's bytes (`Content-Type: application/octet-stream`, up to 64 MB). Answers `SeedApplied`, or `400` with why.

## DELETE /api/seed

Removes the summaries that came from shared data and the kept copy, so this install's own AI writes them on the next summaries run. Answers `{"removed": 812}`. The record of applies stays.

`upnext seed apply [file or address]`, `upnext seed status` and `upnext seed remove` do the same from a terminal.
