import { create } from 'zustand'
import { subscribeWithSelector } from 'zustand/middleware'

// TODO: Import types from existing code
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

    // Actions (TODO: Implement in Phase 2)
    loadScripts: async () => {
      set({ loading: true })
      try {
        // TODO: Invoke Tauri command
        // const scripts = await invoke('list_scripts')
        set({ loading: false })
      } catch (error) {
        set({
          loading: false,
          error: error instanceof Error ? error.message : String(error),
        })
      }
    },

    runScript: async (script) => {
      set({ running: true, currentScript: script, output: [] })
      try {
        // TODO: Invoke Tauri command
        // const result = await invoke('execute_script', { script })
        set({ running: false })
      } catch (error) {
        set({
          running: false,
          error: error instanceof Error ? error.message : String(error),
        })
      }
    },

    stopScript: () => set({ running: false }),

    addOutput: (output) => {
      set((state) => {
        const newOutput = {
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

    clearOutput: () => set({ output: [] }),
  }))
)
