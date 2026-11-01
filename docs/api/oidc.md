# Signing in with an identity provider

With `oidc_issuer`, `oidc_client_id` and `public_url` set, the page another device lands on offers "Sign in with <provider>". It pairs the browser and finds or makes its player. UpNext is the relying party (authorization code flow with PKCE, the provider found by discovery from its issuer). The checks it makes and the limits on waiting sign-ins are in [Sign-in and proxies](../security/sign-in.md).

Register UpNext at the provider as a confidential client (or a public one, with no secret). The redirect address is `public_url` followed by `/api/v1/oidc/callback`. The scopes are `openid`, `profile` and `email`, plus `groups` when `oidc_allowed_groups` is used.

The three `GET` routes below need no token, but not over plain HTTP from a public address (`403`).

## GET /api/oidc

`{"enabled": true, "name": "Authelia"}`: whether the sign-in is offered, and the button's name (`oidc_name`, else the issuer's host).

## GET /api/oidc/login

`303` to the provider's sign-in page. The sign-in is tied to this browser by an `upnext_oidc_signin` cookie. When the sign-in isn't set up, the provider can't be reached or too many sign-ins are waiting, it goes to `/#signin_error=<why>`.

## GET /api/oidc/callback?state=…&code=…

The provider's way back. A state works once.

- The account (`iss` and `sub`) is looked up as a linked player. Players are never matched by name or email.
- While "Link the next sign-in" waits for a player, an account no player has isn't made a player: it waits to be confirmed on this computer, and the browser goes to `/#signin_waiting=<what next>` with no token.
- Otherwise, with `oidc_auto_provision` on (the default), an unknown account gets a new player named from `name`, else `preferred_username`, else "Player N" (with ` 2`, ` 3`… when taken). With it off, an unknown account is refused until a player is linked.
- With `oidc_allowed_groups` set (comma-separated), the token's `groups` claim must hold one of them.

The browser gets a device token bound to the player, of `full` scope and `kind: "oidc"`, named from the browser and the provider ("Android phone (Authelia)"), in the `upnext_device_<port>` cookie, and goes to `/`. On any failure it's `303` to `/#signin_error=<why>` with nothing made.

## POST /api/oidc/logout

`204`, revoking this browser's device token and clearing its cookie. It doesn't sign out of the provider. `400` for a browser that didn't sign in this way.

## Linking an account to a player

These are only on this computer.

- **`POST /api/players/{id}/oidc/link`** `{"pin": "1234"}` (the PIN when the player has one, else `{}`) → the player, with `oidc_link_until`: "Link the next sign-in". For ten minutes, an account no player is linked to that signs in waits to be confirmed as this player (`oidc_waiting`). One player at a time; arming another replaces it. `400` when no issuer is set, `403` from another device or for a wrong PIN, `429` while locked.
- **`POST /api/players/{id}/oidc/confirm`** `{"subject": "u-1", "pin": "1234"}` → the player, now with `oidc`. `subject` names the waiting account, as `oidc_waiting` showed it. The account signs in as this player from then on, so the browser signs in again. An account the player had before is unlinked, and the browsers it signed in are signed out. Needs the PIN again when the player has one. `409` when that account isn't waiting for this player any more; `403` and `429` as above.
- **`DELETE /api/players/{id}/oidc/link`** → the player: the link ends, with any account waiting.
- **`DELETE /api/players/{id}/oidc`** → the player: unlinks their account, ends a link waiting for them, and revokes the device tokens that account signed in as them.

A player's `oidc`, `oidc_link_until` and `oidc_waiting` fields show where this stands ([Players](players.md#get-apiplayers)).

## Settings

`oidc_issuer`, `oidc_client_id`, `oidc_client_secret` (write-only: `GET /api/settings` gives `oidc_client_secret_set`), `oidc_name`, `oidc_auto_provision` (`true`) and `oidc_allowed_groups`. Each is also in the environment (`UPNEXT_OIDC_ISSUER` and so on). They're set only on this computer or in the environment.

`oidc_issuer` is `https://`, or `http://` on this computer or its network. Enter it exactly as the provider's tokens name it. `GET /api/settings` gives each with `<key>_source`, and `oidc_redirect_uri`, the address to register at the provider (null without `public_url`).

Back-channel logout, SCIM and LDAP aren't supported.
