# Who can reach UpNext

Other devices are kept out until you let them in. [The HTTP API's access rules](../api/access.md) have the precise behaviour.

## Tokens and pairing

- **Allow other devices** (Settings › Devices and API) makes the server listen on your network as well. It's off by default, and the container image turns it on.
- Every request that isn't this computer's needs `Authorization: Bearer upnext_…`, a token made in Settings or with `upnext token create`, or a paired browser's cookie. A token is `read` (looks, changes nothing) or `full`. Only a token's SHA-256 is stored, and it's shown once.
- **Pairing** takes a six-digit code that lasts five minutes and works once. Five wrong tries end a waiting code, and anyone who can reach the server can do that. The browser gets an `HttpOnly`, `SameSite=Strict` cookie (`Secure` over HTTPS), listed with the tokens and revoked the same way. The cookie's name carries the port the server listens on, so two UpNexts on different ports of one host don't clear each other's. A container listens on 8787 inside, so it and another UpNext on 8787 on the same host share a name and unpair each other.
- **Asking to be allowed** needs no code. A device on a private address asks, and someone at the computer UpNext runs on allows it after checking that the four-digit number matches the device's. A request lasts two minutes, and only this computer can answer it. Three requests wait at most, one per address (a /64 counts as one), and an address denied three times waits ten minutes. Allow is disabled for a second whenever another request is shown.
- **Through a proxy**, a device may ask only when the proxy is listed in trusted proxies ([Sign-in and proxies](sign-in.md)) and names a client on a private network, such as a tailnet device through `tailscale serve`. Through a proxy that isn't listed, a device pairs with a code.
- With other devices allowed, UpNext announces its name, an install id, its port and its version on the local network (mDNS). The announcement can be turned off.

## Plain HTTP only on private networks

Over `http://`, a token, cookie or pairing code is accepted only from loopback, RFC 1918, link-local, Tailscale's `100.64.0.0/10` and IPv6 unique-local or link-local addresses. From anywhere else UpNext answers `403` and says to use HTTPS: its built-in HTTPS (your certificate and key) or a proxy in front.

## Phones and other computers

The Android app and the app window on another computer pair the same way. [Phones and other computers](apps.md) says how each guards its token.

## Only on this computer

Whatever the token, and also through a proxy on this computer, these answer only on the computer UpNext runs on:

- making tokens and pairing codes, reading a waiting code, and answering a device that asks to be allowed;
- the other computers the app window uses;
- downloading, restoring, importing and deleting backups, and how many backups are kept;
- clearing things before the setup wizard starts again;
- opening the GOG login window;
- settings that name files on the server (the Galaxy database, the certificate and key) and a sync's own Galaxy path.

**One exception: the Galaxy upload.** A full token, a paired browser's included, may upload a copy of GOG Galaxy's `galaxy-2.0.db` (`POST /api/galaxy/upload`), so setup can take it from the computer Galaxy is on. The file must open as SQLite with Galaxy's tables and is at most 1 GB, and it always replaces the one file `galaxy-2.0.db` beside UpNext's database. UpNext reads it in SQLite's defensive mode. A read token can't upload.

## Limits

Each token gets about 10 requests a second, and one pick, one Help me decide, one AI test and one look at the Codex CLI at a time. This computer has no limit.
