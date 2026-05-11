import { create } from 'zustand'
import { subscribeWithSelector } from 'zustand/middleware'

// TODO: Import types from existing code
export interface VirtualPortInfo {
  id: string
  portA: string
  portB: string
  backendType: string
  running: boolean
  createdAt: number
}

interface VirtualPortState {
  ports: VirtualPortInfo[]
  selectedPort: string | null
  loading: boolean
  error: string | null

  // Actions
  createPort: (config: any) => Promise<void>
  stopPort: (id: string) => Promise<void>
  selectPort: (id: string | null) => void
  refreshPorts: () => Promise<void>
}

export const useVirtualPortStore = create<VirtualPortState>()(
  subscribeWithSelector((set, get) => ({
    // Initial state
    ports: [],
    selectedPort: null,
    loading: false,
    error: null,

    // Actions (TODO: Implement in Phase 2)
    createPort: async (config) => {
      set({ loading: true, error: null })
      try {
        // TODO: Invoke Tauri command
        // const id = await invoke('create_virtual_port', { config })
        set({ loading: false })
      } catch (error) {
        set({
          loading: false,
          error: error instanceof Error ? error.message : String(error),
        })
      }
    },

    stopPort: async (id) => {
      try {
        // TODO: Invoke Tauri command
        // await invoke('stop_virtual_port', { id })
      } catch (error) {
        set({ error: error instanceof Error ? error.message : String(error) })
      }
    },

    selectPort: (id) => set({ selectedPort: id }),

    refreshPorts: async () => {
      set({ loading: true })
      try {
        // TODO: Invoke Tauri command
        // const ports = await invoke('list_virtual_ports')
        set({ loading: false })
      } catch (error) {
        set({
          loading: false,
          error: error instanceof Error ? error.message : String(error),
        })
      }
    },
  }))
)
