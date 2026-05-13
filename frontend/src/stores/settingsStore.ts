import { create } from 'zustand'
import { subscribeWithSelector } from 'zustand/middleware'
import { invoke } from '@tauri-apps/api/core'

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
    maxPackets: number
    format: 'hex' | 'ascii' | 'both'
    showTimestamp: boolean
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
  clearError: () => void
}

export const useSettingsStore = create<SettingsState>()(
  subscribeWithSelector((set, get) => ({
    // Initial state
    config: null,
    loading: false,
    saving: false,
    error: null,

    // Load config
    loadConfig: async () => {
      set({ loading: true })
      try {
        const config = await invoke<AppConfig>('get_config')
        set({ config, loading: false, error: null })
      } catch (error) {
        const errorMsg = error instanceof Error ? error.message : 'Failed to load config'
        set({ loading: false, error: errorMsg })
      }
    },

    // Save config
    saveConfig: async () => {
      const { config } = get()
      if (!config) return

      set({ saving: true })
      try {
        await invoke('update_config', { config })
        set({ saving: false, error: null })
      } catch (error) {
        const errorMsg = error instanceof Error ? error.message : 'Failed to save config'
        set({ saving: false, error: errorMsg })
        throw error
      }
    },

    // Update config
    updateConfig: (updates) =>
      set((state) => ({
        config: state.config ? { ...state.config, ...updates } : null,
      })),

    // Reset config
    resetConfig: async () => {
      set({ loading: true })
      try {
        await invoke('reset_config')
        await get().loadConfig()
      } catch (error) {
        const errorMsg = error instanceof Error ? error.message : 'Failed to reset config'
        set({ loading: false, error: errorMsg })
        throw error
      }
    },

    // Clear error
    clearError: () => set({ error: null }),
  }))
)
