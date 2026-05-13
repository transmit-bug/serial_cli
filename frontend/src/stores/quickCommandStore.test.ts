import { describe, it, expect, beforeEach } from 'vitest'
import { useQuickCommandStore, type QuickCommand } from './quickCommandStore'

describe('useQuickCommandStore', () => {
  beforeEach(() => {
    // Reset store before each test
    useQuickCommandStore.setState({ commands: [], loading: false })
  })

  it('starts with empty commands after reset', () => {
    const { commands } = useQuickCommandStore.getState()
    expect(commands).toEqual([])
  })

  it('adds a command', () => {
    const { addCommand, commands } = useQuickCommandStore.getState()
    addCommand({
      name: 'AT',
      data: 'AT',
      mode: 'ascii',
      appendNewline: true,
    })
    const state = useQuickCommandStore.getState()
    expect(state.commands).toHaveLength(1)
    expect(state.commands[0].name).toBe('AT')
    expect(state.commands[0].id).toBeDefined()
  })

  it('updates a command', () => {
    const { addCommand, updateCommand } = useQuickCommandStore.getState()
    addCommand({ name: 'AT', data: 'AT', mode: 'ascii' })
    const id = useQuickCommandStore.getState().commands[0].id

    updateCommand(id, { data: 'ATI' })
    const state = useQuickCommandStore.getState()
    expect(state.commands[0].data).toBe('ATI')
    expect(state.commands[0].name).toBe('AT') // unchanged
  })

  it('deletes a command', () => {
    const { addCommand, deleteCommand } = useQuickCommandStore.getState()
    addCommand({ name: 'AT', data: 'AT', mode: 'ascii' })
    addCommand({ name: 'ATI', data: 'ATI', mode: 'ascii' })
    const id = useQuickCommandStore.getState().commands[0].id

    deleteCommand(id)
    const state = useQuickCommandStore.getState()
    expect(state.commands).toHaveLength(1)
    expect(state.commands[0].name).toBe('ATI')
  })

  it('reorders commands', () => {
    const { addCommand, reorderCommands } = useQuickCommandStore.getState()
    addCommand({ name: 'first', data: '1', mode: 'ascii' })
    addCommand({ name: 'second', data: '2', mode: 'ascii' })
    addCommand({ name: 'third', data: '3', mode: 'ascii' })

    reorderCommands(2, 0) // move third to first
    const state = useQuickCommandStore.getState()
    expect(state.commands[0].name).toBe('third')
    expect(state.commands[1].name).toBe('first')
    expect(state.commands[2].name).toBe('second')
  })

  it('generates unique IDs', () => {
    const { addCommand } = useQuickCommandStore.getState()
    addCommand({ name: 'cmd1', data: '1', mode: 'ascii' })
    addCommand({ name: 'cmd2', data: '2', mode: 'ascii' })

    const state = useQuickCommandStore.getState()
    expect(state.commands[0].id).not.toBe(state.commands[1].id)
  })

  it('supports hex mode commands', () => {
    const { addCommand } = useQuickCommandStore.getState()
    addCommand({ name: 'hex-cmd', data: 'DEADBEEF', mode: 'hex' })

    const state = useQuickCommandStore.getState()
    expect(state.commands[0].mode).toBe('hex')
    expect(state.commands[0].data).toBe('DEADBEEF')
  })
})
