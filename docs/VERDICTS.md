# Backlog verdicts: the contract

Backlog verdicts are players' short answers about how a game fits a backlog and limited time. They are sent one at a time, by choice, to a small service, and published once a day as a CC0 snapshot that installs download whole.

These pages are the contract between UpNext and that service. The service in `services/verdicts/` follows it, and anyone can run their own and point UpNext at it. The machine-readable half is [`verdicts-contract.json`](verdicts-contract.json), which `upnext verdicts contract` prints and a test keeps current. Where the pages and the code disagree, `src/verdicts/` is right.

- [Consent and keys](verdicts/consent.md): the licence, the consent screen and the key a player sends with
- [A verdict](verdicts/verdict.md): its fields, what UpNext fills in, and the text checks
- [The published snapshot](verdicts/snapshot.md): its files, and how an install reads them
- [How the service decides](verdicts/service.md): holds, brigading and withdrawals

The UpNext side is in the API pages ([your verdicts](api/verdicts.md), [verdicts from others](api/community-verdicts.md)), and the operator deploys the service with [its README](../services/verdicts/README.md). `verdicts_url` is `https://verdicts.upnext.games` unless set; set another in Settings › Library data on the computer UpNext runs on, with `upnext config set verdicts_url <address>`, or with `UPNEXT_VERDICTS_URL`.

## The endpoint

Every path is relative to `verdicts_url`. Bodies are JSON, at most 8 KB. Errors are a non-2xx status with `{"error": "<message>"}`, and a refused verdict adds `"problems": [{"field": "text", "why": "has an email address; take it out to send"}]`.

| Status | When |
|---|---|
| `400` | The body isn't a verdict: an unknown field, a missing or unknown answer, a text check, a licence other than `CC0-1.0`, a consent version older than the oldest the service accepts |
| `401` | No key, or not a key's shape |
| `403` | The key is blocked; its writes are refused and its verdicts leave the next snapshot |
| `404` | No such verdict of this key |
| `413`, `415` | A body over 8 KB, or not `application/json` |
| `429` | A limit is reached; `Retry-After` says in seconds when to try again, and UpNext keeps the verdict waiting and sends it again then, while it runs |

- **`POST /v1/verdicts`** sends a verdict, or replaces the key's earlier verdict about the same game (the same `steam_appid`, or, when either has none, the same `gog_id`). `201` for a new one and `200` for a replacement, with `{"id": "<26 characters>", "state": "live", "replaced": false}`. `state` is `held` when the game is being brigaded ([How the service decides](verdicts/service.md)).
- **`DELETE /v1/verdicts/{id}`** withdraws one of the key's verdicts: `204`, or `404`.
- **`DELETE /v1/verdicts`** withdraws all of the key's verdicts: `200` with `{"withdrawn": 12}`. UpNext's "Take down all my verdicts and forget my key" does this, then deletes the key on the install.
- **`GET /v1/verdicts/mine`** lists the key's verdicts, so a restored install can show what it sent: `{"verdicts": [{"id": "…", "state": "live", "created_day": "2026-10-09", "updated_day": "2026-10-09", …the verdict's fields but client}]}`. `state` is `live`, `held`, `withdrawn` (until its answers are deleted) or `removed` (by a moderator).
- **`POST /v1/reports`** reports a published text: `{"verdict_id": "…", "reason": "spam"}`, where `reason` is `spam`, `personal` (personal information), `abuse` (abuse or hate) or `off_topic` (not about this game). `202` with `{}`; the same key reporting the same verdict again counts once. `404` for an id that isn't published.
- **`GET /terms`** is the service's terms and privacy page. **`/admin`** is the moderators' pages, behind Cloudflare Access, and isn't part of this contract.

### Limits

| | Limit |
|---|---|
| New verdicts per key | 20 a day |
| Replacements per key | 10 a day |
| Reports per key | 20 a day |
| Writes per client address | 60 a day across all keys. An IPv6 address counts by its /64 |

## Changing the contract

- **A new answer or question** is a new vocabulary: bump `VOCAB` in `src/verdicts/vocab.rs`. Older verdicts stay readable with their own `vocab`.
- **New consent words** are a new consent version: bump `CONSENT_VERSION` in `src/verdicts/consent.rs` and pin the new text in its test. Raise `OLDEST_CONSENT` in `src/verdicts/verdict.rs` only when older consent must stop being accepted.
- **A different file layout** is a new format: bump `FORMAT` in `src/verdicts/format.rs`. Readers refuse a newer format and say to update.

After any of these, `UPNEXT_UPDATE_VERDICTS=1 cargo test verdicts` rewrites `docs/verdicts-contract.json` and the fixture snapshot. Keep them with the change, and the service follows from the same files.
