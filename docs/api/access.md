# Who can use the API

## This computer

A request from this computer needs nothing. That means a loopback peer address (`127.0.0.1`, `::1`), a loopback `Host` (`localhost`, `127.0.0.1`, `[::1]`), none of a proxy's headers (`Forwarded`, `X-Forwarded-For`, `X-Forwarded-Host`, `X-Forwarded-Proto`, `X-Real-IP`, `Via`) and HTTP/1.1 or newer. That is the UI and the desktop apps.

## Everyone else needs a token

Every other request needs `Authorization: Bearer upnext_…`, or a paired browser's `upnext_device_<port>` cookie ([Tokens and pairing](pairing.md)). That covers another device, a script on another machine, a proxy in front of UpNext and a container's host. Without one, an API request is `401` with `WWW-Authenticate: Bearer realm="UpNext"`.

A token that isn't valid is `401` even from this computer. For a cookie the `401` also clears it, so the UI asks for a new code. The cookie is `upnext_device_<port>`, with the port the server listens on (8787 in a container). The older `upnext_device` is still read.

These need nothing from anyone who can reach the server: the UI's own files, `GET /api/health`, `POST /api/pair`, asking to be allowed and collecting the answer (`POST /api/pair/request` and `GET /api/pair/request/{id}`), `GET /api/steam/callback`, and signing in with an identity provider (`GET /api/oidc`, `GET /api/oidc/login`, `GET /api/oidc/callback`; [Identity provider](oidc.md)). The Steam callback takes only a one-time state from the browser it was given to ([Steam](steam.md#sign-in-with-steam)).

## In the container

In the container image (`UPNEXT_CONTAINER=1`) no request is this computer's, even the host's own browser. Until a browser is paired or a token made, the server prints an hour's pairing code to its log at start. After that, `docker exec upnext upnext pair` makes a five-minute code and `docker exec upnext upnext token create` a token. `upnext pair` asks the running server for a code over loopback, so it works on any headless install.

## Plain HTTP only from a private address

Over `http://`, a token, a cookie, a pairing code or an identity provider's sign-in is taken only from loopback, `10/8`, `172.16/12`, `192.168/16`, link-local, `100.64/10` (Tailscale) and IPv6 unique-local or link-local addresses. From anywhere else it's `403`, saying to use HTTPS. With `tls_cert` and `tls_key` set, HTTPS works on the same port as plain HTTP.

## Scopes

- **`read`**: `GET` and `HEAD` only, and not `GET /api/export`, the token list, `GET /api/steam/login`, `GET /api/gog/bookmark` or `POST /api/seed/offer`.
- **`full`**: everything the UI can do, picks and settings included.

Anything a scope doesn't allow is `403`. Each operation's `security` in the OpenAPI document says which it needs. Some routes answer only on the computer UpNext runs on, whatever the token ([Only on this computer](local-only.md)).

## Hosts and origins

- **Hosts.** An API request must be addressed to a loopback name or to `public_url` ([Proxies](proxies.md)); anything else is `403`. When `public_url` is unset and the server listens beyond loopback, a request with a valid token may name any host.
- **Origins.** A `POST`, `PUT` or `DELETE` with an `Origin` must come from UpNext's own page. A request the browser marks `Sec-Fetch-Site: cross-site` or `same-site`, or that carries another `Origin` (`null` included), is `403` "cross-site request refused", a `GET` too. The ways back from Steam and the identity provider are the exceptions. A write with the paired-browser cookie, or through a proxy that does the login, needs an `Origin` or `Sec-Fetch-Site: same-origin`. Scripts send no `Origin`.

## Limits

Per token (this computer has none; through a proxy that does the login, per client address as the proxy gives it): about 10 requests a second, in bursts of 20, and beyond that `429` with `Retry-After` in seconds. One pick (`POST /api/picks`, `…/more`, `…/too`) and one Help me decide (`POST /api/versus/decide`) run at a time. A second while the first runs is `429` with `Retry-After: 5`.
