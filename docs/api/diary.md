# First-run questions and the play diary

## GET /api/taste/questions

What the first-run questions (`/profile/questions` in the UI) ask. The setup wizard asks them in its "Your taste" step once the library has games.

```json
{"most_played": [{"game": "GameView", "favourite": 18}],
 "few_played": false,
 "loves": [{"about": "facet:mood:cozy", "label": "Cozy", "group": "Mood", "count": 21}],
 "hard_nos": [{"about": "tag:Horror", "label": "Horror", "group": "Genres and themes", "count": 7}],
 "diary": {"on": false, "steam": true}}
```

- `most_played`: your 12 most-played games, each with its favourite row's id when it is one.
- `few_played`: fewer than 6 games with 2 hours or more, so the UI leans on searching for a game rather than showing covers.
- `loves`: the moods found in the library, then the 18 tags on the most games, with how many games each matches. They're saved as likes.
- `hard_nos`: a fixed list of tags such as Horror, Punishing and PvP, each with how many games a hard no on it would leave out. Ones that leave nothing out are dropped, unless you already said it.
- `diary.steam`: a Steam Web API key and Steam ID are set, so the diary can read something.

Every answer is saved as it's given, through the routes it belongs to: favourites and why with [`/api/taste/statements`](taste.md#post-apitastestatements), session lengths and who you play with as habits, and the diary with `PUT /api/taste/diary`.

## The play diary

Off by default (the `play_diary` setting). When it's on, the server reads Steam's recently played games about once an hour (it needs schedules on and the Steam key and ID) and records the minutes each game gained since the last reading. The Play button also records a `launch`, which is the only record of Epic and Amazon games through Heroic, since they report no playtime. The profile reads the diary for the habits it can see over the last 90 days: how long weeknight and weekend sessions run, when you play, and when multiplayer games get played. A habit you state wins over the diary's.

The switch is the household's, one for every player. What it records is each player's own: Steam rows count for the account's owner and the players who play on it, `launch` rows for whoever pressed Play.

### What the diary shows

`GET /api/taste` carries `diary`:

```json
"diary": {"days": 16, "minutes": 1640, "weeknight": {"minutes": 60, "count": 8}, "weekend": {"minutes": 165, "count": 8},
          "times": ["evenings"], "multiplayer": {"minutes": 740, "weekend_share": 0.65, "time": "evenings"}}
```

Each part is null, or empty, until it has enough. Picks and Help me decide read it in About me, and as the usual session when no habit is stated.

### PUT /api/taste/diary

`{"on": true}` turns it on and takes the first reading at once when the Steam key and ID are set. A failed reading shows in `diary.error` and doesn't stop the switch. `{"on": false}` turns it off and forgets the last reading; the rows stay until you delete them. Answers `{"diary": …}`, the `diary` of [`GET /api/taste/sources`](taste-sources.md).

### POST /api/taste/diary/launch

`{"title_id": 41}` answers `{"recorded": true}` when the diary is on and `false` when it's off. `400` for a game not in the library.
