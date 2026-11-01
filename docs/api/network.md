# Other devices and computers on the network

## Listening for other devices

`allow_other_devices` (`false`) makes the server listen beyond loopback. `tls_cert` and `tls_key` (paths, or null; set only on this computer) add HTTPS on the same port. `PUT /api/settings` applies a change to any of them before it answers, and open connections carry on. `GET /api/settings` gives what's in effect as `listen`:

```json
"listen": {"host": "127.0.0.1", "port": 8787, "other_devices": false, "https": false, "error": null,
           "forced": false, "container": false, "this_computer": true}
```

- `error` is set when the last change didn't take (a port in use, a certificate that doesn't load).
- `forced` is true when `UPNEXT_HOST` is beyond loopback, as in a container. It wins over the switch.
- `container` means no browser is this computer, so Settings shows `docker exec upnext upnext pair` and `… token create` instead.
- `this_computer` says the request came from the computer UpNext runs on, where Settings offers tokens and pairing codes.

"Allow other devices" listens on `0.0.0.0`, IPv4 only. Set `host` (`UPNEXT_HOST`) to `::` to take IPv6 as well.

## The announcement

While other devices may use UpNext and `announce` is on (the default), the server announces itself over mDNS as a `_upnext._tcp.local.` service on its port. The TXT values are `v`, `id` (the install id), `name`, `tls` and `ver`. Nothing about the library or its people is announced. A container on Docker's default bridge network announces only inside it; with `network_mode: host` it reaches the network.

`announce` (`true`) and `network_name` (1 to 63 bytes; unset, the host name) apply at once. `GET /api/settings` gives `network_name_shown`, the name in use.

## Using another computer's library

The app window can use another computer's UpNext instead of its own (`upnext app --connect`, or "Use the library on another computer" in the setup wizard). These routes are for this computer only, because they make the server call other computers and keep the tokens it gets. Over there the window is a paired device, so [pairing](allowing.md) applies.

### GET /api/network/computers

```json
{"this": {"id": "…", "name": "attic"},
 "found": [{"id": "4f2a9c01d3e85b67", "name": "den", "version": "0.1.0", "address": "http://192.168.0.42:8787"}],
 "saved": [{"id": "…", "name": "den", "address": "http://192.168.0.42:8787", "open": true}]}
```

`found` is the computers announcing UpNext now, without this one. `saved` is the ones this computer was allowed on before. `open` marks the one the window opens on.

### DELETE /api/network/computers/{id}

`204`: forget one and its token here. It stays in that computer's device list until revoked there.

### POST /api/network/connect

`{"address": "den.lan", "id": "4f2a9c01d3e85b67"}`. The address is a found one or one typed (`den.lan`, `192.168.0.42:8787`, `https://den.tailnet.ts.net`). Without a scheme it's `http://`, and without a port 8787. `id` is optional: the install id it was found with. This asks that computer for a token (`POST /api/pair/request` there) and answers `201`:

```json
{"status": "waiting", "match_code": "4821", "computer": "den", "address": "http://den.lan:8787", "id": null}
```

`400` for something that isn't an address. `409` when `id` is given and another UpNext answers there. `502` when nothing answers there as UpNext or it refused (its reason is given). Any earlier request from here ends.

### GET /api/network/connect

The same shape, after asking the other computer once: `waiting`, `allowed` (the token is saved, and `id` names it), `denied` or `expired`. `404` when no request is waiting. `502` when the other computer doesn't answer.

Once allowed, the page posts `connect:<id>` to the window, which opens on that computer from then on. `connect:here` (Settings › Devices and API on the other computer's page) goes back to this one. Before the token goes there, the window asks the address's `GET /api/health` whether its `id` is still that computer's. When another UpNext answers it opens `/connect?changed=<id>` here instead, and when nothing does, `/connect?unreachable=<id>`. See [Phones and other computers](../security/apps.md) for the trust this rests on.

### DELETE /api/network/connect

`204`: stop waiting.
