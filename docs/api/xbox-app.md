# The Xbox app

On Windows, a sync reads the games the Xbox app installed, Game Pass games included (sync option `xbox`). The Xbox app has no sign-in a program like UpNext may use, so its owned library and playtime can't be read: GOG Galaxy's Xbox plugin or [games added by hand](more-stores.md#games-added-by-hand) bring the rest. A game uninstalled since stays, as owned and not installed.

A game is found in the install folders of each drive, and only real games count: ordinary Store apps are left out. Each file is only read, and nothing in it is run.

## The copies

| Field | Value |
|---|---|
| `source`, `launcher` | `xboxone` |
| `source_id` | the Store id in lower case; the package family name when there's none |
| `installed` | true while the folder is there |
| `store_url` | `https://apps.microsoft.com/detail/{Store id}`, when there is one |
| `run_url` | `shell:AppsFolder\{package family name}!{application id}`, when the application id is one Windows allows |

## Play

`run_url` starts the game through Windows, as the Start menu does. The desktop app opens a `shell:` link only when it has exactly that shape. Browsers don't open these links, so Play does nothing for these games in a browser.

[`GET /api/setup`](setup-wizard.md)'s `stores.xbox` is `{"installed": 3}` (0 off Windows). The profile lists the `xboxone` source ("Xbox app") once it has games.
