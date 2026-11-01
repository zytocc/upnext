# Statuses, staples and stop reasons

UpNext works out where each started game stands from its playtime, when it was last played, HowLongToBeat and its tags. Anything you set wins.

## Status

`GameView.status` is one of these, in the order the library sorts by status:

`untouched`, `sampled`, `playing`, `shelved`, `played`, `done`, `still_playing`, `burnt_out`, `abandoned`, `hidden`, `not_game`.

The first that applies wins: your mark (`override`), then `not_game` when the store's type isn't a game, then playtime:

- `untouched`: no playtime. `sampled`: under `sampled_minutes` (120).
- `playing`: played in the last `shelve_after_days` (60).
- `shelved`: a game with an ending, not a staple, played for less than 80% of its HowLongToBeat main story (or with no known length), and untouched for `shelve_after_days`. Past 80% it was likely finished. A game with no known last-played date counts as shelved.
- `abandoned`: shelved and untouched for `abandon_after_days` (730).
- `played`: anything else, such as endless games, staples, and games played past their main story.

Games with no playtime from their store (Heroic's Epic and Amazon games) stay `untouched` until marked. One chosen from a lineup with Play reads `playing` for `shelve_after_days`, then `untouched` again.

## Marking a game

`PUT /api/games/{id}/override` with `{"status": "shelved", "note": "optional", "reasons": {"codes": ["busy", "grind"], "note": "optional"}}`. Answers with the `GameView`. `null` clears the mark.

The marks (the "Mark as…" menu): for a game with an ending, `done` ("Beat it"), `shelved` (you mean to finish it) and `abandoned` (you don't plan to go back); for an endless game, `still_playing` (now and then) and `burnt_out` (out of picks for `burnout_days`, then back by itself); for either, `hidden` ("Not for me"). `reasons` go only with `shelved`, `abandoned` or `burnt_out` (`400` otherwise).

`override` is the mark as it stands now, and `override_at` when it was set:

- a `shelved` mark untouched for `abandon_after_days` (from the later of the mark and the last play) reads `abandoned`;
- a `burnt_out` mark past its break reads `still_playing`;
- a `shelved`, `abandoned` or `burnt_out` mark on a game played since reads `null`, since you went back to it.

The stored mark stays until you set or clear another.

## Fields

- `kind`: `ending` or `endless`. Endless when the game page's switch says so, or else when its tags name a kind of game with no end state (strategy, sandbox, MMO, sports, party games and the like), when its store categories are multiplayer-only, or when its review tags say `story:none` and `replay:high`.
- `no_ending`: the switch: `true`, `false`, or `null` to infer.
- `staple`: played at least `staple_hours` (100). A label, not a status: a staple can be beaten, and is never shelved or abandoned by inference.
- `shelved_at`, `abandoned_at`: when UpNext found it shelved or abandoned, or when a `shelved` mark turned into `abandoned`; null when you set it or no store says when it was last played.
- `burnt_out_until`: when a `burnt_out` game comes back.
- `stop_reasons`: why you put it down, the latest you said: `{"codes": ["busy", "grind"], "note": "the second act", "source": "menu", "at": 1790000000}`, or null. `source` is `menu`, `pick` or `second_chance`.
- `rejections`, `abandoned_by_picks`, `too` and `chosen` say how it fared in picks ([Lineups](lineups.md#rejections)).

## PUT /api/games/{id}/ending

`{"no_ending": true}`, `false`, or `null` to let UpNext infer it again. This is the game page's "Can be played endlessly" switch. Answers with the `GameView`; `404` for an unknown game.

## Stop reasons

`GET /api/stop-reasons` lists them in the menu's order as `{"code", "group", "label", "aspects"}`. `life` reasons (`busy`, `other_game`, `mood`, `elsewhere`) never count against similar games. `game` reasons (`grind`, `stuck`, `lost_story`, `too_long`, `pacing`, `bugs`, `too_hard`, `boring`) are compared with what reviews say about the review-tag `aspects` listed.

Unknown codes are a `400`. Codes are kept in the menu's order, each once; a blank note is none, and one over 500 characters is a `400`. Empty codes and note record nothing. Every set given is kept, and the game shows the latest. Marking a game again without `reasons` keeps the last ones.

## Settings

`shelve_after_days` (60), `abandon_after_days` (730), `staple_hours` (100) and `burnout_days` (90), in [Settings](settings.md). Days are 1 to 3650 and hours 1 to 100000. A change applies to the next read of the library.
