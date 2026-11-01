# Third-party services

Every service UpNext talks to, what it calls, the terms that apply and what they ask of an app like this one. Requests go from your computer, with your own keys and sign-ins where a service needs them. UpNext's own servers are the shared game data server and the backlog verdicts service; [Privacy](privacy.md) lists what each service is sent.

The facts come from the services' own pages, or archived copies where a page couldn't be loaded. Nothing here is legal advice. Settings › About shows the attributions, and credits sources whose terms ask for none as a courtesy.

- [Stores](third-party/stores.md): Steam, GOG, Epic, their image servers, and the files UpNext reads locally
- [Game data](third-party/data.md): SteamSpy, HowLongToBeat, PCGamingWiki, Wikidata, libretro's thumbnails, ProtonDB, RAWG, IGDB, YouTube and the pricing pages
- [AI](third-party/ai.md): the Claude Code, Codex and Copilot CLIs, the Anthropic and OpenAI APIs, and OpenAI-compatible servers
- [What the project ships and hosts](third-party/project.md): hosting and stores, the Steam copy's Linux libraries, and the Android app's libraries

## Notices

`THIRD_PARTY_NOTICES.txt` at the repository's root lists everything built into UpNext, with versions, licences and the notice texts each asks to be passed on. Every release carries it, and Settings › About links to it.

## Decisions

1. **Steam's store endpoints and the Subscriber Agreement's section 4.C.** UpNext keeps using them, documented here, as every library tool does.
2. **HowLongToBeat.** The scraping stays, off by default, and on only when you turn it on.
3. **GOG's unofficial API.** UpNext uses it, documented as unofficial, as Heroic and Lutris do.
4. **The shared game data.** It publishes only AI-written summaries and review tags derived from Steam reviews, with no review text. That goes against the letter of Steam's and GOG's terms, a risk the project accepts and each release states.

## Publishing data

What the terms say about a CC0 release of summaries and tags made from store reviews:

- **Steam**: content is for "personal, non-commercial use", publishing it needs Valve's written consent, and reviewers license their reviews to Valve only. Summaries UpNext's AI writes are new text, but they're derived from that content, and the contract terms apply whatever the copyright.
- **GOG**: personal use only, and the ban on "extraction tools".
- **SteamSpy**: it publishes no licence, so nothing is granted, and its tags come from Steam's user tags.
- **Can't go into a CC0 release at all**: PCGamingWiki (CC BY-NC-SA), ProtonDB's data (ODbL), RAWG ("No data redistribution"), IGDB (no redistribution without an agreement) and HowLongToBeat.
- **Fine**: Wikidata (CC0).
