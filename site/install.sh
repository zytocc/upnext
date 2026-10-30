#!/bin/sh
# UpNext's installer for Linux, served at https://upnext.games/install.sh:
#
#     curl -fsSL https://upnext.games/install.sh | sh
#
# It installs the newest release from https://github.com/zytocc/upnext/releases
# the way docs/install/linux.md describes by hand: the .deb with apt on
# Debian and Ubuntu, the .rpm with dnf on Fedora or zypper on openSUSE, the
# pacman package on Arch (x86_64), and the Flatpak on SteamOS, whose system
# can't take packages. Anywhere else, or with --user, it puts the release
# archive's two programs in ~/.local/bin and adds UpNext to the launcher,
# with no sudo. Each download is checked against the SHA-256 the release
# publishes beside it before anything is installed. Running it again installs
# the newest release over the old one; your library is never touched.
#
# Options, after `sh -s --` when piping:
#     --user            install into ~/.local/bin, without sudo or packages
#     --version X.Y.Z   install that release instead of the newest
#     --help            print this summary
#
# UPNEXT_VERSION=X.Y.Z does the same as --version.
#
# Everything is inside functions and main runs on the last line, so a
# download cut off halfway runs nothing.

set -eu

REPO=https://github.com/zytocc/upnext
FLATPAK_ID=io.github.zytocc.UpNext
DOCS=$REPO/blob/main/docs/install/linux.md

say() {
    printf 'upnext: %s\n' "$*"
}

fail() {
    printf 'upnext: %s\n' "$*" >&2
    exit 1
}

usage() {
    cat <<'EOF'
Installs UpNext on Linux from its newest GitHub release.

    curl -fsSL https://upnext.games/install.sh | sh
    curl -fsSL https://upnext.games/install.sh | sh -s -- --user

Options:
    --user            install into ~/.local/bin, without sudo or packages
    --version X.Y.Z   install that release instead of the newest
    --help            print this summary

More: https://github.com/zytocc/upnext/blob/main/docs/install/linux.md
EOF
}

need() {
    command -v "$1" >/dev/null 2>&1 || fail "this needs $1, which isn't installed"
}

fetch() {
    # HTTPS only, redirects included (GitHub sends downloads to its storage).
    curl --proto '=https' --tlsv1.2 -fsSL --retry 3 -o "$2" "$1" || fail "couldn't download $1"
}

# The newest full release's version, from where /releases/latest redirects
# (.../releases/tag/vX.Y.Z), so no API call and no rate limit.
latest_version() {
    url=$(curl --proto '=https' --tlsv1.2 -fsSL --retry 3 -o /dev/null -w '%{url_effective}' "$REPO/releases/latest") ||
        fail "couldn't reach $REPO/releases/latest"
    printf '%s\n' "${url##*/tag/v}"
}

