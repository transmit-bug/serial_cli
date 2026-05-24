import { create } from "zustand";
import type { DataPacket, DisplayFormat } from "@/types";

const MAX_PACKETS = 10000;

interface DataStore {
  packets: DataPacket[];
  displayFormat: DisplayFormat;
  autoScroll: boolean;
  maxPackets: number;
  searchQuery: string;
  nextId: number;

  addPacket: (
    direction: "rx" | "tx",
    data: number[],
    timestamp: number,
    decoded?: string,
  ) => void;
  clearBuffer: () => void;
  setDisplayFormat: (format: DisplayFormat) => void;
  toggleAutoScroll: () => void;
  setSearchQuery: (query: string) => void;
}

export const useDataStore = create<DataStore>()((set, get) => ({
  packets: [],
  displayFormat: "mixed",
  autoScroll: true,
  maxPackets: MAX_PACKETS,
  searchQuery: "",
  nextId: 1,

  addPacket: (direction, data, timestamp, decoded) => {
    const { maxPackets, nextId } = get();
    const packet: DataPacket = {
      id: nextId,
      direction,
      timestamp,
      data,
      decoded,
    };
    set((s) => ({
      packets:
        s.packets.length >= maxPackets
          ? [...s.packets.slice(1), packet]
          : [...s.packets, packet],
      nextId: nextId + 1,
    }));
  },

  clearBuffer: () => set({ packets: [], nextId: 1 }),
  setDisplayFormat: (format) => set({ displayFormat: format }),
  toggleAutoScroll: () => set((s) => ({ autoScroll: !s.autoScroll })),
  setSearchQuery: (query) => set({ searchQuery: query }),
}));
