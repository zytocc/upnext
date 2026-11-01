# Wishlist

Games you don't own: suggestions from the series you've played, games you add by name, and your Steam wishlist, read only. Everything here is behind the `wishlist` setting (on by default). With it off there are no suggestions, no Steam wishlist read and no store data fetched, and `GET /api/wishlist` and `from_wishlist` are empty. What's stored stays until it's deleted from [What UpNext knows about you](taste-sources.md). The AI's suggestion in a lineup is on [Wishlist in picks](wishlist-picks.md).

## Where rows come from

- **Your series** (`source: "collection"`): entries of a series you don't own, where the series means something to you (many hours played, or one of its games beaten or a favourite). The reason names that history. Most-played series first, with a few suggestions per series and a cap in all.
- **Your Steam wishlist** (`source: "steam"`, `on_steam: true`), read for your SteamID. A game on it is never a suggestion: a suggestion that turns up there is wishlisted. A game taken off it on Steam loses `on_steam`, and one that only came from Steam is dismissed. UpNext never writes to Steam.
- **By name** (`source: "manual"`), from `POST /api/wishlist`, found with Steam's store search.
- **A lineup** (`source: "pick"`, `batch_id` the lineup): the AI's suggestion of a game you don't own.

**Eligibility.** A suggestion, from a series or a lineup, is shown only when:

- it's a game on Steam's store;
- 80% or more of at least 50 Steam reviews are positive;
- you don't own it on any store UpNext syncs (by Steam appid, then by name with editions folded in);
- you haven't answered it (wishlisted, Not interested, I own it elsewhere);
- it isn't a hard no.

## The `wishlist` task

"Update the wishlist" runs after Fetch details. It takes games now in the library off the wishlist, reads the Steam wishlist, checks your series, then fetches what each row lacks (store details, reviews, tags, art, a HowLongToBeat time). No AI is used. A sync, and `GET /api/wishlist`, also take games now in the library off the wishlist.

## GET /api/wishlist

```json
{"enabled": true, "suggested": [WishlistEntry], "wishlist": [WishlistEntry],
 "dismissed": 3, "owned_elsewhere": 1, "in_library": 2, "steam_account": true,
 "task": {"schedule": "after:details", "enabled": true, "last_run": "Job", "next_run_at": null},
 "rules": {"series_hours": 10, "min_positive": 80, "min_reviews": 50, "per_series": 3}}
```

`suggested` wait for an answer, best rated first; `wishlist` is yours and Steam's, newest first. A `WishlistEntry`:

```json
{"id": 4, "appid": 420, "name": "Half-Life 2: Episode Two", "source": "collection", "reason": "You played …",
 "state": "suggested", "on_steam": false, "series_key": "Q…", "series": "Half-Life", "batch_id": null, "title_id": null,
 "hltb_main": 5.5, "added_at": 1790000000, "updated_at": 1790000000, "wishlisted_at": null,
 "type": "game", "short_description": "…", "genres": ["Action"], "developers": ["Valve"], "release_date": "Oct 10, 2007",
 "header_image": "https://…", "portrait": "https://…", "review_pct": 97, "review_total": 31000,
 "review_desc": "Overwhelmingly Positive", "tags": ["FPS", "Action", "Sci-fi"]}
```

States: `suggested`, `wishlisted`, `dismissed` (Not interested: never suggested again), `owned_elsewhere` (I own it elsewhere: counts as owned from then on) and `in_library` (a sync found it, with its `title_id`). `wishlisted_at` is when it went on the wishlist, here or on Steam.

## POST /api/wishlist

`{"id": 4}` puts a row on the wishlist (a suggestion, or one said no to before). `{"name": "Hades II"}` finds a game on Steam's store and adds it, or wishlists the row it has already, and fetches its store data and art at once. Answers with the `GET` body.

`400` for neither, a name Steam's store doesn't know, or a game you own ("Hades II is in your library already, as Hades II"); `404` for an unknown id; `409` for a game in your library now, or with the setting off; `502` when Steam's store doesn't answer the search.

## PUT /api/wishlist/{id}

`{"state": "wishlisted" | "dismissed" | "owned_elsewhere" | "suggested"}`. Answers with the `GET` body. The UI's Remove is `dismissed`. `400` for another state (`in_library` comes only from a sync); `404` and `409` as above.

## DELETE /api/wishlist/{id}

Forgets the row, so a series may suggest it again, and the next read of your Steam wishlist brings it back if it's there. Answers with the `GET` body; `404` for an unknown id. It works with the setting off.

## Elsewhere

`GET /api/games/{id}` carries `from_wishlist` for a game that came into the library off the wishlist: `{"reason": "…", "source": "collection", "wishlisted_at": 1790000000}`. The game sheet says "From your wishlist (Aug 23): <reason>". Merging titles moves a row's `title_id` to the title that stays.
