import { create } from "zustand";
import { tauriApi } from "@/lib/tauri-api";
import type { ConfigData } from "@/types";

interface SettingsStore {
  config: ConfigData | null;
  loading: boolean;

  loadConfig: () => Promise<void>;
  updateConfig: (config: ConfigData) => Promise<void>;
  resetConfig: () => Promise<void>;
}

export const useSettingsStore = create<SettingsStore>()((set) => ({
  config: null,
  loading: false,

  loadConfig: async () => {
    set({ loading: true });
    try {
      const config = await tauriApi.getConfig();
      set({ config, loading: false });
    } catch {
      set({ loading: false });
    }
  },

  updateConfig: async (config) => {
    await tauriApi.updateConfig(config);
    set({ config });
  },

  resetConfig: async () => {
    await tauriApi.resetConfig();
    const config = await tauriApi.getConfig();
    set({ config });
  },
}));
