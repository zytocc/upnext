# What the project ships and hosts

These host what the project publishes, or are built into what it ships. The app doesn't talk to them.

## Hosting and stores

- **GitHub** hosts the source, the releases, the container image (`ghcr.io`) and CI.
- **Cloudsmith** hosts the apt and dnf package repositories, under its [open-source hosting policy](https://docs.cloudsmith.com/open-source-hosting-policy): free for a public repository with an SPDX licence, in return for a credit that links back to Cloudsmith wherever the repositories are offered. Installs fetch packages and metadata from `dl.cloudsmith.io`, as with any distribution mirror.
- **F-Droid**: the recipe in `contrib/fdroid/` has F-Droid build the Android app from the tagged source. Its [inclusion policy](https://f-droid.org/docs/Inclusion_Policy/) asks for free software, no proprietary libraries and declared anti-features; the app uses nothing from Google Play services and has no anti-features. Google's developer verification applies to the APK either way.
- **Flathub** hosts GNOME's runtime and SDK and the freedesktop SDK's Rust and Node extensions that the Flatpak builds and runs on. UpNext itself isn't on Flathub: each release carries the Flatpak as a bundle, and installing it fetches the runtime from Flathub. The build uses flatpak-builder-tools' generators (MIT) at a pinned commit, and none of them is in the repository.
- **Steam**: the build files in `contrib/steam/` upload UpNext's Steam copy, free and listed under Player Tools, through SteamPipe. The project's Steamworks account takes the [Steam Distribution Agreement](https://partner.steamgames.com/doc/gettingstarted), and the store page follows Valve's [content rules](https://partner.steamgames.com/doc/gettingstarted/contentsurvey), with the content survey's AI disclosure (`contrib/steam/listing/content-survey.md`). The copy carries nothing of the Steamworks SDK.

## The Steam copy's Linux libraries

The Linux depot carries WebKitGTK, GTK, GLib, GStreamer's core and the libraries they need, copied from the Ubuntu 22.04 packages the release's programs are built against ([the Steam build](../../contrib/steam/README.md)). They're unchanged apart from each file's RPATH and the helper programs' folder inside `libwebkit2gtk-4.1`.

Their licences are the LGPL (WebKitGTK, GTK, GLib, GStreamer, libsoup, cairo, pango and others), licences that offer the LGPL or the GPL (GnuTLS's nettle and GMP, elfutils' libdw), the GCC runtime exception (libatomic), and permissive licences. The depot keeps:

- each package's licence in `share/licenses/<package>/copyright`;
- the full texts of the LGPL, the GPL and the other common licences, which those files name only by their path on an Ubuntu system, in `share/licenses/common-licenses/`;
- every package with its version in `share/licenses/PACKAGES.txt`.

Every release with a Steam build carries their source, the Ubuntu source packages of exactly those versions, as `upnext-steam-linux-sources.tar`, which `PACKAGES.txt` names with its address. They stay separate shared libraries that can be swapped for others, as the LGPL asks. Both depots also carry UpNext's own licences and `THIRD_PARTY_NOTICES.txt` in their top folder.

## The Android app's libraries

The libraries built into the APK are listed in `android/gradle/libs.versions.toml`, all under the Apache License 2.0, which asks that a copy of the licence and any NOTICE go with the app. The APK carries them, with the licence text, under "Open-source licences" on the app's first screen, and none needs a credit in Settings › About.
