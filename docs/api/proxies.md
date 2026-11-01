# public_url and proxies

## public_url

`public_url` (setting, or `UPNEXT_PUBLIC_URL`) is the address other devices use, such as `http://upnext.lan:8787` or `https://upnext.example.org`. It's saved as `scheme://host[:port]`, lowercase, without a default port or a trailing `/`; a path is refused. `GET /api/settings` gives it with `public_url_source`.

Steam sign-in (`GET /api/steam/login`) and a pairing's address and QR code use it. Without it, Steam sign-in returns to the host the request named.

## A proxy in front

A proxy in front (Caddy, Traefik, `tailscale serve`) must pass the `Host` header through, which those do by default. For nginx add `proxy_set_header Host $host;` and `proxy_http_version 1.1;`.

A request from loopback that carries a proxy's headers, or that comes over HTTP/1.0 (nginx's default to its upstream), is not treated as this computer's. Its visitors get `401`, saying the request came through a proxy. When the proxy serves HTTPS and `public_url` is its `https://` address, a pairing's cookie is `Secure`. Why: [Sign-in and proxies](../security/sign-in.md).

## Trusted proxies

`trusted_proxies` (setting, or `UPNEXT_TRUSTED_PROXIES`) lists the addresses and ranges of proxies whose forwarding headers UpNext believes, comma- or space-separated, IPv6 too: `127.0.0.1`, `10.0.0.2`, `172.16.0.0/12`. It's empty by default. Ranges wider than a `/8` (IPv6 `/32`) are refused. Only a request from a listed address that carries a proxy's headers is affected. From a listed address:

- **The client's address** is the right-most hop in `Forwarded`'s `for` or in `X-Forwarded-For` that isn't itself listed. A hop that can't be read counts as a public client. The client's address is used for the log line, the limits, and whether plain HTTP is allowed: a public client over plain HTTP is refused a token.
- **HTTPS and the host** come from the same hop: `Forwarded`'s `proto` and `host`, or `X-Forwarded-Proto` and `X-Forwarded-Host`. The host check and `public_url` then come out right behind a proxy that rewrites `Host`.
- **Both kinds of header** in one request (`Forwarded` and `X-Forwarded-*`) must agree, else `400`. A proxy should replace or append to the kind it sets.
- **No header names a player.** `Remote-User`, `X-Forwarded-User`, `Tailscale-User-Login` and the like are ignored. Identity comes from the [identity provider's sign-in](oidc.md) alone.

From any other address, those headers mean what they always do.

## The proxy does the login

`trusted_proxy_login` (setting, or `UPNEXT_TRUSTED_PROXY_LOGIN`; off by default): with it on, a request from a listed proxy needs no token. It lands on "Who's playing?" like a browser on this computer, with PINs as usual, and the choice is kept in that browser's player cookie.

Such a request is not this computer: nothing on the [only on this computer](local-only.md) list, and nothing a `full` token can't do. It acts as an unbound `full` caller. It can choose any player with the `UpNext-Player` header (PINs as usual), list and revoke tokens and devices, and read the export.

- It must be addressed to `public_url` or, when the proxy rewrites `Host` to its upstream's, a loopback name. Otherwise `403`.
- A write must carry an `Origin` or `Sec-Fetch-Site: same-origin`.

Use it only with a proxy that really makes people log in (Authelia, Authentik, oauth2-proxy) or a tailnet (`tailscale serve`), with UpNext reachable only through it. Both settings are set only on this computer or in the environment, and `GET /api/settings` gives them with their `_source`.
