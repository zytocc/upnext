# Tokens and pairing

## Tokens

Tokens are `upnext_` and 40 letters and digits. The token is in the answer that makes it and nowhere else. Backups keep them, and a download or export without keys leaves them out. A restore or an import keeps this install's tokens in place of the copy's, so a token revoked since the backup stays revoked.

### GET /api/tokens

Needs `full` from another device.

```json
{ "tokens": [ { "id": 1, "name": "phone", "scope": "read", "kind": "token", "hint": "upnext_Ab12",
                "created_at": 1790000000, "last_used_at": 1790000300, "player_id": null } ],
  "current": null }
```

- `kind` is `token` (made by hand), `device` (a paired browser) or `oidc` (a browser signed in with the identity provider).
- `player_id` is the player a token or device is bound to and always acts for. When it's null, the request uses the `UpNext-Player` header or the default player.
- `current` is the id of the token the request came with (null from this computer). A token bound to a player gets only itself in `tokens`.

### POST /api/tokens

`{"name": "phone", "scope": "read", "player_id": 2}` makes one, on this computer only: `201` with the row above plus `"token": "upnext_…"`. `player_id` binds it to that player (left out or null: not bound; `400` for an unknown player), with `"pin"` when that player has one (`403` when it's missing or wrong, `429` while locked). A name is 1 to 60 characters and unique (`400` otherwise).

### DELETE /api/tokens/{id}

Revokes one at once: `204`, or `404`. A token bound to a player can revoke only itself (`403` for another).

From a terminal: `upnext token create --name phone --scope read` prints the token alone on stdout; `upnext token list`; `upnext token revoke <id or name>`.

## Pairing a browser with a code

Settings › Devices and API, "Add a device", on the computer UpNext runs on, shows a six-digit code for five minutes, with a QR code of the address. The new device types it and gets a device token of `full` scope as a cookie (`upnext_device_<port>`). The device is then in the token list (`kind: "device"`) and can be revoked like any token. A computer asking to join is [allowed](allowing.md) instead.

### GET /api/pairing

On this computer only. Answers the waiting code or `null`:

```json
{"code": "042917", "expires_at": 1790000300, "address": "http://192.168.0.42:8787", "qr_svg": "<svg…"}
```

`address` is `public_url`, else this computer's address on its network. It is null, with no QR code, while UpNext only listens on this computer. The QR code opens `address` with the code in its fragment (`http://192.168.0.42:8787/#pair=042917`), and the UI pairs with it.

### POST /api/pairing

A new code (`201`, the shape above), on this computer only; any earlier code ends. `?then=setup` makes the QR code open the setup wizard instead of the first page. `then` is `home` (the default) or `setup`; anything else is a `400`. `?player=<id>|shared` says whose device it will be ([Players](players.md)).

### GET /api/pairing/outcome?code=042917

On this computer only: what became of a code shown here. `{"outcome": "waiting", "device": null}` while it can still be typed; `{"outcome": "paired", "device": {"id": 7, "name": "Android phone"}}` once a device used it; `{"outcome": "ended", "device": null}` when it ran out, took five wrong tries, or was ended or replaced. Only the last code that paired a device is remembered, until UpNext restarts. A page waiting on its code asks this, because `GET /api/pairing` is `null` for a used code and an expired one alike.

### DELETE /api/pairing

Ends the code: `204`, on this computer only.

### POST /api/pair

`{"code": "042917", "name": "phone"}` → `201` `{"id": 7, "name": "phone"}` and the cookie. `name` is optional: when left out it's worked out from the browser ("Android phone"), with ` 2`, ` 3`… when taken. Needs no token, but not over plain HTTP from a public address (`403`). A wrong code is `401`, with the tries left. Five wrong tries end the code. Make a new one and pair again. `404` when no code is waiting. A code works once.

With `"wants": "token"`, for an app that keeps the token itself (the Android app), no cookie is set. The answer adds the token, the only time it's seen, and what the app needs to find the computer again: `{"id": 7, "name": "Pixel 9", "token": "upnext_…", "computer": "den", "computer_id": "4f2a9c01d3e85b67"}`. When `public_url` is set, `address` is added and the token is taken only at that name. `wants` is `cookie` (the default) or `token`.

Why these limits: [Who can reach UpNext](../security/devices.md).
