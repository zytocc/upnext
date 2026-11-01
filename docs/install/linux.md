# Linux

## One command

```sh
curl -fsSL https://upnext.games/install.sh | sh
```

It installs the newest release with the package manager: the `.deb` with apt on Debian and Ubuntu, the `.rpm` with dnf on Fedora or zypper on openSUSE, the pacman package on Arch (x86_64), and the Flatpak on SteamOS. Each download is checked against its SHA-256 first.

On any other distribution, or with `| sh -s -- --user` in place of `| sh`, it puts `upnext` and `upnext-app` in `~/.local/bin` and UpNext in your launcher, without sudo. Run it again to upgrade; `| sh -s -- --version 0.1.2` installs a given release. [site/install.sh](../../site/install.sh) is the script.

## Debian, Ubuntu and Fedora

Each [release](https://github.com/zytocc/upnext/releases) has packages for x86_64 and arm64: a `.deb` for Debian 12 and 13 and Ubuntu 22.04, 24.04 and 26.04, and an `.rpm` for Fedora 43 to 45. Set `V` to the release's version; on arm64 the files end in `_arm64.deb` and `.aarch64.rpm`.

```sh
V=0.1.2
# Debian and Ubuntu
curl -LO "https://github.com/zytocc/upnext/releases/download/v$V/upnext_$V-1_amd64.deb"
curl -LO "https://github.com/zytocc/upnext/releases/download/v$V/upnext_$V-1_amd64.deb.sha256"
sha256sum -c "upnext_$V-1_amd64.deb.sha256"
sudo apt install "./upnext_$V-1_amd64.deb"
# Fedora
curl -LO "https://github.com/zytocc/upnext/releases/download/v$V/upnext-$V-1.x86_64.rpm"
curl -LO "https://github.com/zytocc/upnext/releases/download/v$V/upnext-$V-1.x86_64.rpm.sha256"
sha256sum -c "upnext-$V-1.x86_64.rpm.sha256"
sudo dnf install "./upnext-$V-1.x86_64.rpm"
# openSUSE Tumbleweed: the same .rpm, which isn't signed
sudo zypper install --allow-unsigned-rpm "./upnext-$V-1.x86_64.rpm"
```

The package puts `upnext` and `upnext-app` in `/usr/bin` and UpNext in your launcher under Games. Open it from there, or run `upnext app` for the window or `upnext serve` for the web UI. To start it at login, run `systemctl --user enable --now upnext`. If you ran `upnext install-desktop` before, run `upnext install-desktop --remove`, or your own launcher entry hides the package's.

To update, install the newer package. To uninstall, run `systemctl --user disable --now upnext` if you turned the service on, then `sudo apt remove upnext`, `sudo dnf remove upnext` or `sudo zypper remove upnext`; your library in `~/.local/share/upnext` stays.

If you had UpNext from `cargo install` before, run `cargo uninstall upnext`, or that copy runs instead. Remove a copied `~/.config/systemd/user/upnext.service` too, since it overrides the package's, and run `systemctl --user daemon-reload`.

## Arch Linux

Each release has a `PKGBUILD` for `upnext-bin`, which installs that release's Linux archive, checked against its SHA-256, as a pacman package:

```sh
mkdir upnext-bin && cd upnext-bin
curl -fsSLO https://github.com/zytocc/upnext/releases/latest/download/PKGBUILD
less PKGBUILD        # read it first, as with any PKGBUILD
makepkg -si          # downloads the release archive, checks it, builds and installs the package
```

The same `PKGBUILD` builds on Arch Linux ARM (aarch64). Or download `upnext-bin-<version>-1-x86_64.pkg.tar.zst` from the release and run `sudo pacman -U ./upnext-bin-<version>-1-x86_64.pkg.tar.zst`; give pacman the downloaded file, not the link, because the package isn't signed.

The package puts `upnext` and `upnext-app` in `/usr/bin` and UpNext in your launcher under Games. To start it at login, run `systemctl --user enable --now upnext`. To update, build the new release's `PKGBUILD` the same way. To uninstall, `sudo pacman -R upnext-bin`; your library stays.

## Release archive

Each release also has an archive for Linux on x86_64 and arm64, with `upnext` and `upnext-app` and the web UI built in. It needs glibc 2.35 or newer, and WebKitGTK 4.1 and GTK 3 (`sudo apt install libwebkit2gtk-4.1-0 libgtk-3-0` on Debian and Ubuntu, `sudo pacman -S webkit2gtk-4.1 gtk3` on Arch, `sudo dnf install webkit2gtk4.1 gtk3` on Fedora).

```sh
curl -LO https://github.com/zytocc/upnext/releases/latest/download/upnext-x86_64-unknown-linux-gnu.tar.xz
curl -LO https://github.com/zytocc/upnext/releases/latest/download/upnext-x86_64-unknown-linux-gnu.tar.xz.sha256
sha256sum -c upnext-x86_64-unknown-linux-gnu.tar.xz.sha256
tar xf upnext-x86_64-unknown-linux-gnu.tar.xz
install -Dm755 -t ~/.local/bin upnext-x86_64-unknown-linux-gnu/upnext upnext-x86_64-unknown-linux-gnu/upnext-app
upnext serve                               # then open http://127.0.0.1:8787
```

On arm64, the archive is `upnext-aarch64-unknown-linux-gnu.tar.xz`. `~/.local/bin` has to be on your `PATH`. To start it at login, see [Running UpNext](../running.md#start-it-at-login-linux). To update, repeat the steps. To uninstall, delete the two programs; your library stays.

[Flatpak](flatpak.md) covers any Linux with Flatpak and the Steam Deck, and [Building from source](source.md) the rest.
