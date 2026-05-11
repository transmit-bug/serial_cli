import { create } from 'zustand'
import { subscribeWithSelector } from 'zustand/middleware'

export interface DataPacket {
  id: string
  portId: string
  data: number[]
  timestamp: number
  direction: 'rx' | 'tx'
  displayFormat: 'hex' | 'ascii' | 'mixed'
}

interface DataState {
  rxPackets: DataPacket[]
  txPackets: DataPacket[]
  maxPackets: number
  displayFormat: DataPacket['displayFormat']

  // Actions
  addRxPacket: (packet: Omit<DataPacket, 'id'>) => void
  addTxPacket: (packet: Omit<DataPacket, 'id'>) => void
  clearPackets: () => void
  setDisplayFormat: (format: DataPacket['displayFormat']) => void
}

export const useDataStore = create<DataState>()(
  subscribeWithSelector((set, get) => ({
    // Initial state
    rxPackets: [],
    txPackets: [],
    maxPackets: 10000,
    displayFormat: 'hex',

    // Actions (TODO: Implement in Phase 2)
    addRxPacket: (packet) => {
      set((state) => {
        const newPacket = { ...packet, id: `${Date.now()}-${Math.random()}` }
        let rxPackets = [...state.rxPackets, newPacket]

        // FIFO cleanup (max 10000 packets)
        if (rxPackets.length > state.maxPackets) {
          rxPackets = rxPackets.slice(rxPackets.length - state.maxPackets)
        }

        return { rxPackets }
      })
    },

    addTxPacket: (packet) => {
      set((state) => {
        const newPacket = { ...packet, id: `${Date.now()}-${Math.random()}` }
        let txPackets = [...state.txPackets, newPacket]

        if (txPackets.length > state.maxPackets) {
          txPackets = txPackets.slice(txPackets.length - state.maxPackets)
        }

        return { txPackets }
      })
    },

    clearPackets: () => set({ rxPackets: [], txPackets: [] }),

    setDisplayFormat: (format) =>
      set((state) => ({
        rxPackets: state.rxPackets.map((p) => ({ ...p, displayFormat: format })),
        txPackets: state.txPackets.map((p) => ({ ...p, displayFormat: format })),
      })),
  }))
)
