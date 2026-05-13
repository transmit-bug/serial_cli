import { create } from 'zustand'
import { subscribeWithSelector } from 'zustand/middleware'
import { invoke } from '@tauri-apps/api/core'
import { toast } from 'sonner'
import { useConnectionStore } from './connectionStore'
import { useDataStore } from './dataStore'

export type QuickCommandMode = 'hex' | 'ascii'

export interface QuickCommand {
  id: string
  name: string
  data: string
  mode: QuickCommandMode
  /** Optional protocol to use for encoding */
  protocol?: string
  /** Whether to append \r\n automatically */
  appendNewline?: boolean
}

interface QuickCommandState {
  commands: QuickCommand[]
  loading: boolean

  // Actions
  addCommand: (cmd: Omit<QuickCommand, 'id'>) => void
  updateCommand: (id: string, updates: Partial<QuickCommand>) => void
  deleteCommand: (id: string) => void
  reorderCommands: (fromIndex: number, toIndex: number) => void

  // Execute a quick command (send data to port)
  executeCommand: (cmd: QuickCommand) => Promise<void>

  // Persistence
  loadCommands: () => void
  saveCommands: () => void
}

const STORAGE_KEY = 'serial-cli-quick-commands'

const DEFAULT_COMMANDS: QuickCommand[] = [
  { id: 'default-at', name: 'AT', data: 'AT', mode: 'ascii', appendNewline: true },
  { id: 'default-ati', name: 'ATI', data: 'ATI', mode: 'ascii', appendNewline: true },
  { id: 'default-reset', name: 'AT+RST', data: 'AT+RST', mode: 'ascii', appendNewline: true },
]

export const useQuickCommandStore = create<QuickCommandState>()(
  subscribeWithSelector((set, get) => ({
    // Initial state
    commands: [],
    loading: false,

    // Add a new command
    addCommand: (cmd) => {
      const newCmd: QuickCommand = {
        ...cmd,
        id: `${Date.now()}-${Math.random().toString(36).slice(2, 8)}`,
      }
      set((state) => ({
        commands: [...state.commands, newCmd],
      }))
      get().saveCommands()
    },

    // Update an existing command
    updateCommand: (id, updates) => {
      set((state) => ({
        commands: state.commands.map((cmd) =>
          cmd.id === id ? { ...cmd, ...updates } : cmd
        ),
      }))
      get().saveCommands()
    },

    // Delete a command
    deleteCommand: (id) => {
      set((state) => ({
        commands: state.commands.filter((cmd) => cmd.id !== id),
      }))
      get().saveCommands()
    },

    // Reorder commands (drag & drop)
    reorderCommands: (fromIndex, toIndex) => {
      set((state) => {
        const commands = [...state.commands]
        const [removed] = commands.splice(fromIndex, 1)
        commands.splice(toIndex, 0, removed)
        return { commands }
      })
      get().saveCommands()
    },

    // Execute a quick command: send data to the connected port
    executeCommand: async (cmd) => {
      const { portId } = useConnectionStore.getState()
      const { addTxPacket } = useDataStore.getState()

      if (!portId) {
        toast.error('未连接到串口')
        return
      }

      let data: number[] = []

      // Parse input data based on mode
      if (cmd.mode === 'hex') {
        const hex = cmd.data.replace(/\s/g, '')
        if (!/^[0-9A-Fa-f]*$/.test(hex)) {
          toast.error(`快捷指令 "${cmd.name}" 包含无效的十六进制数据`)
          return
        }
        for (let i = 0; i < hex.length; i += 2) {
          data.push(parseInt(hex.substring(i, 2), 16))
        }
      } else {
        let text = cmd.data
        if (cmd.appendNewline) {
          text += '\r\n'
        }
        data = Array.from(new TextEncoder().encode(text))
      }

      // Apply protocol encoding if specified
      if (cmd.protocol) {
        try {
          const encodedData = await invoke<number[]>('protocol_encode', {
            protocolName: cmd.protocol,
            data,
          })
          data = encodedData
        } catch (encodeError) {
          toast.error(`快捷指令 "${cmd.name}" 协议编码失败`)
          return
        }
      }

      // Send data
      try {
        const bytesWritten = await invoke<number>('send_data', {
          portId,
          data,
        })

        addTxPacket({
          portId,
          direction: 'tx',
          data,
          timestamp: Date.now(),
        })

        toast.success(`已发送: ${cmd.name} (${bytesWritten} 字节)`)
      } catch (error) {
        toast.error(`快捷指令 "${cmd.name}" 发送失败: ${error instanceof Error ? error.message : '未知错误'}`)
      }
    },

    // Load commands from localStorage
    loadCommands: () => {
      try {
        const stored = localStorage.getItem(STORAGE_KEY)
        if (stored) {
          const parsed = JSON.parse(stored) as QuickCommand[]
          if (Array.isArray(parsed) && parsed.length > 0) {
            set({ commands: parsed })
            return
          }
        }
      } catch {
        // Ignore parse errors
      }
      // Use defaults if nothing stored
      set({ commands: DEFAULT_COMMANDS })
      get().saveCommands()
    },

    // Save commands to localStorage
    saveCommands: () => {
      try {
        const { commands } = get()
        localStorage.setItem(STORAGE_KEY, JSON.stringify(commands))
      } catch {
        // Ignore storage errors
      }
    },
  }))
)

// Auto-load on store creation
if (typeof window !== 'undefined') {
  useQuickCommandStore.getState().loadCommands()
}
