// Records what the online demo answers with (src/demo/recorded.json): the
// mock library's first player alone, served by a real `upnext serve`, and
// every read the web UI makes of it. The lineups come from a stand-in model
// in this file that picks the games written down in LINEUPS, with the
// reasons in REASONS, so each request gets games that fit it.
//
//   node scripts/demo-record.mjs          writes src/demo/recorded.json
//   node scripts/demo-record.mjs --check  records again and fails when the
//                                         answers' shape (keys and types)
//                                         differs from the committed file
//
// It needs a built server: target/debug/upnext, or UPNEXT_BIN. Nothing is
// read from this machine and nothing is sent anywhere but the two local
// servers it starts.
import { spawn, spawnSync } from 'node:child_process'
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs'
import { createServer } from 'node:http'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
import { fileURLToPath } from 'node:url'
import { verdictsService } from '../e2e/fake-verdicts.mjs'

const here = fileURLToPath(new URL('.', import.meta.url))
const out = join(here, '../src/demo/recorded.json')
const bin = process.env.UPNEXT_BIN ?? join(here, '../../target/debug/upnext')
const check = process.argv.includes('--check')
const port = Number(process.env.UPNEXT_DEMO_PORT ?? 8796)
const fakePort = Number(process.env.UPNEXT_DEMO_FAKE_PORT ?? 8795)
const base = `http://127.0.0.1:${port}`

// What the demo's Settings shows in place of the scratch folder and the
// stand-in model's address.
const SHOWN_DATA = '/home/you/.local/share/upnext'
const SHOWN_HOME = '/home/you'
const SHOWN_AI = 'http://den.lan:8080/v1'
const SHOWN_MODEL = 'demo'
const SHOWN_VERDICTS = 'https://verdicts.upnext.games'

// The lineups the demo can show: one for each routine (by its name), the
// typed requests with the words the demo matches a visitor's request on,
// and their picks and spares by name, best first.
const LINEUPS = [
  { routine: 'Weeknight wind-down', games: ['Citizen Sleeper', 'Dorfromantik', 'Baba Is You', 'Mini Metro', 'ISLANDERS', 'A Short Hike'] },
  { routine: 'Saturday co-op', games: ['Valheim', 'Terraria', 'Divinity: Original Sin 2', 'Satisfactory', 'Cuphead', 'Northgard'] },
  { routine: 'Couch with a pad', games: ['Cuphead', 'Ori and the Will of the Wisps', 'Spiritfarer', 'TUNIC', 'Hollow Knight', "Death's Door"] },
  // The Pick page's "Or try" suggestions, as a visitor clicks them.
  {
    prompt: 'A quiet hour',
    words: ['quiet', 'hour', 'calm', 'cozy', 'cosy', 'relax', 'relaxing', 'chill', 'tonight', 'gentle', 'tired'],
    games: ['A Short Hike', 'Unpacking', 'ISLANDERS', 'Mini Metro', 'Dorfromantik', 'Spiritfarer'],
  },
  {
    prompt: 'A weekend story',
    words: ['weekend', 'long', 'rpg', 'epic', 'fantasy', 'sink', 'big', 'saga'],
    games: ["Baldur's Gate 3", 'Disco Elysium - The Final Cut', 'Mass Effect Legendary Edition', 'Pentiment', 'Divinity: Original Sin 2', 'Kentucky Route Zero: TV Edition'],
  },
  {
    prompt: 'Something I can actually finish',
    words: ['finish', 'end', 'ending', 'complete', 'beat', 'short'],
    games: ['Gone Home', "Hellblade: Senua's Sacrifice", 'Citizen Sleeper', 'Night in the Woods', 'A Short Hike', 'Unpacking'],
  },
  {
    prompt: 'Co-op night',
    words: ['co-op', 'coop', 'friends', 'friend', 'together', 'multiplayer', 'online', 'party'],
    games: ['Valheim', 'Terraria', 'Satisfactory', 'Divinity: Original Sin 2', 'Cuphead', 'Northgard'],
  },
  // These two are pick modes, from candidates the server chooses. The mock
  // library has two ways into a new genre, so New to me shows two.
  { prompt: 'New to me', words: ['new', 'different', 'genre'], count: 2, games: ['Dorfromantik', 'Brotato'] },
  { prompt: 'A hidden gem', words: ['gem', 'gems', 'hidden', 'underrated'], games: [] },
  {
    prompt: 'Surprise me',
    words: ['surprise', 'anything', 'whatever', 'something'],
    games: ['Outer Wilds', 'Return to Monkey Island', 'Slay the Spire', "Heaven's Vault", 'Ori and the Will of the Wisps', 'Baba Is You'],
  },
  // The request box's example, and two more kinds of request.
  {
    prompt: 'About 2 hours, something atmospheric with a good story',
    words: ['story', 'atmospheric', 'atmosphere', 'narrative', 'hours', 'evening', 'mood', 'moody'],
    games: ['Kentucky Route Zero: TV Edition', "Hellblade: Senua's Sacrifice", 'Gone Home', 'Night in the Woods', "Heaven's Vault", 'Citizen Sleeper'],
  },
  {
    prompt: 'Quick runs I can stop at any time',
    words: ['quick', 'run', 'runs', 'roguelike', 'roguelite', 'minutes', 'break', 'stop', 'lunch', 'fast'],
    games: ['Slay the Spire', 'Vampire Survivors', 'Monster Train', 'Brotato', 'FTL: Faster Than Light', 'Loop Hero'],
  },
  {
    prompt: 'A puzzle game that makes me think',
    words: ['puzzle', 'puzzles', 'think', 'brain', 'clever', 'logic', 'mystery'],
    games: ['Baba Is You', 'Outer Wilds', "Heaven's Vault", 'Mini Metro', 'Return to Monkey Island', 'Dorfromantik'],
  },
]