# Download one release file and its .sha256 into $TMP, and stop unless they
# agree. The .sha256 holds "<hash> *<file>".
download() {
    file=$1
    say "downloading $file"
    fetch "$REPO/releases/download/v$VERSION/$file" "$TMP/$file"
    fetch "$REPO/releases/download/v$VERSION/$file.sha256" "$TMP/$file.sha256"
    expected=$(cut -d ' ' -f 1 "$TMP/$file.sha256")
    case $expected in
        *[!0-9a-f]* | "") fail "$file.sha256 doesn't hold a SHA-256" ;;
    esac
    [ ${#expected} -eq 64 ] || fail "$file.sha256 doesn't hold a SHA-256"
    actual=$(sha256sum "$TMP/$file" | cut -d ' ' -f 1)
    [ "$actual" = "$expected" ] || fail "$file doesn't match its SHA-256 (expected $expected, got $actual); nothing was installed"
    say "checked $file against its SHA-256"
}

# sudo, doas, or nothing for root. Empty output with status 1: neither.
privilege() {
    if [ "$(id -u)" -eq 0 ]; then
        printf '\n'
    elif command -v sudo >/dev/null 2>&1; then
        printf 'sudo\n'
    elif command -v doas >/dev/null 2>&1; then
        printf 'doas\n'
    else
        return 1
    fi
}

os_release() {
    # The value of one key of /etc/os-release, unquoted.
    [ -r /etc/os-release ] || return 0
    sed -n "s/^$1=//p" /etc/os-release | tr -d '"' | head -n 1
}

# How to install here: deb, rpm-dnf, rpm-zypper, pacman, flatpak or user.
choose_method() {
    id=$(os_release ID)
    if [ "$id" = steamos ]; then
        if [ "$ARCH" = x86_64 ] && command -v flatpak >/dev/null 2>&1; then
            echo flatpak
        else
            echo user
        fi
        return
    fi
    # Fedora Silverblue, Kinoite and other image-based systems take no
    # packages from dnf.
    if [ -e /run/ostree-booted ] || ! SUDO=$(privilege); then
        echo user
    elif command -v apt-get >/dev/null 2>&1 && command -v dpkg >/dev/null 2>&1; then
        echo deb
    elif command -v dnf >/dev/null 2>&1; then
        echo rpm-dnf
    elif command -v zypper >/dev/null 2>&1; then
        echo rpm-zypper
    elif command -v pacman >/dev/null 2>&1 && [ "$ARCH" = x86_64 ]; then
        echo pacman
    else
        echo user
    fi
}

install_deb() {
    case $ARCH in x86_64) deb_arch=amd64 ;; *) deb_arch=arm64 ;; esac
    file="upnext_${VERSION}-1_${deb_arch}.deb"
    download "$file"
    # apt reads the file as its own unprivileged user, which needs to see it.
    chmod 755 "$TMP"
    chmod 644 "$TMP/$file"
    say "installing $file with apt (WebKitGTK and GTK come with it)"
    $SUDO apt-get install -y "$TMP/$file" </dev/null ||
        fail "apt couldn't install $file (its message is above); other ways to install: $DOCS"
}

install_rpm() {
    file="upnext-${VERSION}-1.${ARCH}.rpm"
    download "$file"
    if [ "$1" = dnf ]; then
        say "installing $file with dnf (WebKitGTK and GTK come with it)"
        $SUDO dnf install -y "$TMP/$file" </dev/null ||
            fail "dnf couldn't install $file (its message is above); other ways to install: $DOCS"
    else
        # The .rpm isn't signed, which zypper refuses unless told.
        say "installing $file with zypper (WebKitGTK and GTK come with it)"
        $SUDO zypper --non-interactive install --allow-unsigned-rpm "$TMP/$file" </dev/null ||
            fail "zypper couldn't install $file (its message is above); other ways to install: $DOCS"
    fi
}

install_pacman() {
    file="upnext-bin-${VERSION}-1-x86_64.pkg.tar.zst"
    download "$file"
    # The local file, which pacman takes without a signature (it's checked
    # against the release's SHA-256 above instead).
    say "installing $file with pacman (WebKitGTK and GTK come with it)"
    $SUDO pacman -U --noconfirm "$TMP/$file" </dev/null ||
        fail "pacman couldn't install $file (its message is above); other ways to install: $DOCS"
}

install_flatpak() {
    file="upnext-${VERSION}-x86_64.flatpak"
    download "$file"
    # A bundle names no place to update from, so a new release replaces the
    # installed copy; its library in ~/.var/app stays.
    if flatpak info --user "$FLATPAK_ID" >/dev/null 2>&1; then
        say "removing the installed Flatpak first (your library stays)"
        flatpak uninstall --user -y --noninteractive "$FLATPAK_ID" </dev/null
    fi
    say "installing $file with Flatpak (the GNOME runtime comes from Flathub)"
    flatpak install --user -y --noninteractive "$TMP/$file" </dev/null ||
        fail "flatpak couldn't install $file (its message is above); Flathub has to be set up first"
}

