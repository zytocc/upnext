# UpNext's Steam copy

UpNext on Steam is free, listed under Player Tools, and uses no Steamworks SDK. It is the release's own programs laid out as two SteamPipe depots, one for Windows and one for Linux and SteamOS. This folder has what makes and uploads them, the store text, and the library images "Add to Steam" gives a non-Steam shortcut. The Steamworks steps, the first-build checks and the upload are below; [`docs/dev/RELEASE.md`](../../docs/dev/RELEASE.md#the-steam-copy) has the per-release row.

| File | What it is |
|---|---|
| `build-depots.sh` | Both depots from a release: fetches the tag's x86_64 Linux archive and Windows zip (or takes them from a folder), checks each against its `.sha256`, and lays out `linux/` and `windows/`. |
| `bundle-linux.py` | The Linux depot: the programs with WebKitGTK and every library they need but those in `linux-exclude.txt`, the launcher, the licences and the third-party notices. |
| `fetch-sources.py` | The Ubuntu source packages of every library in the Linux depot, at exactly the versions it carries, as one tar for the release (below). |
| `linux-exclude.txt` | The libraries the Linux depot leaves to the computer, and why. |
| `upnext.sh` | The Linux depot's launcher, which Steam runs. |
| `smoke-test.sh` | Runs the Linux depot in a container that has no GTK, WebKitGTK or GLib. |
| `upload.sh` | Fills in `build/` with the ids from Steamworks and runs steamcmd; `--dry-run` only shows what it would do. |
| `build/` | SteamPipe's app build and the two depot builds, with `@…@` for the ids. |
| `listing/` | Drafts of the store page's text and the content survey's answers. |
| `library/` | The library images (capsule, header, hero, logo) "Add to Steam" writes for a non-Steam shortcut, at the sizes Steam's library uses. |

## The depots

**Windows:** `upnext.exe`, `upnext-app.exe`, the licences, the third-party notices (`THIRD_PARTY_NOTICES.txt`) and README from the release's zip, and `steam-copy.txt`. Steam runs `upnext-app.exe`, with `--big-screen` for the launch option "UpNext". The app window needs Microsoft's WebView2 Runtime, as the MSI install does.

**Linux and SteamOS:**

```
upnext.sh                     the launcher Steam runs
LICENSE-MIT, LICENSE-APACHE   UpNext's licences, and the README and CHANGELOG, from the release's archive
THIRD_PARTY_NOTICES.txt       the licences of the crates and npm packages built into the programs
bin/upnext, bin/upnext-app    the release's programs
bin/steam-copy.txt            tells UpNext it is the Steam copy
lib/                          WebKitGTK, GTK, GLib, GStreamer's core and the rest, with their RPATH set to this folder
lib/x86_64-linux-gnu/webkit2gtk-4.1/   WebKitGTK's web, network and GPU processes and its injected bundle
lib/gio/modules/              TLS (for GOG's sign-in window) and dconf
lib/gdk-pixbuf/               the SVG image loader and its list
lib/gstreamer-1.0/            empty on purpose (its README.txt says why)
share/glib-2.0/schemas/       GTK's settings schemas, which its file chooser needs
share/licenses/               each bundled package's licence, the common licences they name, and PACKAGES.txt with their versions and where their source is
```

[How the Linux depot works](linux-depot.md) explains where its libraries come from, what it leaves to the computer, and the two changes made to them.

## The libraries' source

