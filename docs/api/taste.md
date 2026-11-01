# Taste profile

Favourites with why, likes, dislikes and hard nos, habits, and patterns counted from how you play. `/api/profile` is the Steam profile, so these live under `/api/taste`. Patterns and how picks read the profile are on [Patterns](taste-patterns.md); the sources are on [Taste sources](taste-sources.md).

## Subjects

A like, dislike or hard no is about one subject:

- `facet:<scale>:<value>`, a review-tag value, such as `facet:grind:heavy`;
- `tag:<tag>`, such as `tag:Horror`;
- `length:short` (main story under 10 hours), `length:medium` (10 to 30) or `length:long` (over 30);
- `text:<your words>`, read only by the AI.

A tag matches a game's ten strongest tags and its store genres. A hard no reaches less far: the five strongest tags from Steam or GOG, plus store genres.

## GET /api/taste

```json
{"favourites": [{"id": 3, "game": "GameView", "why": [{"key": "aspect:story", "label": "the story"}],
                 "choices": [{"key": "tag:Open World", "label": "Open World"}], "note": "the last hour", "updated_at": 1790000000}],
 "statements": [{"id": 7, "kind": "hard_no", "about": "tag:Horror", "label": "Horror", "note": null, "matches": 34, "updated_at": 1790000000}],
 "habits": [{"id": 9, "about": "weeknight", "value": 90, "note": null, "updated_at": 1790000000}],
 "patterns": [{"id": 12, "key": "drift:facet:grind:heavy", "sentence": "You drift away from games with heavy grind.", "sense": "drift",
               "summary": "4 abandoned, 1 shelved, 3 of them for the grind; 1 played through", "confidence": 0.97, "sure": true,
               "games": [{"id": 41, "name": "…", "outcome": "abandoned", "at": 1780000000, "weight": 0.93, "reasons": ["grind"]}],
               "hidden": false, "since": 1790000000}],
 "hidden_patterns": [], "updated_at": 1790000000, "judged": {"kept": 41, "left": 25},
 "suggestions": ["GameView"], "subjects": [{"about": "facet:mood:scary", "label": "Scary", "group": "Mood", "count": 52}],
 "diary": {"...": "..."},
 "task": {"schedule": "after:sync", "enabled": true, "last_run": "Job", "next_run_at": null},
 "rules": {"half_life_days": 365, "min_games": 4}}
```

- `favourites`, most played first. `why` is what you chose it for; `choices` is what the game offers.
- `statements`: likes, dislikes and hard nos. `matches` counts the games in the library it's about (null for your own words).
- `habits`: `weeknight` and `weekend` (minutes a session usually is), `times`, `who` and `where`, with the same values as a [routine](routines.md), each with an optional note.
- `patterns`, surest first, then `hidden_patterns`. A pattern's `games` list its side first, heaviest first. `summary` counts the recent games, then the older ones.
- `suggestions`: about 12 of your most-played games that aren't favourites. `subjects`: what a like or dislike can be about, with how many games each matches. `diary`: see [the play diary](diary.md#what-the-diary-shows).

## POST /api/taste/statements

- `{"kind": "favourite", "title_id": 41, "why": ["aspect:story", "tag:Co-op"], "note": "…"}`: a game already a favourite takes the new reasons.
- `{"kind": "like" | "dislike" | "hard_no", "about": "tag:Horror", "note": "…"}`: a subject that already has one becomes this kind.
- `{"kind": "context", "about": "who", "why": ["solo", "friends"], "note": "…"}`: a habit's answer replaces the one before.

Answers `201` with the row. `400` for an unknown kind, subject, reason or choice, a game not in the library, a note over 300 characters, or `pattern`.

## PUT /api/taste/statements/{id}

- A like, dislike or hard no: `{"kind": "hard_no"}` and/or `{"note": "…"}` (an empty note clears it).
- A favourite or a habit: `{"why": [...]}` and/or `{"note": "…"}`.
- A pattern: `{"hidden": true}`, `{"hidden": false}` or `{"confirm": true}`, which makes it a like (stick) or a dislike (drift) about its subject and answers with that row.

Answers with the row. `400` for a change that doesn't fit the kind; `404` for an unknown id.

## DELETE /api/taste/statements/{id}

`204`. A pattern can't be deleted (`400`); hide it instead. `404` for an unknown id.
