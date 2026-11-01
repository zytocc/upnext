# Jobs

A job is background work: a sync, an enrich, a backup, a restore, an import, a taste profile count or a download of other players' verdicts. It runs in the server (started here, or by its schedule) or in a terminal (`upnext sync`, `upnext enrich`, `upnext backup`). The API shows both, and one job runs at a time across them. A running job whose process stops reporting (closed, killed or crashed) is marked `failed`. The last 50 jobs are kept with their logs, and so is each scheduled task's last run.

## The Job

```json
{"id": 42, "kind": "enrich", "task": "details", "source": "terminal", "status": "running",
 "started_at": 1758990000, "finished_at": null, "heartbeat_at": 1758990630,
 "total": 1600, "done": 1040, "current": null, "message": "in parallel: Steam store 1040 · Steam reviews 1038 · …",
 "per_minute": 31.2, "eta_secs": 1010, "failed": 3, "summary": null, "error": null, "cancel_requested": false,
 "lanes": [{"name": "Steam reviews", "done": 1038, "total": 1600, "failed": 2, "state": "waiting", "current": "Portal 2",
            "note": "Steam asked us to slow down", "retry_at": 1758990670}],
 "options": {"hltb": false, "reviews": true, "refresh": false, "limit": null},
 "ai_usage": null}
```

- `kind`: `sync`, `enrich`, `backup`, `restore`, `import`, `taste` or `verdicts`. A restore or an import starts only from [Backups](backups.md).
- `task`: the [scheduled task](schedules.md) it ran as or did the work of, or null for other work, such as a retry of a few titles or a single source.
- `source`: `ui`, `schedule` or `terminal`.
- `status`: `running`, `done`, `failed` or `cancelled`. `total` may be 0 while unknown. `summary` says what a finished job did ("2,000 games, 3 new"; "1,600 titles: 3 matched, 1,500 reviews, 5 errors").
- **A partial sync.** When one account (a player's Steam account, Heroic folder, Galaxy database, Epic or GOG sign-in) fails while the rest sync, the job is `done` but its summary starts "Partial:" and names what failed. Each failure is in `failed` and `failures`, with lane `sync` and the account as `source`. A sync where something failed and nothing was read is `failed`, and its `error` names each account and why.
- `per_minute` and `eta_secs`: the recent rate and the time left at that rate. Both are null until there is enough to go on.
- `lanes` (enrich only), in the order they started. One per rate-limited service and one per pass over the library, each with `done`, `total`, `failed`, `current` and a `state`: `running`, `waiting` (a service asked it to slow down or failed; `note` says which and `retry_at` when it tries again), `stopped` (it gave up for the rest of the run, such as for a rejected key or a usage limit; `note` says why) or `done`. While lanes run, the job's `done` is the slowest lane's.
- `failed`: failures so far, one per title and source.
- `ai_usage`: null unless the job made AI calls. Otherwise the calls' totals ([AI usage](ai-usage.md)) plus `estimated_tokens`, the tokens the whole job should take. It is null until each AI lane with work left has had a call back, and after the job ends.
- `options`: for an enrich, every source with the value it ran with ([Sync and enrich sources](sources.md#enrich-options)). For a sync, one boolean per library, keyed as `GET /api/sync/sources` lists them, plus `steam_local` and `galaxy_db`. For a backup, `{"keep": 4}`; for a restore or an import, the file's `name`.

[Starting and following jobs](job-routes.md) has the routes.
