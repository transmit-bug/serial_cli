import { create } from 'zustand'
import { subscribeWithSelector } from 'zustand/middleware'
import { invoke } from '@tauri-apps/api/core'

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

interface PortStatus {
  port_id: string
  port_name: string
  is_open: boolean
  config: SerialConfig
  stats: {
    bytes_sent: number
    bytes_received: number
    packets_sent: number
    packets_received: number
    last_activity: number | null
  }
}

interface ConnectionState {
  // State
  status: ConnectionStatus
  portId: string | null
  portName: string | null
  config: SerialConfig | null
  error: string | null
  portStatus: PortStatus | null

  // Actions
  connect: (portName: string, config: SerialConfig) => Promise<void>
  disconnect: () => Promise<void>
  checkHealth: () => Promise<boolean>
  setStatus: (status: ConnectionStatus) => void
  setError: (error: string | null) => void
}

export const useConnectionStore = create<ConnectionState>()(
  subscribeWithSelector((set, get) => ({
    // Initial state
    status: 'disconnected',
    portId: null,
    portName: null,
    config: null,
    error: null,
    portStatus: null,

    // Connect to a serial port
    connect: async (portName, config) => {
      set({ status: 'connecting', error: null })
      try {
        const portId = await invoke<string>('open_port', {
          portName,
          config,
        })

        // Start data sniffing
        try {
          await invoke('start_sniffing', { portId })
          console.log('Started data sniffing for port:', portId)
        } catch (sniffError) {
          console.error('Failed to start sniffing:', sniffError)
          // Don't fail connection if sniffing fails
        }

        set({
          status: 'connected',
          portId,
          portName,
          config,
          error: null,
          portStatus: {
            port_id: portId,
            port_name: portName,
            is_open: true,
            config,
            stats: {
              bytes_sent: 0,
              bytes_received: 0,
              packets_sent: 0,
              packets_received: 0,
              last_activity: null,
            },
          },
        })
      } catch (error) {
        const errorMsg = error instanceof Error ? error.message : String(error)
        set({ status: 'error', error: errorMsg })
        throw error
      }
    },

    // Disconnect from current port
    disconnect: async () => {
      const { portId } = get()
      if (portId) {
        try {
          await invoke('close_port', { portId })
          set({
            status: 'disconnected',
            portId: null,
            portName: null,
            config: null,
            error: null,
            portStatus: null,
          })
        } catch (error) {
          const errorMsg = error instanceof Error ? error.message : String(error)
          set({ error: errorMsg })
          throw error
        }
      }
    },

    // Check if current port is still healthy
    checkHealth: async () => {
      const { portId } = get()
      if (!portId) return false

      try {
        const isHealthy = await invoke<boolean>('check_port_health', { portId })
        if (!isHealthy) {
          set({
            status: 'disconnected',
            portId: null,
            portName: null,
            config: null,
            portStatus: null,
          })
        }
        return isHealthy
      } catch (error) {
        console.error('Failed to check port health:', error)
        return false
      }
    },

    setStatus: (status) => set({ status }),
    setError: (error) => set({ error }),
  }))
)
