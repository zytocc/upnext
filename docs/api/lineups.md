# Lineups

## Which games picks offer

Picks offer `untouched`, `sampled` and `shelved` games, `still_playing` games and endless or staple `played` ones (pitched as a session, and ranked lower unless the request asks for something familiar). They offer an `abandoned` game as a second chance, at most one in a lineup with its spares, unless the request asks for games put down. They leave out `playing`, `burnt_out` during its break, `done`, `hidden`, `not_game`, and a `played` game with an ending that is no staple.

## The lineup

The AI is asked for `count` picks and `2 × count` spares. Picks that break a rule (two of one series, two second chances) mean asking again, and a spare that does is dropped. A lineup has:

- `shown`: how many picks it shows (the `count` asked for);
- `picks`: the picks on screen in slot order, each with `slot` (from 0) and `feedback`, the last thing done with it here (`{"action", "text", "at"}`, or null). A slot with no pick waits for new spares;
- `spares`: spares not used yet, best first (`slot` null);
- `removed`: picks taken off, in the order they went, each with the `feedback` that took it off.

When another call changed the lineup while feedback, `too` or `switch` was working from it, the later one is `409` and writes nothing.

## POST /api/picks/{batch_id}/feedback

`{"title_id": 1234, "action": "not_tonight", "text": null, "reasons": null}`. Answers with the lineup as it stands after. `404` when the game isn't on screen in this lineup, or for an unknown lineup; `400` for an unknown action.

- `chosen`: Play. Recorded, and the pick stays.
- `not_tonight`: takes the pick off and puts the next spare in its slot, with no AI call. Nothing about the game changes.
- `not_for_me`: Never recommend this. Marks the game `hidden`, then as `not_tonight`.
- `started`: "I've started it…", with `text` how it went, a mark's name (`done`, `shelved`, `abandoned`, `hidden`, `still_playing`, `burnt_out`), and `reasons` as for [marking a game](statuses.md#marking-a-game), stored with source `pick`. When the mark takes it out of picks it's taken off and the next spare goes in; with `shelved` or `still_playing` it stays.
- `too` has its own call (`400` here).

The next spare is the best one that doesn't repeat a series on screen or add a second second chance. With none, the slot stays empty.

## POST /api/picks/{batch_id}/too

`{"title_id": 1234, "word": "long"}`. The word may come as typed ("It's too long.") and is tidied to `long`, 1 to 40 characters, else a `400`. One AI call (kind `too`) answers with a spare without the problem for the slot and, for any other pick on screen with the same problem, another.

```json
{"batch": "PickBatch",
 "swaps": [{"slot": 0, "out": 1234, "into": 77}, {"slot": 2, "out": 99, "into": 78}],
 "ai_error": null}
```

The turned-down game's feedback is `too` with the word, and it counts as a rejection; the others it took off are `same_problem`, which doesn't. `into` is null when no spare was free of the problem, and the slot waits for new spares. With no usable spare there's no call.

When the call fails, the swap happens all the same: the first usable spare the word doesn't rule out takes the slot, and `ai_error` says why the AI couldn't check. Without the AI, a length word such as `long` or `short` rules out a spare not known to be shorter or longer by HowLongToBeat, `old` and `new` go by release date, and any other word rules out a spare whose tags, review tags or summary match it.

## POST /api/picks/{batch_id}/more

`{"sources": ["steam"], "input": "controller"}`, both optional; big screen sends `input` again, since the lineup doesn't keep it. New spares for a lineup whose spares ran out: a pick call for its empty slots and `2 × shown` spares, leaving out every game the lineup has held. A lineup with no empty slot comes back unchanged. Errors as for a pick, and `404` for an unknown lineup.

## Rejections

"It's too ___" and Never recommend this are rejections; Not tonight never is.

- A `shelved` game turned down twice since it was last played or marked is `abandoned` (`abandoned_at` is the second rejection, `abandoned_by_picks` is true) and out of picks.
- An `abandoned` game turned down once since it was abandoned or marked is out of picks, and stays `abandoned`.
- Untouched and sampled games stay in picks however often they're turned down.

`GameView` fields: `rejections` (times it was turned down while it has this status), `abandoned_by_picks`, `too` (the last "It's too ___" said about it, `{"word": "long", "at": 1790000000}`) and `chosen` (the last time it was chosen with Play, `{"at": 1790000000, "prompt": "a relaxing 2 hours"}`).
