import { create } from 'zustand'
import { subscribeWithSelector } from 'zustand/middleware'

// TODO: Import types from existing code
export interface SerialConfig {
  baudrate: number
  databits: number
  stopbits: number
  parity: string
  timeout_ms: number
  flow_control: string
  dtr_enable: boolean
  rts_enable: boolean
}

export type ConnectionStatus = 'disconnected' | 'connecting' | 'connected' | 'error'

interface ConnectionState {
  // State
  status: ConnectionStatus
  port: string | null
  config: SerialConfig | null
  error: string | null

  // Actions
  connect: (port: string, config: SerialConfig) => Promise<void>
  disconnect: () => Promise<void>
  setStatus: (status: ConnectionStatus) => void
  setError: (error: string | null) => void
}

export const useConnectionStore = create<ConnectionState>()(
  subscribeWithSelector((set, get) => ({
    // Initial state
    status: 'disconnected',
    port: null,
    config: null,
    error: null,

    // Actions (TODO: Implement in Phase 2)
    connect: async (port, config) => {
      set({ status: 'connecting' })
      try {
        // TODO: Invoke Tauri command
        // await invoke('open_port', { portName: port, config })
        set({ status: 'connected', port, config, error: null })
      } catch (error) {
        set({ status: 'error', error: error instanceof Error ? error.message : String(error) })
      }
    },

    disconnect: async () => {
      const { port } = get()
      if (port) {
        try {
          // TODO: Invoke Tauri command
          // await invoke('close_port', { portId: port })
          set({ status: 'disconnected', port: null, config: null, error: null })
        } catch (error) {
          set({ error: error instanceof Error ? error.message : String(error) })
        }
      }
    },

    setStatus: (status) => set({ status }),
    setError: (error) => set({ error }),
  }))
)
