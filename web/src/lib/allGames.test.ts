import { afterEach, describe, expect, it, vi } from 'vitest'
import { api } from '../api'
import type { GameView } from '../types'
import { allGames } from './allGames'

describe('A list', () => {
  afterEach(() => vi.restoreAllMocks())

  it('reads every page of its games, past the first thousand', async () => {
    const game = (id: number) => ({ id, name: `Game ${id}` }) as unknown as GameView
    const all = Array.from({ length: 2345 }, (_, i) => game(i + 1))
    const games = vi.spyOn(api, 'games').mockImplementation(async (q = {}) => {
      const from = q.offset ?? 0
      return { total: all.length, items: all.slice(from, from + (q.limit ?? 50)) }
    })
    const got = await allGames({ status: ['shelved'] })
    expect(got.items.map((g) => g.id)).toEqual(all.map((g) => g.id))
    expect(games.mock.calls.map(([q]) => q?.offset)).toEqual([0, 1000, 2000])
    expect(games.mock.calls.every(([q]) => q?.status?.[0] === 'shelved')).toBe(true)
  })
})
