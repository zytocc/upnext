import { useQueryClient } from '@tanstack/react-query'
import type { AddedBody, OwnGame } from '../../types'
import { invalidateLibrary } from './sheetActions'

/** The form's fields, as typed: strings for the inputs, kept apart from the body sent. */
export type Fields = {
  name: string
  from: string
  cover_url: string
  play_url: string
  hours: string
  last_played: string
  installed: boolean
  look_up: boolean
}

/** A Unix time as the date input's `YYYY-MM-DD`, in UTC as the server keeps it. */
function dateOf(unix: number): string {
  return unix > 0 ? new Date(unix * 1000).toISOString().slice(0, 10) : ''
}

export function fieldsOf(g?: OwnGame): Fields {
  return {
    name: g?.name ?? '',
    from: g?.from ?? '',
    cover_url: g?.cover_url ?? '',
    play_url: g?.play_url ?? '',
    hours: g?.minutes != null ? String(+(g.minutes / 60).toFixed(2)) : '',
    last_played: dateOf(g?.last_played ?? 0),
    installed: g?.installed ?? false,
    look_up: g?.look_up ?? true,
  }
}

export function bodyOf(f: Fields): AddedBody {
  const text = (s: string) => s.trim() || null
  const hours = f.hours.trim() === '' ? null : Number(f.hours)
  return {
    name: f.name,
    from: text(f.from),
    cover_url: text(f.cover_url),
    play_url: text(f.play_url),
    hours: hours != null && Number.isFinite(hours) ? hours : null,
    last_played: text(f.last_played),
    installed: f.installed,
    look_up: f.look_up,
  }
}

/** Everything that shows an added game, after one is added, changed or removed. */
export function useAddedChanged() {
  const qc = useQueryClient()
  return (titleId?: number | null) => {
    invalidateLibrary(qc)
    qc.invalidateQueries({ queryKey: ['added-games'] })
    if (titleId != null) qc.invalidateQueries({ queryKey: ['game', titleId] })
  }
}
