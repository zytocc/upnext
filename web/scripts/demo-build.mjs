// Builds the online demo into dist-demo/: the web UI in demo mode
// (src/demo/), Cloudflare Pages' header and redirect files from demo/, and
// the third-party notices that Settings › About links to.
import { copyFileSync, readdirSync } from 'node:fs'
import { join } from 'node:path'
import { fileURLToPath } from 'node:url'
import { build } from 'vite'

const web = fileURLToPath(new URL('..', import.meta.url))
const out = join(web, 'dist-demo')

await build({ root: web, mode: 'demo', build: { outDir: out, emptyOutDir: true } })
for (const f of readdirSync(join(web, 'demo'))) copyFileSync(join(web, 'demo', f), join(out, f))
copyFileSync(join(web, '../THIRD_PARTY_NOTICES.txt'), join(out, 'third-party-notices.txt'))
console.log(`Built the demo in ${out}`)
