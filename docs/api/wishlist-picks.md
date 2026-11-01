# Wishlist in picks

With the `wishlist` setting on, a lineup the AI picks from the whole library (not New to you or Hidden gems, and not one picked without AI) can show two things beyond your library.

## A wishlisted pick

The AI is offered some of your wishlisted games (here or on Steam) that you don't own and that aren't a hard no. At most one may stand in for one of the picks, and that pick becomes its `alternate`, so switching needs no second call. In the lineup it's a pick like the others:

```json
{"id": -12, "appid": 1030300, "name": "Hollow Knight: Silksong", "reason": "…", "game": null, "slot": 1, "feedback": null,
 "owned": false,
 "wishlist": WishGame,
 "alternate": {"id": 1145360, "name": "Hades", "reason": "…", "game": "GameView"}}
```

- `id` is minus its wishlist entry's id, since no title has a negative id.
- A `WishGame` is what the card shows (the wishlist row's name, store text, art, review figures and tags). Once its wishlist entry is deleted, the pick keeps a `WishGame` made from its own name and appid.
- A wishlisted pick that doesn't hold up is dropped and the lineup stands as picked, with no second call.
- Its card's one action is the switch: `feedback` and `too` answer `400` for it. New spares never list the wishlist.

### POST /api/picks/{batch_id}/switch

`{"title_id": -12}`, the wishlisted pick's `id`. Its `alternate` takes the slot at once, with no AI call. When the alternate can't go on screen any more, the best usable spare does, as for Not tonight. The wishlisted pick moves to `removed` with `owned: false` and no `feedback`. Answers with the lineup. `400` for a game you own or one with no alternate; `404` for an unknown lineup or a game not on screen.

## The suggestion

The pick call may name one game you don't own, from the AI's own knowledge, when it fits the request and your taste unusually well. UpNext checks it against the [eligibility rules](wishlist.md#where-rows-come-from) before it's shown, within a short time limit. A suggestion that fails a check, or that Steam doesn't answer for in time, isn't shown and nothing is stored.

Once it passes, it waits on the wishlist as a suggestion (`source: "pick"`, `batch_id` the lineup), with its store data and art fetched, and the lineup carries it:

```json
"suggestion": {"game": WishGame, "reason": "Not in your library, but …"}
```

`game.state` is where it stands now, so a lineup read later shows the answer given since (`PUT /api/wishlist/{id}` answers it, as on the Lists page). `suggestion` is null otherwise, and always for lineups picked without AI or in a mode.
