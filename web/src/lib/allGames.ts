import { api } from '../api'
import type { GamesQuery, GamesResponse } from '../types'

/** The most games one request answers with (the server's cap). */
const PAGE = 1000

/** Every game a list's query matches, a page at a time: a list shows them
 *  all, sorted here, so it can't stop at the first page. */
export async function allGames(query: GamesQuery): Promise<GamesResponse> {
  const first = await api.games({ ...query, limit: PAGE, offset: 0 })
  const items = [...first.items]
  while (items.length < first.total) {
    const next = await api.games({ ...query, limit: PAGE, offset: items.length })
    if (next.items.length === 0) break
    items.push(...next.items)
  }
  return { ...first, items }
}
