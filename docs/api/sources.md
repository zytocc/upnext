# Sync and enrich sources

## GET /api/sync/sources

The libraries a sync reads, in order: `[{"id": "steam", "label": "Steam", "blocked": null}, {"id": "gog", "label": "GOG account", "blocked": "not connected"}]`. The ids are `steam`, `heroic`, `gog` (blocked without a GOG sign-in), `epic` (blocked without an Epic sign-in or the Epic Games Launcher), `ea`, `ubisoft` and `xbox` (blocked when nothing they installed is found; Windows only) and `galaxy` (blocked without `galaxy_db`).

## Enrich options

One boolean per source, keyed by its id, plus `refresh` (fetch again what titles have already), `limit` (at most this many games) and `titles` (a list of title ids to limit the run to). A source left out takes its default, and an unknown key is ignored. Enrich always runs the Steam-store name match for non-Steam games.

`GET /api/enrich/sources` lists them in run order: `[{"id": "hltb", "label": "HowLongToBeat", "default": false, "note": "slow", "blocked": null}]`. `note` is a word on cost ("slow", "uses the AI"), and `blocked` says why the settings keep a source from running ("needs a key", "needs an IGDB login", "Linux mode is off"). The ids are `wikidata`, `series`, `art`, `reviews`, `review_texts`, `tags`, `protondb`, `gog_meta`, `igdb`, `rawg`, `pcgw`, `hltb` (off by default), `libretro` (box art for [games added from an emulator](added-games.md#look-ups)), `trailers`, `digests` (off), `series_guides` (off), `second_chances` (off), `hidden_gems` (off) and `wishlist` (off; its own task).

How an enrich runs is on [Jobs](jobs.md#the-job). Schedules for them are on [Scheduled jobs](schedules.md).
