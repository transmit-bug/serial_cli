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

  refreshPorts: () => Promise<void>;
  createPort: (config: CreateVirtualPortConfig) => Promise<void>;
  stopPort: (id: string) => Promise<void>;
  getStats: (id: string) => Promise<VirtualPortStats | null>;
  checkHealth: (id: string) => Promise<boolean>;
  loadCapturedPackets: (id: string) => Promise<void>;
  setSelectedPort: (id: string | null) => void;
}

export const useVirtualPortStore = create<VirtualPortStore>()((set) => ({
  ports: [],
  selectedPort: null,
  capturedPackets: [],
  healthMap: {},
  loading: false,

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
    }));
  },

  getStats: async (id) => {
    try {
      return await tauriApi.getVirtualPortStats(id);
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
}));
