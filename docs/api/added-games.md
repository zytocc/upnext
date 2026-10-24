# Games added from no store

[API index](../API.md) · [More stores](more-stores.md) · [Games](games.md#sources-and-launchers) · [Sync and enrich sources](sources.md)

Some games come from no store UpNext reads: a PlayStation game in an emulator, a game played only with a mod, a VR game from a headset's own store. Each one is added by hand, one at a time, and joins the player's library as an owned copy with `source` `other`, `launcher` `manual` and a `source_id` of its own (`added-…`). The lists of titles for the EA app and the other stores without a sign-in are [another route](more-stores.md#games-added-by-hand).

## The game

`POST /api/library/added` and `PUT /api/library/added/{id}` take the same body:

```json
{"name": "Demon's Souls", "from": "RPCS3", "cover_url": "https://example.org/cover.png",
 "play_url": null, "hours": 12.5, "last_played": "2026-10-20", "installed": true, "look_up": true}
```

- `name`: needed, at most 200 characters. Spaces are tidied and ™, ® and © signs dropped.
- `from`: where it's from, in the player's words ("RPCS3", "Meta Quest", "Half-Life RTX"), at most 60 characters, or null. The AI's list of a game's stores shows it, and an emulator's or console's name finds its box art (below).
- `cover_url`: an `https` link to a cover, or null.
- `play_url`: a launcher's link that starts the game, or null. Only `steam`, `heroic`, `goggalaxy`, `com.epicgames.launcher` and `uplay` links are taken, the ones the desktop app opens: nothing that runs a program or opens a file, since anyone who may change the library sets it. A Steam shortcut's `steam://rungameid/…` link works for an emulator added to Steam.
- `hours`: hours played, 0 to 100,000, or null when unknown. They're kept as minutes.
- `last_played`: a date, `YYYY-MM-DD`, not in the future, or null.
- `installed`: false unless given.
- `look_up`: true unless given false. It says whether fetching details looks the game up by name.

`400` says what's wrong: no name, a field too long, a cover that isn't `https`, another kind of Play link, a date that isn't one, or the same name from the same place already added.

## Routes

- `GET /api/library/added` gives this player's games added this way, newest first: `{"games": [{"copy_id": 812, "title_id": 640, "name": "Demon's Souls", "from": "RPCS3", "cover_url": null, "play_url": null, "minutes": 750, "last_played": 1792497600, "installed": true, "look_up": true, "steam_matched": false}]}`. `last_played` is a Unix time (noon UTC on the date given), 0 when there's none. `title_id` is the title it's grouped into, for [`GET /api/games/{id}`](games.md).
- `POST /api/library/added` adds a game and answers `201` with `{"game": {…}, "looking_up": true}`. With `look_up` on, an enrich job for that one title starts; `looking_up` is false when another job was already running, and the next fetch of details finds it.
- `PUT /api/library/added/{id}` changes every field of the copy `id` (`copy_id`) and answers `200` in that shape, or `404` when it isn't a game this player added. A new name, or `look_up` turned off, drops the Steam match and what the look-ups by name found for the old name (when no other copy shares the title), so a wrong match goes with "Not this game". A new name or place also drops a libretro cover.
- `DELETE /api/library/added/{id}` takes it out of the library: `204`, or `404`. Marks, ratings and diary entries stay with the title, as for a game a store stops reporting.

In the games' own lists, a copy has `from` (null for every store but `other`) and `look_up` (true but for a game kept from look-ups). A title whose every copy has `look_up` false and no Steam app id is left out of fetching details.

## Look-ups

With `look_up` on, fetching details treats the game as any non-Steam game: a match by name on the Steam store, then Wikidata and PCGamingWiki by name, and HowLongToBeat, IGDB and RAWG where they're on. A game Steam doesn't sell gets two more, with no keys:

- **Genres and year from Wikidata.** A game Wikidata found by name (and Steam didn't) gets its genres (P136) and earliest release year (P577), asked once a game.
- **Box art from libretro** (enrich option `libretro`, "Box art from libretro", on by default). When `from` names an emulator or console, the libretro thumbnail server's list of box art for that system is read (kept 30 days in the database) and a cover with the same name is used, the USA one first. Names match without their brackets ("(USA)", "[!]") and with "The" moved to the front. A game with a cover link of its own, or a Steam match, isn't looked up there.

`from` names a system when it holds, as whole words, an emulator's name or a console's: RPCS3 or PS3, shadPS4 or PS4, PCSX2 or PS2, DuckStation or PS1, PPSSPP or PSP, Vita3K or Vita, Dolphin or GameCube (and Wii), Cemu or Wii U, mGBA or Game Boy Advance, and the Nintendo, Sega, Atari and NEC consoles libretro has folders for. Up to three of libretro's folders are tried, in the order named. The full table is `SYSTEMS` in `src/libretro.rs`.
