# The published snapshot

The service builds a snapshot once a day, under `verdicts_url`, in three files. Each is complete, not a change to the one before.

- **`verdicts.json`**, the manifest, under a stable name and written last. It is served with an `ETag`, so an unchanged day costs one small request.
- **`verdicts-summary-<date>.jsonl.gz`**: the games with at least 3 verdicts. This is what installs download, always whole and never a game at a time, so the host never learns which games anyone owns.
- **`verdicts-all-<date>.jsonl.gz`**: every published verdict, for anyone who wants the raw data. Installs don't download it.

The dated files are kept for six days.

Both data files are gzip'd JSON lines, and the first line is a header whose `sha256` is of everything after it, uncompressed. Months (`YYYY-MM`) are the finest time in any published line, and no line carries a key, its hash, a client version or anything that links two verdicts to one person.

**The manifest:**
```json
{ "upnext_verdicts": 1, "date": "2026-10-09", "vocab": 1, "games": 4, "verdicts": 37,
  "licence": "CC0-1.0", "notice": "UpNext backlog verdicts: … CC0 1.0 …",
  "summary": { "file": "verdicts-summary-2026-10-09.jsonl.gz", "bytes": 1072, "sha256": "…" },
  "all": { "file": "verdicts-all-2026-10-09.jsonl.gz", "bytes": 1055, "sha256": "…" } }
```
`games` is the summary's games and `verdicts` the full file's verdicts. `bytes` and `sha256` are of each file as downloaded, and the files sit beside the manifest.

**A header:**
```json
{ "upnext_verdicts": 1, "kind": "summary", "date": "2026-10-09", "vocab": 1, "count": 4, "verdicts": 35,
  "sha256": "…", "licence": "CC0-1.0", "notice": "…" }
```
`kind` is `summary` or `all`, `count` is the lines after the header, and `verdicts` is the verdicts those lines count (the sum of the games' `n` in a summary).

**A game in the summary:**
```json
{ "steam_appid": 292030, "gog_id": "1207664663", "n": 6,
  "counts": { "outcome": {"finished": 3, "shelved": 1, "stopped": 2}, "worth": {"yes": 3, "for_some": 2, "no": 1},
              "hours": {"10_20": 1, "80_plus": 1}, "clicked": {"later": 2, "never": 1},
              "limited_time": {"no": 3, "mostly": 2}, "factors": {"hard_to_resume": 5},
              "stopped_for": {"too_long": 3, "pacing": 3},
              "hours_by_outcome": {"finished": {"80_plus": 1}, "stopped": {"10_20": 1}} },
  "texts": [ { "id": "…", "outcome": "stopped", "worth": "no", "hours": "10_20",
               "stop_note": "once Velen is done", "month": "2026-08" } ] }
```
- `n` is the verdicts counted, at least 3. Every count is how many verdicts gave that answer, and answers nobody gave are left out. `outcome` and `worth` add up to `n`; no other question's counts add up to more than `n`.
- `hours_by_outcome` is the hours of the verdicts with each outcome.
- `texts` are up to 20 verdicts with a `text`, a `stop_note` or both, newest month first and then by id, so their order says nothing finer than the month. A text held or hidden by moderation isn't there.
- **Which verdicts are one game's:** those with the same `steam_appid`. A verdict with only a `gog_id` joins the game whose verdicts carry both that GOG id and an appid, and is a game of its own otherwise. `format::summarise` in `src/verdicts/format.rs` is the reference.

**A verdict in the full file** is the verdict as sent without `licence`, `consent` and `client`, with its `id` and `month`: `{"id": "…", "steam_appid": 1145360, "outcome": "finished", "worth": "yes", "hours": "10_20", …, "vocab": 1, "month": "2026-10"}`. Ids are 26 lowercase base32 characters.

## Reading it

UpNext reads every file strictly, as it reads its shared game data. A field the format doesn't list, a licence other than `CC0-1.0`, a format newer than its own, a wrong checksum or count, counts that don't add up, a month after the snapshot's date, a game twice, a game with fewer than 3 verdicts, or a text that fails a [text check](verdict.md#text-checks) refuses the whole file, and the snapshot it had stays in place. An answer from a newer vocabulary than the reader's is dropped rather than refused.

`upnext verdicts check <folder or file>` reads a snapshot from disk and says what it holds. With the full file beside the manifest, it also checks that the summary is what the full file sums up to.

With "Backlog verdicts from other players" on (`verdicts_receive`), an install runs the "Backlog verdicts" job once a day. It asks for the manifest, and when that names a newer day, downloads the summary and puts it in place of the one it kept; an older day never replaces a newer one. It sends no key and nothing else about the player.

The snapshot isn't signed. The summary's SHA-256 and size in the manifest catch a damaged download, and the HTTPS connection says it came from the service, so `verdicts_url` is `https://` (plain `http://` only to this computer). Games are matched to the library by Steam appid, then by the GOG id of a GOG copy.

`upnext verdicts sync` runs the job. `upnext verdicts apply <file, folder or address>` puts a downloaded snapshot in place, for an install that can't reach the service; one no newer than the kept one needs `--force`. [Backlog verdicts from others](../api/community-verdicts.md) has what the game page, the library and picks show of it.
