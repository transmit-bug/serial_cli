import { create } from 'zustand'
import { subscribeWithSelector } from 'zustand/middleware'
import { invoke } from '@tauri-apps/api/core'

export interface ScriptInfo {
  name: string
  path: string
  description: string
  modifiedAt: number
}

export interface ScriptOutput {
  id: string
  type: 'log' | 'error' | 'result'
  message: string
  timestamp: number
}

interface ScriptState {
  scripts: ScriptInfo[]
  currentScript: string | null
  output: ScriptOutput[]
  loading: boolean
  running: boolean
  error: string | null

  // Actions
  loadScripts: () => Promise<void>
  runScript: (script: string) => Promise<void>
  stopScript: () => void
  addOutput: (output: Omit<ScriptOutput, 'id' | 'timestamp'>) => void
  clearOutput: () => void
  saveScript: (name: string, content: string) => Promise<void>
  deleteScript: (name: string) => Promise<void>
}

export const useScriptStore = create<ScriptState>()(
  subscribeWithSelector((set, get) => ({
    // Initial state
    scripts: [],
    currentScript: null,
    output: [],
    loading: false,
    running: false,
    error: null,

    // Load all scripts
    loadScripts: async () => {
      set({ loading: true })
      try {
        const scripts = await invoke<ScriptInfo[]>('list_scripts')
        set({ scripts, loading: false, error: null })
      } catch (error) {
        const errorMsg = error instanceof Error ? error.message : 'Failed to load scripts'
        set({ loading: false, error: errorMsg })
      }
    },

    // Run script
    runScript: async (script) => {
      set({ running: true, currentScript: script, output: [], error: null })
      try {
        const result = await invoke<string>('execute_script', { script })
        get().addOutput({ type: 'result', message: result })
        set({ running: false })
      } catch (error) {
        const errorMsg = error instanceof Error ? error.message : 'Failed to run script'
        get().addOutput({ type: 'error', message: errorMsg })
        set({ running: false, error: errorMsg })
      }
    },

    // Stop script
    stopScript: () => set({ running: false, currentScript: null }),

    // Add output line
    addOutput: (output) => {
      set((state) => {
        const newOutput: ScriptOutput = {
          ...output,
          id: `${Date.now()}-${Math.random()}`,
          timestamp: Date.now(),
        }
        let logs = [...state.output, newOutput]

        // Limit to 1000 lines
        if (logs.length > 1000) {
          logs = logs.slice(logs.length - 1000)
        }

        return { output: logs }
      })
    },

    // Clear output
    clearOutput: () => set({ output: [] }),

    // Save script
    saveScript: async (name, content) => {
      try {
        await invoke('save_script', { name, content })
        await get().loadScripts()
      } catch (error) {
        const errorMsg = error instanceof Error ? error.message : 'Failed to save script'
        set({ error: errorMsg })
        throw error
      }
    },

    // Delete script
    deleteScript: async (name) => {
      try {
        await invoke('delete_script', { name })
        await get().loadScripts()
      } catch (error) {
        const errorMsg = error instanceof Error ? error.message : 'Failed to delete script'
        set({ error: errorMsg })
        throw error
      }
    },
  }))
)
