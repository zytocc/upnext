# The library

## GET /api/games

```json
{"total": 530, "items": ["GameView", "..."]}
```

Every parameter is optional:

- `status`: a comma list, such as `untouched,sampled`, in lower case as [Statuses](statuses.md) writes them.
- `staple`: `true` for your staples only, whatever their status.
- `source`: a comma list, such as `steam,gog`. Any store id works, and one the library has no games from finds none. A title owned on GOG and elsewhere counts as GOG's.
- `installed`: `true` for installed games only.
- `controller`: `true` for games Steam says a gamepad works with, fully or in part. This is big screen's Controller filter.
- `q`: a search, ignoring case. A game matches when the query is part of its name, or when every word of it is found in its name, editions, genres, tags, themes, categories or developers. Name matches sort first.
- `genre`, `tag`: an exact match, one of each. One no game has finds none.
- `linux`: `ok` (native, platinum or gold), `native`, `issues` (silver, bronze or borked) or `unknown` (no data, or pending). See [Game details](details.md#linux).
- `facet`: a review-tag value the title must have, written `name:value` as [`GET /api/review-tags`](review-tags.md#get-apireview-tags) lists them, such as `facet=sessions:short` or `facet=mood:cozy`. Repeat it or give a comma list; a title must have all of them.
- `praised`: an aspect players praised, such as `praised=story`. Repeatable.
- `not_criticized`: an aspect players neither criticized nor split on, such as `not_criticized=grind`. Only titles with review tags qualify. Repeatable.
- `rating`: `rated`, `unrated`, `high` (4 or 5 stars) or `low` (1 or 2). See [Ratings](ratings.md).
- `backlog`: `respects_time` for games most players said respect limited time, or `clicks_fast` for those most said clicked in the first hour. See [Backlog verdicts from others](community-verdicts.md).
- `sort`: `name`, `playtime`, `last_played`, `review`, `hltb`, `release`, `metacritic` or `rating` (your stars; unrated games come last either way). `name` unless asked.
- `order`: `asc` or `desc`; `asc` for name and `desc` for the rest unless asked.
- `limit` (100 unless asked, at most 1000) and `offset` (0).

An unknown status, `linux` value, facet, aspect, `rating` or `backlog` is a `400`, not an empty list. `Shelved`, in the wrong case, is a `400` too.

## GET /api/games/{id}

The `GameView` for a title id, with the fields only the game page has (`tags_by_source`, `from_wishlist`). `404` if there's no such title. A player with a PIN opens the pages of games they hid only after unlocking them ([Hiding games](#hiding-games)).

## Play later

- `PUT /api/games/{id}/later` saves a game to Play later and `DELETE /api/games/{id}/later` takes it off. Both answer with the `GameView`, whose `saved_at` is when it was saved (Unix seconds), or null.
- `GET /api/play-later` gives the saved games, most recently saved first, as `GameView`s.

`GameView.trailer_url` is a YouTube trailer when IGDB knows one (the `trailers` enrich source), otherwise a YouTube search for "<name> trailer".

## Help me decide

`POST /api/versus/decide` with `{"ids": [a, b], "prompt": "optional original request"}` answers `{"pick_id", "reason", "model", "usage", "cost_usd", "api_usd"}`. The AI (the picks' backend and `model`) picks one of the two and explains why in two or three sentences.

`400` unless there are exactly two different ids, or when `prompt` is over 2,000 characters; `404` for an unknown title; `502` when the AI fails; `503` with the reason when no AI is set up. One runs at a time per caller ([Limits](access.md#limits)).

## Hiding games

- `PUT /api/games/{id}/hide` `{"scope": "me" | "others" | "everyone"}` answers `{"scope", "still_seen_by": ["Sam", "Kit"], "through_shared": ["Kit"]}`. `me` takes the game out of your own library, search, picks and statistics. `others` keeps your copies out of every other player's, unless they have it through their own account or another shared one. `everyone` does both. `others` and `everyone` need a copy of your own (`400`). Hiding teaches the taste profile nothing. `still_seen_by` names who still sees it, and `through_shared` those who see it only through another player's shared account. Both are empty for `me`.
- `DELETE /api/games/{id}/hide` answers `204`.
- A `GameView` has `hidden_scope` (`me`, `others`, `everyone` or null) for how the request's player hid it, and `own_copy`, true when one of its copies is in a store account of theirs. Another player's hiding is not shown.
- `GET /api/hidden` lists Hidden by me: `[{"title_id", "name", "scope", "at", "own_copy"}]`, newest first. For a player with a PIN it works only within ten minutes of `POST /api/hidden/unlock` `{"pin"}` in this browser or with this token (`403` otherwise), and the pages of their hidden games open in that time too. `DELETE /api/hidden/unlock` locks it again (`204`).
- A game hidden from a player is named to them nowhere. In jobs, job logs and `GET /api/logs` it reads "a hidden game".
