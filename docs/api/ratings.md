# Ratings

Two ratings, each kept apart from what you did with a game. A pick's rating says how well the suggestion fit what its lineup was asked for. A game's rating is your own, 1 to 5 stars.

## PUT /api/picks/{batch_id}/rating

`{"title_id": 1234, "verdict": "not_a_fit", "reason": "wrong_length"}`. `verdict` is `good` or `not_a_fit`, or `null` to take the rating back. `reason` goes with `not_a_fit` only and is optional: `wrong_length`, `wrong_mood`, `wrong_genre` or `played_out`. Answers with the lineup.

A rating isn't a card action: the pick stays in its slot. Any pick of the lineup can be rated, spares and picks taken off included. Each pick has `rating`, `{"verdict", "reason", "at"}`, or null.

`400` for an unknown verdict or reason, or a reason with `good`. `404` for an unknown lineup or a game that isn't in it.

## PUT /api/games/{id}/rating

`{"rating": 4}`, 1 to 5, or `null` to clear it. Answers with the `GameView`, whose `my_rating` is the rating and `rated_at` when it was given. `400` outside 1 to 5; `404` for no such title.

`GameView.pick_rating` is the latest rating the game had as a pick, from any lineup: `{"verdict", "reason", "at", "prompt"}`, where `prompt` is what that lineup was asked for (empty for none), or null.

## How picks read them

- **Game ratings** are part of the taste profile and never fade. About me lists the games you rated 4 or 5 and 1 or 2, newest first; a 3 says neither.
- **In the ranking**, a rated candidate moves up or down with its stars. A game you rated 4 or 5 counts like a favourite for likeness, and one you rated 1 or 2 counts against candidates like it. `ranked_by` and the reasons say so.
- **Pick ratings** count for 90 days, since they were about a request. The AI is told how recent picks landed and leans toward what worked for requests like this one. In the ranking, a game rated not a fit in that time ranks lower; a good pick changes nothing there.

## Elsewhere

- Merging titles keeps the surviving title's rating, or takes the other's when it has none. Pick ratings follow their games.
- `GET /api/games` takes `rating` and `sort=rating` ([The library](library.md)), and `GET /api/stats` gives `rated`. `upnext list --sort rating` sorts the same way.
- [Taste sources](taste-sources.md) list, delete and export them.
