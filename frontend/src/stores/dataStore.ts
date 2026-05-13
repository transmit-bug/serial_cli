import { create } from 'zustand'
import { subscribeWithSelector } from 'zustand/middleware'
import { listen } from '@tauri-apps/api/event'
import type { DataEvent } from '@/types/tauri'

export interface DataPacket {
  id: string
  portId: string
  data: number[]
  timestamp: number
  direction: 'rx' | 'tx'
  displayFormat?: 'hex' | 'ascii' | 'mixed'
}

interface DataState {
  rxPackets: DataPacket[]
  txPackets: DataPacket[]
  maxPackets: number
  displayFormat: 'hex' | 'ascii' | 'mixed'
  showTimestamp: boolean

  // Actions
  addRxPacket: (packet: Omit<DataPacket, 'id'>) => void
  addTxPacket: (packet: Omit<DataPacket, 'id'>) => void
  clearRxPackets: () => void
  clearTxPackets: () => void
  clearPackets: () => void
  setDisplayFormat: (format: 'hex' | 'ascii' | 'mixed') => void
  setShowTimestamp: (show: boolean) => void
  setMaxPackets: (max: number) => void

  // Internal
  startListening: () => () => void
}

export const useDataStore = create<DataState>()(
  subscribeWithSelector((set, get) => ({
    // Initial state
    rxPackets: [],
    txPackets: [],
    maxPackets: 10000,
    displayFormat: 'hex',
    showTimestamp: true,

    // Add RX packet
    addRxPacket: (packet) => {
      set((state) => {
        const newPacket = {
          ...packet,
          id: `${Date.now()}-${Math.random()}`,
          displayFormat: state.displayFormat,
        }
        let rxPackets = [...state.rxPackets, newPacket]

        // FIFO cleanup (max 10000 packets)
        if (rxPackets.length > state.maxPackets) {
          rxPackets = rxPackets.slice(rxPackets.length - state.maxPackets)
        }

        return { rxPackets }
      })
    },

    // Add TX packet
    addTxPacket: (packet) => {
      set((state) => {
        const newPacket = {
          ...packet,
          id: `${Date.now()}-${Math.random()}`,
          displayFormat: state.displayFormat,
        }
        let txPackets = [...state.txPackets, newPacket]

        if (txPackets.length > state.maxPackets) {
          txPackets = txPackets.slice(txPackets.length - state.maxPackets)
        }

        return { txPackets }
      })
    },

    // Clear all packets
    clearPackets: () => set({ rxPackets: [], txPackets: [] }),

    // Clear RX packets only
    clearRxPackets: () => set({ rxPackets: [] }),

    // Clear TX packets only
    clearTxPackets: () => set({ txPackets: [] }),

    // Set display format
    setDisplayFormat: (format) =>
      set({
        displayFormat: format,
        rxPackets: [],
        txPackets: [],
      }),

    // Set show timestamp
    setShowTimestamp: (show) => set({ showTimestamp: show }),

    // Set max packets
    setMaxPackets: (max) =>
      set((state) => {
        const nextState: Partial<DataState> = { maxPackets: max }

        // Trim existing packets if needed
        if (state.rxPackets.length > max) {
          nextState.rxPackets = state.rxPackets.slice(-max)
        }
        if (state.txPackets.length > max) {
          nextState.txPackets = state.txPackets.slice(-max)
        }

        return nextState as DataState
      }),

    // Start listening to Tauri events
    startListening: () => {
      const unlistenPromises = [
        // Listen for data-received events
        listen<DataEvent>('data-received', (event) => {
          const { addRxPacket } = get()
          addRxPacket({
            portId: event.payload.port_id,
            direction: 'rx',
            data: event.payload.data,
            timestamp: event.payload.timestamp,
          })
        }),

        // Listen for data-sent events
        listen<DataEvent>('data-sent', (event) => {
          const { addTxPacket } = get()
          addTxPacket({
            portId: event.payload.port_id,
            direction: 'tx',
            data: event.payload.data,
            timestamp: event.payload.timestamp,
          })
        }),
      ]

      // Return cleanup function
      return () => {
        unlistenPromises.forEach((promise) => {
          promise.then((unlisten) => unlisten())
        })
      }
    },
  }))
)

// Auto-start listening when store is created
let cleanup: (() => void) | null = null
if (typeof window !== 'undefined') {
  cleanup = useDataStore.getState().startListening()
}

// Cleanup on hot module reload
if (typeof window !== 'undefined' && (window as any).__TAURI_UNLISTEN__) {
  ;(window as any).__TAURI_UNLISTEN__.push(() => {
    cleanup?.()
  })
}
