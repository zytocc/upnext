// Builds the online demo into dist-demo/: the web UI in demo mode
// (src/demo/), Cloudflare Pages' header and redirect files from demo/, and
// the third-party notices that Settings › About links to.
import { createHash } from 'node:crypto'
import { copyFileSync, readFileSync, readdirSync, writeFileSync } from 'node:fs'
import { join } from 'node:path'
import { fileURLToPath } from 'node:url'
import { build } from 'vite'

const web = fileURLToPath(new URL('..', import.meta.url))
const out = join(web, 'dist-demo')

await build({ root: web, mode: 'demo', build: { outDir: out, emptyOutDir: true } })
for (const f of readdirSync(join(web, 'demo'))) copyFileSync(join(web, 'demo', f), join(out, f))
// The page's inline scripts (the theme, set before the app loads) run by
// their hashes, so the Content-Security-Policy allows no other inline script.
const html = readFileSync(join(out, 'index.html'), 'utf8')
const hashes = [...html.matchAll(/<script>([\s\S]*?)<\/script>/g)].map((m) => `'sha256-${createHash('sha256').update(m[1]).digest('base64')}'`)
const headers = join(out, '_headers')
writeFileSync(headers, readFileSync(headers, 'utf8').replace("'self' INLINE_SCRIPTS", ["'self'", ...hashes].join(' ')))
copyFileSync(join(web, '../THIRD_PARTY_NOTICES.txt'), join(out, 'third-party-notices.txt'))
console.log(`Built the demo in ${out}`)
