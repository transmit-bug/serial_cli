import { create } from "zustand";
import { tauriApi } from "@/lib/tauri-api";

interface LogStore {
  lines: string[];
  loading: boolean;
  autoRefresh: boolean;
  filter: string;

  loadLogs: () => Promise<void>;
  setAutoRefresh: (enabled: boolean) => void;
  setFilter: (filter: string) => void;
  clearLogs: () => Promise<void>;
}

export const useLogStore = create<LogStore>((set, get) => ({
  lines: [],
  loading: false,
  autoRefresh: false,
  filter: "",

  loadLogs: async () => {
    set({ loading: true });
    try {
      const lines = await tauriApi.readLogs(2000);
      set({ lines, loading: false });
    } catch {
      set({ loading: false });
    }
  },

  setAutoRefresh: (enabled) => {
    set({ autoRefresh: enabled });
    if (enabled) {
      get().loadLogs();
    }
  },

  setFilter: (filter) => set({ filter }),

  clearLogs: async () => {
    await tauriApi.clearLogs();
    set({ lines: [] });
  },
}));
