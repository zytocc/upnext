# Running the backlog verdicts service

Run everything here from `services/verdicts/` on your own machine. The service's README says what it does and how to test it locally.

## The free tier

Cloudflare's free tier covers the service until it has many thousands of verdicts; check Cloudflare's current limits before launch.

| | Free tier | What the service uses |
|---|---|---|
| Workers requests | 100,000 a day ([limits](https://developers.cloudflare.com/workers/platform/limits/)) | One per verdict sent or report, and one small manifest request per install per day |
| Workers CPU | 10 ms per request and per Cron Trigger run | The daily run's CPU grows with the number of published verdicts |
| R2 | 10 GB-month, 1 million writes and 10 million reads a month, no egress charge ([pricing](https://developers.cloudflare.com/r2/pricing/)) | Three writes and a list a day; one read per download |

The first limit you will hit is the daily run's 10 ms of CPU, with many thousands of verdicts. `npx wrangler tail` then shows the scheduled run ending with Error 1102 (CPU time exceeded), and `/verdicts.json` stops getting a new date. The fix is Workers Paid, $5 a month per account ([pricing](https://developers.cloudflare.com/workers/platform/pricing/)), with no code change. Installs keep the last snapshot meanwhile.

## First deployment

You need Node 22 or newer and a Cloudflare account (the free plan). The real ids go in `wrangler.local.toml`, which git ignores; none goes in the repository.

1. **Sign in and install.**
   ```sh
   cd services/verdicts
   npm ci
   npx wrangler login          # opens a browser; `npx wrangler whoami` checks it
   ```
2. **Make the database and the bucket.**
   Turn R2 on once per account first (dashboard, Storage & Databases › R2 Object Storage; it asks for a payment method even on the free plan), or `r2 bucket create` fails with error 10042.
   ```sh
   npx wrangler d1 create upnext-verdicts
   npx wrangler r2 bucket create upnext-verdicts
   ```
   `d1 create` prints the database's `database_id`. The bucket stays private.
3. **Make your own configuration.**
   ```sh
   cp wrangler.toml wrangler.local.toml
   ```
   In `wrangler.local.toml`, set `database_id` to the id from step 2. `OPERATOR` and `CONTACT_URL`, shown on `/terms`, name "UpNext contributors" and the repository's GitHub issues; a copy run by someone else sets its own. Leave `ACCESS_TEAM_DOMAIN` and `ACCESS_AUD` empty for now; the admin pages answer 404 until they're set.
4. **Create the tables and deploy.**
   ```sh
   npm run migrate:remote      # wrangler d1 migrations apply upnext-verdicts --remote --config wrangler.local.toml
   npm run deploy              # wrangler deploy --config wrangler.local.toml
   ```
   `migrate:remote` asks for confirmation. Pass `--remote` or `--local` yourself when you run wrangler's D1 commands by hand. `deploy` prints the Worker's address, `https://upnext-verdicts.<your-subdomain>.workers.dev`, and registers the daily Cron Trigger (03:17 UTC). The account's `workers.dev` subdomain is part of that public address, so choose a neutral one in the dashboard (Workers & Pages, the account's subdomain).
5. **Give it a domain of its own.** With a domain in the same Cloudflare account, add `routes = [{ pattern = "verdicts.<your domain>", custom_domain = true }]` and `workers_dev = false` to `wrangler.local.toml` and deploy again: the Worker then answers only at `https://verdicts.<your domain>`. The rest of this file calls it the service's address.
6. **Check it.** `curl https://<the service's address>/` answers with a short text, and `/terms` shows the terms page; read them before launch. `/verdicts.json` answers 404 until the first daily run has written a snapshot.
7. **Give UpNext the address.** The project's own service is `https://verdicts.upnext.games`, set as `verdicts::DEFAULT_URL` in `src/verdicts.rs`. A copy run by someone else sets the `verdicts_url` setting instead.

## The moderators' pages

`/admin` lists reported texts, held games and the newest texts, finds a verdict by id, and acts on one: hide or show its text, remove it, release held verdicts, and block or unblock the key that wrote it. Each action is logged with the moderator's sign-in address and the day (`/admin/log`). The pages need Cloudflare Access in front of them; without it they stay closed (404).

1. In the Cloudflare dashboard, open **Zero Trust** and choose a team name when asked. Your team's address is `https://<team>.cloudflareaccess.com`. The free plan is enough.
2. In Zero Trust's identity provider settings, a one-time PIN by email works with no setup. To sign in with GitHub instead, add GitHub there, with `https://<team>.cloudflareaccess.com/cdn-cgi/access/callback` as the OAuth app's callback URL ([GitHub](https://developers.cloudflare.com/cloudflare-one/integrations/identity-providers/github/)).
3. Under **Access controls › Applications**, add a **self-hosted** application:
   - Domain: the service's address from step 5 above, with two paths, `admin` and `admin/*` (a path ending in `/*` doesn't cover the path itself, [app paths](https://developers.cloudflare.com/cloudflare-one/policies/access/app-paths/)).
   - A policy that allows the moderators' email addresses, and nobody else.
   - Don't use the Worker's own **"Protect this Worker behind Access"** switch, on the Worker's page in the dashboard: it would put the whole Worker behind Access, including the endpoint installs send to and the snapshot they download.
   - If the dashboard won't take a path on the `workers.dev` address, give the Worker a custom domain of its own and add the application there; Cloudflare's notes on Access for `workers.dev` addresses are in [Workers › Cloudflare Access](https://developers.cloudflare.com/workers/configuration/cloudflare-access/).
4. Open the application's **Additional settings** and copy its **Application Audience (AUD) Tag**.
5. In `wrangler.local.toml`, set `ACCESS_TEAM_DOMAIN = "https://<team>.cloudflareaccess.com"` and `ACCESS_AUD` to the tag, then `npm run deploy` again.
6. Open `https://<the service's address>/admin`: Access asks you to sign in, and the page says who you're signed in as. In a private window without signing in, the same address must not show the page.

## A rate limit in front (optional)

A Cloudflare rate limiting rule in front of the Worker answers a burst before it reaches the Worker, so refused requests don't use the free tier's 100,000 a day. Rules belong to a zone, so the Worker needs a custom domain in a zone of yours.

1. Give the Worker a custom domain (step 5 of the first deployment) and use that address as `verdicts_url`.
2. In that zone, open **Security › WAF › Rate limiting rules** and create one rule (the free plan allows one):
   - If incoming requests match: URI Path starts with `/v1/`.
   - With the same characteristics: IP.
   - When rate exceeds: 20 requests per 10 seconds.
   - Then: Block, for 10 seconds.
3. Check it: about 25 quick `curl -s -o /dev/null -w '%{http_code}\n' -X DELETE -H 'Authorization: Bearer upnext_v_<64 zeros>' https://<your domain>/v1/verdicts` show `200` and then `429`.

`/verdicts.json` and the snapshot files are outside `/v1/` and aren't counted.

## Moderating

A new text reaches installs only with the next daily snapshot (03:17 UTC), so a glance at "New texts since yesterday" before then catches most problems.

- **Reported texts.** A text reported by three different keys, each with a published verdict of its own, is held out of the snapshot. Show it again if it's fine (its reports are cleared), or hide it. Its verdict's answers stay counted either way. A text with fewer reports, or reports from keys with no published verdict (which cost nothing to make), is listed too, still shown: hide it, or dismiss its reports if it's fine.
- **Held games.** A game that suddenly gets many new verdicts has them held. Release them one at a time or all of the game's at once; held verdicts nobody reported are released by the daily run after seven days.
- **Removing a verdict** takes it out of the next snapshot and its answers and text are deleted by that run. Use it for personal information or abuse; the writer's key can still send other verdicts.
- **Blocking a key** takes all its verdicts out of the next snapshot and refuses its new verdicts and reports. Unblocking puts its live verdicts back.
- **A takedown request** from someone who lost their key: find the verdict by its id (the published files carry it) and remove it.

## Watching it

- `npx wrangler tail --config wrangler.local.toml` streams the Worker's log. Each daily run logs one line, `{"daily":{"date":…,"games":…,"verdicts":…,"released":…,"cleared":…,"deletedFiles":[…]}}`; a refused admin request logs why.
- `curl -s https://<the service's address>/verdicts.json` shows the latest snapshot's date and counts; it should be today's after 03:17 UTC.
- `upnext verdicts check <folder>` checks a snapshot after you download `verdicts.json` and its two files into a folder.

## Updating

```sh
cd services/verdicts
git pull
npm ci
npm run migrate:remote     # only applies migrations not applied yet
npm run deploy
```

When `wrangler.toml` changes in the repository, make the same change in `wrangler.local.toml`.

## Running it locally

`npm run migrate:local` and `npm run dev` run the Worker on `http://localhost:8787` with a local D1 database and bucket under `.wrangler/`, and `curl "http://localhost:8787/cdn-cgi/local/scheduled?cron=17+3+*+*+*"` runs the daily snapshot. No account is needed, and nothing local reaches the deployed service.
