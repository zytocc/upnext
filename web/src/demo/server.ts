// The online demo's stand-in for the UpNext server: it answers the web UI's
// API calls from a recording of the mock library (recorded.json, written
// by scripts/demo-record.mjs) and keeps what a visitor changes in memory,
// in this tab alone. Calls that would reach outside the computer (syncs,
// sign-ins, sending a verdict) or need a real server are turned down.
import type { DecideResult, GameView, HiddenTitle, OverrideStatus, Pick, PickBatch, Routine, Status, TooOutcome } from '../types'

type Json = Record<string, unknown>

/** A recorded lineup; a typed request's also has the words a visitor's request is matched on. */
export type RecordedLineup = PickBatch & { words?: string[] }

export interface Recorded {
  /** When it was recorded, in Unix seconds: every time in it moves forward to today. */
  recorded_at: number
  /** GET answers by path and query. */
  answers: Record<string, unknown>
  /** GET paths the server said no to, with the status and message. */
  refused: Record<string, { status: number; error: string }>
  /** Every game in full, in the library's order. */
  games: GameView[]
  lineups: RecordedLineup[]
  /** A roll of the dice, with no games: the demo draws its own, with this reason. */
  random: PickBatch & { reason: string }
}

export interface DemoAnswer {
  status: number
  body?: unknown
}

/** What a call the demo can't make answers with. */
export const NOT_IN_DEMO =
  'The demo has no server behind it, so it can’t do this. Install UpNext to connect your stores, sync, sign in or send anything.'

// Unix seconds from 2001 to 2039: any whole number in this range in the
// recording is a time.
const FIRST_TIME = 1_000_000_000
const LAST_TIME = 2_200_000_000

/** Every time in `value` moved by `by` seconds, so "last played 3 days ago" stays 3 days ago. */
export function moveTimes<T>(value: T, by: number): T {
  if (Array.isArray(value)) return value.map((v) => moveTimes(v, by)) as T
  if (value && typeof value === 'object') {
    return Object.fromEntries(Object.entries(value).map(([k, v]) => [k, moveTimes(v, by)])) as T
  }
  if (typeof value === 'number' && Number.isInteger(value) && value >= FIRST_TIME && value <= LAST_TIME) return (value + by) as T
  return value
}

const ok = (body?: unknown): DemoAnswer => (body === undefined ? { status: 204 } : { status: 200, body })
const no = (status: number, error: string): DemoAnswer => ({ status, body: { error } })
const clone = <T>(v: T): T => structuredClone(v)

function shuffle<T>(items: T[], random: () => number): T[] {
  const out = [...items]
  for (let i = out.length - 1; i > 0; i--) {
    const j = Math.floor(random() * (i + 1))
    ;[out[i], out[j]] = [out[j], out[i]]
  }
  return out
}

/** The words of a request, lower case. */
const words = (s: string) => s.toLowerCase().match(/[a-z0-9-]+/g) ?? []

