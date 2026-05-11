import { create } from 'zustand'
import { subscribeWithSelector } from 'zustand/middleware'
import { invoke } from '@tauri-apps/api/core'
import type { VirtualPortInfo, VirtualPortStats, VirtualPortConfig, CapturedPacket } from '@/types/tauri'

interface VirtualPortState {
  ports: Map<string, VirtualPortInfo>
  portStats: Map<string, VirtualPortStats>
  capturedPackets: Map<string, CapturedPacket[]>
  selectedPort: string | null
  loading: boolean
  error: string | null

  // Actions
  createPort: (config: VirtualPortConfig) => Promise<string>
  stopPort: (id: string) => Promise<void>
  listPorts: () => Promise<void>
  getPortStats: (id: string) => Promise<VirtualPortStats>
  getCapturedPackets: (id: string) => Promise<CapturedPacket[]>
  selectPort: (id: string | null) => void
  clearError: () => void
}

export const useVirtualPortStore = create<VirtualPortState>()(
  subscribeWithSelector((set, get) => ({
    // Initial state
    ports: new Map(),
    portStats: new Map(),
    capturedPackets: new Map(),
    selectedPort: null,
    loading: false,
    error: null,

    // Create virtual port
    createPort: async (config) => {
      set({ loading: true, error: null })
      try {
        const id = await invoke<string>('create_virtual_port', { config })

        // Refresh list after creation
        await get().listPorts()

        set({ loading: false })
        return id
      } catch (error) {
        const errorMsg = error instanceof Error ? error.message : 'Failed to create virtual port'
        set({ loading: false, error: errorMsg })
        throw error
      }
    },

    // Stop virtual port
    stopPort: async (id) => {
      set({ loading: true })
      try {
        await invoke('stop_virtual_port', { id })

        // Remove from local state
        set((state) => {
          const ports = new Map(state.ports)
          const stats = new Map(state.portStats)
          const packets = new Map(state.capturedPackets)

          ports.delete(id)
          stats.delete(id)
          packets.delete(id)

          return { ports, portStats: stats, capturedPackets: packets, loading: false }
        })
      } catch (error) {
        const errorMsg = error instanceof Error ? error.message : 'Failed to stop virtual port'
        set({ loading: false, error: errorMsg })
        throw error
      }
    },

    // List all virtual ports
    listPorts: async () => {
      set({ loading: true })
      try {
        const ports = await invoke<VirtualPortInfo[]>('list_virtual_ports')
        const portsMap = new Map(ports.map((p) => [p.id, p]))
        set({ ports: portsMap, loading: false, error: null })
      } catch (error) {
        const errorMsg = error instanceof Error ? error.message : 'Failed to list virtual ports'
        set({ loading: false, error: errorMsg })
      }
    },

    // Get port stats
    getPortStats: async (id) => {
      try {
        const stats = await invoke<VirtualPortStats>('get_virtual_port_stats', { id })
        set((state) => {
          const portStats = new Map(state.portStats)
          portStats.set(id, stats)
          return { portStats }
        })
        return stats
      } catch (error) {
        console.error(`Failed to get stats for port ${id}:`, error)
        throw error
      }
    },

    // Get captured packets
    getCapturedPackets: async (id) => {
      try {
        const packets = await invoke<CapturedPacket[]>('get_captured_packets', { id })
        set((state) => {
          const capturedPackets = new Map(state.capturedPackets)
          capturedPackets.set(id, packets)
          return { capturedPackets }
        })
        return packets
      } catch (error) {
        console.error(`Failed to get captured packets for port ${id}:`, error)
        throw error
      }
    },

    // Select port
    selectPort: (id) => set({ selectedPort: id }),

    // Clear error
    clearError: () => set({ error: null }),
  }))
)
