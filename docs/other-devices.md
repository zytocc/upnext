# Other devices and scripts

The computer UpNext runs on needs nothing. Other devices are kept out until you let them in. Turn on **Allow other devices** (Settings › Devices and API) so UpNext listens on your network as well as on `127.0.0.1`.

## Pairing

A phone or another browser asks, and you allow it:

1. Open UpNext's address on the phone and choose **Ask to be allowed**.
2. Every UpNext window and browser on the computer it runs on shows "A phone called … wants to use UpNext" with a four-digit number. Check that the phone shows the same number and choose **Allow**. With two or more players, choose whose phone it is first.

`upnext pair requests` and `upnext pair approve 4821` do the same in a terminal.

Or type a code. **Add a device** shows a six-digit code that lasts five minutes, with a QR code of the address; scanning it pairs with nothing to type. `upnext pair` prints a code from a terminal.

Either way the browser keeps a cookie and shows in the token list as a paired device you can revoke. The [Android app](install/android.md) finds the computer on the network and asks the same way.

A script uses a token. Make one in Settings, or with `upnext token create --name x --scope read|full`: `read` looks and changes nothing, `full` does everything. Send it as `Authorization: Bearer upnext_…`:

```sh
curl -H "Authorization: Bearer $TOKEN" http://upnext.lan:8787/api/v1/play-later
```

The API is versioned under `/api/v1/`. [The HTTP API](API.md) is the guide, and the OpenAPI document is at `/api/v1/openapi.json`.

## HTTPS and proxies

Over plain HTTP, other devices can only connect from your home network or Tailscale. For anything further, use HTTPS. Either give UpNext a certificate (Settings, HTTPS, or `UPNEXT_TLS_CERT` and `UPNEXT_TLS_KEY`; HTTPS then works on the same port), or put a proxy in front.

`tailscale serve --bg 8787` is the easiest proxy. Caddy, Traefik and others work too; keep the request's `Host` (nginx: `proxy_set_header Host $host;`). Then set "Address for other devices" (`public_url`) to the address they use.

## Found on the network

While other devices are allowed, UpNext announces itself on your home network under the computer's name, so another computer's app window can find it. Settings › Devices and API changes the name or turns the announcement off. On Windows, the firewall must let UpNext receive on UDP port 5353 for another computer to find it; the address can always be typed instead.

## The library on another computer

`upnext app --connect`, or "Is your library on another computer?" on setup's first page, lists the computers on the network that run UpNext. Choose one and it asks that computer to allow this one, as a phone does. Once someone there taps Allow, the window shows that computer's UpNext and opens on it from then on. Games still start on the computer the window runs on. "Use this computer's library", in Settings › Devices and API, goes back.

## Behind your own sign-in

Two optional settings in Settings › Devices and API fit UpNext into a home server's sign-in: **Sign in with an identity provider** (any OpenID Connect provider, set with `UPNEXT_OIDC_ISSUER`, `UPNEXT_OIDC_CLIENT_ID` and `UPNEXT_OIDC_CLIENT_SECRET`) and **Trusted proxies** (`trusted_proxies`, `UPNEXT_TRUSTED_PROXIES`). Never turn on "The proxy does the login" (`trusted_proxy_login`) for Tailscale Funnel or any proxy that lets anyone through. [Sign-in and proxies](security/sign-in.md) has the rules, [OIDC](api/oidc.md) and [proxies](api/proxies.md) the setup, and [Who can reach UpNext](security/devices.md) the limits for devices.
