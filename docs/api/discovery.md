# Discovery: new to you, hidden gems and roll the dice

Three pick modes narrow the candidates. New to you and Hidden gems draw from the library, and Roll the dice draws by chance. The first two are asked for in the request's own words ("New to me", "new to me, 2 hours", "A hidden gem for tonight"; whole words) or with `mode` (`new_to_you`, `gems`, `random`) on `POST /api/picks`. Picks keep their usual rules (one per series, one second chance), the rest of the request still counts, and new spares (`…/more`) stay in the lineup's mode. Too few games for the count is a `400`: "no candidates: only 1 of your games is new to you (…), cannot pick 3".

## New to you (`new_to_you`)

A way into a genre or mechanic you have barely played, in a game that is gentle to start (short, and not hard or punishing). A game you dislike or ruled out is not a candidate.

- **A lineup spreads**: each pick stands for a different genre or mechanic (spares are exempt). The AI is asked again when two picks can't. Picks without AI keep such a game as a spare. When too few games can make the count the pick is a `400`: "only 2 of the eligible games can share a lineup (one game per series, each a way into something different), cannot pick 3".

### GET /api/new-to-you/{id}

```json
{"ways": [{"about": "tag:Deckbuilding", "tag": "Deckbuilding", "phrase": "deckbuilders", "kind": "mechanic"}]}
```

The genres and mechanics a game is a way into for you now, mechanics first. On "Not for me" for a New to you pick, the UI offers to make the first of these a hard no: `about` goes straight into [`POST /api/taste/statements`](taste.md#post-apitastestatements). `404` for a game not in the library.

Settings: `new_to_you_max_hours` (8, 1 to 200) is the longest main story that counts as gentle, and `unfamiliar_share` (2, 0 to 100, percent) is the share of your played hours below which a genre is new to you ([Settings](settings.md)).

## Roll the dice (`random`)

Only `"mode": "random"` asks for it. The request's words don't steer it. The Pick page's dice button sends it with `count` 1 and an empty request.

- A game is **off your radar** when picks offer it and you haven't started it, or last played it a year ago or more. Endless games and staples never are.
- Every eligible game has the same chance. The draw leaves out `exclude_ids` and the picks of the last three lineups (unless that leaves too few), and keeps one game per series and one second chance.
- The lineup is saved like any other with no AI call, even when one is set up: `model` null, `attempts` 0, `candidates_considered` the games it drew from, and `ranked_by` one line, "chance alone: drawn at random from 42 games off your radar". History calls it picked without AI, "It's too ___" takes the next spare the word doesn't rule out, and new spares are drawn at random again.
- Too few games is a `400`: "no candidates: none of your games is off your radar yet (…)".

## Hidden gems

The `gems` pick mode and the shelf on the Lists page draw from unplayed, short, well-rated games the AI has rated as unusual. A gem you play leaves the pool.

The `hidden_gems` task (off until turned on, like every AI job; the Lists page's Find hidden gems runs it) has the AI rate each game in the pool that isn't rated yet or was rated by an older prompt (all of them on `refresh`). It runs on the summaries backend (kind `hidden_gems`, counted per game) and is skipped, with a log line, when no AI is set up. Picks leave party games out.

### GET /api/gems

```json
{"gems": [{"game": "GameView",
           "rating": {"studio": 3, "mechanics": 2, "play": 3, "party": false, "line": "You unpack a life, box by box.",
                      "prompt_version": 1, "model": "<model>", "written_at": 1790000000},
           "score": 8, "rarest": [["Wholesome", 2], ["Cozy", 5], ["Pixel Graphics", 8]]}],
 "pool": 80, "rated": 80, "unrated": 0, "party": 6,
 "ai": {"available": true, "reason": null},
 "task": {"schedule": "off", "enabled": true, "last_run": "Job", "next_run_at": null},
 "rules": {"min_positive": 93, "min_reviews": 500, "max_hours": 5.0, "count": 12}}
```

The shelf is the `gem_count` (12) best-rated games in the pool that aren't party games. `unrated` is what the next run rates, and `party` the rated party games left off. `ai` says whether the summaries backend can run, and why not. `task.enabled` is the `auto_sync` setting.

`?all=true` adds `ranked`: every game in the pool, the shelf's order first, then party games, then the ones not rated yet (`rating` and `score` null).

Settings: `gem_min_positive` (93, 50 to 100), `gem_min_reviews` (500, 0 to 1,000,000), `gem_max_hours` (5, 1 to 200) and `gem_count` (12, 1 to 100) ([Settings](settings.md)).
