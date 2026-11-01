# Second chances

The Second chance part of the Lists page's Abandoned tab shows abandoned games whose reasons might no longer hold. For each, "Then" is why you stopped (`stop_reasons`) and "Now" is what player reviews written since say about it, written by the AI in the Second chances job.

- **On the page:** every `abandoned` game, unless you've answered for it there since you last played it. Playing it again, and putting it down again, brings it back.
- **"Now"** is written from the reviews posted after it was put down. With no review since, there's nothing to check against and no call is made.
- **The `second_chances` task** (off until someone turns it on; the page's Check now runs it) goes over the games on the page and the ones shelved again for a second try, re-reading their reviews when the stored ones are old (always on `refresh`). With no AI set up it logs that it was skipped.

## GameView fields

`second_chance` is null, or:

```json
{"now": "2025 and 2026 reviews say most of the bugs are smoothed out…", "verdict": "partly",
 "line": "2025–2026 reviews: crashes mostly fixed, some jank remains.", "written_at": 1790000000, "model": "sonnet",
 "outdated": false, "answer": null, "answered_at": null}
```

- `verdict` is `better` (what stopped you is fixed or gone), `partly`, `same` (still there) or `unclear`. With no reason about the game (only life reasons, or none) it's only ever `better` or `unclear`.
- `outdated` is true once the reasons or the reviews it was written from have changed.
- `answer` is the page's: `willing`, `still_no` or `not_for_me`.

`second_try` is true for a game shelved again with "Willing to try again".

## In picks

- A second chance whose up-to-date `verdict` is `same` isn't offered.
- "Willing to try again" marks the game `shelved`, so it's offered like any shelved game, and one rejection in picks abandons it for good (`abandoned_by_picks`); it doesn't come back to the page.
- Still no counts as a rejection: the game stays `abandoned` and out of picks.
- The AI sees why you stopped and what the reviews say since. A lineup picked without AI adds the same to the reason.

## GET /api/second-chances

```json
{"games": [{"game": "GameView", "stopped_at": 1705104000, "reviews_since": 8, "due": false}],
 "ai": {"available": true, "reason": null},
 "task": {"schedule": "off", "enabled": true, "last_run": "Job", "next_run_at": null}}
```

Games where reviews say things changed come first (`better`, then `partly`), then the ones not checked or unclear, then `same`. `game` is the slim list shape. `due` means the next run writes its "Now". `ai` is the summaries' backend's state, and `task.enabled` the `auto_sync` setting.

## POST /api/second-chances/{id}

Any one of `{"answer": "willing" | "still_no" | "not_for_me"}`, `{"undo": true}` or `{"reasons": {"codes": ["bugs"], "note": "your reply"}}`. Answers with the `GameView`.

- `answer` is for a game on the page, else a `409`. `willing` marks it `shelved`, `not_for_me` marks it `hidden`, and `still_no` leaves the mark alone.
- `undo` takes the answer back, putting back the mark the game had before.
- `reasons` are checked as for [marking a game](statuses.md#marking-a-game) and stored with source `second_chance`. Clearing every code and the note records that no reasons stand.
- An unknown answer, an answer with `undo`, or none of the three is a `400`; an unknown game a `404`.
