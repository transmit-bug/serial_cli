import { create } from "zustand";
import { tauriApi } from "@/lib/tauri-api";
import { listen } from "@tauri-apps/api/event";
import type { ServerStatus, ServerConnectionInfo } from "@/types";

interface ServerState {
  status: ServerStatus;
  loading: boolean;
  error: string | null;
}

interface ServerActions {
  startServer: () => Promise<void>;
  stopServer: () => Promise<void>;
  refreshStatus: () => Promise<void>;
  setError: (error: string | null) => void;
}

export const useServerStore = create<ServerState & ServerActions>(
  (set, get) => ({
    status: {
      running: false,
      socket_path: "",
      started_at: 0,
      active_connections: 0,
      total_requests: 0,
      total_errors: 0,
      connections: [],
    },
    loading: false,
    error: null,

    startServer: async () => {
      set({ loading: true, error: null });
      try {
        const status = await tauriApi.startServer();
        set({ status, loading: false });
      } catch (err) {
        set({ error: String(err), loading: false });
      }
    },

    stopServer: async () => {
      set({ loading: true, error: null });
      try {
        await tauriApi.stopServer();
        set({
          status: {
            running: false,
            socket_path: "",
            started_at: 0,
            active_connections: 0,
            total_requests: 0,
            total_errors: 0,
            connections: [],
          },
          loading: false,
        });
      } catch (err) {
        set({ error: String(err), loading: false });
      }
    },

    refreshStatus: async () => {
      try {
        const status = await tauriApi.getServerStatus();
        set({ status, error: null });
      } catch (err) {
        set({ error: String(err) });
      }
    },

    setError: (error) => set({ error }),
  }),
);

// Setup event listener for server status changes
export function setupServerEventListener() {
  return listen<{ running: boolean; socket_path: string }>(
    "server-status-changed",
    (event) => {
      const { running, socket_path } = event.payload;
      useServerStore.setState((state) => ({
        status: {
          ...state.status,
          running,
          socket_path,
        },
      }));
    },
  );
}
