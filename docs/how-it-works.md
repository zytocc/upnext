# How it works

1. **Sync** brings in your games from each store. Steam brings playtime. Heroic doesn't track it, so its games count as untouched until you mark them.
2. **Titles** group a game's copies across stores, so remasters and editions become one game. You can merge and split by hand.
3. **Fetch details** adds store details, review scores, tags, player reviews and, on Linux, ProtonDB tiers from Steam, GOG, SteamSpy, Wikidata and PCGamingWiki, and from IGDB and RAWG with your keys.
4. **The AI's summaries** (optional) read each game's player reviews and write "People say" and "Vibes", plus review tags such as difficulty, pace and session length.
5. **Status**: untouched, sampled (under 2 hours), playing, shelved (has an ending, under 80% through, untouched for 60 days), abandoned (shelved for 2 years) or played, plus what you mark yourself. The thresholds are settings.
6. **Picks**: your untouched, sampled and shelved games are ranked by reviews, the words and time in your request, and your taste profile. The best 120 or so go to the AI, which chooses from them. Without an AI, picks come from the ranking alone.
7. **Playing**: a pick links straight into Steam, Heroic, Galaxy, the Epic Games Launcher, Ubisoft Connect or, from the app window on Windows, the Xbox app.

## Other ways to pick

Ask for the first two in a request; the others have their own place in the app:

- **New to you** ("new to me"): short, gentle games in a genre or mechanic you've barely played, one genre per pick.
- **A hidden gem** ("a hidden gem for tonight"): short, highly rated games the AI judged unusual, for their studio, their mechanics or how they play. The Lists page's Find hidden gems rates them first.
- **Second chances**: games you abandoned, with what players have said about them since.
- **Roll the dice**, beside Curate: one game you haven't started, or haven't played in a year, drawn at random with no AI.

## The play diary

Turn on the play diary (Profile, under Where it comes from) and UpNext notes which Steam games gained playtime each hour, and each time you press Play. Your profile then shows your habits, such as how long your weeknight sessions run, and picks use them.

[The HTTP API](API.md) has the details of each step.
