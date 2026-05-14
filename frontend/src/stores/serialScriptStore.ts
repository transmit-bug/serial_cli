import { create } from 'zustand'
import { subscribeWithSelector } from 'zustand/middleware'
import { invoke } from '@tauri-apps/api/core'
import type { ScriptStatus } from '@/types/tauri'

interface SerialScriptState {
  // State
  scriptStatus: ScriptStatus | null
  loading: boolean
  error: string | null

  // Actions
  checkScriptStatus: (portId: string) => Promise<void>
  attachScript: (portId: string, scriptSource: string) => Promise<void>
  detachScript: (portId: string) => Promise<void>
  clear: () => void
}

export const useSerialScriptStore = create<SerialScriptState>()(
  subscribeWithSelector((set) => ({
    // Initial state
    scriptStatus: null,
    loading: false,
    error: null,

    // Check script status for a port
    checkScriptStatus: async (portId) => {
      try {
        const status = await invoke<ScriptStatus>('get_script_status', { portId })
        set({ scriptStatus: status, error: null })
      } catch {
        set({ scriptStatus: null, error: null })
      }
    },

    // Attach a script to a port
    attachScript: async (portId, scriptSource) => {
      set({ loading: true, error: null })
      try {
        await invoke('attach_script', { portId, scriptSource })
        // Refresh status
        const status = await invoke<ScriptStatus>('get_script_status', { portId })
        set({ scriptStatus: status, loading: false, error: null })
      } catch (error) {
        const errorMsg = error instanceof Error ? error.message : 'Failed to attach script'
        set({ loading: false, error: errorMsg })
        throw error
      }
    },

    // Detach script from a port
    detachScript: async (portId) => {
      set({ loading: true, error: null })
      try {
        await invoke('detach_script', { portId })
        set({ scriptStatus: null, loading: false, error: null })
      } catch (error) {
        const errorMsg = error instanceof Error ? error.message : 'Failed to detach script'
        set({ loading: false, error: errorMsg })
        throw error
      }
    },

    // Clear state (e.g., on disconnect)
    clear: () => set({ scriptStatus: null, loading: false, error: null }),
  }))
)