// Why each game fits, as a model would say it.
const REASONS = {
  'Citizen Sleeper': 'A quiet, tender story told a day at a time, so an hour ends at a natural stop.',
  Dorfromantik: 'Placing tiles is calm and unhurried, and a round ends on its own when the stack runs out.',
  'Baba Is You': 'Each level is a few minutes of thinking, so you can stop after any one of them.',
  'Mini Metro': 'A map takes about fifteen minutes, and it gets busy without ever getting loud.',
  ISLANDERS: 'A whole island fits in half an hour, with nothing to manage between sessions.',
  'A Short Hike': 'Nothing in it hurries you, and you can see the summit in one sitting.',
  Valheim: 'Building a longhouse and sailing out together fills an afternoon, and it rewards talking it through.',
  Terraria: 'Easy to drop into together, with something to dig up or build whatever each of you likes.',
  'Divinity: Original Sin 2': 'A full party campaign where friends can split up, scheme and still finish a fight together.',
  Satisfactory: 'Factories are better planned out loud, and a long session shows real progress.',
  Cuphead: 'Short, hard fights that two players can retry together without losing much time.',
  'Ori and the Will of the Wisps': 'Beautiful and flowing on a pad, with save points close together.',
  Spiritfarer: 'Gentle and warm, and it plays well with a pad from the couch.',
  TUNIC: 'A small, secretive adventure that feels right on a controller.',
  'Hollow Knight': 'Tight and atmospheric on a pad, with benches that make good places to stop.',
  "Death's Door": 'Short, sharp areas and great controller feel, so a session always gets somewhere.',
  Northgard: 'Matches against or beside friends fit an afternoon, and talking tactics is half the fun.',
  'Gone Home': 'A short, quiet story you uncover room by room, and an evening takes you to the end.',
  'Kentucky Route Zero: TV Edition': 'Each act is about an hour of strange, beautiful road trip, and it lingers after.',
  "Hellblade: Senua's Sacrifice": 'Intense and atmospheric, with a story that holds you the whole way through.',
  'Night in the Woods': 'Funny and sad small-town days, told in chapters that each make a good evening.',
  "Heaven's Vault": 'You piece together an old language as you go, so every session leaves you a little sharper.',
  Unpacking: 'Calm and wordless: you learn a life from the things in the boxes, and a few moves fill an hour.',
  "Baldur's Gate 3": 'A huge, reactive campaign that rewards a whole weekend, and you have never started it.',
  'Disco Elysium - The Final Cut': 'All story and choices, with writing that makes long sessions fly by.',
  'Mass Effect Legendary Edition': 'Three games of space opera to settle into, with a crew you will get attached to.',
  Pentiment: 'A slow, beautifully drawn mystery that suits a long, unhurried read.',
  'Skyrim Special Edition': 'Endless to wander, so a weekend never runs out of places to go.',
  'Slay the Spire': 'A run is under an hour and every floor is a clean place to stop.',
  'Vampire Survivors': 'Thirty-minute runs that need almost no thought to start.',
  'Monster Train': 'Fast deckbuilding runs that you can pause between any two battles.',
  Brotato: 'Waves last a minute or so, and a whole run fits in a coffee break.',
  'FTL: Faster Than Light': 'Each jump is a small decision, so you can step away between any two of them.',
  'Loop Hero': 'The loop plays itself while you plan, and it pauses whenever you need it to.',
  'Outer Wilds': 'You tried it once; it is all curiosity and clues, and it clicks once a loop or two makes sense.',
  'Hotline Miami': 'Short, loud and brilliant: a level takes minutes, and the soundtrack carries you through.',
  'Return to Monkey Island': 'Gentle puzzles and good jokes, with chapters that break an evening into easy pieces.',
}