export function createDemo(recorded: Recorded, now: () => number = Date.now, random: () => number = Math.random) {
  const seconds = () => Math.floor(now() / 1000)
  const data = moveTimes(clone(recorded), seconds() - recorded.recorded_at)

  const games = new Map(data.games.map((g) => [g.id, g]))
  const order = data.games.map((g) => g.id)
  /** Each game's status before any change, for when a status set here is cleared. */
  const recordedStatus = new Map(data.games.map((g) => [g.id, g.status]))
  const batches = new Map<string, PickBatch>()
  /** Lineups made here, newest first. */
  const history: string[] = []
  const routinesAnswer = data.answers['/api/v1/routines'] as { routines: Routine[] } & Json
  let routines = routinesAnswer.routines
  let settings = data.answers['/api/v1/settings'] as Json
  let made = 0
  let typed = 0

  const game = (id: number) => games.get(id)
  /** A game as a list shows it: without the tags by source, as the server sends it. */
  const listed = (g: GameView): GameView => ({ ...g, tags_by_source: {} })

  function setOverride(g: GameView, status: OverrideStatus | null) {
    g.override = status
    g.status = (status ?? recordedStatus.get(g.id) ?? 'untouched') as Status
    const t = seconds()
    g.shelved_at = status === 'shelved' ? t : null
    g.abandoned_at = status === 'abandoned' ? t : null
  }

  // ---- The library -------------------------------------------------------

  function listGames(q: URLSearchParams) {
    const csv = (k: string) => q.get(k)?.split(',').map((s) => s.trim()).filter(Boolean)
    const yes = (k: string) => q.get(k) === 'true' || q.get(k) === '1'
    const statuses = csv('status')
    const sources = csv('source')
    const text = q.get('q')?.trim().toLowerCase()
    const genre = q.get('genre')?.toLowerCase()
    const tag = q.get('tag')?.toLowerCase()
    const rating = q.get('rating')
    let items = order.map((id) => games.get(id)!).filter((g) => {
      if (statuses && !statuses.includes(g.status)) return false
      if (q.has('staple') && g.staple !== yes('staple')) return false
      if (sources && !g.owned_on.some((s) => sources.includes(s))) return false
      if (yes('installed') && !g.installed) return false
      if (yes('controller') && !['full', 'partial'].includes(String((g as unknown as Json).controller_support))) return false
      if (genre && !g.genres.some((x) => x.toLowerCase() === genre)) return false
      if (tag && !g.tags.some((x) => x.toLowerCase() === tag)) return false
      if (rating === 'rated' && g.my_rating == null) return false
      if (rating === 'unrated' && g.my_rating != null) return false
      if (rating === 'high' && !((g.my_rating ?? 0) >= 4)) return false
      if (rating === 'low' && !(g.my_rating != null && g.my_rating <= 2)) return false
      if (text) {
        const hay = [g.name, ...g.editions, ...g.genres, ...g.tags, ...g.themes, ...g.categories, ...g.developers].join(' ').toLowerCase()
        if (!text.split(/\s+/).every((t) => hay.includes(t))) return false
      }
      return true
    })
    const sort = q.get('sort') ?? 'name'
    const desc = q.get('order') ? q.get('order') === 'desc' : !['name', 'status', 'source', 'linux'].includes(sort)
    const key = (g: GameView): number | string | null => {
      switch (sort) {
        case 'playtime':
          return g.playtime_minutes
        case 'last_played':
          return g.last_played
        case 'review':
          return g.review_pct
        case 'hltb':
          return g.hltb_main
        case 'metacritic':
          return g.metacritic
        case 'rating':
          return g.my_rating ?? null
        case 'release':
          return g.release_date ? Date.parse(g.release_date) || null : null
        case 'status':
          return g.status
        case 'source':
          return g.owned_on.join(',')
        default:
          return g.name.toLowerCase()
      }
    }
    items = [...items].sort((a, b) => {
      const ka = key(a)
      const kb = key(b)
      // Games with nothing to sort by go last either way, as on the server.
      if (ka == null || kb == null) return ka == null ? (kb == null ? 0 : 1) : -1
      const o = ka < kb ? -1 : ka > kb ? 1 : 0
      return (desc ? -o : o) || a.name.toLowerCase().localeCompare(b.name.toLowerCase())
    })
    if (text) {
      // Names that start with the words come first.
      const rank = (g: GameView) => (g.name.toLowerCase() === text ? 0 : g.name.toLowerCase().startsWith(text) ? 1 : 2)
      items = [...items].sort((a, b) => rank(a) - rank(b))
    }
    const limit = Math.min(Math.max(Number(q.get('limit') ?? 100), 1), 1000)
    const offset = Number(q.get('offset') ?? 0)
    return { total: items.length, items: items.slice(offset, offset + limit).map(listed) }
  }

  // ---- Lineups -----------------------------------------------------------

  /** A pick of a game as it is now. */
  const fresh = (p: Pick): Pick => (p.owned !== false && game(p.id) ? { ...p, game: listed(game(p.id)!) } : p)
  const usable = (id: number, skip: Set<number>) => {
    const g = game(id)
    return !!g && !skip.has(id) && !['hidden', 'done', 'played', 'abandoned', 'not_game'].includes(g.status)
  }

  /** The recorded lineup closest to a request: by its routine, or by the words they share, or the next in turn. */
  function lineupFor(prompt: string, routineId: number | null): RecordedLineup {
    if (routineId != null) {
      const r = data.lineups.find((l) => l.routine?.id === routineId)
      if (r) return r
    }
    const typedOnes = data.lineups.filter((l) => l.words)
    const same = typedOnes.find((l) => l.prompt.toLowerCase() === prompt.trim().toLowerCase())
    if (same) return same
    const asked = new Set(words(prompt))
    let best: RecordedLineup | null = null
    let bestScore = 0
    for (const l of typedOnes) {
      const score = (l.words ?? []).filter((w) => asked.has(w)).length
      if (score > bestScore) [best, bestScore] = [l, score]
    }
    if (best) return best
    typed += 1
    return typedOnes[(typed - 1) % typedOnes.length]
  }

  function save(batch: PickBatch) {
    batches.set(batch.batch_id, batch)
    history.unshift(batch.batch_id)
    return batch
  }

  function newBatch(template: PickBatch, prompt: string, picks: Pick[], spares: Pick[], routine: Routine | null): PickBatch {
    made += 1
    return {
      ...clone(template),
      batch_id: `demo-${made}`,
      prompt,
      created_at: seconds(),
      picks: picks.map((p, slot) => fresh({ ...p, slot, feedback: null, rating: null } as Pick)),
      spares: spares.map((p) => fresh({ ...p, slot: null, feedback: null, rating: null } as Pick)),
      removed: [],
      routine,
    }
  }

  function makeLineup(body: Json): DemoAnswer {
    const prompt = typeof body.prompt === 'string' ? body.prompt : ''
    const count = Math.min(Math.max(Number(body.count ?? 3), 1), 6)
    const skip = new Set<number>(Array.isArray(body.exclude_ids) ? (body.exclude_ids as number[]) : [])
    const routineId = typeof body.routine_id === 'number' ? body.routine_id : null
    const routine = routineId != null ? (routines.find((r) => r.id === routineId) ?? null) : null
    if (routineId != null && !routine) return no(404, `no routine ${routineId}`)
    if (body.mode === 'random') {
      const { reason, ...template } = data.random
      const offered = shuffle(
        order.filter((id) => usable(id, skip) && ['untouched', 'sampled'].includes(game(id)!.status)),
        random,
      )
      const pick = (id: number) => ({ id, appid: game(id)!.appid, name: game(id)!.name, reason, slot: null, feedback: null, rating: null, owned: true, game: game(id)! }) as Pick
      const batch = newBatch(template, prompt, offered.slice(0, count).map(pick), offered.slice(count, count + 6).map(pick), routine)
      batch.ranked_by = [`chance alone: drawn at random from ${offered.length} games off your radar`]
      batch.candidates_considered = offered.length
      return ok(save(batch))
    }
    const template = lineupFor(routine ? `${routine.name} ${routine.note ?? ''}` : prompt, routineId)
    const all = [...template.picks, ...template.spares].filter((p) => usable(p.id, skip))
    const batch = newBatch(template, prompt, all.slice(0, count), all.slice(count), routine ?? template.routine ?? null)
    delete (batch as Partial<RecordedLineup>).words
    return ok(save(batch))
  }

  /** Takes a pick off the screen with what was done to it; the best spare takes its slot. */
  function takeOff(batch: PickBatch, pick: Pick, feedback: Pick['feedback']) {
    batch.picks = batch.picks.filter((p) => p.id !== pick.id)
    batch.removed.push({ ...pick, slot: null, feedback })
    const onScreen = new Set(batch.picks.map((p) => p.id))
    const next = batch.spares.find((s) => usable(s.id, onScreen))
    if (next) {
      batch.spares = batch.spares.filter((s) => s.id !== next.id)
      batch.picks.push({ ...next, slot: pick.slot })
      batch.picks.sort((a, b) => (a.slot ?? 0) - (b.slot ?? 0))
    }
  }

  function lineupCall(batchId: string, action: string, body: Json): DemoAnswer {
    const batch = batches.get(batchId)
    if (!batch) return no(404, 'no such lineup')
    if (action === 'more') {
      // New spares from the games off your radar that this lineup hasn't shown.
      const held = new Set([...batch.picks, ...batch.spares, ...batch.removed].map((p) => p.id))
      const reason = 'Another one from your backlog that fits what you asked for.'
      const more = order.filter((id) => usable(id, held) && ['untouched', 'sampled'].includes(game(id)!.status)).slice(0, 6)
      for (const id of more) {
        const g = game(id)!
        batch.spares.push({ id, appid: g.appid, name: g.name, reason, slot: null, feedback: null, rating: null, owned: true, game: listed(g) } as Pick)
      }
      const free = [...Array(batch.shown).keys()].filter((slot) => !batch.picks.some((p) => p.slot === slot))
      for (const slot of free) {
        const next = batch.spares.shift()
        if (next) batch.picks.push({ ...next, slot })
      }
      batch.picks.sort((a, b) => (a.slot ?? 0) - (b.slot ?? 0))
      return ok(batch)
    }
    const titleId = Number(body.title_id)
    const pick = batch.picks.find((p) => p.id === titleId)
    if (!pick) return no(404, 'that game isn’t on screen in this lineup')
    const at = seconds()
    if (action === 'rating') {
      pick.rating = body.verdict ? ({ verdict: body.verdict, reason: body.reason ?? null, at } as Pick['rating']) : null
      return ok(batch)
    }
    if (action === 'switch') return no(400, `${pick.name} is in your library: there's nothing to switch`)
    if (action === 'too') {
      const word = String(body.word ?? '').trim()
      if (!word) return no(400, 'say what it’s too much of')
      const slot = pick.slot ?? 0
      takeOff(batch, pick, { action: 'too', text: word, at })
      const into = batch.picks.find((p) => p.slot === slot)?.id ?? null
      const outcome: TooOutcome = { batch, swaps: [{ slot, out: pick.id, into }], ai_error: null }
      return ok(outcome)
    }
    // A card's own actions.
    const g = game(titleId)
    switch (body.action) {
      case 'chosen':
        pick.feedback = { action: 'chosen', text: null, at }
        if (g) g.chosen = { at, prompt: batch.prompt }
        return ok(batch)
      case 'not_tonight':
        takeOff(batch, pick, { action: 'not_tonight', text: null, at })
        return ok(batch)
      case 'not_for_me':
        if (g) setOverride(g, 'hidden')
        takeOff(batch, pick, { action: 'not_for_me', text: null, at })
        return ok(batch)
      case 'started': {
        const how = String(body.text ?? '') as OverrideStatus
        if (!['done', 'shelved', 'abandoned', 'still_playing', 'burnt_out'].includes(how)) return no(400, 'say how it went')
        if (g) setOverride(g, how)
        const feedback = { action: 'started' as const, text: how, at }
        if (how === 'shelved' || how === 'still_playing') pick.feedback = feedback
        else takeOff(batch, pick, feedback)
        return ok(batch)
      }
      default:
        return no(400, `unknown action ${JSON.stringify(body.action)}`)
    }
  }

  function pastLineups(q: URLSearchParams) {
    const limit = Math.min(Math.max(Number(q.get('limit') ?? 20), 1), 200)
    const offset = Number(q.get('offset') ?? 0)
    const find = words(q.get('q') ?? '')
    let list = history.map((id) => batches.get(id)!)
    if (q.get('order') === 'oldest') list = [...list].reverse()
    if (find.length) {
      list = list.filter((b) => {
        const hay = [b.prompt, ...[...b.picks, ...b.removed].map((p) => p.name)].join(' ').toLowerCase()
        return find.every((w) => hay.includes(w))
      })
    }
    return list.slice(offset, offset + limit).map((b) => ({ ...b, picks: b.picks.map(fresh) }))
  }

  /** Versus: the shorter of the two, said the way a model would. */
  function decide(body: Json): DemoAnswer {
    const ids = Array.isArray(body.ids) ? (body.ids as number[]) : []
    const [a, b] = ids.map(game)
    if (!a || !b || a.id === b.id) return no(400, 'name two different games')
    const short = (a.hltb_main ?? 99) <= (b.hltb_main ?? 99) ? a : b
    const other = short === a ? b : a
    const answer: DecideResult = {
      pick_id: short.id,
      reason: `${short.name} fits an evening better: it’s the shorter of the two, so you’ll see more of it before ${other.name} would get going.`,
      model: (data.lineups[0]?.model as string | null) ?? null,
      usage: null,
      cost_usd: null,
      api_usd: null,
    }
    return ok(answer)
  }

  // ---- One game ------------------------------------------------------------

  function gameCall(method: string, id: number, what: string, body: Json): DemoAnswer {
    const g = game(id)
    if (!g) return no(404, `no game ${id}`)
    const at = seconds()
    if (what === 'override' && method === 'PUT') {
      setOverride(g, (body.status as OverrideStatus | null) ?? null)
      return ok(g)
    }
    if (what === 'rating' && method === 'PUT') {
      g.my_rating = typeof body.rating === 'number' ? body.rating : null
      g.rated_at = g.my_rating == null ? null : at
      return ok(g)
    }
    if (what === 'ending' && method === 'PUT') {
      g.no_ending = typeof body.no_ending === 'boolean' ? body.no_ending : null
      return ok(g)
    }
    if (what === 'later') {
      g.saved_at = method === 'PUT' ? at : null
      return ok(g)
    }
    if (what === 'hide') {
      g.hidden_scope = method === 'PUT' ? ((body.scope as GameView['hidden_scope']) ?? 'me') : null
      return method === 'PUT' ? ok({ scope: g.hidden_scope, still_seen_by: [], through_shared: [] }) : ok()
    }
    return no(403, NOT_IN_DEMO)
  }

  // ---- Everything else -------------------------------------------------------

  function read(path: string, q: URLSearchParams, full: string): DemoAnswer {
    if (path === '/api/v1/games') return ok(listGames(q))
    const one = path.match(/^\/api\/v1\/games\/(\d+)$/)
    if (one) return game(Number(one[1])) ? ok(game(Number(one[1]))) : no(404, 'no such game')
    if (path === '/api/v1/play-later') {
      const later = order.map(game).filter((g): g is GameView => g?.saved_at != null)
      return ok(later.sort((a, b) => (b.saved_at ?? 0) - (a.saved_at ?? 0)))
    }
    if (path === '/api/v1/hidden') {
      const hidden: HiddenTitle[] = order
        .map(game)
        .filter((g): g is GameView => !!g?.hidden_scope)
        .map((g) => ({ title_id: g.id, name: g.name, scope: g.hidden_scope!, at: seconds(), own_copy: g.own_copy }))
      return ok(hidden)
    }
    if (path === '/api/v1/picks/history') return ok(pastLineups(q))
    if (path === '/api/v1/routines') return ok({ ...routinesAnswer, routines })
    if (path === '/api/v1/settings') return ok(settings)
    // The Activity panel's list, whatever it asks for.
    if (path === '/api/v1/jobs') return ok(data.answers['/api/v1/jobs?limit=20'])
    for (const key of [full, path]) {
      if (key in data.answers) return ok(data.answers[key])
      if (key in data.refused) return no(data.refused[key].status, data.refused[key].error)
    }
    return no(404, NOT_IN_DEMO)
  }

  function write(method: string, path: string, body: Json): DemoAnswer {
    if (method === 'POST' && path === '/api/v1/picks') return makeLineup(body)
    const lineup = path.match(/^\/api\/v1\/picks\/([^/]+)\/(feedback|too|more|switch|rating)$/)
    if (lineup) return lineupCall(decodeURIComponent(lineup[1]), lineup[2], body)
    const one = path.match(/^\/api\/v1\/games\/(\d+)\/([a-z]+)$/)
    if (one) return gameCall(method, Number(one[1]), one[2], body)
    if (method === 'POST' && path === '/api/v1/versus/decide') return decide(body)
    if (path === '/api/v1/routines' && method === 'POST') {
      const id = Math.max(0, ...routines.map((r) => r.id)) + 1
      const routine = { days: [], times: [], who: null, place: null, input: null, minutes: null, note: null, ...body, id } as unknown as Routine
      routines = [...routines, routine]
      return ok(routine)
    }
    const routine = path.match(/^\/api\/v1\/routines\/(\d+)$/)
    if (routine) {
      const id = Number(routine[1])
      if (!routines.some((r) => r.id === id)) return no(404, `no routine ${id}`)
      if (method === 'DELETE') {
        routines = routines.filter((r) => r.id !== id)
        return ok()
      }
      const changed = { days: [], times: [], who: null, place: null, input: null, minutes: null, note: null, ...body, id } as unknown as Routine
      routines = routines.map((r) => (r.id === id ? changed : r))
      return ok(changed)
    }
    // The look of the app: theme and the like. Settings that reach out (keys, sign-ins) stay as they are.
    if (method === 'PUT' && path === '/api/v1/settings') {
      const looks = Object.fromEntries(Object.entries(body).filter(([k]) => k.startsWith('theme') || k === 'linux_mode'))
      if (Object.keys(looks).length !== Object.keys(body).length) return no(403, NOT_IN_DEMO)
      settings = { ...settings, ...looks }
      return ok(settings)
    }
    if (method === 'PUT' && path === '/api/v1/players/current') {
      const players = (data.answers['/api/v1/players'] as { players: unknown[] }).players
      return ok(players[0])
    }
    // The tour, taken or skipped.
    if (method === 'PUT' && path === '/api/v1/setup' && Object.keys(body).every((k) => k === 'tour')) {
      const state = data.answers['/api/v1/setup/state'] as Json
      Object.assign(state, { tour: body.tour, tour_offer: false })
      return ok(state)
    }
    if (method === 'POST' && path === '/api/v1/taste/diary/launch') return ok({ recorded: false })
    if (path === '/api/v1/hidden/unlock') return ok()
    return no(403, NOT_IN_DEMO)
  }

  return {
    /** The answer to one API call. `url` is the path with its query. */
    answer(method: string, url: string, body?: unknown): DemoAnswer {
      const u = new URL(url, 'http://demo.invalid')
      const path = u.pathname.replace(/^\/api\/(?!v1\/)/, '/api/v1/')
      const full = path + u.search
      const answer = method === 'GET' ? read(path, u.searchParams, full) : write(method, path, (body && typeof body === 'object' ? body : {}) as Json)
      // A copy, as from a server: the page never holds the demo's own objects.
      return answer.body === undefined ? answer : { ...answer, body: clone(answer.body) }
    },
  }
}

export type Demo = ReturnType<typeof createDemo>
