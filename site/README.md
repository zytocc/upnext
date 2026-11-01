# upnext.games

UpNext's website: one page in plain HTML and CSS, with no build step, no outside scripts or fonts, and no trackers. `main.js` only types the example requests and adds copy buttons. The colours and fonts are the app's own (`web/src/index.css`), with the fonts' licences in `fonts/`.

To look at it, serve the folder (`python3 -m http.server -d site 8080`) and open http://127.0.0.1:8080.

## Screenshots

`images/pick.webp` is `docs/images/pick.png` as WebP; the others are of the made-up library from `upnext mock-library`. Each has smaller and larger sizes for the page's `srcset`. After a new screenshot (`UPNEXT_README_SHOT=1 npx playwright test readme-shot` in `web/`, which also writes `web/test-results/pick-3200.png`), make the sizes again:

```sh
convert docs/images/pick.png -resize 800x -quality 82 site/images/pick-800.webp
```

## Deploying

The site is a Cloudflare Pages project on `upnext.games`, separate from the verdicts service on `verdicts.upnext.games`. Its settings: no framework preset, an empty build command, `site` as the build output directory, `site/*` as the only build watch path, and Web Analytics off. To deploy by hand: `npx wrangler pages deploy site --project-name upnext-site`.

`_headers` sets the security headers, including a content security policy that allows only the site's own files. `404.html` is the page for addresses that don't exist.

## install.sh

`install.sh` is the Linux installer behind `curl -fsSL https://upnext.games/install.sh | sh` ([Linux](../docs/install/linux.md#one-command)). It finds the newest release each time it runs, so a release needs no change here. A change to it reaches everyone as soon as the site deploys: check it with `shellcheck -s sh site/install.sh` and run it on each kind of distribution it handles first.

The Download buttons go to `releases/latest`, the newest full release.