install_user() {
    need tar
    need xz
    target="${ARCH}-unknown-linux-gnu"
    file="upnext-${target}.tar.xz"
    download "$file"
    tar -xJf "$TMP/$file" -C "$TMP"
    bin="$HOME/.local/bin"
    mkdir -p "$bin"
    for program in upnext upnext-app; do
        # A new file renamed over the old one, so a running copy keeps its own.
        cp "$TMP/upnext-${target}/$program" "$bin/.$program.new"
        chmod 755 "$bin/.$program.new"
        mv -f "$bin/.$program.new" "$bin/$program"
    done
    say "put upnext and upnext-app in $bin"
    # Both programs link WebKitGTK and GTK, which packages would have
    # brought along.
    if ldd "$bin/upnext" 2>/dev/null | grep -q 'not found'; then
        say "UpNext needs WebKitGTK 4.1 and GTK 3, which aren't installed here. Install them with"
        say "  sudo apt install libwebkit2gtk-4.1-0 libgtk-3-0   (Debian, Ubuntu)"
        say "  sudo dnf install webkit2gtk4.1 gtk3               (Fedora)"
        say "  sudo pacman -S webkit2gtk-4.1 gtk3                (Arch)"
        say "then run: upnext install-desktop"
    else
        "$bin/upnext" install-desktop || say "couldn't add UpNext to the launcher; \`upnext app\` opens the window"
    fi
    case ":$PATH:" in
        *":$bin:"*) ;;
        *) say "$bin isn't on your PATH yet; on most distributions it is after you log in again" ;;
    esac
}

main() {
    user=no
    VERSION=${UPNEXT_VERSION:-}
    while [ $# -gt 0 ]; do
        case $1 in
            --user) user=yes ;;
            --version)
                [ $# -ge 2 ] || fail "--version needs a version, such as 0.1.2"
                VERSION=$2
                shift
                ;;
            --version=*) VERSION=${1#--version=} ;;
            -h | --help)
                usage
                return 0
                ;;
            *) fail "unknown option $1 (try --help)" ;;
        esac
        shift
    done

    [ "$(uname -s)" = Linux ] || fail "this installer is for Linux; for Windows, see $REPO/blob/main/docs/install/windows.md"
    case $(uname -m) in
        x86_64 | amd64) ARCH=x86_64 ;;
        aarch64 | arm64) ARCH=aarch64 ;;
        *) fail "UpNext's releases are for x86_64 and arm64, not $(uname -m)" ;;
    esac
    need curl
    need sha256sum

    [ -n "$VERSION" ] || VERSION=$(latest_version)
    VERSION=${VERSION#v}
    case $VERSION in
        [0-9]*.[0-9]*.[0-9]*) ;;
        *) fail "\"$VERSION\" isn't a release version, such as 0.1.2" ;;
    esac
    case $VERSION in
        *[!0-9A-Za-z.-]*) fail "\"$VERSION\" isn't a release version, such as 0.1.2" ;;
    esac

    SUDO=
    if [ "$user" = yes ]; then
        method=user
    else
        method=$(choose_method)
        SUDO=$(privilege) || SUDO=
    fi

    TMP=$(mktemp -d)
    trap 'rm -rf "$TMP"' EXIT
    trap 'exit 1' HUP INT TERM

    say "installing UpNext $VERSION"
    case $method in
        deb) install_deb ;;
        rpm-dnf) install_rpm dnf ;;
        rpm-zypper) install_rpm zypper ;;
        pacman) install_pacman ;;
        flatpak) install_flatpak ;;
        user) install_user ;;
    esac

    say "UpNext $VERSION is installed"
    case $method in
        flatpak)
            say "open it from your launcher, or run: flatpak run $FLATPAK_ID"
            ;;
        user)
            say "run \`upnext app\` for the window or \`upnext serve\` for the web UI"
            say "to remove it: upnext install-desktop --remove; rm ~/.local/bin/upnext ~/.local/bin/upnext-app"
            ;;
        *)
            say "open it from your launcher, or run \`upnext app\` for the window or \`upnext serve\` for the web UI"
            say "to start it at login: systemctl --user enable --now upnext"
            # An older copy earlier on the PATH would hide the package's.
            found=$(command -v upnext 2>/dev/null || true)
            if [ -n "$found" ] && [ "$found" != /usr/bin/upnext ]; then
                say "note: $found comes before /usr/bin/upnext on your PATH; remove it to use the package"
            fi
            ;;
    esac
}

main "$@"