Most of the bundled libraries are under the LGPL or the GPL, which ask whoever ships them to make their source available. Each release with a Steam build carries the Ubuntu source packages of every bundled library as `upnext-steam-linux-sources.tar`, made by `fetch-sources.py`; [The libraries' source](sources.md) says how it's made and checked.

## Trying it

```sh
cargo build --release --locked --bins            # with web/dist built first
contrib/steam/bundle-linux.py --bin-dir target/release --out target/steam-linux
contrib/steam/smoke-test.sh target/steam-linux ubuntu:24.04   # the glibc of the machine it was built on, or newer
```

The smoke test needs Docker and Xvfb (and ImageMagick to check the window drew). In a fresh home folder it runs `upnext --version`, a first `upnext serve`, which makes `~/.local/share/upnext/upnext.db`, then `upnext.sh --big-screen` as Steam would; it checks that the window's server answers, that "Add to Steam" says this is the Steam copy, that WebKitGTK's processes run from the depot, and lists the libraries each process loaded from outside it.

`upload.sh --dry-run` with made-up ids shows the filled-in build files without steamcmd:

```sh
STEAM_APP_ID=1000 STEAM_DEPOT_LINUX=1001 STEAM_DEPOT_WINDOWS=1002 contrib/steam/upload.sh --depots target/steam --dry-run
```

## Steamworks (once)

Valve's rules change; where Steamworks shows something different from the steps here, follow Steamworks.

1. **Partner account.** Sign up at `partner.steamgames.com` with the account that will own UpNext: identity, tax interview and bank details. Pay the Steam Direct fee for one product.
2. **The app.** Create it as software (not a game) in the Player Tools category, named UpNext, developer and publisher "UpNext contributors". Pricing: free.
3. **Depots** (SteamPipe, Depots): add "UpNext Linux" for Linux and SteamOS and "UpNext Windows" for Windows, 64-bit, and note their ids beside the app id. Publish.
4. **Launch options** (Installation, General); the first for each system is what Play runs:

   | System | Executable | Arguments | Description |
   |---|---|---|---|
   | Windows | `upnext-app.exe` | `--big-screen` | UpNext |
   | Windows | `upnext-app.exe` | | UpNext (desktop window) |
   | Linux and SteamOS | `upnext.sh` | `--big-screen` | UpNext |
   | Linux and SteamOS | `upnext.sh` | | UpNext (desktop window) |

5. **Linux runtime** (Installation, Linux runtime): Steam Linux Runtime 3.0 ("sniper"). The depot also runs without it.
6. **Steam Input:** make the Gamepad template the app's default controller configuration, so a Deck's controls and a pad reach big screen as a gamepad.
7. **A beta branch** (SteamPipe, Builds): add `beta`, the branch uploads go to. Publish.
8. **The builder account.** Make a separate Steam account for uploads, with Steam Guard by email, and give it only the permission to edit this app's builds (Users and Permissions). On a Linux machine, get Valve's steamcmd (`curl -fsSLO https://steamcdn-a.akamaihd.net/client/installer/steamcmd_linux.tar.gz`, unpack, `./steamcmd.sh`) and sign the account in once: `./steamcmd.sh +login <builder> +quit`, with its password and the Steam Guard code. steamcmd keeps the sign-in in `config/config.vdf` in its own folder (or `~/Steam/config/config.vdf`).
9. **The store page**, from `listing/` ([its README](listing/README.md) has the fields and image sizes): the short and long descriptions, the content survey's answers, the system requirements, the website and support fields, and the images. Turn on Coming Soon and submit the page for review.
10. **The first build** (upload it by hand, below), installed from the `beta` branch on a Steam Deck and on Windows (the app's Properties, Betas) and checked as in the next section. Then set it live on the default branch (SteamPipe, Builds) and submit the build for review.
11. **Release** once both reviews have passed and the Coming Soon period is over. Then add a Steam line to the install pages.

## Checking the first build

With the first build on the `beta` branch:

1. **Steam Deck, Gaming mode:** install UpNext from the library. "UpNext" opens full screen in big screen with the Deck's controls moving the focus (if they move a mouse, the default isn't the Gamepad template). The setup finds the Deck's Steam library. Press Play on a pick: the game starts, UpNext closes, and Gaming mode returns to its library after the game.
2. **The Deck, inside the runtime:** while it runs, `cat /proc/$(pgrep -f bin/upnext-app)/environ | tr '\0' '\n' | grep -E '^(Steam|PRESSURE_VESSEL|container)'` shows `SteamGamepadUI` (or `SteamTenfoot`) and whether it runs in pressure-vessel's container; `ls ~/.local/share/upnext` shows the database. If it isn't in the container, Steamworks' Linux runtime setting didn't take.
3. **The Deck, Desktop mode:** start UpNext from Steam: big screen, and Play leaves it open.
4. **The GOG sign-in window** opens from Settings › Accounts and loads GOG's page over HTTPS.
5. **A file chooser** (Settings › Backups, upload a backup) opens and shows its icons. They come from the computer's theme, which inside the runtime is sniper's.
6. **Claude Code** installed in `~/.local/bin` on the Deck runs from inside the runtime: an AI pick works.
7. **Windows:** install from Steam. "UpNext" opens big screen and "UpNext (desktop window)" the usual window. With the MSI also installed, both use `%LOCALAPPDATA%\upnext` and the second one started joins the first's server. Play from Big Picture closes UpNext and starts the game.
8. **A Linux desktop with Steam:** the same as 1 in Big Picture, and Settings › Big screen says this is the Steam copy instead of offering Add to Steam.

## Uploading a build by hand

Make the depots on Ubuntu 22.04, the system the release's programs are built on; elsewhere, in a container:

```sh
cd ~/src/upnext
docker run --rm -v "$PWD:/src" -w /src ubuntu:22.04 sh -c '
  apt-get update -qq && apt-get install -y -qq --no-install-recommends ca-certificates curl xz-utils python3 \
    libwebkit2gtk-4.1-0 libgtk-3-0 librsvg2-common glib-networking dconf-gsettings-backend libglib2.0-bin patchelf binutils &&
  contrib/steam/build-depots.sh --tag vX.Y.Z --out target/steam &&
  contrib/steam/fetch-sources.py --depot target/steam/linux --out target/steam-sources/upnext-steam-linux-sources.tar'
contrib/steam/smoke-test.sh target/steam/linux ubuntu:22.04     # optional: Docker and Xvfb
gh release upload vX.Y.Z target/steam-sources/upnext-steam-linux-sources.tar target/steam-sources/upnext-steam-linux-sources.tar.sha256 --repo zytocc/upnext
export STEAM_APP_ID=… STEAM_DEPOT_LINUX=… STEAM_DEPOT_WINDOWS=…   # from Steamworks
contrib/steam/upload.sh --depots target/steam --dry-run
STEAM_USERNAME=<builder> STEAMCMD=~/steamcmd/steamcmd.sh contrib/steam/upload.sh --depots target/steam
```

`build-depots.sh` fetches the tag's Linux archive and Windows zip from the release, checks them against their `.sha256` files, and lays out `target/steam/linux` and `target/steam/windows`. `fetch-sources.py` makes the tar of the Linux depot's library sources, which goes on the release before any build reaches Steam ([The libraries' source](sources.md)). `upload.sh` uploads both depots as one build and sets it live on `beta`; `--branch` names another beta branch, never the default one, which you set live in Steamworks after checking. Run it on Linux so the Linux depot keeps its executable bits.

## Setting up the workflow

After the manual route has worked once, the release workflow can do it on every tag. The `build-steam-depots` job makes the depots, runs the smoke test on the Linux one, and keeps them as the run's `steam-depots` artifact and the sources as `steam-sources`. The `attach-steam-sources` job attaches `upnext-steam-linux-sources.tar` and its `.sha256` to the release. The `publish-steam` job then uploads the depots to `beta`. A missing variable or secret leaves a notice and the release stands.

```sh
gh variable set UPNEXT_STEAM_DEPOTS --body on --repo zytocc/upnext
gh variable set STEAM_APP_ID --body <app id> --repo zytocc/upnext
gh variable set STEAM_DEPOT_LINUX --body <Linux depot id> --repo zytocc/upnext
gh variable set STEAM_DEPOT_WINDOWS --body <Windows depot id> --repo zytocc/upnext
# steamcmd's bootstrap, which the job checks before running it:
curl -fsSLO https://steamcdn-a.akamaihd.net/client/installer/steamcmd_linux.tar.gz && sha256sum steamcmd_linux.tar.gz
gh variable set STEAMCMD_SHA256 --body <that hash> --repo zytocc/upnext
gh secret set STEAM_USERNAME --repo zytocc/upnext                 # the builder account's name
base64 -w0 ~/steamcmd/config/config.vdf | gh secret set STEAM_CONFIG_VDF --repo zytocc/upnext
gh variable set UPNEXT_STEAM_UPLOAD --body on --repo zytocc/upnext
```

`STEAM_BRANCH` names a branch other than `beta`. When `publish-steam` fails asking for a Steam Guard code, sign the builder in again with steamcmd and set `STEAM_CONFIG_VDF` from the new file. When Valve replaces steamcmd's bootstrap, the checksum fails: download it, check it's Valve's, and set `STEAMCMD_SHA256` again.
