#!/usr/bin/env bash
# Writes the winget manifests for one published release: the three files
# microsoft/winget-pkgs keeps for each version of Zytoc.UpNext, under
# <winget-pkgs checkout>/manifests/z/Zytoc/UpNext/<version>/.
#
#   contrib/winget/render.sh 0.1.2 ../winget-pkgs
#
# The package is the release's per-user MSI (contrib/windows/upnext.wxs). The
# script downloads the MSI's .sha256 from the release for InstallerSha256,
# and takes the release date from this checkout's CHANGELOG.md, so the
# release has to be published and its CHANGELOG section dated. With msitools
# installed (`msiinfo`), it also downloads the MSI, checks it against that
# SHA-256 and adds its ProductCode, which lets winget find an installed copy
# directly; without it the UpgradeCode, which every UpNext MSI shares, does.
# contrib/winget/README.md says how to submit the files.
set -euo pipefail

if [[ $# -ne 2 || ! $1 =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
  echo "usage: $0 X.Y.Z <winget-pkgs checkout>" >&2
  exit 2
fi
version=$1
root=$(cd "$(dirname "$0")/../.." && pwd)
out="$2/manifests/z/Zytoc/UpNext/$version"

id=Zytoc.UpNext
schema=1.10.0
# upnext.wxs's UpgradeCode, the same for every release.
upgrade_code='{829B9437-D64F-4555-9CE8-971A5EA0CAC1}'
msi=upnext-x86_64-pc-windows-msvc.msi
url="https://github.com/zytocc/upnext/releases/download/v$version/$msi"

release_date=$(sed -n "s/^## \[$version\] - \([0-9-]*\)$/\1/p" "$root/CHANGELOG.md")
[[ $release_date =~ ^[0-9]{4}-[0-9]{2}-[0-9]{2}$ ]] || {
  echo "CHANGELOG.md has no dated section for $version" >&2
  exit 1
}

sha256=$(curl --proto '=https' --tlsv1.2 -fsSL "$url.sha256" | cut -d ' ' -f 1)
[[ $sha256 =~ ^[0-9a-f]{64}$ ]] || {
  echo "$url.sha256 doesn't hold a SHA-256" >&2
  exit 1
}

product_code=
if command -v msiinfo >/dev/null; then
  tmp=$(mktemp -d)
  trap 'rm -rf "$tmp"' EXIT
  curl --proto '=https' --tlsv1.2 -fsSL -o "$tmp/$msi" "$url"
  echo "$sha256  $tmp/$msi" | sha256sum -c --quiet
  product_code=$(msiinfo export "$tmp/$msi" Property | tr -d '\r' | sed -n 's/^ProductCode\t//p')
  [[ $product_code =~ ^\{[0-9A-F-]{36}\}$ ]] || {
    echo "couldn't read the MSI's ProductCode" >&2
    exit 1
  }
else
  echo "msiinfo (msitools) isn't installed: leaving out the ProductCode" >&2
fi

mkdir -p "$out"

cat >"$out/$id.yaml" <<EOF
# yaml-language-server: \$schema=https://aka.ms/winget-manifest.version.$schema.schema.json

PackageIdentifier: $id
PackageVersion: $version
DefaultLocale: en-US
ManifestType: version
ManifestVersion: $schema
EOF

{
  cat <<EOF
# yaml-language-server: \$schema=https://aka.ms/winget-manifest.installer.$schema.schema.json

PackageIdentifier: $id
PackageVersion: $version
InstallerLocale: en-US
Platform:
- Windows.Desktop
InstallerType: wix
Scope: user
InstallModes:
- interactive
- silent
- silentWithProgress
UpgradeBehavior: install
Commands:
- upnext
ReleaseDate: $release_date
AppsAndFeaturesEntries:
- DisplayName: UpNext
  Publisher: UpNext contributors
EOF
  [[ -z $product_code ]] || echo "  ProductCode: '$product_code'"
  cat <<EOF
  UpgradeCode: '$upgrade_code'
Installers:
- Architecture: x64
  InstallerUrl: $url
  InstallerSha256: ${sha256^^}
EOF
  [[ -z $product_code ]] || echo "  ProductCode: '$product_code'"
  cat <<EOF
ManifestType: installer
ManifestVersion: $schema
EOF
} >"$out/$id.installer.yaml"

cat >"$out/$id.locale.en-US.yaml" <<EOF
# yaml-language-server: \$schema=https://aka.ms/winget-manifest.defaultLocale.$schema.schema.json

PackageIdentifier: $id
PackageVersion: $version
PackageLocale: en-US
Publisher: UpNext contributors
PublisherUrl: https://github.com/zytocc/upnext
PublisherSupportUrl: https://github.com/zytocc/upnext/issues
PrivacyUrl: https://github.com/zytocc/upnext/blob/main/docs/privacy.md
PackageName: UpNext
PackageUrl: https://upnext.games
License: MIT OR Apache-2.0
LicenseUrl: https://github.com/zytocc/upnext#licence
ShortDescription: Pick what to play next from the games you already own.
Description: |-
  UpNext gathers your game libraries from Steam, Epic, GOG, Amazon, the EA app, Ubisoft Connect and the Xbox app into one list and looks up what each game is like.
  Tell it what you're in the mood for and it suggests a few games from your backlog that fit, with the reasons, from an AI you choose or, without one, ranked by UpNext itself.
  It runs on your own computer, keeps everything in a local database, and has an app window, a web UI, a big-screen mode for a controller and a command line.
Moniker: upnext
Tags:
- backlog
- games
- game-library
- gog
- recommendations
- steam
ReleaseNotesUrl: https://github.com/zytocc/upnext/releases/tag/v$version
Documentations:
- DocumentLabel: Documentation
  DocumentUrl: https://github.com/zytocc/upnext/blob/main/docs/README.md
ManifestType: defaultLocale
ManifestVersion: $schema
EOF

echo "Wrote $out"
