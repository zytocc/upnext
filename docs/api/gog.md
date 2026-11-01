# GOG

After a one-time sign-in, a sync reads the owned games and each game's playtime from GOG's own API (source `gog`). Playtime covers games played on Windows through GOG Galaxy too. Heroic stays the launcher for those games where it's installed. Otherwise `run_url` is `goggalaxy://openGameView/gog_{id}`. The `gog` profile source counts as configured with Heroic or a sign-in, with a `note` such as `"Heroic + GOG account"`. The sign-in is kept on the player's GOG connection.

## GET /api/gog

The `GogStatus`: `{"connected": false, "user_id": null, "username": null, "auth_url": "https://auth.gog.com/auth?client_id=...", "login_pending": false, "login_error": null}`. `user_id` and `username` are set when connected. `login_pending` is true while a sign-in window is open, and `login_error` says why the last window failed.

## POST /api/gog/login

`{"code": "<the code from the redirect, or the whole redirect address>"}` answers with the `GogStatus`, connected. `400` `{"error": …}` when the code is wrong or has expired. This is the way to paste GOG's landing-page address by hand.

## DELETE /api/gog

Deletes the stored sign-in and answers with the `GogStatus`.

## The sign-in window

GOG's sign-in sends the browser only to its own `https://embed.gog.com/on_login_success` page, so a browser tab can't hand the code to UpNext. `POST /api/gog/window` opens a sign-in window on the computer running UpNext (`upnext gog-window`). When the window reaches GOG's landing page it reports the code, the server trades it for a sign-in, and the window closes. The UI polls `GET /api/gog` while `login_pending` is true.

It answers with the `GogStatus`. `409`, saying to sign in with the link and paste instead, when a window is open already, the server was built without the app window, or the computer has no desktop to open it on (`environment.display` in [`GET /api/setup`](setup-wizard.md)). Only on this computer.

## The "Send to UpNext" bookmark

Where no window can open (a container, or a browser on another device), the wizard and Settings offer a bookmark, "Send to UpNext", to drag to the bookmarks bar once. It holds UpNext's address and this install's bookmark key.

1. `GET /api/gog/bookmark` gives `{"key": "<64 hex>"}`. It needs a `full` token or a paired browser. A copy without keys leaves it out, and a restore or an import keeps this install's.
2. `POST /api/gog/browser` gives `{"state": "<32 hex>", "auth_url": "https://auth.gog.com/auth?…", "expires_at": 1759600000}`: a one-time state for the device that asked, good for ten minutes. The page opens `auth_url`.
3. On GOG's success page, the bookmark sends the tab to UpNext's `/gog/callback#key=…&code=…`.
4. That page sends `POST /api/gog/callback` with `{"state": "…", "code": "…", "key": "…"}` (the code, or GOG's whole address), which answers with the `GogStatus`, connected.

The callback is `403` without this install's key, or when the state wasn't issued to this device, was used already or expired. A `403` for a missing key leaves the state waiting for the real one. It is `400` when GOG turns the code down. The first try that has the key uses the state up, whatever its outcome.
