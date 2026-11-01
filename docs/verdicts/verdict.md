# A verdict

One verdict is one player's answer about one game. It is sent as JSON with exactly these fields; any other field refuses it.

| Field | Type | Values | Rules |
|---|---|---|---|
| `steam_appid` | integer | The game's Steam appid | At least one of `steam_appid` and `gog_id` |
| `gog_id` | string | The game's GOG product id, digits only | |
| `outcome` | string | `finished`, `playing`, `shelved`, `stopped`, `endless`, `bounced` | Required |
| `worth` | string | `yes`, `for_some`, `no` | Required |
| `hours` | string | `under_2`, `2_5`, `5_10`, `10_20`, `20_40`, `40_80`, `80_plus` | Never exact playtime |
| `clicked` | string | `first_hour`, `few_hours`, `later`, `much_later`, `never` | |
| `sessions` | string | `short`, `medium`, `long` | Review tags' `sessions` scale, value for value |
| `limited_time` | string | `yes`, `mostly`, `no` | |
| `factors` | list of strings | Helps: `save_anywhere`, `easy_to_resume`, `short_chunks`, `recap`. Hurts: `hard_to_resume`, `long_stretches`, `padding`, `obligations`, `needs_a_group` | Up to 3, each once, only with `limited_time` |
| `stop_point` | string | `at_credits`, `before_endgame`, `anytime`, `push_through` | |
| `stop_note` | string | Up to 80 characters ("after chapter 3") | Only with `stop_point`; [text checks](#text-checks) |
| `stopped_for` | list of strings | The game reasons UpNext asks for when a game is put down: `grind`, `stuck`, `lost_story`, `too_long`, `pacing`, `bugs`, `too_hard`, `boring` | Each once, only when `outcome` is `shelved`, `stopped` or `bounced`. Life reasons (`busy`, `other_game`, `mood`, `elsewhere`) are refused |
| `text` | string | Up to 500 characters | [Text checks](#text-checks) |
| `vocab` | integer | The vocabulary the answers come from: `1` | |
| `licence` | string | `CC0-1.0` | |
| `consent` | integer | The consent version accepted: `1` | |
| `client` | string | The UpNext version that sent it, such as `0.1.0` | Letters, digits, `.`, `-` and `+`, up to 32. Not published |

Optional fields are left out rather than sent as `null`, though `null` reads as left out. Characters are Unicode scalar values (in JavaScript, `[...text].length`), not bytes. Every field's question, answers and words in the form are in [`verdicts-contract.json`](../verdicts-contract.json) under `fields`, with the "most players" line each answer reads as on the game page and, for `clicked` and `sessions`, the review tag it agrees with (`clicked` maps `first_hour` to `hook:fast`, `few_hours` to `hook:gradual`, `later` and `much_later` to `hook:slow`, and `never` to nothing).

A verdict carries no Steam ID or store account, name, exact playtime, or time finer than a day.

## Pre-filled from the install

UpNext fills in what it can stand behind, and the player changes or adds the rest (`upnext verdicts draft <game>` shows it for a game):

- `outcome` from the player's mark: Beat it is `finished`, Shelved and Burnt out are `shelved`, Abandoned is `stopped`, Still playing is `endless`, Not for me is `bounced`. An unmarked game being played now is `playing`.
- `hours` from the title's playtime, as its range, when a store reports one.
- `sessions` from the play diary, when it's on and holds at least five sessions of the game.
- `stop_point` is `anytime` for a game with no ending.
- `stopped_for` from the game reasons the player gave for putting it down. Life reasons and the note typed with them stay on the install.
- `worth` and the rest are always asked.

## Text checks

`text` and `stop_note` are checked in the app as the person types and again by the service. A text is refused, saying which check it failed, when it is blank, too long, or matches one of these patterns (case-insensitive; the first that matches is the one named). Invisible characters that change no words are removed first (`text_ignored` in the contract), so a zero-width space before `.com` is still a link.

| Code | Refuses |
|---|---|
| `control` | Invisible control characters (new lines, tabs and carriage returns are fine), and the characters that reorder text as it's shown, U+202A to U+202E and U+2066 to U+2069 |
| `wide` | Full-width letters, digits, `＠`, `．` and `／`; the full-width `：`, `！`, `？` and brackets Chinese and Japanese text uses are fine |
| `steam_profile` | `steamcommunity`, `s.team/` and `steam://` links |
| `email` | Email addresses |
| `link` | Anything with `://`, `www.`, or a word ending in a common domain (`.com`, `.net`, `.org`, `.io`, `.gg`, `.tv` and others) |
| `handle` | `@handles` |
| `long_number` | Any run of 17 or more digits, which could be a Steam ID |
| `phone` | Nine or more digits with at most two spaces, dots, dashes or brackets between each |

The patterns are in [`verdicts-contract.json`](../verdicts-contract.json) under `text_checks`. The checks guard against accidents and are not moderation: a game called "Agar.io" in a text is refused as a link, and so is Japanese typed with full-width digits. Look-alike letters from other alphabets aren't caught; reports and the operator are the guard there.
