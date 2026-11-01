# Flatpak and Steam Deck

UpNext's Flatpak, `io.github.zytocc.UpNext`, runs on any Linux with Flatpak and on a Steam Deck. Download `upnext-<version>-x86_64.flatpak` from the [latest release](https://github.com/zytocc/upnext/releases/latest) and install it. It needs the GNOME runtime from Flathub, so set up Flathub first ([flathub.org/setup](https://flathub.org/setup)). UpNext itself isn't on Flathub. To build the Flatpak yourself, see [contrib/flatpak](../../contrib/flatpak/README.md).

```sh
flatpak install --user ./upnext-<version>-x86_64.flatpak
flatpak run io.github.zytocc.UpNext                           # the app window; it's in your launcher too
flatpak run --command=upnext io.github.zytocc.UpNext doctor   # the command line
```

To update, run `flatpak install --user --reinstall ./upnext-<new version>-x86_64.flatpak`; on Flatpak older than 1.16.2, run `flatpak uninstall --user io.github.zytocc.UpNext` first. Your library stays, and `flatpak update` updates the runtime only. To uninstall, run `flatpak uninstall --user io.github.zytocc.UpNext`.

## What the sandbox changes

- **What it reads.** Steam's and Heroic's usual folders, read-only. For a library somewhere else, `flatpak override --user --filesystem=/path/to/Steam:ro --env=UPNEXT_STEAM_DIR=/path/to/Steam io.github.zytocc.UpNext` lets it read the folder and points UpNext at it (`UPNEXT_HEROIC_DIR` for Heroic). The setup wizard shows the same lines.
- **Where it keeps things.** In `~/.var/app/io.github.zytocc.UpNext`, apart from a native install's. To move a library between the two, download a backup in one (Settings › Backups, with keys and sign-ins) and import it in the other.
- **It runs while its window is open.** There is no service and no [start at login](../running.md#start-it-at-login-linux); schedules run while the window is open.
- **The AI.** An API key or your own server work as anywhere. For Claude Code, run `flatpak run --command=upnext io.github.zytocc.UpNext install-claude`, then give it a token from `claude setup-token` with `flatpak override --user --env=CLAUDE_CODE_OAUTH_TOKEN=<the token> io.github.zytocc.UpNext` and start UpNext again. The Codex and Copilot CLIs can't be used from the Flatpak.
- **Add to Steam** can't change Steam's files from the sandbox, so add UpNext in Steam yourself (below).

## On a Steam Deck

1. In Desktop mode, install the Flatpak from Konsole as above.
2. Open Steam there and choose Games › Add a Non-Steam Game to My Library. Tick UpNext and add it.
3. In its Properties, put ` --big-screen` at the end of the launch options.
4. On its page, choose the controller icon, the layout, Templates, Gamepad.

Back in Gaming mode, UpNext is in your library under Non-Steam and opens in [big screen](../big-screen.md#add-to-steam). A SteamOS update keeps UpNext and your library.
