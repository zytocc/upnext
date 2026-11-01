# Patterns, and how picks read the profile

## Patterns

A pattern says a kind of game (a review-tag value, a tag or a length) is one you tend to stick with or to leave.

- Games you stuck with: favourites, staples, beaten or played-through games, games still being played, and picks you chose and played since. Games you left: abandoned or shelved ones, hidden ones, and ones turned down with "It's too ___". Life reasons for shelving or abandoning don't count.
- Recent games count for more than old ones. `pattern_half_life_days` sets how fast that fades.
- A pattern needs `pattern_min_games` games on its side that still count, a clear difference from your usual, and enough confidence that it's real. `confidence` is that figure, and 95% and over is `sure`.
- Overlapping patterns show once, the surer one. At most 15 show.
- A subject you've made a like, dislike or hard no isn't a pattern.
- The `taste` job stores them, and editing a statement or deleting a source counts them again at once. Hiding one ("Not really") holds when it comes back.

## How picks read the profile

`POST /api/picks`, new spares (`POST /api/picks/{batch_id}/more`) and `POST /api/versus/decide` read the profile:

- **Hard nos** UpNext can check (a review-tag value, a tag, a length) leave a game out of the pool, so it isn't counted in `candidates_considered`. A favourite is never left out. Hard nos in your own words go to the AI only.
- **The ranking** that orders the AI's candidates and makes picks without AI moves a game up for each like it matches and for likeness to your favourites. It moves a game down for each dislike and for each pattern of games you drift away from, unless the request asks for that subject.
- **With no time in the request**, your usual session (the `weeknight` habit, or `weekend` from Friday 5 pm to Sunday) stands in.
- **The prompt** gets About me: favourites with why, likes, dislikes and hard nos, the patterns shown on the page and the habits you stated. Help me decide gets the same.
- **One change of pace**: a vague request for 3 or more picks, with likes or patterns on your profile, gets one pick outside them, never a hard no, and its reason says so. Picks without AI keep the last slot for it.

Without AI, `ranked_by` and the reasons name what was counted, for example `likeness to your favourites`.
