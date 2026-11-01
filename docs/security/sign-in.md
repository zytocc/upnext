# Sign-in and proxies

## Signing in with an identity provider

Set an issuer, a client id and `public_url`, on this computer or in the environment, to turn it on. UpNext uses the authorization code flow with PKCE (S256). The state works once, lasts ten minutes and is tied to the browser by a cookie, and a nonce is checked.

UpNext checks the ID token itself:

- a signature by one of the keys the provider publishes (RSA, ECDSA or Ed25519; `none` and the client secret are refused);
- the issuer the provider's discovery names, which must be the issuer set;
- the audience and `azp`;
- the expiry, and `iat` and `nbf` with two minutes' leeway;
- the nonce.

Starting a sign-in needs no token, so it is limited. Three wait at most from one address (an IPv6 /64), and a fourth replaces that address's oldest. Thirty-two wait in all, after which a new one is refused. Known gap: about 11 addresses (or /64s), each holding three unfinished sign-ins, can keep anyone new from starting one for ten minutes.

A signed-in browser gets a device token bound to the account's player, with the same cookie and revoking as a paired phone, and signing out revokes it. An account is its linked player:

- **Link the next sign-in** (on this computer, with that player's PIN, for ten minutes) makes the next new account that signs in wait, unlinked and with no token. Someone at this computer sees its name, username, email and subject, enters the player's PIN again and confirms it is that player.
- With auto-provisioning on (the default), every account the provider signs in gets its own player. Players are never matched by name or email. If the provider has accounts that shouldn't use UpNext, set `oidc_allowed_groups` or turn auto-provisioning off.

The client secret is write-only and shows as `<set>` in logs. Known gap: the provider isn't asked again after it pairs the browser, so disabling an account there doesn't sign out its browsers here. Settings › Players (Unlink) or revoking the device does.

## Trusted proxies

`trusted_proxies` lists the addresses whose forwarding headers UpNext believes. It's empty by default, and ranges wider than /8 are refused. A request with none of a proxy's headers is judged as before, even from a listed address.

From a listed address, the client's address is the right-most hop in `X-Forwarded-For` or `Forwarded` that isn't itself listed, and one that can't be read counts as a public client. HTTPS and the host are taken as that hop's proxy recorded them. A request carrying both kinds of header must have them agree or is refused. The plain-HTTP rule, the limits and the log then apply to the client and not the proxy. No header names a player: `Remote-User` and the like are ignored.

With **The proxy does the login** (`trusted_proxy_login`, off by default), a listed proxy's visitors need no token and land on the player picker, PINs as usual. Each is an unbound full caller, as someone at the keyboard is: they can choose any player with the `UpNext-Player` header, list and revoke tokens and paired devices, and read the export, but can't do what only this computer can. That trusts everything that can connect from a listed address, so turn it on only for a proxy that makes everyone sign in, or a tailnet's `tailscale serve`, with UpNext reachable only through it (listening on loopback, or firewalled). Leave it off behind Tailscale Funnel, which lets anyone in. Its writes still need UpNext's own `Origin`.

## Not on the open internet

To reach UpNext away from home, put it behind Tailscale, a VPN, or a reverse proxy with its own login, and keep the token layer on as well, unless you've listed that proxy and let it do the login.

The token layer stays on behind a proxy even when the proxy rewrites `Host`. A proxy that speaks HTTP/1.1, strips every forwarding header and rewrites `Host` to a loopback address can't be told from a browser on this computer, and passes its visitors off as this computer. Don't set one up that way.
