# Backlog verdicts from others

Other players' backlog verdicts come from the service's daily snapshot ([VERDICTS.md](../VERDICTS.md)). The "Backlog verdicts" job downloads it whole and keeps it, so game pages and picks have it with no network. Nothing is sent to get it. A failed download changes nothing. Games are matched when the library is read, by Steam appid or the GOG id of a GOG copy, so a game synced later gets its verdicts without a new download.

They show only while `verdicts_receive` is on (off by default; on from the setup wizard). Turning it off hides them everywhere and keeps the snapshot.

## GET /api/verdicts/community

`{"on": true, "chosen": true, "service": true, "snapshot": CommunitySnapshot, "licence": "CC0-1.0"}`. `chosen` says someone turned it on or off; until then the game page offers it once. `service` says a `verdicts_url` is set. A **CommunitySnapshot** is `{"date", "source", "applied_at", "checked_at", "games", "verdicts", "matched"}`: the snapshot's day (null before the first), where it came from (the service's address or a file), when it was applied and last checked, its games and verdicts, and how many of its games are in this library.

## GET /api/games/{id}/verdicts

The game page's "Backlog verdicts" section: the status above, flattened, and `verdicts`, null when the game has fewer than 3 or they're off:

```json
{"n": 23, "headline": "Worth it on a backlog, say 18 of 23 players",
 "lines": ["Clicks within the first hour", "Fine in sessions under an hour", "You can save anywhere", "Fine to stop after the credits"],
 "counts_line": null,
 "outcomes": [{"outcome": "finished", "label": "Finished: usually 10 to 20 h", "count": 17},
              {"outcome": "endless", "label": "Plays it now and then: usually 10 to 20 h", "count": 6}],
 "texts": [{"id": "aaaaaaaaaaaaaaaaaaaaaaaaan", "outcome": "endless", "about": "No ending; I play it now and then · 20 to 40 hours · Worth it: Yes",
            "text": "…", "stop_note": null, "month": "2026-10"}]}
```

- `lines` are up to four one-line answers, each shown only when one answer has a clear majority from 5 verdicts. Under 5 verdicts `lines` is empty and `counts_line` gives the counts ("2 say it's worth it, 1 says no").
- `outcomes` are how far people got, most common first.
- `texts` are the newest 20 texts, each with its verdict id for [reporting](verdicts-sending.md#post-apiverdictsreports). `404` for a title that isn't there.

## On games and picks

- `GameView.backlog` is null unless they're on and the game has at least 3: `{"n", "headline", "short", "lines", "facets"}`. `short` is the headline for a table cell ("Worth it, 18 of 23"). `facets` are the clear majorities, such as `clicked:first_hour`, `sessions:short`, `limited_time:yes`, `stop_point:at_credits` and `outcome:finished`.
- `GET /api/games?backlog=respects_time` lists games most players said respect limited time, and `backlog=clicks_fast` those most said clicked in the first hour.
- A pick has `backlog`, a line for its card when the game has at least 5 verdicts: "Backlog players: clicks fast, fine in short sessions (23)".
- `verdicts_prompt` (on by default) lets the AI read the verdicts in picks, "It's too ___", Help me decide and Second Chance, below your own history. `upnext prompt --with-verdicts` and `--without-verdicts` show either way. The UI shows verdicts whenever receiving is on.

## Jobs and settings

- The `verdicts` task ("Backlog verdicts"), daily by default, is skipped while receiving is off or no service is set. `POST /api/jobs` with `{"task": "verdicts"}` runs it now.
- `verdicts_receive` (with `verdicts_receive_source`) and `verdicts_prompt` are set from anywhere. `verdicts_url` must be `https://`, or plain `http://` only to `localhost`, `127.0.0.1` or `[::1]`. Any other address is a `400`, and `UPNEXT_VERDICTS_URL` set to one stops UpNext starting.
- `upnext verdicts sync` runs the job. `upnext verdicts apply <file, folder or address>` puts a downloaded snapshot in place (a manifest, its summary file or a folder holding them) and turns receiving on. A snapshot no newer than the one kept is skipped with a warning unless `--force` is given.
