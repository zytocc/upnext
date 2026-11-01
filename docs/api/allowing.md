# Allowing a device with a tap

In place of typing a code, a device can ask, and someone at the computer allows it with a tap. A phone's browser asks from the pair page. The Android app and the app window on another computer ask after finding the computer on the network. The code stays as the fallback for a computer whose screen nobody sees.

## POST /api/pair/request

`{"name": "Pixel 9", "kind": "phone", "wants": "cookie"}`, all optional. `name` is as for `POST /api/pair`, `kind` is one of `phone`, `tablet`, `tv`, `computer`, and `wants` is `cookie` for a browser or WebView or `token` for an app that keeps the token itself. Answers `201`:

```json
{"id": "9c1e…", "secret": "…", "match_code": "4821", "expires_at": 1790000120, "computer": "den", "computer_id": "4f2a9c01d3e85b67"}
```

The device shows the four-digit `match_code`, and the computer shows it too, so nobody allows a neighbour's request by mistake.

Needs no token, but only from a private address (`403` from any other, HTTPS included): the client's own, or, through a proxy listed in `trusted_proxies` ([Proxies](proxies.md)), the client's as the proxy names it. Through a proxy that isn't listed it's `403` whatever the address. Pair with a code instead, or list the proxy.

A request waits two minutes. One waits per address, and asking again replaces it. Limits answer `429` with `Retry-After`: ten seconds between asks per address (IPv6 by /64), ten minutes after three denials, and 30 seconds while three requests already wait (two per IPv6 /64). Each request is logged with how to answer it without a screen: `upnext pair approve 4821`, or `docker exec upnext upnext pair approve 4821` in the container.

## GET /api/pair/request/{id}

With the secret in `UpNext-Pair-Secret`. Answers `{"status": "waiting"}`, `{"status": "denied", …}` or `{"status": "allowed", "id": 9, "name": "Pixel 9", "token": null, "address": null}`. Allowed, a request that wanted a cookie gets the `upnext_device_<port>` cookie, as `POST /api/pair` gives it. One that wanted a token gets it in `token`, the only time it's seen. `address` is `public_url` when it's set, and the token is then taken only at that name. An answer is collected once, then the request is gone (`404`, as for an unknown id, a wrong secret or an expired request). Needs no token, and the same private-address rule applies.

## GET /api/pairing/requests

On this computer only. The requests waiting, oldest first, for the dialog every UpNext window and browser on the computer shows:

```json
{"requests": [{"id": "9c1e…", "name": "Pixel 9", "kind": "phone", "match_code": "4821", "from": "192.168.0.42", "via": null, "expires_at": 1790000120}], "players": []}
```

`from` is the address it asked from, the client's as a listed proxy names it, and `via` is that proxy's address, or null when it asked directly. `players` lists `{"id", "name"}` when there are two or more players, and is empty with one.

## POST /api/pairing/requests/{id}

On this computer only. `{"allow": true, "player_id": 2}` answers `204`. The device gets a device token of `full` scope when it collects, named as it asked. With `player_id` it's bound to that player, and needs `"pin"` when that player has one. Null or left out makes it the household's. `{"allow": false}` denies it. `404` when it isn't waiting any more, `400` for an unknown player.

## GET /api/pair/open

With `Authorization: Bearer upnext_…`: `303` to `/`, with that token as the `upnext_device_<port>` cookie. The app window on another computer opens this first, since a webview sends a header only with the first page it loads. On this computer, with no token, it only goes to `/`.
