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

export interface ConnectionEntry {
  portId: string;
  portName: string;
  status: ConnectionStatus;
  config: SerialConfig;
  portStatus: PortStats | null;
  connectedAt: number | null;
  error: string | null;
}

interface ConnectionStore {
  availablePorts: PortInfo[];
  connections: ConnectionEntry[];
  activePortId: string | null;
  pendingPort: string | null;
  defaultConfig: SerialConfig;
  serverOccupiedPorts: Set<string>;

  refreshPorts: () => Promise<void>;
  connect: (portName: string, config?: Partial<SerialConfig>) => Promise<void>;
  disconnect: (portId: string) => Promise<void>;
  disconnectAll: () => Promise<void>;
  setActivePort: (portId: string | null) => void;
  setPendingPort: (portName: string | null) => void;
  setServerOccupiedPorts: (ports: string[]) => void;
  removePort: (portId: string) => void;
  setDefaultConfig: (config: Partial<SerialConfig>) => void;
  setPortError: (portId: string, error: string | null) => void;
}

const pollingTimers = new Map<string, ReturnType<typeof setInterval>>();

const startPolling = (portId: string) => {
  stopPolling(portId);
  const poll = async () => {
    try {
      const status = await tauriApi.getPortStatus(portId);
      useConnectionStore.setState((s) => {
        const entry = s.connections.find((c) => c.portId === portId);
        if (!entry) return s;
        return {
          connections: s.connections.map((c) =>
            c.portId === portId ? { ...c, portStatus: status.stats } : c,
          ),
        };
      });
    } catch {
      // ignore polling errors
    }
  };
  const timer = setInterval(poll, 2000);
  pollingTimers.set(portId, timer);
  poll();
};

const stopPolling = (portId: string) => {
  const timer = pollingTimers.get(portId);
  if (timer) {
    clearInterval(timer);
    pollingTimers.delete(portId);
  }
};

export const useConnectionStore = create<ConnectionStore>()((set, get) => ({
  availablePorts: [],
  connections: [],
  activePortId: null,
  pendingPort: null,
  defaultConfig: DEFAULT_CONFIG,
  serverOccupiedPorts: new Set(),

  refreshPorts: async () => {
    try {
      const ports = await tauriApi.listPorts();
      set({ availablePorts: ports });
    } catch (e) {
      set({ availablePorts: [] });
    }
  },

  connect: async (_portName, configOverride) => {
    const { pendingPort, connections, defaultConfig, availablePorts } = get();
    const portName = pendingPort;
    if (!portName) return;
    if (
      connections.some(
        (c) => c.portName === portName && c.status === "connected",
      )
    )
      return;

    // Check if this is a virtual port
    const isVirtual = availablePorts.find(
      (p) => p.port_name === portName,
    )?.is_virtual;

    const finalConfig = { ...defaultConfig, ...configOverride };

    const entry: ConnectionEntry = {
      portId: "",
      portName,
      status: "connecting",
      config: finalConfig,
      portStatus: null,
      connectedAt: null,
      error: null,
    };

    set((s) => ({
      connections: [...s.connections, entry],
      activePortId: s.activePortId ?? "pending",
    }));

    try {
      const portId = await tauriApi.openPort(portName, finalConfig, isVirtual);

      set((s) => ({
        connections: s.connections.map((c) =>
          c.portName === portName && c.status === "connecting"
            ? { ...c, portId, status: "connected", connectedAt: Date.now() }
            : c,
        ),
        activePortId: portId,
      }));

      await tauriApi.startSniffing(portId);
      startPolling(portId);
    } catch (e) {
      set((s) => ({
        connections: s.connections.map((c) =>
          c.portName === portName && c.status === "connecting"
            ? { ...c, status: "error", error: String(e) }
            : c,
        ),
      }));
    }
  },

  disconnect: async (portId) => {
    try {
      await tauriApi.stopSniffing(portId);
      await tauriApi.closePort(portId);
    } catch {
      // Port may already be closed
    }
    stopPolling(portId);
    set((s) => {
      const newConnections = s.connections.map((c) =>
        c.portId === portId
          ? {
              ...c,
              status: "disconnected" as ConnectionStatus,
              portStatus: null,
              connectedAt: null,
            }
          : c,
      );
      let newActive = s.activePortId;
      if (s.activePortId === portId) {
        const next = newConnections.find((c) => c.status === "connected");
        newActive = next?.portId ?? null;
      }
      return { connections: newConnections, activePortId: newActive };
    });
  },

  disconnectAll: async () => {
    const { connections } = get();
    await Promise.all(
      connections
        .filter((c) => c.status === "connected")
        .map((c) => get().disconnect(c.portId)),
    );
  },

  setActivePort: (portId) => set({ activePortId: portId }),

  setPendingPort: (portName) => set({ pendingPort: portName }),

  setServerOccupiedPorts: (ports) =>
    set({ serverOccupiedPorts: new Set(ports) }),

  removePort: (portId) =>
    set((s) => {
      const newConnections = s.connections.filter((c) => c.portId !== portId);
      let newActive = s.activePortId;
      if (s.activePortId === portId) {
        newActive =
          newConnections.find((c) => c.status === "connected")?.portId ?? null;
      }
      return { connections: newConnections, activePortId: newActive };
    }),

  setDefaultConfig: (config) =>
    set((s) => ({ defaultConfig: { ...s.defaultConfig, ...config } })),

  setPortError: (portId, error) =>
    set((s) => ({
      connections: s.connections.map((c) =>
        c.portId === portId ? { ...c, error } : c,
      ),
    })),
}));
