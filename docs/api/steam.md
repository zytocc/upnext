# Steam

The Steam Web API key (`steam_api_key`, write-only) and the SteamID (`steam_id`, 17 digits) are [settings](settings.md). Without a SteamID set, UpNext reads the one most recently signed in to the local Steam client (`steam_id_source` is then `steam client`). A SteamID belongs to whoever linked it: one another player's Steam connection has already is a `400`, naming whose it is ([Accounts](accounts.md)).

## POST /api/steam/test

Calls Steam with the key and SteamID in use. `{"ok": true, "persona_name": "player", "avatar": "...", "game_count": 812}`, or `400` `{"error": "..."}` (a bad key, a private profile and so on). The name and picture are kept and shown as `steam_persona` and `steam_avatar`.

## Sign in with Steam

Signing in with Steam (OpenID) gives the SteamID, never the API key, which Settings still asks for.

1. The UI sends the browser to `/api/steam/login`, which redirects to Steam's OpenID page. The sign-in has to come back to the browser that started it. It needs a `full` token, a paired browser or this computer, since it saves the SteamID.
2. Steam sends the browser back to `/api/steam/callback`, which needs no token. The server checks the answer with Steam, stores the SteamID, and sends the browser to `/settings?steam=ok`, or `/settings?steam=error&reason=...`. A SteamID another player's connection has is refused there too.
3. `/api/steam/login?then=setup` comes back to `/setup?steam=ok` (or `…error…`) instead, and `then=player` to `/setup/player?steam=…`. Any other `then` is ignored.

Where Steam sends the browser back to is in [public_url and proxies](proxies.md#public_url).

## What a sync reads

With the key, a sync reads the owned games and playtime from the Web API. Without it, it reads the local Steam client's files, or both with a sync's `steam_local` option. For an account other than the one signed in to Steam last, the client's files list only the games it played or installed, and the rest aren't taken as removed.

`steam_include_hidden` (`false`): games hidden in the Steam client are left out. With the Web API alone and no local client, nothing is known to be hidden. `true` syncs them like any other game.

Adding UpNext to Steam as a shortcut for big screen is [Add UpNext to Steam](steam-shortcut.md).
