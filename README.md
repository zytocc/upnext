# UpNext

**Pick what to play next from the games you already own.**

Launchers like Playnite and Heroic gather your games in one place. UpNext tells you which one to play tonight.

Website: [upnext.games](https://upnext.games)

<img src="docs/images/pick.png" width="800" alt="The Pick page: a request, and a lineup of three games with why each fits and what players say about it">

*The Pick page on the made-up library from `upnext mock-library`. The cover art belongs to the games' publishers.*

## How it works

1. UpNext reads your libraries from Steam, Epic, GOG, Amazon, the EA app, Ubisoft Connect and the Xbox app, and merges them into one list.
2. It looks up what each game is like: genres, length, review scores and what players say.
3. You say what you're in the mood for, such as "about an hour, something calm with a bit of story". It suggests a few games from your backlog that fit, and says why.

The suggestions come from an AI you choose: your Claude or ChatGPT subscription, an API key, or a model on your own machine. Without an AI, UpNext ranks your games itself.

It runs on your own computer and keeps everything in a local database. There are no accounts and no analytics; [Privacy](docs/privacy.md) lists every service it talks to.

## Try it

[The online demo](https://demo.upnext.games) runs in your browser on a made-up library, with nothing to install.

With Docker:

```sh
git clone https://github.com/zytocc/upnext && cd upnext
docker compose -f contrib/docker/compose.yaml up -d
docker logs upnext      # shows the address and a code to pair your browser
```

Or install it:

| Platform | Get it |
|---|---|
| Windows 10 and 11 | [The installer](https://github.com/zytocc/upnext/releases/latest/download/upnext-x86_64-pc-windows-msvc.msi) ([guide](docs/install/windows.md)) |
| Linux | `curl -fsSL https://upnext.games/install.sh \| sh` ([guide](docs/install/linux.md), with `.deb`, `.rpm`, Arch and archive downloads) |
| Steam Deck, any Linux | [The Flatpak](docs/install/flatpak.md) |
| Home server, NAS | [The container image](docs/install/docker.md) |
| Android | [The app](docs/install/android.md), to use UpNext on your computer from a phone or TV |
| From source | [Rust and Node](docs/install/source.md) |

The first time it opens, a short setup asks for your stores and your AI. `upnext doctor` prints the paths it uses and anything missing.

## What else it does

- **One library.** A game you own on two stores shows once.
- **Picks that fit.** Ask for a length, a mood or "something I can actually finish". Turn a lineup down and say why, rate the picks, or have it decide between two games.
- **A taste profile you can read and edit.** Favourites, likes and hard nos, each with where it came from, plus routines for nights with friends or on the couch.
- **More ways in.** Second chances for games you abandoned, hidden gems, genres new to you and a roll of the dice.
- **Everywhere in the house.** A web UI, an app window, a big-screen mode for a controller, an Android app and a command line.

[The documentation](docs/README.md) covers everything else.

## Contributing

Bug reports, ideas and pull requests are welcome. [CONTRIBUTING.md](CONTRIBUTING.md) has the setup and the checks; [SECURITY.md](SECURITY.md) says how to report a vulnerability privately.

UpNext is developed with AI coding assistants, mostly Claude Code. Every change goes through the same tests and CI as any other.

## Code signing policy

Free code signing provided by [SignPath.io](https://about.signpath.io/), certificate by [SignPath Foundation](https://signpath.org/).

Windows releases are signed from the first release SignPath Foundation accepts; until then they aren't signed. Only files the `release` workflow (`.github/workflows/release.yml`) builds from this repository are signed, and each is published with its SHA-256 checksum.

- **Committers and reviewers:** [zytoc](https://github.com/zytoc). Every change from anyone else, such as a pull request, is reviewed by a committer before it's merged.
- **Approvers:** [zytoc](https://github.com/zytoc), who approves each release's signing by hand.

**Privacy policy:** UpNext has no accounts and no analytics. [Privacy](docs/privacy.md) lists every service it talks to and what each is sent.

## Licence

UpNext is licensed under either of the [Apache License, Version 2.0](LICENSE-APACHE) or the [MIT licence](LICENSE-MIT), at your option. Unless you say otherwise, any contribution you submit for inclusion in UpNext is licensed the same way.

Game data comes from the services in [Third-party services](docs/THIRD_PARTY.md), under their own terms. UpNext isn't affiliated with Valve, GOG, Epic, Amazon or any of those services. The libraries, fonts and icons built into UpNext keep their own licences, listed in [THIRD_PARTY_NOTICES.txt](THIRD_PARTY_NOTICES.txt). Questions are welcome in the [GitHub issues](https://github.com/zytocc/upnext/issues).

## Supporting UpNext

UpNext is free and stays free. If you'd like to support the work, you can on [Ko-fi](https://ko-fi.com/zytocc), or through the Sponsor button at the top of the repository.
