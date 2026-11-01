# Picks without AI

With no AI set up (the chosen backend's own check fails), or with `"without_ai": true`, `POST /api/picks` makes the lineup from UpNext's own ranking with no AI call. `upnext pick --without-ai` does the same in a terminal, and so does `upnext pick` with no AI set up.

The lineup is the same shape, with:

- `ranked_by`: what the ranking went by, in plain words, and only what counted for some game. A reason in `avoid` it can't read ends the list with a line saying only the AI reads reasons in your own words.
- `model` null, `attempts` 0, `usage`, `cost_usd` and `api_usd` null, and `candidates_considered` every game it could choose from.
- Each pick's `reason` built from the data: review score, main-story length against your time, and what matched your request. A shelved game or a second chance with reasons starts with what you said when you stopped.

The lineup is the top `count` of the ranking, then `2 × count` spares, with one game per series and one second chance (unless asked for more). Each lineup varies the ranking a little, so lineups differ, and games shown in the last few lineups rank lower.

`avoid` is read where it can be:

- `too <word>` (the menu's "Too long" and "Too slow", and the words said with "It's too ___") holds back games the word rules out against the excluded games, as [`/too`](lineups.md#post-apipicksbatch_idtoo) reads words.
- "Want something newer" (or older) holds back games no newer (or older) than them.
- "Not in the mood for these genres" holds back games sharing their genres.
- "Want something …" adds its words to the request.

On a lineup picked without AI:

- `POST /api/picks/{batch_id}/too` makes no call: the first usable spare the word doesn't rule out takes the slot, and `ai_error` is null.
- `POST /api/picks/{batch_id}/more` takes the next games in the ranking, steering away from games like the ones said to be "too ___".

Help me decide stays an AI feature: with no AI set up it answers `503` with the reason.

The [taste profile](taste-patterns.md), [ratings](ratings.md), [routines](routines.md) and [other players' verdicts](community-verdicts.md) add their own lines to `ranked_by` and the reasons.
