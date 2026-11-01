# What the wizard reads

## GET /api/setup

What every step shows, read fresh each time, with no AI or network call.

```json
{"environment": {"os": "linux", "mode": "serve", "container": false, "flatpak": null, "display": true, "gog_window": true, "client_local": true},
 "stores": {"libraries": [{"id": "steam", "label": "Steam", "blocked": null}, {"id": "gog", "label": "GOG account", "blocked": "not connected"}],
   "steam": {"path": "/home/you/.local/share/Steam", "missing": null, "error": null, "account": "76561197960287930",
             "account_source": "steam client", "persona": "Alex", "local_games": 812, "web_api": false,
             "local_accounts": [{"steam_id": "76561197960287930", "persona": "Alex", "player": "Alex"}],
             "other_users": []},
   "heroic": {"path": "/home/you/.config/heroic", "missing": null, "error": null, "games": {"amazon": 1, "epic": 5, "gog": 2},
              "owner": "Alex", "other_users": []},
   "gog": {"connected": false, "username": null}, "galaxy": {"path": null, "exists": false},
   "epic": {"connected": false, "display_name": null, "launcher_dir": null, "installed": null},
   "ea": {"installed": 0}, "ubisoft": {"installed": 0}, "xbox": {"installed": 0}},
 "keys": {"steam_api_key": false, "steam_id": true, "rawg": false, "igdb": false},
 "ai": {"backend": "claude-cli", "available": true, "error": null, "claude": "/home/you/.local/bin/claude", "claude_install": null, "claude_token": false},
 "library": {"titles": 99, "copies": 99, "multi_store": 0, "by_source": {"steam": 95, "gog": 2, "epic": 2, "amazon": 0},
   "enriched": 99, "unenriched": 0, "details_minutes": 0.0, "last_sync": 1790986302, "job": null},
 "setup": {"needed": false, "step": "done", "...": "..."},
 "you": {"id": 1, "name": "Player 1", "placeholder": true, "suggested": "Alex"},
 "background": {"sync_waiting": false, "details_waiting": false, "seed": {"wanted": true, "state": "applied", "written": 412, "error": null}}}
```

## environment

- `mode`: `app` (the desktop app), `service` (the systemd unit sets `UPNEXT_MODE=service`) or `serve`. `container` is true in the container image.
- `flatpak` is the Flatpak's app id (`io.github.zytocc.UpNext`) inside one, else null.
- `display`: the server's computer has a screen to open a window on. Always true inside `upnext app`, always false in a container.
- `gog_window` says whether the [GOG sign-in window](gog.md#the-sign-in-window) can open.
- `client_local`: the browser is on the server's computer, so paths and windows mean something to the reader.

## stores

- `libraries` is [`GET /api/sync/sources`](sources.md).
- `steam` and `heroic` give the folder found or set, else `missing` (`not found`, or `not mounted` in a container), and `error` when it was found but couldn't be read.
- Steam's `account` is the SteamID in use, with `account_source`. `web_api` says whether the key and SteamID are both set. `local_games` is what the client's caches list for that account before any sync ([Steam](steam.md#what-a-sync-reads)).
- `steam.local_accounts` lists every account signed in to this computer's Steam client, most recent first, with `player` (whose Steam connection has it, or null). For the first player, `account` is the client's most recent account; for anyone else it's their own SteamID, else the first local account nobody has, which their setup offers them.
- `heroic.owner` is the player whose Heroic connection reads the folder shown. Another player's Heroic is a folder of their own (`heroic_dir`).
- `other_users` lists Steam and Heroic folders found in other users' home folders, for a request from this computer and never in a container. A player's setup offers each one.

## The rest

- `keys`: whether each is set. The values never appear.
- `ai`: the chosen backend and whether it can run, with no call. `claude` is where the Claude CLI was found, or null. `claude_install` is a container's install of Claude Code (`UPNEXT_INSTALL_CLAUDE=1`): `{"installing": true, "error": null}` while it runs, an `error` when it failed, null otherwise. `claude_token` says whether `CLAUDE_CODE_OAUTH_TOKEN` is set for the server.
- `library`: the counts, `details_minutes` (an estimate for the titles without details), the last sync and the running job.
- `setup` is [`GET /api/setup/state`](setup.md#get-apisetupstate), and `background` is what [`POST /api/setup/background`](setup.md#post-apisetupbackground) answers.
- `you` is the request's player. `placeholder` is true while their name is one UpNext made up ("Player 1"). `suggested` is then a name to offer: the Steam account's `persona`, else the first player's Steam profile name, else the user name UpNext runs under. It is null when they have a name of their own, nothing fits, or another player has it.
