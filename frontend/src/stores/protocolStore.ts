import { create } from 'zustand'
import { subscribeWithSelector } from 'zustand/middleware'
import { invoke } from '@tauri-apps/api/core'
import type { ProtocolInfo } from '@/types/tauri'

interface ProtocolState {
  protocols: ProtocolInfo[]
  activeProtocol: string | null
  loading: boolean
  error: string | null

  // Actions
  loadProtocols: () => Promise<void>
  setActiveProtocol: (name: string | null) => void
  enableProtocol: (name: string) => Promise<void>
  disableProtocol: (name: string) => Promise<void>
  reloadProtocol: (name: string) => Promise<void>
  clearError: () => void
}

export const useProtocolStore = create<ProtocolState>()(
  subscribeWithSelector((set, get) => ({
    // Initial state
    protocols: [],
    activeProtocol: null,
    loading: false,
    error: null,

    // Load all protocols
    loadProtocols: async () => {
      set({ loading: true })
      try {
        const protocols = await invoke<ProtocolInfo[]>('list_protocols')
        set({ protocols, loading: false, error: null })
      } catch (error) {
        const errorMsg = error instanceof Error ? error.message : 'Failed to load protocols'
        set({ loading: false, error: errorMsg })
      }
    },

    // Set active protocol
    setActiveProtocol: (name) => set({ activeProtocol: name }),

    // Enable/load protocol
    enableProtocol: async (name) => {
      try {
        // For custom protocols, the file should already be saved by the frontend.
        // We rely on the backend to find it by name in its protocols directory.
        // Since load_protocol expects { path: String }, we'd need to resolve the path.
        // For now, protocol activation is handled directly by ProtocolPanel which stores
        // the filePath on the Protocol object.
        await get().loadProtocols()
      } catch (error) {
        const errorMsg = error instanceof Error ? error.message : 'Failed to enable protocol'
        set({ error: errorMsg })
        throw error
      }
    },

    // Disable/unload protocol
    disableProtocol: async (name) => {
      try {
        await invoke('unload_protocol', { name })
        await get().loadProtocols()
      } catch (error) {
        const errorMsg = error instanceof Error ? error.message : 'Failed to disable protocol'
        set({ error: errorMsg })
        throw error
      }
    },

    // Reload protocol
    reloadProtocol: async (name) => {
      try {
        await invoke('reload_protocol', { name })
        await get().loadProtocols()
      } catch (error) {
        const errorMsg = error instanceof Error ? error.message : 'Failed to reload protocol'
        set({ error: errorMsg })
        throw error
      }
    },

    // Clear error
    clearError: () => set({ error: null }),
  }))
)
