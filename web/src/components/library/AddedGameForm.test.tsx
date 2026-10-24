import { afterEach, describe, expect, it, vi } from 'vitest'
import { fireEvent, render, screen, waitFor } from '@testing-library/react'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { api, ApiError } from '../../api'
import type { AddedSaved, OwnGame } from '../../types'
import { ToastProvider } from '../Toast'
import { AddedGameForm } from './AddedGameForm'
import { bodyOf, fieldsOf } from './addedGame'

const demons: OwnGame = {
  copy_id: 41,
  title_id: 9,
  name: "Demon's Souls",
  from: 'RPCS3',
  cover_url: null,
  play_url: null,
  minutes: 125,
  last_played: 1_792_497_600,
  installed: true,
  look_up: true,
  steam_matched: false,
}

function mount(editing?: OwnGame) {
  vi.spyOn(api, 'addedGames').mockResolvedValue({ games: [{ ...demons, from: 'shadPS4' }] })
  const saved: AddedSaved = { game: demons, looking_up: true }
  const add = vi.spyOn(api, 'addGame').mockResolvedValue(saved)
  const edit = vi.spyOn(api, 'editAddedGame').mockResolvedValue(saved)
  const onDone = vi.fn()
  const client = new QueryClient({ defaultOptions: { queries: { retry: false }, mutations: { retry: false } } })
  render(
    <QueryClientProvider client={client}>
      <ToastProvider>
        <AddedGameForm editing={editing} onDone={onDone} onCancel={vi.fn()} />
      </ToastProvider>
    </QueryClientProvider>,
  )
  return { add, edit, onDone }
}

afterEach(() => vi.restoreAllMocks())

describe('Adding a game from no store', () => {
  it('sends what was typed, with empty fields left out', async () => {
    const { add, onDone } = mount()
    fireEvent.change(screen.getByLabelText('Name'), { target: { value: "Demon's Souls" } })
    fireEvent.change(screen.getByLabelText("Where it's from"), { target: { value: ' RPCS3 ' } })
    fireEvent.change(screen.getByLabelText('Hours played'), { target: { value: '2.5' } })
    fireEvent.click(screen.getByRole('button', { name: 'Add it' }))
    await waitFor(() => expect(onDone).toHaveBeenCalled())
    expect(add).toHaveBeenCalledWith({
      name: "Demon's Souls",
      from: 'RPCS3',
      cover_url: null,
      play_url: null,
      hours: 2.5,
      last_played: null,
      installed: false,
      look_up: true,
    })
  })

  it('offers the places already used first', async () => {
    mount()
    await waitFor(() => expect(document.querySelector('datalist option')?.getAttribute('value')).toBe('shadPS4'))
  })

  it('shows why the server turned it down', async () => {
    const { add } = mount()
    add.mockRejectedValue(new ApiError(400, 'the Play link has to be a launcher’s link'))
    fireEvent.change(screen.getByLabelText('Name'), { target: { value: 'Okami' } })
    fireEvent.click(screen.getByRole('button', { name: 'Add it' }))
    expect((await screen.findByRole('alert')).textContent).toContain('Play link')
  })

  it('changes a game added before, keeping its hours to the minute', async () => {
    const { edit, onDone } = mount(demons)
    expect((screen.getByLabelText('Name') as HTMLInputElement).value).toBe("Demon's Souls")
    expect((screen.getByLabelText('Last played') as HTMLInputElement).value).toBe('2026-10-20')
    fireEvent.click(screen.getByRole('button', { name: 'Save' }))
    await waitFor(() => expect(onDone).toHaveBeenCalled())
    const body = edit.mock.calls[0][1]
    expect(edit.mock.calls[0][0]).toBe(41)
    expect(Math.round((body.hours ?? 0) * 60)).toBe(125)
    expect(body).toMatchObject({ from: 'RPCS3', last_played: '2026-10-20', installed: true })
  })
})

describe('The form’s fields', () => {
  it('round-trip a saved game', () => {
    const body = bodyOf(fieldsOf(demons))
    expect(body).toMatchObject({ name: "Demon's Souls", from: 'RPCS3', cover_url: null, installed: true, look_up: true })
  })

  it('leave out hours that are not a number', () => {
    expect(bodyOf({ ...fieldsOf(), name: 'Game', hours: 'lots' }).hours).toBeNull()
  })
})
