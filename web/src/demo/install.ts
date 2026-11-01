// The online demo's start: the web UI's API calls go to the stand-in in
// server.ts, in this tab, and nothing goes to a server. Only a build made
// with `npm run build:demo` loads this file.
import { createDemo, type Recorded } from './server'
// The recording loads as a file of its own, about a megabyte before compression.
import recordedUrl from './recorded.json?url'

/** How long an answer takes, so loading states show as they would. A
 *  lineup takes longer, as the real one asks an AI. */
const wait = (method: string, path: string) => (method === 'POST' && path.endsWith('/api/v1/picks') ? 900 : 40)

export async function install() {
  const real = window.fetch.bind(window)
  const demo = createDemo((await (await real(recordedUrl)).json()) as Recorded)
  window.fetch = async (input, init) => {
    const href = typeof input === 'string' ? input : input instanceof URL ? input.href : input.url
    const url = new URL(href, window.location.href)
    if (url.origin !== window.location.origin || !url.pathname.startsWith('/api/')) return real(input, init)
    const method = (init?.method ?? (input instanceof Request ? input.method : 'GET')).toUpperCase()
    let body: unknown
    if (typeof init?.body === 'string') {
      try {
        body = JSON.parse(init.body)
      } catch {
        body = undefined
      }
    }
    const answer = demo.answer(method, url.pathname + url.search, body)
    await new Promise((r) => setTimeout(r, wait(method, url.pathname)))
    const text = answer.body === undefined || answer.status === 204 ? null : JSON.stringify(answer.body)
    return new Response(text, { status: answer.status, headers: { 'Content-Type': 'application/json' } })
  }
}
