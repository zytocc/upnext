# Games added by hand

Two ways bring in games from no store UpNext reads. A game is added one at a time (an emulator game, a mod, a VR game from a headset's own store), or a store without a usable sign-in gets a list of titles typed or pasted in. Either way they join the player's library as owned copies with launcher `manual`.

## One game at a time

An added game has `source` `other`, `launcher` `manual` and a `source_id` of its own (`added-…`).

`POST /api/library/added` and `PUT /api/library/added/{id}` take the same body:

```json
{"name": "Demon's Souls", "from": "RPCS3", "cover_url": "https://example.org/cover.png",
 "play_url": null, "hours": 12.5, "last_played": "2026-10-20", "installed": true, "look_up": true}
```

- `name` is needed, at most 200 characters.
- `from` is where it's from, in the player's words ("RPCS3", "Meta Quest"), at most 60 characters, or null. An emulator's or console's name finds its box art.
- `cover_url` is an `https` link, or null.
- `play_url` is a launcher link that starts the game, or null. Only `steam`, `heroic`, `goggalaxy`, `com.epicgames.launcher` and `uplay` links are taken. A Steam shortcut's `steam://rungameid/…` link works for an emulator added to Steam.
- `hours` is 0 to 100,000, or null when unknown.
- `last_played` is `YYYY-MM-DD`, not in the future, or null.
- `installed` is false unless given.
- `look_up` is true unless given false. It says whether fetching details looks the game up by name.

`400` says what's wrong: no name, a field too long, a cover that isn't `https`, another kind of Play link, a bad date, or the same name from the same place already added.

### Routes

- `GET /api/library/added` gives this player's games added this way, newest first: `{"games": [{"copy_id": 812, "title_id": 640, "name": "Demon's Souls", "from": "RPCS3", "cover_url": null, "play_url": null, "minutes": 750, "last_played": 1792497600, "installed": true, "look_up": true, "steam_matched": false}]}`. `last_played` is 0 when there's none. `title_id` is the title it's grouped into, for [`GET /api/games/{id}`](games.md).
- `POST /api/library/added` adds a game and answers `201` with `{"game": {…}, "looking_up": true}`. With `look_up` on, an enrich job for that one title starts. `looking_up` is false when another job was already running, and the next fetch of details finds it.
- `PUT /api/library/added/{id}` changes every field of the copy `id` (`copy_id`) and answers `200` in that shape, or `404` when it isn't a game this player added. A new name, or `look_up` turned off, drops the Steam match and what the look-ups found for the old name.
- `DELETE /api/library/added/{id}` takes it out of the library: `204`, or `404`. Marks, ratings and diary entries stay with the title.

In the games' own lists, a copy has `from` (null for every store but `other`) and `look_up`. A title whose every copy has `look_up` false and no Steam app id is left out of fetching details.

### Look-ups

With `look_up` on, fetching details treats the game as any non-Steam game: a match by name on the Steam store, then Wikidata, PCGamingWiki, HowLongToBeat, IGDB and RAWG where they're on. A game Steam doesn't sell also gets genres and a year from Wikidata, and, when `from` names an emulator or console, box art from libretro (enrich option `libretro`, on by default). A game with a cover link of its own, or a Steam match, isn't looked up there.

## Lists of titles for stores without a sign-in

The EA app, Ubisoft Connect, Battle.net and the Xbox app have no sign-in a program like UpNext may use, so a list of titles typed or pasted in brings their games in. Each title is an owned, uninstalled copy with the store's `source` (`origin`, `uplay`, `battlenet` or `xboxone`), `launcher` `manual` and the title's own key as `source_id`. Enrich finds its details on Steam by name.

- `GET /api/library/by-hand` gives `{"stores": [{"store": "origin", "label": "EA app", "titles": ["Dead Space", "Mass Effect Legendary Edition"]}, {"store": "uplay", "label": "Ubisoft Connect", "titles": []}]}`: every such store, in that order.
- `PUT /api/library/by-hand/{store}` with `{"titles": ["Dead Space", "Far Cry 5\nAnno 1800"]}` replaces the store's list and answers `{"total": 3, "added": 2, "removed": 1, "lists": {…}}`. Titles are one a line (an entry may hold several lines). Blank lines, bullets, ™, ® and © signs and repeats are dropped. A title left off leaves the library, and `{"titles": []}` takes them all out. `400` for any other store or more than 5,000 titles.
