import { useState, type FormEvent } from 'react'
import { useMutation, useQuery } from '@tanstack/react-query'
import { api, ApiError } from '../../api'
import type { AddedBody, AddedSaved, OwnGame } from '../../types'
import { useToast } from '../useToast'
import { bodyOf, fieldsOf, useAddedChanged, type Fields } from './addedGame'

/** Places a game from no store often comes from, offered as the player types.
 *  An emulator's or console's name also finds the box art on libretro. */
const PLACES = ['RPCS3', 'shadPS4', 'PCSX2', 'DuckStation', 'PPSSPP', 'Dolphin', 'Cemu', 'RetroArch', 'Meta Quest', 'SteamVR']

/** Add a game from no store, or change one added before (`editing`). */
export function AddedGameForm({
  editing,
  onDone,
  onCancel,
}: {
  editing?: OwnGame
  onDone: (saved: AddedSaved) => void
  onCancel: () => void
}) {
  const [f, setF] = useState<Fields>(() => fieldsOf(editing))
  const [today] = useState(() => new Date().toISOString().slice(0, 10))
  const set = <K extends keyof Fields>(k: K, v: Fields[K]) => setF((x) => ({ ...x, [k]: v }))
  const changed = useAddedChanged()
  const toast = useToast()
  // The player's own places come first among the suggestions.
  const { data: added } = useQuery({
    queryKey: ['added-games'],
    queryFn: api.addedGames,
    staleTime: 60_000,
  })
  const places = [...new Set([...(added?.games ?? []).map((g) => g.from).filter((s): s is string => !!s), ...PLACES])]
  const save = useMutation({
    mutationFn: (body: AddedBody) => (editing ? api.editAddedGame(editing.copy_id, body) : api.addGame(body)),
    onSuccess: (saved) => {
      changed(saved.game.title_id)
      if (editing?.title_id != null && editing.title_id !== saved.game.title_id) changed(editing.title_id)
      toast.success(editing ? `Saved ${saved.game.name}` : `Added ${saved.game.name}`, saved.looking_up ? 'Looking it up now.' : undefined)
      onDone(saved)
    },
  })
  const submit = (ev: FormEvent) => {
    ev.preventDefault()
    if (f.name.trim()) save.mutate(bodyOf(f))
  }
  const id = editing ? `added-${editing.copy_id}` : 'added-new'
  const label = 'flex flex-col gap-1 text-[14px] font-semibold'
  const hint = 'text-[12.5px] font-normal text-dim'

  return (
    <form onSubmit={submit} className="flex max-w-[640px] flex-col gap-4" aria-label={editing ? `Change ${editing.name}` : 'Add a game'}>
      {!editing && (
        <p className="m-0 text-[14.5px] leading-[1.6] text-muted">
          For a game from no store UpNext reads: one in an emulator, one played with a mod, a VR game. It's looked up by name like the rest.
        </p>
      )}
      <div className="grid gap-4 sm:grid-cols-2">
        <label className={label}>
          Name
          <input
            id={`${id}-name`}
            autoFocus={!editing}
            required
            maxLength={200}
            className="field h-11"
            value={f.name}
            onChange={(e) => set('name', e.target.value)}
            autoComplete="off"
          />
        </label>
        <label className={label}>
          Where it's from
          <input
            list={`${id}-places`}
            maxLength={60}
            className="field h-11"
            placeholder="RPCS3, Meta Quest…"
            value={f.from}
            onChange={(e) => set('from', e.target.value)}
            autoComplete="off"
          />
          <datalist id={`${id}-places`}>
            {places.map((p) => (
              <option key={p} value={p} />
            ))}
          </datalist>
        </label>
        <label className={label}>
          Hours played
          <input
            type="number"
            min={0}
            max={100000}
            step="any"
            inputMode="decimal"
            className="field h-11"
            value={f.hours}
            onChange={(e) => set('hours', e.target.value)}
          />
        </label>
        <label className={label}>
          Last played
          <input
            type="date"
            max={today}
            className="field h-11"
            value={f.last_played}
            onChange={(e) => set('last_played', e.target.value)}
          />
        </label>
        <label className={label}>
          <span>
            Cover link <span className={hint}>(optional, https)</span>
          </span>
          <input
            type="url"
            className="field h-11"
            placeholder="https://"
            value={f.cover_url}
            onChange={(e) => set('cover_url', e.target.value)}
          />
        </label>
        <label className={label}>
          <span>
            Play link <span className={hint}>(optional, a launcher's)</span>
          </span>
          <input
            className="field h-11"
            placeholder="steam://rungameid/…"
            value={f.play_url}
            onChange={(e) => set('play_url', e.target.value)}
            autoComplete="off"
          />
        </label>
      </div>
      <div className="flex flex-col gap-2 text-[14.5px]">
        <label className="flex min-h-11 cursor-pointer items-center gap-3">
          <input
            type="checkbox"
            checked={f.installed}
            onChange={(e) => set('installed', e.target.checked)}
            className="m-0 h-[18px] w-[18px] accent-accent"
          />
          Installed
        </label>
        <label className="flex min-h-11 cursor-pointer items-center gap-3">
          <input
            type="checkbox"
            checked={f.look_up}
            onChange={(e) => set('look_up', e.target.checked)}
            className="m-0 h-[18px] w-[18px] accent-accent"
          />
          <span>
            Look it up by name <span className={hint}>for details, genres and box art</span>
          </span>
        </label>
      </div>
      {save.error && (
        <p className="m-0 text-[14px] text-bad" role="alert">
          {save.error instanceof ApiError ? save.error.message : String(save.error)}
        </p>
      )}
      <div className="flex flex-wrap items-center gap-3">
        <button type="submit" className="btn-outline h-11" disabled={!f.name.trim() || save.isPending}>
          {save.isPending ? 'Saving…' : editing ? 'Save' : 'Add it'}
        </button>
        <button type="button" className="btn-quiet min-h-11 px-2" disabled={save.isPending} onClick={onCancel}>
          Never mind
        </button>
      </div>
    </form>
  )
}
