# Game details

Fetching details (the `details` and `reviews` [scheduled tasks](schedules.md), from the sources in [Sync and enrich sources](sources.md)) fills in these `GameView` fields. Each is null or empty until its source has run.

## Tags, themes and ratings

```json
{
  "tags": ["Story Rich", "Atmospheric", "Stealth", "..."],
  "tags_by_source": {"steam": ["..."], "reviews": ["Relaxing", "Short sessions"], "gog": ["..."], "igdb": ["..."]},
  "themes": ["Horror", "Third person", "Single player"],
  "gog_rating": 4.5, "gog_rating_count": 1200,
  "igdb_rating": 81.3, "igdb_rating_count": 240,
  "rawg_rating": 4.2, "rawg_ratings_count": 900,
  "rawg_url": "https://rawg.io/games/...", "pcgw_url": "https://www.pcgamingwiki.com/wiki/...",
  "wikidata_url": "https://www.wikidata.org/wiki/Q..."
}
```

- `tags` is one de-duplicated list from every source, at most 30.
- `tags_by_source` keeps them apart (`steam`, `reviews`, `gog`, `rawg`, `igdb`, `pcgw`), on the game page only. `reviews` holds the plain labels of a game's moods, scales and skills ("Relaxing", "Easy", "Short sessions").
- `themes` are the themes, perspectives, modes and art styles that IGDB and PCGamingWiki list. `categories` include GOG's features.
- `genres` and `developers` fall back to RAWG, then PCGamingWiki, when the stores have none, and `metacritic` falls back to RAWG. RAWG's and GOG's ratings stand in for the review score in picks when Steam has none.
- `rawg_url`, `pcgw_url` and `wikidata_url` are the source pages. RAWG asks for a visible link back wherever its data is shown, so a client that shows `rawg_url` data links to RAWG.

## What players wrote

```json
"review_texts": [
  {"source": "steam", "positive": true, "rating": null, "votes": 61, "hours": 11, "title": null, "posted": 1758000000, "text": "I enjoyed this quite a bit..."},
  {"source": "gog", "positive": null, "rating": 5, "votes": 12, "hours": null, "title": "Masterpiece", "posted": 1757000000, "text": "..."}
],
"digest": "A stealth game whose story and atmosphere carry it; the stealth itself is simple and the ending divides people.",
"vibes": "You creep through rain-slick streets..."
```

- `review_texts` are whole reviews: up to 10 English ones from Steam and up to 8 from GOG. Steam reviews have a thumb (`positive`), votes and hours played; GOG's have 1 to 5 stars (`rating`) and a title. `posted` is in Unix seconds, or null.
- `digest` (People say) is the AI's critical look in at most 40 words.
- `vibes` is what playing it feels like, in 3 or 4 sentences; null when the reviews say nothing about the experience.
- The summaries job (`digests`) writes `digest`, `vibes` and [`review_tags`](review-tags.md) in one AI call.

### POST /api/games/{id}/reviews

Fetches the title's player reviews now (Steam's, else GOG's for its GOG copy) and answers with the `GameView`. The game page calls it for a title whose summary came from [shared data](seed.md), which has no reviews stored. `400` when the title has neither store; `404` for no such title; `502` when the store doesn't answer.

## Linux

These are filled in only while Linux compatibility is active. `linux_mode` is `auto` (on when the server runs on Linux), `on` or `off`, as `linux_modes` lists them. `GET /api/settings` also gives `linux_detected` and `linux_active`.

- `linux_native`: Steam lists a Linux build (or null).
- `protondb_tier`, `protondb_trending` and `protondb_total` (the report count): ProtonDB's view, not looked up for games with a native build.
- `linux`: `native`, else the ProtonDB tier (`platinum`, `gold`, `silver`, `bronze`, `borked` or `pending`), or null when unknown. "Runs well" is native, platinum or gold.

Picks then favour games that run well and push bronze and borked down.
