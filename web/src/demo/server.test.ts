import { describe, expect, it } from 'vitest'
import type { GameView, PickBatch, TooOutcome } from '../types'
import raw from './recorded.json?raw'
import { createDemo, moveTimes, NOT_IN_DEMO, type Recorded } from './server'

const recorded = JSON.parse(raw) as Recorded
const DAY = 86_400

/** A demo a year after the recording, with dice that always roll low. */
function demo() {
  return createDemo(recorded, () => (recorded.recorded_at + 365 * DAY) * 1000, () => 0)
}

const body = <T>(a: { body?: unknown }) => a.body as T

describe('the online demo', () => {
  it('moves every time forward to today, and leaves other numbers alone', () => {
    expect(moveTimes({ at: 1_790_000_000, id: 44, appid: 1_135_690, hours: 3.5 }, 10)).toEqual({
      at: 1_790_000_010,
      id: 44,
      appid: 1_135_690,
      hours: 3.5,
    })
    const played = recorded.games.find((g) => g.last_played > 0)!
    const now = body<GameView>(demo().answer('GET', `/api/v1/games/${played.id}`))
    expect(now.last_played - played.last_played).toBe(365 * DAY)
  })

  it('searches, filters and sorts the library as the server does', () => {
    const d = demo()
    const found = body<{ total: number; items: GameView[] }>(d.answer('GET', '/api/v1/games?q=hades'))
    expect(found.items.map((g) => g.name)).toEqual(['Hades'])
    const abandoned = body<{ items: GameView[] }>(d.answer('GET', '/api/v1/games?status=abandoned&limit=500'))
    expect(abandoned.items.length).toBeGreaterThan(0)
    expect(abandoned.items.every((g) => g.status === 'abandoned')).toBe(true)
    const byTime = body<{ items: GameView[] }>(d.answer('GET', '/api/v1/games?sort=playtime&limit=3'))
    const times = byTime.items.map((g) => g.playtime_minutes)
    expect(times).toEqual([...times].sort((a, b) => b - a))
  })

  it('answers a suggestion with its recorded lineup, under the request as typed', () => {
    const d = demo()
    const batch = body<PickBatch>(d.answer('POST', '/api/v1/picks', { prompt: 'A quiet hour', count: 3 }))
    const quiet = recorded.lineups.find((l) => l.prompt === 'A quiet hour')!
    expect(batch.picks.map((p) => p.name)).toEqual(quiet.picks.map((p) => p.name))
    const typed = body<PickBatch>(d.answer('POST', '/api/v1/picks', { prompt: 'some co-op with friends tonight', count: 3 }))
    expect(typed.prompt).toBe('some co-op with friends tonight')
    expect(typed.picks.map((p) => p.name)).toEqual(recorded.lineups.find((l) => l.prompt === 'Co-op night')!.picks.map((p) => p.name))
  })

  it('leaves out turned-down games, and a spare takes the slot of a game taken off', () => {
    const d = demo()
    const quiet = recorded.lineups.find((l) => l.prompt === 'A quiet hour')!
    const first = quiet.picks[0]
    const batch = body<PickBatch>(d.answer('POST', '/api/v1/picks', { prompt: 'A quiet hour', count: 3, exclude_ids: [first.id] }))
    expect(batch.picks.map((p) => p.id)).not.toContain(first.id)

    const shown = batch.picks[0]
    const after = body<PickBatch>(d.answer('POST', `/api/v1/picks/${batch.batch_id}/feedback`, { title_id: shown.id, action: 'not_tonight' }))
    expect(after.picks).toHaveLength(3)
    expect(after.picks.find((p) => p.slot === 0)?.id).not.toBe(shown.id)
    expect(after.removed.map((p) => p.id)).toEqual([shown.id])

    const too = body<TooOutcome>(d.answer('POST', `/api/v1/picks/${batch.batch_id}/too`, { title_id: after.picks[1].id, word: 'long' }))
    expect(too.swaps[0].out).toBe(after.picks[1].id)

    const history = body<PickBatch[]>(d.answer('GET', '/api/v1/picks/history?limit=20'))
    expect(history.map((b) => b.batch_id)).toEqual([batch.batch_id])
  })

  it('rolls the dice from games not started yet', () => {
    const batch = body<PickBatch>(demo().answer('POST', '/api/v1/picks', { prompt: '', count: 3, mode: 'random' }))
    expect(batch.mode).toBe('random')
    expect(batch.picks).toHaveLength(3)
    for (const p of batch.picks) expect(['untouched', 'sampled']).toContain(p.owned !== false && p.game.status)
  })

  it('keeps a status, a rating and Play later in the tab', () => {
    const d = demo()
    const g = recorded.games.find((g) => g.status === 'untouched')!
    expect(body<GameView>(d.answer('PUT', `/api/v1/games/${g.id}/override`, { status: 'abandoned' })).status).toBe('abandoned')
    expect(body<GameView>(d.answer('PUT', `/api/v1/games/${g.id}/override`, { status: null })).status).toBe('untouched')
    expect(body<GameView>(d.answer('PUT', `/api/v1/games/${g.id}/rating`, { rating: 5 })).my_rating).toBe(5)
    d.answer('PUT', `/api/v1/games/${g.id}/later`)
    expect(body<GameView[]>(d.answer('GET', '/api/v1/play-later')).map((x) => x.id)).toEqual([g.id])
    // A new demo starts from the recording again.
    expect(body<GameView>(demo().answer('GET', `/api/v1/games/${g.id}`)).my_rating ?? null).toBeNull()
  })

  it('turns down what needs a real server, and changes no setting that reaches out', () => {
    const d = demo()
    for (const [method, path] of [
      ['POST', '/api/v1/jobs'],
      ['POST', '/api/v1/gog/login'],
      ['POST', '/api/v1/verdicts/1/send'],
      ['POST', '/api/v1/players'],
      ['POST', '/api/v1/taste/statements'],
    ]) {
      expect(d.answer(method, path, {})).toEqual({ status: 403, body: { error: NOT_IN_DEMO } })
    }
    expect(d.answer('PUT', '/api/v1/settings', { anthropic_api_key: 'x' }).status).toBe(403)
    const looks = body<{ theme: string }>(d.answer('PUT', '/api/v1/settings', { theme: 'daylight' }))
    expect(looks.theme).toBe('daylight')
  })

  it('answers every recorded read, and says no where the server did', () => {
    const d = demo()
    for (const path of Object.keys(recorded.answers)) expect(d.answer('GET', path).status, path).toBe(200)
    for (const [path, no] of Object.entries(recorded.refused)) expect(d.answer('GET', path).status, path).toBe(no.status)
  })
})