// The lineup the next pick call is for: its games and the routine's name or the request.
let current = null

/** What the stand-in model answers: the current lineup's games, in its
 *  order, from the candidates the server sent. */
function answer(body) {
  const prompt = body.messages?.map((m) => m.content).join('\n') ?? ''
  const schema = body.response_format?.json_schema?.schema?.properties ?? {}
  const candidates = [...prompt.matchAll(/^#(\d+) ([^|[\n]+)/gm)].map((m) => ({ id: Number(m[1]), name: m[2].trim() }))
  const reply = (content) => ({
    model: SHOWN_MODEL,
    usage: { prompt_tokens: 6000, completion_tokens: 300, total_tokens: 6300 },
    choices: [{ finish_reason: 'stop', message: { role: 'assistant', content: JSON.stringify(content) } }],
  })
  if (schema.pick_id) return reply({ pick_id: candidates[0]?.id ?? null, reason: 'Shorter, and closer to what you enjoy.' })
  const wanted = (current?.games ?? []).map((n) => candidates.find((c) => c.name === n)).filter(Boolean)
  const missing = (current?.games ?? []).filter((n) => !candidates.some((c) => c.name === n))
  // A game the server no longer offers for this request leaves a gap that
  // the first other candidates fill.
  if (missing.length) console.warn(`Not among the candidates for ${current.routine ?? current.prompt}: ${missing.join(', ')}`)
  const n = schema.picks?.maxItems ?? 3
  const spares = schema.spares?.maxItems ?? 0
  for (const c of candidates) if (wanted.length < n && !wanted.includes(c)) wanted.push(c)
  const games = wanted.map((c) => ({ id: c.id, name: c.name, reason: REASONS[c.name] ?? 'Fits the request.' }))
  return reply({ picks: games.slice(0, n), spares: games.slice(n, n + spares) })
}

function fakeModel() {
  return createServer((req, res) => {
    let raw = ''
    req.on('data', (chunk) => (raw += chunk))
    req.on('end', () => {
      if (req.method === 'GET' && req.url === '/health') return void res.writeHead(200).end('ok')
      if (req.method === 'GET' && req.url === '/v1/models') {
        return void res.writeHead(200, { 'content-type': 'application/json' }).end(JSON.stringify({ data: [{ id: SHOWN_MODEL }] }))
      }
      if (verdictsService(req, res, raw)) return
      if (req.method !== 'POST' || !req.url?.endsWith('/chat/completions')) return void res.writeHead(404).end()
      try {
        const body = JSON.stringify(answer(JSON.parse(raw)))
        res.writeHead(200, { 'content-type': 'application/json' }).end(body)
      } catch (e) {
        console.error(e instanceof Error ? e.message : e)
        res.writeHead(500).end()
      }
    })
  }).listen(fakePort, '127.0.0.1')
}

const data = mkdtempSync(join(tmpdir(), 'upnext-demo-'))
const home = join(data, 'home', 'you')
const env = {
  ...process.env,
  HOME: home,
  USER: 'sam',
  LOGNAME: 'sam',
  UPNEXT_CONFIG_DIR: data,
  UPNEXT_DATA_DIR: data,
  UPNEXT_STEAM_DIR: join(data, 'no-steam'),
  UPNEXT_HEROIC_DIR: join(data, 'no-heroic'),
  UPNEXT_LLM: 'openai',
  UPNEXT_OPENAI_BASE_URL: `http://127.0.0.1:${fakePort}/v1`,
  UPNEXT_OPENAI_MODEL: SHOWN_MODEL,
  UPNEXT_VERDICTS_URL: `http://127.0.0.1:${fakePort}/verdicts`,
}
delete env.USERNAME

let server = null
let model = null
function stop() {
  server?.kill()
  model?.close()
  rmSync(data, { recursive: true, force: true })
}

async function waitFor(url) {
  for (let i = 0; i < 300; i++) {
    try {
      if ((await fetch(url)).ok) return
    } catch {
      /* not up yet */
    }
    await new Promise((r) => setTimeout(r, 200))
  }
  throw new Error(`${url} never answered`)
}

class Refused extends Error {
  constructor(status, body) {
    super(`${status} ${JSON.stringify(body)}`)
    this.status = status
    this.body = body
  }
}

async function call(method, path, body) {
  const res = await fetch(base + path, {
    method,
    headers: { Accept: 'application/json', ...(body ? { 'Content-Type': 'application/json' } : {}) },
    body: body ? JSON.stringify(body) : undefined,
  })
  if (!res.ok) throw new Refused(res.status, await res.json().catch(() => null))
  return res.status === 204 ? null : res.json()
}

// The reads the pages make that take no id. The library itself and each
// game are recorded apart (`games`).
const READS = [
  '/api/v1/profile',
  '/api/v1/config',
  '/api/v1/stats',
  '/api/v1/second-chances',
  '/api/v1/review-tags',
  '/api/v1/gems',
  '/api/v1/gems?all=true',
  '/api/v1/wishlist',
  '/api/v1/taste',
  '/api/v1/taste/sources',
  '/api/v1/taste/sources/reactions',
  '/api/v1/taste/sources/ratings',
  '/api/v1/taste/sources/diary',
  '/api/v1/taste/questions',
  '/api/v1/stop-reasons',
  '/api/v1/routines',
  '/api/v1/jobs?limit=20',
  '/api/v1/jobs/scheduled',
  '/api/v1/sync/sources',
  '/api/v1/enrich/sources',
  '/api/v1/gog',
  '/api/v1/settings',
  '/api/v1/collections',
  '/api/v1/llm/models',
  '/api/v1/llm/codex',
  '/api/v1/llm/copilot',
  '/api/v1/llm/openai',
  '/api/v1/ai/usage?days=30',
  '/api/v1/ai/plan',
  '/api/v1/ai/prices',
  '/api/v1/ai/estimates',
  '/api/v1/backups',
  '/api/v1/seed',
  '/api/v1/tokens',
  '/api/v1/pairing',
  '/api/v1/pairing/requests',
  '/api/v1/steam/shortcut',
  '/api/v1/setup',
  '/api/v1/setup/state',
  '/api/v1/epic',
  '/api/v1/library/by-hand',
  '/api/v1/library/added',
  '/api/v1/health',
  '/api/v1/network/computers',
  '/api/v1/network/connect',
  '/api/v1/oidc',
  '/api/v1/players',
  '/api/v1/accounts',
  '/api/v1/ai-accounts',
  '/api/v1/verdicts',
  '/api/v1/verdicts/vocab',
  '/api/v1/verdicts/consent',
  '/api/v1/verdicts/community',
]

// The other reads about one game, for every game in the library.
const GAME_READS = [
  (id) => `/api/v1/games/${id}/next-in-series`,
  (id) => `/api/v1/games/${id}/series`,
  (id) => `/api/v1/games/${id}/verdicts`,
  (id) => `/api/v1/new-to-you/${id}`,
  (id) => `/api/v1/verdicts/draft/${id}`,
]

async function record() {
  const mock = spawnSync(bin, ['mock-library', '--one-player'], { env, encoding: 'utf8' })
  if (mock.status !== 0) throw new Error(`mock-library failed: ${mock.error ?? mock.stderr}`)
  // Another server already on the port would answer in this one's place.
  if (await fetch(`${base}/api/health`).then(() => true, () => false)) throw new Error(`something already listens on port ${port}`)
  model = fakeModel()
  server = spawn(bin, ['serve', '--port', String(port)], { env, stdio: 'ignore' })
  await waitFor(`http://127.0.0.1:${fakePort}/health`)
  await waitFor(`${base}/api/health`)

  // A read that says no (a 404 when there's nothing to show) is kept with
  // its status, and the demo answers it the same way.
  // The player's own name, so the demo doesn't ask for one.
  await call('PUT', '/api/v1/players/1', { name: 'Alex' })
  const answers = {}
  const refused = {}
  const read = async (path) => {
    try {
      answers[path] = await call('GET', path)
    } catch (e) {
      if (!(e instanceof Refused) || e.status >= 500) throw e
      refused[path] = { status: e.status, error: e.body?.error ?? '' }
    }
  }
  for (const path of READS) await read(path)
  const list = await call('GET', '/api/v1/games?limit=1000')
  const games = []
  for (const g of list.items) {
    games.push(await call('GET', `/api/v1/games/${g.id}`))
    for (const path of GAME_READS) await read(path(g.id))
  }

  // Each lineup is asked once, in a library where nothing was turned down.
  const routines = answers['/api/v1/routines'].routines
  const lineups = []
  for (const l of LINEUPS) {
    current = l
    if (l.routine) {
      const r = routines.find((r) => r.name === l.routine)
      if (!r) throw new Error(`the mock library has no routine ${l.routine}`)
      lineups.push(await call('POST', '/api/v1/picks', { prompt: '', count: 3, routine_id: r.id }))
    } else {
      lineups.push({ words: l.words, ...(await call('POST', '/api/v1/picks', { prompt: l.prompt, count: l.count ?? 3 })) })
    }
  }
  current = null
  // A roll of the dice: the demo draws its own games, so only the wording is kept.
  const { picks: drawn, spares: _spares, ...rolled } = await call('POST', '/api/v1/picks', { prompt: '', count: 3, mode: 'random' })
  const random = { ...rolled, picks: [], spares: [], reason: drawn[0].reason }

  const settings = answers['/api/v1/settings']
  settings.listen.port = 8787
  const recorded = { recorded_at: Math.floor(Date.now() / 1000), answers, refused, games, lineups, random }
  return JSON.parse(
    JSON.stringify(recorded)
      .replaceAll(`http://127.0.0.1:${fakePort}/v1`, SHOWN_AI)
      .replaceAll(`http://127.0.0.1:${fakePort}/verdicts`, SHOWN_VERDICTS)
      .replaceAll(home, SHOWN_HOME)
      .replaceAll(data, SHOWN_DATA),
  )
}

/** A value's keys, all the way down: an array's items are merged, so a
 *  key that only some items have counts. Values themselves don't count:
 *  they change with the date and the dice. */
function keys(v) {
  if (Array.isArray(v)) return v.map(keys).reduce(merge, null)
  if (v && typeof v === 'object') return Object.fromEntries(Object.keys(v).map((k) => [k, keys(v[k])]))
  return null
}

function merge(a, b) {
  if (!a || !b) return a ?? b
  const out = { ...a }
  for (const [k, v] of Object.entries(b)) out[k] = merge(out[k] ?? null, v)
  return out
}

/** Every key path that one of `a` and `b` has and the other doesn't. A
 *  value that is an object in one and empty or null in the other doesn't
 *  count: a game with no series, say. */
function differences(a, b, path) {
  if (!a || !b) return []
  const out = []
  for (const k of new Set([...Object.keys(a), ...Object.keys(b)])) {
    if (!(k in a)) out.push(`${path}.${k}: new`)
    else if (!(k in b)) out.push(`${path}.${k}: gone`)
    else out.push(...differences(a[k], b[k], `${path}.${k}`))
  }
  return out
}

try {
  const fresh = await record()
  if (check) {
    const committed = JSON.parse(readFileSync(out, 'utf8'))
    const { recorded_at: _a, ...was } = committed
    const { recorded_at: _b, ...now } = fresh
    const diff = differences(keys(was), keys(now), 'recorded')
    if (diff.length) {
      console.error('The demo data no longer matches the API. In web/, run `node scripts/demo-record.mjs` and commit src/demo/recorded.json.')
      for (const d of diff.slice(0, 40)) console.error(`  ${d}`)
      process.exitCode = 1
    } else {
      console.log('The demo data matches the API.')
    }
  } else {
    writeFileSync(out, JSON.stringify(fresh) + '\n')
    console.log(`Wrote ${out}`)
  }
} catch (e) {
  console.error(e instanceof Error ? e.message : e)
  process.exitCode = 1
} finally {
  stop()
}
