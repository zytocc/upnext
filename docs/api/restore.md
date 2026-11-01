# Restoring and importing

Both answer only on this computer, whatever the token ([Only on this computer](local-only.md)). Backups are on [Backups](backups.md).

## POST /api/backups/{name}/restore

A `restore` `Job` (`202`). It backs up the database as it is first, then puts the backup in place. The server keeps running and every page shows the restored library at once.

This install's secret keys, `steam_id` with the Steam profile and each player's GOG and Epic sign-ins are kept where the backup has none. This install's API tokens replace the backup's, so a token revoked since stays revoked. Everything else is the backup's: library, statuses, lists, picks, settings, schedules, AI usage and job history. Jobs the backup caught running are closed as `failed`.

`400` when the file isn't a database, or is from a newer UpNext or from before the first release. `404` for an unknown name. `409` while a job runs.

## POST /api/backups/import?name=old-laptop.db

The database file as the request's body (any content type, up to 2 GB). An `import` `Job` (`202`) that does what a restore does, with an `import` backup first. The file is checked before the job starts, so one UpNext can't use is a `400` with a plain reason. `409` while a job runs.

## Opening an older database

The server and the command line back up a database with an older schema before upgrading it. A database from a newer UpNext, or from a development build made before the first release, isn't opened.
