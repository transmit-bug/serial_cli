import { create } from 'zustand'
import { subscribeWithSelector } from 'zustand/middleware'

// TODO: Import types from existing code
export interface AppConfig {
  serial: {
    defaultBaudrate: number
    timeoutMs: number
  }
  protocols: {
    hotReload: boolean
    customDir: string
  }
  virtualPorts: {
    backend: string
    monitor: boolean
  }
  display: {
    theme: 'dark' | 'light'
    fontSize: number
    fontFamily: string
  }
}

interface SettingsState {
  config: AppConfig | null
  loading: boolean
  saving: boolean
  error: string | null

  // Actions
  loadConfig: () => Promise<void>
  saveConfig: () => Promise<void>
  updateConfig: (updates: Partial<AppConfig>) => void
  resetConfig: () => Promise<void>
}

export const useSettingsStore = create<SettingsState>()(
  subscribeWithSelector((set, get) => ({
    // Initial state
    config: null,
    loading: false,
    saving: false,
    error: null,

    // Actions (TODO: Implement in Phase 2)
    loadConfig: async () => {
      set({ loading: true })
      try {
        // TODO: Invoke Tauri command
        // const config = await invoke('get_config')
        set({ loading: false })
      } catch (error) {
        set({
          loading: false,
          error: error instanceof Error ? error.message : String(error),
        })
      }
    },

    saveConfig: async () => {
      const { config } = get()
      if (!config) return

      set({ saving: true })
      try {
        // TODO: Invoke Tauri command
        // await invoke('save_config', { config })
        set({ saving: false })
      } catch (error) {
        set({
          saving: false,
          error: error instanceof Error ? error.message : String(error),
        })
      }
    },

    updateConfig: (updates) =>
      set((state) => ({
        config: state.config ? { ...state.config, ...updates } : null,
      })),

    resetConfig: async () => {
      set({ loading: true })
      try {
        // TODO: Invoke Tauri command
        // await invoke('reset_config')
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
