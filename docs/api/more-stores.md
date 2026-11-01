# Epic, Galaxy and the other stores

## Epic

Epic games come from Heroic where it's installed, and without it from the Epic sign-in or the Epic Games Launcher's installs. All make copies with `source` and `launcher` `epic` and the Epic app name as `source_id`, so a game seen both ways is one row and Heroic keeps it.

- **The Epic sign-in.** `auth_url` is Epic's sign-in page, which ends on a page of JSON holding an `authorizationCode`. That code (or the whole page) is posted back. The sign-in is kept on the player's Epic connection and renewed at each sync. A sync reads the owned library and the playtime Epic keeps. Add-ons, demos, engine downloads and private test builds are left out.
- **The Epic Games Launcher's installs.** On Windows, the launcher's manifests (elsewhere set with `epic_manifests` or `UPNEXT_EPIC_MANIFESTS`) list the installed games, with no sign-in. Without the sign-in, a game uninstalled since stays, as owned and not installed.

`GET /api/epic` gives `{"connected": false, "display_name": null, "auth_url": "https://www.epicgames.com/id/login?redirectUrl=…", "error": null}`. `error` says why the sign-in ended when Epic refused to renew it at a sync. The stored sign-in is then deleted.

`POST /api/epic/login` with `{"code": "<the authorizationCode, or the whole page>"}` answers in that shape, connected, with the `display_name`. `400` when there's no code in it or Epic turns it down (a code works once, for a few minutes).

`DELETE /api/epic` deletes the stored sign-in and answers in that shape. The Epic games already read stay, as owned. `upnext epic-login [code]` and `upnext epic-logout` do the same from a terminal.

`GET /api/settings` has `epic` (that shape) and `epic_launcher_dir` (the manifests folder found, or null), and `GET /api/config` has `epic_connected`. The `epic` profile source counts as configured with Heroic, the sign-in or the launcher, with a `note` naming which.

## Installed games on Windows

On Windows, a sync reads the games these installed (sync options `ea`, `ubisoft` and `xbox`). A game uninstalled since stays, as owned and not installed. None tracks playtime.

- **The EA app.** Copies have `source` and `launcher` `origin` and no `run_url`.
- **Ubisoft Connect.** Copies have `source` and `launcher` `uplay`, and `run_url` `uplay://launch/{id}/0`.
- **The Xbox app.** Its game folders, as [The Xbox app](xbox-app.md) says.

[`GET /api/setup`](setup-wizard.md)'s `stores` has `ea`, `ubisoft` and `xbox`, `{"installed": 3}` (0 off Windows). The profile lists the `origin`, `uplay` and `xboxone` sources once they have games.

## Games added by hand

Lists of titles for the EA app, Ubisoft Connect, Battle.net and the Xbox app, and single games, are on [Games added by hand](added-games.md).

## GOG Galaxy

With a Galaxy database (`galaxy_db`, or one uploaded in the [wizard](setup.md#post-apigalaxyupload)), a sync reads every store Galaxy is linked to. It leaves out games Galaxy says aren't owned (Steam Family Sharing), subscription titles (EA Play), and betas, test servers, playtests, demos and teasers.

- `galaxy_skip`, such as `["origin"]`: the integrations a sync leaves out. `PUT` takes them comma-separated (`"origin,xboxone"`; `ea` and `xbox` are taken for those two, and `gog` is refused). A sync removes the copies of a store it names.
- `galaxy_integrations`: every store a Galaxy sync has brought games from, GOG's own first: `[{"platform": "origin", "label": "EA app", "games": 55, "last_added": "2024-05-18", "connected": true, "left_out": false, "seen_at": 1790000000}]`. `connected` is what Galaxy says, or null. Galaxy calls unlinked plugins connected too, so it's reported and never acted on.
