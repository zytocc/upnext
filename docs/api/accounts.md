# Store accounts and AI accounts

Every route here acts for the request's player, so each is `403` while nobody has been chosen ([Players](players.md)) and takes the `UpNext-Player` header. Only the owner changes a connection or an account. None returns a sign-in or a key's value.

## Store accounts

- A **StoreConnection** is a library a player linked, shown only to its owner. `kind` is `steam`, `gog`, `epic`, `heroic` or `galaxy`; `settings` holds text values; `signed_in` says whether it holds a GOG or Epic sign-in; `last_sync` and `count` are null before the first sync.
- A **StoreAccount** is one store's library read through a connection, as the asker sees it. `sharing` is `everyone`, `chosen` or `owner`. `games` counts its copies, not counting removed ones. `uses` says the asker sees its games (on until they turn it off), and `plays_on` that its playtime counts for them. `players` is for its owner only, one entry per other player, `{"player_id": 2, "name": "Sam", "offered": false, "uses": true, "plays_on": false}`, otherwise `[]`.

- `GET /api/accounts` answers `{"connections": [StoreConnection], "accounts": [StoreAccount]}`: this player's connections, and every store account they own or are offered, theirs first.
- `POST /api/connections` `{"kind": "steam", "settings": {...}}` links a library for this player and answers `201`; the next sync fills it. `settings` holds only its kind's keys: `steam_id` (a SteamID64) and `steam_include_hidden` for Steam, `heroic_dir` for Heroic, `galaxy_db` and `galaxy_skip` for Galaxy, `epic_manifests` for Epic, none for GOG. `400` for an unknown kind, a key its kind doesn't have, a bad value or a `steam_id` another player's connection has. `403` for `galaxy_db`, `heroic_dir` or `epic_manifests` from another device, since they name a folder or file on this computer.
- `PUT /api/connections/{id}` `{"settings": {...}}` answers with the connection. A key set to `null` or `""` is cleared and the others are kept. Errors as for `POST`, and `403` for a connection that isn't theirs.
- `DELETE /api/connections/{id}` answers `204`, removing the connection with its accounts and their games. Marks and history on those games stay, so linking it again brings them back.
- `PUT /api/accounts/{id}` `{"name", "sharing", "players": [{"player_id", "offered", "plays_on"}]}`, each optional, answers with the account. The name is 1 to 60 characters. `offered` is whether `chosen` sharing offers the account to that player. Hours count for a player only from their own accounts unless `plays_on` ("also plays on it") is set for them. `400` for a bad name or `sharing`, or a `player_id` that's the owner's or no player's; `403` for an account that isn't theirs.
- `PUT /api/accounts/{id}/use` `{"uses": false}` is a player's own switch for an account offered to them. Off takes its games out of their library and keeps their marks and history. `400` for their own account; `403` for one not offered to them.

A sync reads every connection. `upnext sync --account <id>`, or a sync job's `connection` option, reads one. The GOG and Epic sign-ins (`/api/gog/…`, `/api/epic/…`) are the requesting player's own.

## AI accounts

An **AiAccount** is one backend with its models and keys. `backend` is one of `llm_backends`, or null for the environment's or the built-in one. `settings` holds the models, address, effort and thinking settings, and `keys_set` the names of the keys it holds. `sharing` is `everyone`, `chosen` or `owner`, and `offered_to` lists player ids for its owner with `chosen` sharing, otherwise `[]`. `in_use` says the asker's picks run on it, `library` that the library's own AI work does, and `this_computer` marks the first account, which the environment's AI settings and this computer's signed-in Claude, Codex and Copilot CLIs belong to.

- `GET /api/ai-accounts` lists every AI account this player owns or is offered, theirs first.
- `POST /api/ai-accounts` `{"backend", "settings", "sharing", "offered_to", "library"}` answers `201` with the account. `settings` are AI settings and keys as `PUT /api/settings` takes them. `sharing` is `everyone` unless `offered_to` is sent without it, which makes it `chosen`. A player with no AI account starts using it. `400` for an unknown backend, a key that isn't an AI setting, a bad value or `sharing`, or an `offered_to` id that's the owner's or no player's. `403` for `"library": true` from a token bound to a player.
- `PUT /api/ai-accounts/{id}` takes the same body, each field optional (null or `""` clears a setting). `"library": true` runs the library's own AI work (summaries, series guides, hidden gems) on it, and `false` changes nothing; a bound token can't send `library` (`403`). From another device, a new `openai_base_url` clears the account's `openai_api_key` unless the same request sends one. `403` for an account that isn't theirs.
- `DELETE /api/ai-accounts/{id}` answers `204`. Whoever used it gets picks without AI until they choose another, and the library's AI work stops until one is chosen for it.
- `PUT /api/ai-accounts/{id}/use` (no body) makes it the AI this player's picks, Help me decide and Second Chance run on. `403` for one that isn't theirs and isn't offered to them.

A Claude Code CLI account other than this computer's needs its own token (`claude_oauth_token`, from `claude setup-token`). A Codex CLI account signs in under its own folder (`CODEX_HOME=<data folder>/codex/<id> codex login`) and a Copilot CLI account the same way (`COPILOT_HOME=<data folder>/copilot/<id> copilot login`). A Copilot account never uses this computer's own GitHub sign-in or `GH_TOKEN`.

`GET /api/ai/plan` reads the request player's AI account ([Prices and plans](ai-prices.md#get-apiaiplan)).
