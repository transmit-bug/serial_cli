import { create } from "zustand";
import { tauriApi } from "@/lib/tauri-api";
import type {
  ConnectionStatus,
  PortInfo,
  PortStats,
  SerialConfig,
} from "@/types";

const DEFAULT_CONFIG: SerialConfig = {
  baudrate: 115200,
  databits: 8,
  stopbits: 1,
  parity: "None",
  timeout_ms: 1000,
  flow_control: "None",
};

interface ConnectionStore {
  portId: string | null;
  portName: string | null;
  status: ConnectionStatus;
  config: SerialConfig;
  availablePorts: PortInfo[];
  error: string | null;
  portStatus: PortStats | null;
  connectedAt: number | null;

  refreshPorts: () => Promise<void>;
  connect: (portName: string, config?: Partial<SerialConfig>) => Promise<void>;
  disconnect: () => Promise<void>;
  checkHealth: () => Promise<boolean>;
  setConfig: (config: Partial<SerialConfig>) => void;
  setError: (error: string | null) => void;
  startStatusPolling: () => void;
  stopStatusPolling: () => void;
}

let pollingTimer: ReturnType<typeof setInterval> | null = null;

export const useConnectionStore = create<ConnectionStore>()((set, get) => ({
  portId: null,
  portName: null,
  status: "disconnected",
  config: DEFAULT_CONFIG,
  availablePorts: [],
  error: null,
  portStatus: null,
  connectedAt: null,

  refreshPorts: async () => {
    try {
      const ports = await tauriApi.listPorts();
      set({ availablePorts: ports });
    } catch (e) {
      set({ error: String(e) });
    }
  },

  connect: async (portName, configOverride) => {
    const { config, status } = get();
    if (status === "connected" || status === "connecting") return;

    const finalConfig = { ...config, ...configOverride };
    set({ status: "connecting", error: null });

    try {
      const portId = await tauriApi.openPort(portName, finalConfig);
      set({
        portId,
        portName,
        status: "connected",
        config: finalConfig,
        connectedAt: Date.now(),
      });

      await tauriApi.startSniffing(portId);
      get().startStatusPolling();
    } catch (e) {
      set({ status: "error", error: String(e) });
    }
  },

  disconnect: async () => {
    const { portId } = get();
    if (!portId) return;

    get().stopStatusPolling();

    try {
      await tauriApi.stopSniffing(portId);
      await tauriApi.closePort(portId);
    } catch {
      // Port may already be closed
    }
    set({
      portId: null,
      portName: null,
      status: "disconnected",
      error: null,
      portStatus: null,
      connectedAt: null,
    });
  },

  checkHealth: async () => {
    const { portId } = get();
    if (!portId) return false;
    try {
      return await tauriApi.checkPortHealth(portId);
    } catch {
      return false;
    }
  },

  setConfig: (config) => set((s) => ({ config: { ...s.config, ...config } })),
  setError: (error) => set({ error }),

  startStatusPolling: () => {
    get().stopStatusPolling();
    const poll = async () => {
      const { portId, status } = get();
      if (!portId || status !== "connected") return;
      try {
        const portStatus = await tauriApi.getPortStatus(portId);
        set({ portStatus: portStatus.stats });
      } catch {
        // ignore polling errors
      }
    };
    pollingTimer = setInterval(poll, 2000);
    poll();
  },

  stopStatusPolling: () => {
    if (pollingTimer) {
      clearInterval(pollingTimer);
      pollingTimer = null;
    }
  },
}));
