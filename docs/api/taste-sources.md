# What UpNext knows about you

The profile page's "What UpNext knows about you" lists every source the profile reads, shows each one and deletes it.

## GET /api/taste/sources

```json
{"library": {"games": 2000, "played": 400, "hours": 3500, "stores": ["steam", "gog", "epic"], "last_sync": 1790000000},
 "statuses": {"games": 12, "by": {"done": 5, "hidden": 7}},
 "reactions": {"picks": {"chosen": 4, "not_tonight": 9, "too": 3}, "pick_ratings": {"good": 7, "not_a_fit": 3},
               "stop_reasons": 6, "second_chance_answers": 1},
 "ratings": {"games": 3, "by": {"5": 1, "2": 2}},
 "answers": {"favourites": 8, "likes": 3, "dislikes": 1, "hard_nos": 2, "habits": 2, "routines": 1, "not_really": 1},
 "diary": {"on": true, "rows": 28, "days": 16, "launches": 3, "since": 1789239600, "last_reading": 1790888400, "error": null, "paused": null},
 "wishlist": {"on": true, "steam": 4, "added": 2, "suggested": 3, "answered": 5, "bought": 1}}
```

- `reactions.picks` counts what was done on pick cards, by action; `pick_ratings` the picks you rated, by verdict; `stop_reasons` the games with reasons.
- `diary` is the [play diary](diary.md). `paused` says why it's on but not reading (no Steam Web API key or Steam ID, or schedules turned off).
- `wishlist` counts the [wishlist](wishlist.md) rows by where they came from and what became of them.

## GET /api/taste/sources/{source}

- `reactions`: every reaction, newest first. `what` is a card action (`chosen`, `not_tonight`, `too`, `same_problem`, `not_for_me`, `started`), `good_pick` or `not_a_fit`, `reasons` or `second_chance`; `text` carries the reason or answer.
- `ratings`: every rating you gave a game, newest first.
- `diary`: every row of the play diary, newest first. `source` is `steam` (minutes played in that hour) or `launch` (the Play button, `minutes` 0).
- `404` for the others, which have pages of their own (Library, Lists) or are the profile page.

## DELETE /api/taste/sources/{source}

Answers `{"deleted": n}`, then counts the patterns again.

- `statuses`: every mark you set is cleared.
- `reactions`: every card action and pick rating, every set of reasons for stopping, and every Second Chance answer (the marks those answers set stay). Games turned down in picks can come back to them.
- `ratings`: every game's rating. Pick ratings stay, with the reactions.
- `answers`: everything you stated, your routines, and which patterns you said weren't right.
- `diary`: every row, and the last reading the next one counts from. It stays on if it was on.
- `wishlist`: every row. Your Steam wishlist comes back at the next update while the wishlist is on.
- `library`: `400`. It comes from your stores at each sync.

## GET /api/taste/export

The whole profile as a file (`upnext-profile-YYYYMMDD.json`): your statements, statuses, reactions, play diary, `ratings` and `routines`. A token bound to a player gets that player's.

## Settings

`pattern_half_life_days` (365, 1 to 3650) and `pattern_min_games` (4, 2 to 50), in [Settings](settings.md).
