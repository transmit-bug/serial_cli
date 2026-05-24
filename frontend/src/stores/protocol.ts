import { create } from "zustand";
import { tauriApi } from "@/lib/tauri-api";
import type { ProtocolInfo } from "@/types";

interface ProtocolStore {
  protocols: ProtocolInfo[];
  activeProtocol: string | null;
  loading: boolean;

  loadProtocols: () => Promise<void>;
  setActiveProtocol: (
    portId: string,
    protocolName: string | null,
  ) => Promise<void>;
  loadCustomProtocol: (path: string) => Promise<void>;
  unloadProtocol: (name: string) => Promise<void>;
  reloadProtocol: (name: string) => Promise<void>;
}

export const useProtocolStore = create<ProtocolStore>()((set) => ({
  protocols: [],
  activeProtocol: null,
  loading: false,

  loadProtocols: async () => {
    set({ loading: true });
    try {
      const protocols = await tauriApi.listProtocols();
      set({ protocols, loading: false });
    } catch {
      set({ loading: false });
    }
  },

  setActiveProtocol: async (portId, protocolName) => {
    if (!protocolName) {
      set({ activeProtocol: null });
      return;
    }
    await tauriApi.setPortProtocol(portId, protocolName);
    set({ activeProtocol: protocolName });
  },

  loadCustomProtocol: async (path) => {
    await tauriApi.loadProtocol(path);
    const protocols = await tauriApi.listProtocols();
    set({ protocols });
  },

  unloadProtocol: async (name) => {
    await tauriApi.unloadProtocol(name);
    set((s) => ({
      protocols: s.protocols.filter((p) => p.name !== name),
      activeProtocol: s.activeProtocol === name ? null : s.activeProtocol,
    }));
  },

  reloadProtocol: async (name) => {
    await tauriApi.reloadProtocol(name);
  },
}));
