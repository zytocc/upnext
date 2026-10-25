# The Xbox app

[API index](../API.md) · [More stores](more-stores.md) · [Games](games.md#sources-and-launchers) · [Sync and enrich sources](sources.md)

On Windows, a sync reads the games the Xbox app installed, Game Pass games included (sync option `xbox`). The Xbox app has no sign-in a program like UpNext may use, so its owned library and playtime can't be read: GOG Galaxy's Xbox plugin or [games added by hand](more-stores.md#games-added-by-hand) bring the rest. A game uninstalled since stays, as owned and not installed, and goes to player 1's Xbox app connection like the computer's other installs.

## What is read

- **The install folders.** Each drive from `C:` to `Z:` with a hidden `.GamingRoot` file at its root installs into the folders that file names, relative to the root: `RGBX`, a count, then UTF-16 names each ended by a 0. A name with `..`, a drive letter or a control character is dropped. A drive whose file names none, or has no file, is looked at for `XboxGames`.
- **A game.** A folder in those folders whose `Content` folder, or the folder itself, holds a `MicrosoftGame.config`. Ordinary Store apps don't have one, so they're left out.
- **Its ids.** The package's `appxmanifest.xml` beside the config, else the config, gives the package's `Identity` (`Name` and `Publisher`). The package family name is `Name_` and the 13-character publisher id Windows works out from `Publisher`. The manifest's first `Application Id`, else the config's first PC `Executable Id`, is the application id. The config's `StoreId` is the Microsoft Store's product id.
- **Its name.** The config's `ShellVisuals DefaultDisplayName`, else the manifest's `DisplayName`, else the folder's name. A name that is an `ms-resource:` reference, which only Windows can look up, is skipped.

Each file is read only up to 1 MiB, comments are dropped before it's read, and nothing in it is run.

## The copies

| Field | Value |
|---|---|
| `source`, `launcher` | `xboxone`, GOG Galaxy's name for the store |
| `source_id` | the Store id in lower case, as Galaxy's Xbox copies have it; the package family name when there's none |
| `installed` | true while the folder is there |
| `store_url` | `https://apps.microsoft.com/detail/{Store id}`, when there is one |
| `run_url` | `shell:AppsFolder\{package family name}!{application id}`, when the application id is one Windows allows |

## Play

`run_url` starts the game through Windows, as the Start menu does. A Store package's `.exe` can't be run directly. The desktop app opens a `shell:` link only when it has exactly that shape (a webview's `%5C` and `%21` are read as `\` and `!`), since a `shell:` link can name any folder or program. Browsers don't open these links, so Play does nothing for these games in a browser.

[`GET /api/setup`](setup-wizard.md)'s `stores.xbox` is `{"installed": 3}` (0 off Windows). The profile lists the `xboxone` source ("Xbox app") once it has games.
