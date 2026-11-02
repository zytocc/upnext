# Changelog

All notable changes to UpNext are listed here. The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and versions follow [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- On Windows, the games the Xbox app installed come in by themselves, Game Pass games included, and Play starts them through Windows. Owned games you haven't installed, and playtime, still come from GOG Galaxy or a list typed in. Sync option: `xbox`.
- One command installs UpNext on Linux: `curl -fsSL https://upnext.games/install.sh | sh`. It picks the right package for your distribution, checks its checksum, and updates UpNext when run again ([Linux](docs/install/linux.md#one-command)).
- An online demo at [demo.upnext.games](https://demo.upnext.games) shows UpNext on a made-up library in your browser, with nothing to install. What you change there stays in that browser tab.

### Changed

- The documentation is shorter and simpler, with fewer pages.
- The Windows programs carry the Visual C++ runtime inside them, so they no longer need the Visual C++ Redistributable installed.

## [0.1.2] - 2026-10-26

### Added

- Library › Add a game adds a game from a store UpNext can't read, such as an emulator or a headset's store. Give it a name and, if you like, a cover, a Play link, hours played and where it's from. Details come from Wikidata, and box art from libretro for emulator games. You can change or remove it from its game sheet. Thanks to avi12 for asking.
- A GitHub Copilot plan can run the AI through GitHub's Copilot CLI: install it, run `copilot login`, then choose "GitHub Copilot" in Settings › Picks. Copilot Free users choose Auto.

### Changed

- Resting the mouse on a pick's art fades through the game's screenshots inside the art, with its top tags and dots to choose one. Keyboard users get the same from a "Preview" button.

### Fixed

- The Claude Code CLI works as an AI backend when it prints its messages as a JSON array, as it does in verbose mode. Thanks to avi12 for the fix.
- A pick answer with a few bad entries no longer fails the pick; the bad ones are skipped. "It's too ___" keeps the AI's swap when one of its extras doesn't fit.
- OpenAI's own endpoint, and routers that pass its strict mode on, take UpNext's schema on the first request.
- The Lists page shows every game of a long list, not only the first 1,000.
- Cancelling a library sync stops it before the next library and ends it as cancelled.
- A Steam account with only free games syncs as an empty library instead of reading as a private profile.
- A GOG sync whose library page comes back incomplete fails and leaves the library as it was.
- Requests run as the right player when their settings can't be read for a moment.
- A sync with nothing to read says "No library to read yet", and games Steam doesn't sell stop showing as missing details.
- Pick requests read lengths only from real numbers and units ("stamina" is no longer a minute), and no longer crash on letters such as "İ".
- Smaller fixes to the game sheet, lineups that run out of games, previews and clearing the library.

### Security

- An uploaded GOG Galaxy database is read in SQLite's defensive mode, so a crafted file can't run more than plain functions.
- A token runs one Settings AI test and one Codex CLI check at a time.
- Updated rustls-webpki, ctutils and tokio for RUSTSEC-2026-0382, -0385 and -0386.

## [0.1.1] - 2026-10-16

### Added

- A Microsoft Store package (MSIX) of the Windows app. It opens the same library as the installer's and keeps it when uninstalled.

### Changed

- A new icon: a d-pad with Up pressed.
- The Flatpak is a file on each release. [Flatpak and Steam Deck](docs/install/flatpak.md) says how to update an installed copy, since `flatpak update` doesn't.

### Security

- Updated rustls for RUSTSEC-2026-0349: someone on the network could push UpNext's HTTPS connections down to classical-only key exchange.

## [0.1.0] - 2026-10-13

The first public release.

- One library from Steam, Epic, GOG, Amazon and GOG Galaxy, plus the EA app's and Ubisoft Connect's installed games on Windows and games added by hand. A game you own twice is one title.
- Details for every game: genres, tags, play length, review scores, what players say and Linux compatibility, from the stores and open game databases.
- Picks: say what you're in the mood for and get a few games that fit, with the reasons. The AI is yours: a Claude Code or Codex subscription, an API key, or a model on your own machine. Without an AI, UpNext ranks the games itself.
- A taste profile you can read and edit, ratings, routines, Second chances, Hidden gems, New to you, Roll the dice and Help me decide.
- Several players on one computer, each with their own marks, picks and taste profile. Sign in with your own identity provider (OpenID Connect).
- Big screen, a layout for a gamepad, a Steam Deck or a TV, and Add to Steam.
- Backlog verdicts: short answers from other players on how a game fits a backlog, shared under CC0. Nothing is sent until you send a verdict.
- UpNext's shared game data, so popular games have summaries from the start.
- Other devices: pair a phone, tablet or browser by tapping Allow on the computer. An Android app is included.
- A web UI, an app window on Linux and Windows, a command line, and an HTTP API under `/api/v1` with an OpenAPI document.
- Install as an `.msi`, `.deb`, `.rpm`, Arch package, Flatpak, container image or from source.

The [README](README.md) and [the documentation](docs/README.md) describe everything in full.
