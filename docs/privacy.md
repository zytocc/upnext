# Privacy

Everything UpNext knows is in its database on your computer ([where it is](running.md#where-it-keeps-things)). There's no account and no analytics. UpNext's own servers are the shared game data server, which your computer downloads from at setup and when you ask, and the backlog verdicts service, which hears from your computer only when you publish a verdict or while verdicts from other players are on (the setup wizard turns them on; Settings › Library data turns them off). These are the services UpNext talks to, and what each gets:

| Service | When | What it's sent |
|---|---|---|
| Steam Web API | Sync, the play diary (hourly, if on), the wishlist | Your Steam API key and SteamID; it answers with your games, playtime, recent play, profile name and wishlist |
| Steam sign-in | When you choose Sign in with Steam | Your browser goes to Steam; UpNext gets back your SteamID |
| Steam store | Fetching details | Steam app ids, and the names of games from other stores to find them on Steam |
| SteamSpy, ProtonDB | Fetching details (ProtonDB only on Linux) | Steam app ids |
| GOG | Sync, once you connect your account; fetching details and series covers, with no account | Your GOG sign-in, for sync; GOG product ids, and series games' store-page names to find their ids |
| Epic Games | Sync, once you sign in to Epic | Your Epic sign-in; it answers with your games, their store details and your playtime |
| Wikidata, PCGamingWiki, libretro | Fetching details and series | Game names, Steam app ids and console names |
| IGDB, RAWG | Fetching details, with your own credentials | Your credentials, game names |
| HowLongToBeat | Only when you turn it on | Game names |
| Your identity provider | Only if you set one up, when someone signs in with it | Your browser goes to it; UpNext reads its public settings and keys and sends it the sign-in's code with UpNext's client id and secret; it answers with who signed in |
| UpNext's shared game data (`https://data.upnext.games`, or another address you give) | Once during setup, unless you untick it, and when you ask for it again (Settings › Library data, or `upnext seed apply`) | Nothing about you; like any request, it shows the server your address and UpNext's version |
| UpNext's backlog verdicts service (`https://verdicts.upnext.games`, or another address in `verdicts_url`) | Daily, if verdicts from other players are on; and when you publish, withdraw or report a verdict | The daily download sends nothing about you; like any request, it shows the service your address and UpNext's version. A verdict you publish goes with your player's random key, never your name, accounts or library, and is published under CC0 ([Backlog verdicts](backlog-verdicts.md)) |
| Steam, GOG, IGDB, Epic and libretro image servers, and the Anthropic, OpenAI and OpenRouter pricing pages | Cover art on a page; prices weekly | Your browser loads the images directly, so the image servers see your IP address. The pricing pages get nothing about you |
| **The AI you chose** | When you ask for picks, or run an AI job | Below |

## What goes to the AI

UpNext sends the AI only what the task needs, and never your keys, SteamID, account names, sign-ins or file paths.

- **Picks**: your request, about 120 candidate games with their details and your playtime, your taste profile and ratings, and earlier feedback. Run `upnext prompt "…"` to see it all.
- **Review summaries**: each game's name and public player reviews from Steam and GOG.
- **Series guides**: a series' name and its games, each with its year, length, review score and how long you played it.
- **Second chances**: an abandoned game, what you said about why you stopped, and player reviews written since.
- **Hidden gems**: unplayed games with their details, tags, publishers and review summaries.

With an AI CLI, the call goes through that CLI on your plan, and Copilot's index of past sessions lists UpNext's calls. With an API key, it goes to Anthropic or OpenAI under your key. With a local server, nothing leaves your machine.

On your network, UpNext announces its name, an install id, its port and its version while other devices are allowed (Settings › Devices and API turns that off), and the Android app sends a device's name and kind to a computer it asks to be allowed on.

[Third-party services](THIRD_PARTY.md) lists every service with its terms, and [SECURITY.md](../SECURITY.md) covers who can reach the server.
