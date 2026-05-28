import { create } from "zustand";
import { tauriApi } from "@/lib/tauri-api";
import type {
  CapturedPacket,
  CreateVirtualPortConfig,
  VirtualPortInfo,
  VirtualPortStats,
} from "@/types";

interface VirtualPortStore {
  ports: VirtualPortInfo[];
  selectedPort: string | null;
  capturedPackets: CapturedPacket[];
  healthMap: Record<string, boolean>;
  loading: boolean;

  // Enhanced state
  statsMap: Record<string, VirtualPortStats>;
  throughputMap: Record<string, number>;
  prevBytesMap: Record<string, number>;
  createFormOpen: boolean;

  refreshPorts: () => Promise<void>;
  createPort: (config: CreateVirtualPortConfig) => Promise<void>;
  stopPort: (id: string) => Promise<void>;
  getStats: (id: string) => Promise<VirtualPortStats | null>;
  checkHealth: (id: string) => Promise<boolean>;
  loadCapturedPackets: (id: string) => Promise<void>;
  setSelectedPort: (id: string | null) => void;
  setCreateFormOpen: (open: boolean) => void;
  sendToPort: (portEnd: string, data: number[]) => Promise<number>;
}

export const useVirtualPortStore = create<VirtualPortStore>()((set, get) => ({
  ports: [],
  selectedPort: null,
  capturedPackets: [],
  healthMap: {},
  loading: false,

  // Enhanced state
  statsMap: {},
  throughputMap: {},
  prevBytesMap: {},
  createFormOpen: false,

  refreshPorts: async () => {
    set({ loading: true });
    try {
      const ports = await tauriApi.listVirtualPorts();
      set({ ports, loading: false });
    } catch {
      set({ loading: false });
    }
  },

  createPort: async (config) => {
    await tauriApi.createVirtualPort(config);
    const ports = await tauriApi.listVirtualPorts();
    set({ ports });
  },

  stopPort: async (id) => {
    await tauriApi.stopVirtualPort(id);
    set((s) => ({
      ports: s.ports.filter((p) => p.id !== id),
      selectedPort: s.selectedPort === id ? null : s.selectedPort,
      statsMap: {
        ...s.statsMap,
        [id]: undefined as unknown as VirtualPortStats,
      },
      throughputMap: { ...s.throughputMap, [id]: 0 },
    }));
  },

  getStats: async (id) => {
    try {
      const stats = await tauriApi.getVirtualPortStats(id);
      if (stats) {
        // Compute throughput delta
        const { prevBytesMap, throughputMap } = get();
        const prevBytes = prevBytesMap[id] ?? stats.bytes_bridged;
        const delta = stats.bytes_bridged - prevBytes;
        set({
          statsMap: { ...get().statsMap, [id]: stats },
          throughputMap: { ...throughputMap, [id]: delta },
          prevBytesMap: { ...prevBytesMap, [id]: stats.bytes_bridged },
        });
      }
      return stats;
    } catch {
      return null;
    }
  },

  checkHealth: async (id) => {
    try {
      const healthy = await tauriApi.checkVirtualPortHealth(id);
      set((s) => ({ healthMap: { ...s.healthMap, [id]: healthy } }));
      return healthy;
    } catch {
      set((s) => ({ healthMap: { ...s.healthMap, [id]: false } }));
      return false;
    }
  },

  loadCapturedPackets: async (id) => {
    try {
      const packets = await tauriApi.getCapturedPackets(id);
      set({ capturedPackets: packets });
    } catch {
      set({ capturedPackets: [] });
    }
  },

  setSelectedPort: (id) => set({ selectedPort: id }),

  setCreateFormOpen: (open) => set({ createFormOpen: open }),

  sendToPort: async (portEnd: string, data: number[]) => {
    const { selectedPort } = get();
    if (!selectedPort) throw new Error("No virtual port selected");
    return tauriApi.sendToVirtualPort(selectedPort, portEnd, data);
  },
}));
