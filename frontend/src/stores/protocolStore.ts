import { create } from 'zustand'
import { subscribeWithSelector } from 'zustand/middleware'

// TODO: Import types from existing code
export interface ProtocolInfo {
  name: string
  version: string
  description: string
  author: string
  enabled: boolean
}

interface ProtocolState {
  protocols: ProtocolInfo[]
  activeProtocol: string | null
  loading: boolean
  error: string | null

  // Actions
  loadProtocols: () => Promise<void>
  setActiveProtocol: (name: string) => void
  enableProtocol: (name: string) => Promise<void>
  disableProtocol: (name: string) => Promise<void>
  reloadProtocol: (name: string) => Promise<void>
}

export const useProtocolStore = create<ProtocolState>()(
  subscribeWithSelector((set, get) => ({
    // Initial state
    protocols: [],
    activeProtocol: null,
    loading: false,
    error: null,

    // Actions (TODO: Implement in Phase 2)
    loadProtocols: async () => {
      set({ loading: true })
      try {
        // TODO: Invoke Tauri command
        // const protocols = await invoke('list_protocols')
        set({ loading: false })
      } catch (error) {
        set({
          loading: false,
          error: error instanceof Error ? error.message : String(error),
        })
      }
    },

    setActiveProtocol: (name) => set({ activeProtocol: name }),

    enableProtocol: async (name) => {
      try {
        // TODO: Invoke Tauri command
        // await invoke('load_protocol', { name })
      } catch (error) {
        set({ error: error instanceof Error ? error.message : String(error) })
      }
    },

    disableProtocol: async (name) => {
      try {
        // TODO: Invoke Tauri command
        // await invoke('unload_protocol', { name })
      } catch (error) {
        set({ error: error instanceof Error ? error.message : String(error) })
      }
    },

    reloadProtocol: async (name) => {
      try {
        // TODO: Invoke Tauri command
        // await invoke('reload_protocol', { name })
      } catch (error) {
        set({ error: error instanceof Error ? error.message : String(error) })
      }
    },
  }))
)
