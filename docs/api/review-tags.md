# Review tags

The review summary and the review tags come from one AI call over the same player reviews. The vocabulary is fixed and `GET /api/review-tags` returns it. `GameView.review_tags` is null until a title's summary has been written with tags:

```json
"review_tags": {
  "scales": {"difficulty": "hard", "learning": "steep", "sessions": "short"},
  "mood": ["tense", "dark"],
  "skills": ["reflexes", "timing"],
  "aspects": [
    {"aspect": "combat", "verdict": "praised"},
    {"aspect": "grind", "verdict": "criticized"},
    {"aspect": "ending", "verdict": "divisive"}
  ],
  "notes": ["slow first hours"]
}
```

- `scales`: at most one value per scale and at most five scales, the best-supported ones. A scale the reviews don't speak to is left out. Values run from low to high.
- `mood`: up to 2 values. `skills`: up to 3.
- `aspects`: at most 8, praised first, then criticized, then divisive.
- `notes`: at most 3 short points the vocabulary can't express.

Lists carry `review_tags` too.

## GET /api/review-tags

The vocabulary, with counts over the whole library. The UI draws the scales and filter menus from it.

```json
{
  "tagged": 1600,
  "scales": [
    {"name": "difficulty", "title": "Difficulty", "means": "how hard players say it is, not what the store says",
     "values": [{"value": "easy", "label": "Easy", "count": 129}, {"value": "moderate", "label": "Moderate difficulty", "count": 33}],
     "known": 297}
  ],
  "sets": [
    {"name": "mood", "title": "Mood", "means": "the feel of playing it", "max": 2, "values": [], "known": 735},
    {"name": "skills", "title": "Asks for", "...": "..."}
  ],
  "aspects": [{"name": "grind", "praised": 0, "criticized": 84, "divisive": 6}]
}
```

- `tagged` counts the titles with review tags.
- `scales` come in vocabulary order, each one's `values` from low to high. `title` is the name the game page shows.
- `known` counts the titles with any value of that scale or set. Some are thin (sessions: fewer than one title in ten), so a filter on them should say how many games could match at all.
- `aspects` counts the titles whose players praised, criticized or split on each aspect.

Filter the library by them with `facet`, `praised` and `not_criticized` on [`GET /api/games`](library.md).

## In picks and Help me decide

The AI sees each candidate's feel and its praised, criticized and divisive aspects. The ranking that comes before it reads them too:

- A request's words ask for facets ("chill" for a relaxed pace or a relaxing or cozy mood, "a big world" for an open structure), and a match counts like a tag match.
- A word after "no", "not" or "without" in the same clause doesn't count for the games that have it, and "no grind" asks for `grind:none`.
- With a short time budget, games whose players call them good for short sessions rank higher.
