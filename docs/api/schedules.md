# Scheduled jobs

Settings › Jobs lists twelve tasks (the UI hides `wishlist` while the wishlist is off). Jobs are on [Jobs](jobs.md).

| `task` | Label | What it runs | Default |
|---|---|---|---|
| `sync` | Sync libraries | a sync of the libraries in its options | daily |
| `details` | Fetch details | an enrich of the sources in its options (every source but HowLongToBeat, the wishlist and the AI's), for what titles don't have yet | after `sync` |
| `reviews` | Refresh player reviews | an enrich of `reviews`, `review_texts` and `gog_meta`, with `refresh` | monthly |
| `digests` | People say, Vibes and tags | an enrich of `digests` (AI) | off |
| `series_guides` | Series guides | an enrich of `series_guides` (AI) | off |
| `second_chances` | Second chances | an enrich of `second_chances` (AI; [Second chances](second-chances.md)) | off |
| `hidden_gems` | Find hidden gems | an enrich of `hidden_gems` (AI; [Hidden gems](discovery.md#hidden-gems)) | off |
| `wishlist` | Update the wishlist | an enrich of `wishlist`, skipped while the wishlist is off ([Wishlist](wishlist.md)) | after `details` |
| `taste` | Update your profile | a `taste` job, counting the [patterns](taste-patterns.md) again (no AI, no network) | after `sync` |
| `hltb` | HowLongToBeat | an enrich of `hltb` | off |
| `backup` | Back up the database | a backup job, keeping 4 ([Backups](backups.md)) | weekly |
| `verdicts` | Backlog verdicts | a download of other players' verdicts, skipped while receiving is off ([Backlog verdicts from others](community-verdicts.md)) | daily |

- **Schedules** are `off`, `daily`, `weekly`, `monthly` (every 30 days) or `after:<task>`. A task runs one period after its last run. One that has never run, or whose schedule just changed, waits a full period. An `after` task runs when that task ends `done`, from the server, the UI or a terminal; a failed or cancelled sync starts nothing.
- **The scheduler** runs in `upnext serve`. It starts the first due task in table order, and only when the `auto_sync` setting is on (it switches every schedule), the library has been synced once (the first sync is yours), and no job is running. A run missed while the server was stopped happens after it starts.
- **Which task a job was.** A job started as a task records it. A command-line enrich of every title counts as the first task in table order whose sources it ran, so `upnext enrich --with hltb` is `details` and `upnext enrich --only digests` is `digests`.

## GET /api/jobs/scheduled

```json
{"enabled": true, "waiting_for_first_sync": false, "tasks": [
  {"task": "details", "label": "Fetch details", "schedule": "after:sync", "uses_ai": false, "chosen": false,
   "options": {"wikidata": true, "series": true, "art": true, "reviews": true, "review_texts": true, "tags": true, "protondb": true,
               "gog_meta": true, "igdb": true, "rawg": true, "pcgw": true, "trailers": true},
   "keep": null, "limit": null, "last_run": "Job", "next_run_at": null, "next_after": "sync"}]}
```

- `enabled` is the `auto_sync` setting. `chosen`: someone set this task's schedule or options.
- `options`: every library (sync) or source (details) on or off, or null for tasks without options. `keep`: the backups kept (backup only). `limit`: the titles one run takes (AI tasks only), or null.
- `last_run`: its last run, however it ended, or null. Its next run is `next_run_at` (in the past means as soon as nothing else runs), or `next_after`, the task whose next run that ends done starts it, or neither when it's off.

## PUT /api/jobs/scheduled/{task}

`{"schedule": "weekly", "options": {"igdb": false}, "keep": 4}`. Every field is optional, and options are merged into the saved ones. `limit` (an AI task only) caps the titles one run takes, so a big first run goes in parts (`{"schedule": "daily", "limit": 300}`); `0` takes it off. Answers in the shape of `GET`.

`400`, with the reason, for an unknown schedule or option, a `keep` outside 1 to 100 or on anything but the backup, a `limit` on a task that doesn't use the AI, or a task after itself or two that would start each other. `403` for a `keep` from another device; `404` for an unknown task.

The libraries a sync reads and the sources an enrich runs are on [Sync and enrich sources](sources.md).
