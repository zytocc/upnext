# First-run setup

The UI's `/setup` page is the first-run wizard. Every page goes there while the library is empty and setup is neither done nor skipped. Settings › Setup restarts it. The steps reuse `PUT /api/settings`, `POST /api/jobs` and `PUT /api/jobs/scheduled/{task}`. What it reads is on [What the wizard reads](setup-wizard.md).

Each player also has a short setup of their own at `/setup/player`. `GET /api/setup/state`, `GET /api/setup`, `PUT /api/setup` and `POST /api/setup/reset` answer for the request's player and take the `UpNext-Player` header.

## GET /api/setup/state

```json
{"needed": false, "step": "done", "setup": "done", "tour": "open", "tour_offer": true,
 "player_needed": false, "player_step": null, "player_setup": "done"}
```

- `needed`: the wizard shows (the library has no titles and `setup` is `open`). The UI checks it before any other page.
- `step`: the step the wizard last reached, or null: `welcome`, `where`, `steam`, `stores`, `more_stores`, `extras`, `ai`, `ai_jobs`, `sync`, `details`, `taste`, `players` or `done`. The `players` step offers [`you.suggested`](setup-wizard.md) as the first player's name.
- `setup` and `tour`: `open`, `done` or `skipped`. `tour_offer` is true once setup no longer shows, the library has games and the tour is still `open`.
- `player_needed`: the UI sends the request's player to their own setup before any other page. True with two or more players, once the wizard no longer shows, while their `player_setup` is `open`. The first player's own setup is the wizard, so it starts `done`; anyone else's starts `open`. `player_step` is `you`, `games`, `ai`, `taste` or `done`.
- While a default player with a PIN hasn't been chosen, `player_needed` and `tour_offer` are false.

## PUT /api/setup

Any of `{"step": "sync", "setup": "skipped", "tour": "done", "player_step": "games", "player_setup": "done"}`. Answers with the state. `400` for an unknown step. `tour`, `player_step` and `player_setup` are the player's own. `step` and `setup` are the household's, which a token bound to one player can't change (`403`).

## POST /api/setup/reset

Any of `{"accounts": true, "ai": true, "taste": true, "history": true, "library": true}`. Clears what's chosen before "Restart the setup wizard" opens the wizard. Answers `{"backup": "upnext-manual-20261009-181500.db", "from_env": ["steam_api_key"]}`: a backup is taken first, and `from_env` lists cleared settings that still have a value from the environment or `.env`.

- `accounts`: the Steam, IGDB, RAWG and Galaxy settings, and the GOG and Epic sign-ins.
- `ai`: the AI backend choices and their model, key, address and effort settings.
- `taste`: profile statements, routines and the play diary.
- `history`: statuses, ratings, reasons, Play later, Second Chance answers, your verdicts and every lineup.
- `library`: every game and title, the wishlist and other players' verdicts; `taste` and `history` go with it. Settings, tokens, jobs and the record of AI calls stay.

With two or more players, `accounts`, `taste` and `history` clear only the player's own. The API keys are everyone's and stay, and "can be played endlessly" is a fact about the game and stays. `library` is everyone's.

Only on this computer (`403` from another device, so never in a container), never while a job runs (`409`), and `400` when nothing is chosen.

## POST /api/setup/background

Any of `{"sync": "if_changed", "details": "wait", "seed": true}`. Answers `{"sync_waiting": false, "details_waiting": false, "seed": {"wanted": true, "state": "waiting", "written": null, "error": null}}`.

The wizard runs the first sync and Fetch details in the background while the person carries on. It's the household's, so it takes no `UpNext-Player` header, and it needs a `full` token (`403` for a read token).

- `sync`: `if_changed` syncs unless the last sync read the same accounts and folders and is running, waiting or brought games. `again` syncs anyway. While a job runs the sync waits (`sync_waiting`).
- `details`: `wait` holds the schedules on a first run only, until details are asked for, setup ends, or an hour passes, so the data-source keys of the next step are in first. `after_sync` fetches details once the sync ends (`details_waiting`). Nothing starts when every game has its details.
- `seed`: [shared data](seed.md) is applied when the wizard's first sync ends with games, unless `seed` is `false`. `seed.state` is `idle`, `waiting`, `applying`, `applied` (with `written`), `failed` (with `error`) or `skipped`. `skipped` means there was nothing to do or nothing to fetch, which isn't an error.

What was asked lives in the server's memory, and a restart forgets it.

## POST /api/galaxy/upload

The bytes of a `galaxy-2.0.db` (`Content-Type: application/octet-stream`), for a Galaxy database copied from another computer. Up to 1 GB (`413` over that). The file is checked, saved next to UpNext's database, and `galaxy_db` is set to it. Answers `{"path": "/home/you/.local/share/upnext/galaxy-2.0.db", "games": 212}`.

- `400` with the reason for an empty file or one that isn't a Galaxy database; nothing is kept.
- `409` while another upload is running, or while a sync holds the kept file (try again once it's done).
- `408` for an upload that stalls or takes over 15 minutes.
- `500` naming the folder for any other failure to save.

A `full` token can upload. Typing a `galaxy_db` path is only for this computer.

## From another device or a container

`GET /api/setup` takes any token; the writes take a `full` one. `environment.client_local` is false. The GOG sign-in window, file paths in `PUT /api/settings` and importing a database stay on the server's computer. Steam's sign-in comes back through `GET /api/steam/callback` ([Steam](steam.md#sign-in-with-steam)).
