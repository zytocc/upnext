# Routines

A routine is a named session that puts together what the habits keep apart: when, who with, where, on what and for how long ("Friday with friends": Friday evenings and late nights, with friends, at a desk, on keyboard and mouse, 2 hours). They're edited on the Profile, after Habits, and the Pick page offers them as chips, with the one that fits the time chosen to start with. A routine is personal, like the habits.

```json
{"id": 3, "name": "Friday with friends", "days": ["fri"], "times": ["evenings", "late_nights"],
 "who": "friends", "place": "desk", "input": "keyboard_mouse", "minutes": 120, "note": null}
```

- `name`: up to 40 characters, and no two the same, ignoring case.
- `days`: any of `mon` to `sun`; empty for any day.
- `times`: any of `mornings`, `afternoons`, `evenings` and `late_nights`; empty for any time.
- `who`: `solo`, `partner`, `friends` or `kids`. `place`: `desk`, `couch` or `deck`. `input`: `keyboard_mouse` or `controller`. Each may be null.
- `minutes`: 15 to 720, or null. `note`: up to 300 characters, read by the AI only.

At most 20 routines.

## GET /api/routines

`{"routines": [Routine], "now": 3}`, in the order they were made. `now` is the routine that fits the day and time of day now (local time), or null. Only a routine with days or times can fit. When several do, the narrowest wins, then the first made. The small hours before 5 am count as the night before.

## POST /api/routines

A routine without `id`; only `name` is required. Answers `201` with the routine. `400` with the reason in `error` for a name that's empty, too long or taken, a value out of range, a 21st routine, or a body it can't read. A body not sent as `Content-Type: application/json` is a `415`.

## PUT /api/routines/{id}

A routine, as for `POST`, replacing the one there. `400` and `415` as for `POST`; `404` for no such routine.

## DELETE /api/routines/{id}

`204`; `404` for no such routine.

## How picks read a routine

`POST /api/picks` with `routine_id` picks for that routine. The lineup keeps the routine as it was then (`routine` on the lineup), and its new spares keep to it even after the routine changes or is deleted.

The AI is told the routine, and it outranks the habits; your request outranks both. The ranking reads each game's tags, Steam store categories and IGDB modes, and leaves alone a game that says nothing:

- With others (`partner`, `friends`, `kids`), co-op and multiplayer games rank higher and single-player-only ones lower. On your own, online-only multiplayer games rank lower. With kids, gory, violent, sexual or horror games rank lower.
- On a controller (`input: controller`, or `place: deck` with no input said), controller support ranks higher, and a game whose Steam details list none ranks lower. Keyboard and mouse changes nothing.
- The routine's length is the time budget when the request gives none.

`ranked_by` names the routine and what it counted.

From the command line: `upnext pick --routine "Friday with friends"` and `upnext prompt --routine …`, by name in any case. `upnext taste` lists the routines and which fits now.
