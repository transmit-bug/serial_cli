import { listen } from "@tauri-apps/api/event";
import { create } from "zustand";
import { tauriApi } from "@/lib/tauri-api";
import type {
  RemoteConnectionInfo,
  RemoteDataEvent,
  RemoteDevice,
  RemoteOpenResult,
  RemotePortInfo,
  RemoteRecvResult,
  RemoteStreamErrorEvent,
} from "@/types";

function hexToText(hex: string): string {
  const clean = hex.replace(/\s+/g, "");
  if (!/^[0-9a-fA-F]*$/.test(clean) || clean.length % 2 !== 0) return "";
  const bytes: number[] = [];
  for (let i = 0; i < clean.length; i += 2) {
    bytes.push(parseInt(clean.slice(i, i + 2), 16));
  }
  return new TextDecoder()
    .decode(new Uint8Array(bytes))
    .split("")
    .filter((ch) => ch.charCodeAt(0) >= 0x20 && ch.charCodeAt(0) !== 0x7f)
    .join("");
}

interface RemoteState {
  devices: RemoteDevice[];
  loading: boolean;
  error: string | null;

  // Workbench (selected device)
  activeDeviceId: string | null;
  ports: RemotePortInfo[];
  connections: RemoteConnectionInfo[];
  workbenchLoading: boolean;
  workbenchError: string | null;

  // Data streaming (per connection_id)
  streaming: Record<string, boolean>;
  rxBuffers: Record<string, string>;
}

interface RemoteActions {
  loadDevices: () => Promise<void>;
  addDevice: (name: string, host: string, port: number) => Promise<void>;
  updateDevice: (
    id: string,
    name: string,
    host: string,
    port: number,
  ) => Promise<void>;
  deleteDevice: (id: string) => Promise<void>;
  testDevice: (id: string) => Promise<boolean>;
  selectDevice: (id: string | null) => Promise<void>;
  refreshPorts: () => Promise<void>;
  refreshConnections: () => Promise<void>;
  openRemotePort: (
    port: string,
    baudrate: number,
  ) => Promise<RemoteOpenResult | null>;
  closeRemoteConnection: (connectionId: string) => Promise<void>;
  sendRemoteData: (connectionId: string, data: number[]) => Promise<void>;
  recvRemoteData: (
    connectionId: string,
    timeoutMs: number,
  ) => Promise<RemoteRecvResult | null>;
  startStream: (connectionId: string) => Promise<void>;
  stopStream: (connectionId: string) => Promise<void>;
  stopDeviceStreams: (deviceId: string) => Promise<void>;
  appendStreamData: (event: RemoteDataEvent) => void;
  streamError: (event: RemoteStreamErrorEvent) => void;
  streamClosed: (deviceId: string, connectionId: string) => void;
  clearRx: (connectionId: string) => void;
  setError: (error: string | null) => void;
  setWorkbenchError: (error: string | null) => void;
}

export const useRemoteStore = create<RemoteState & RemoteActions>(
  (set, get) => ({
    devices: [],
    loading: false,
    error: null,

    activeDeviceId: null,
    ports: [],
    connections: [],
    workbenchLoading: false,
    workbenchError: null,

    streaming: {},
    rxBuffers: {},

    loadDevices: async () => {
      set({ loading: true, error: null });
      try {
        const devices = await tauriApi.getRemoteDevices();
        set({ devices, loading: false });
      } catch (err) {
        set({ error: String(err), loading: false });
      }
    },

    addDevice: async (name, host, port) => {
      set({ loading: true, error: null });
      try {
        const devices = await tauriApi.addRemoteDevice(name, host, port);
        set({ devices, loading: false });
      } catch (err) {
        set({ error: String(err), loading: false });
        throw err;
      }
    },

    updateDevice: async (id, name, host, port) => {
      set({ loading: true, error: null });
      try {
        const devices = await tauriApi.updateRemoteDevice(id, name, host, port);
        set({ devices, loading: false });
      } catch (err) {
        set({ error: String(err), loading: false });
        throw err;
      }
    },

    deleteDevice: async (id) => {
      set({ loading: true, error: null });
      try {
        await get().stopDeviceStreams(id);
        const devices = await tauriApi.deleteRemoteDevice(id);
        set((s) => ({
          devices,
          loading: false,
          activeDeviceId: s.activeDeviceId === id ? null : s.activeDeviceId,
          ports: s.activeDeviceId === id ? [] : s.ports,
          connections: s.activeDeviceId === id ? [] : s.connections,
        }));
      } catch (err) {
        set({ error: String(err), loading: false });
      }
    },

    testDevice: async (id) => {
      try {
        await tauriApi.testRemoteDevice(id);
        return true;
      } catch {
        return false;
      }
    },

    selectDevice: async (id) => {
      // Stop streams of the device we're leaving
      const { activeDeviceId } = get();
      if (activeDeviceId && activeDeviceId !== id) {
        await get().stopDeviceStreams(activeDeviceId);
      }
      set({
        activeDeviceId: id,
        ports: [],
        connections: [],
        workbenchError: null,
      });
      if (id) {
        await Promise.all([get().refreshPorts(), get().refreshConnections()]);
      }
    },

    refreshPorts: async () => {
      const { activeDeviceId } = get();
      if (!activeDeviceId) return;
      set({ workbenchLoading: true, workbenchError: null });
      try {
        const ports = await tauriApi.remotePortList(activeDeviceId);
        set({ ports, workbenchLoading: false });
      } catch (err) {
        set({ workbenchError: String(err), workbenchLoading: false });
      }
    },

    refreshConnections: async () => {
      const { activeDeviceId } = get();
      if (!activeDeviceId) return;
      try {
        const connections = await tauriApi.remoteConnectionList(activeDeviceId);
        set({ connections });
      } catch {
        // Non-fatal; keep existing list
      }
    },

    openRemotePort: async (port, baudrate) => {
      const { activeDeviceId } = get();
      if (!activeDeviceId) return null;
      set({ workbenchLoading: true, workbenchError: null });
      try {
        const result = await tauriApi.remoteOpenPort(
          activeDeviceId,
          port,
          baudrate,
        );
        await get().refreshConnections();
        set({ workbenchLoading: false });
        return result;
      } catch (err) {
        set({ workbenchError: String(err), workbenchLoading: false });
        return null;
      }
    },

    closeRemoteConnection: async (connectionId) => {
      const { activeDeviceId } = get();
      if (!activeDeviceId) return;
      try {
        // Stop streaming for this connection first
        if (get().streaming[connectionId]) {
          await tauriApi.stopRemoteSubscribe(activeDeviceId, connectionId);
        }
        await tauriApi.remoteCloseConnection(activeDeviceId, connectionId);
        set((s) => ({
          connections: s.connections.filter(
            (c) => c.connection_id !== connectionId,
          ),
          streaming: { ...s.streaming, [connectionId]: false },
        }));
      } catch (err) {
        set({ workbenchError: String(err) });
      }
    },

    startStream: async (connectionId) => {
      const { activeDeviceId } = get();
      if (!activeDeviceId) return;
      try {
        await tauriApi.startRemoteSubscribe(activeDeviceId, connectionId);
        set((s) => ({ streaming: { ...s.streaming, [connectionId]: true } }));
      } catch (err) {
        set({ workbenchError: String(err) });
      }
    },

    stopStream: async (connectionId) => {
      const { activeDeviceId } = get();
      if (!activeDeviceId) return;
      try {
        await tauriApi.stopRemoteSubscribe(activeDeviceId, connectionId);
        set((s) => ({ streaming: { ...s.streaming, [connectionId]: false } }));
      } catch (err) {
        set({ workbenchError: String(err) });
      }
    },

    stopDeviceStreams: async (deviceId) => {
      const { streaming } = get();
      const running = Object.entries(streaming)
        .filter(([, on]) => on)
        .map(([connectionId]) => connectionId);
      for (const connectionId of running) {
        try {
          await tauriApi.stopRemoteSubscribe(deviceId, connectionId);
        } catch {
          // Best-effort cleanup
        }
      }
      set({ streaming: {} });
    },

    appendStreamData: (event) => {
      // Event contract is snake_case (matches the Rust backend + mock):
      // { device_id, connection_id, data, bytes_read, timestamp }
      const connectionId = event.connection_id;
      if (!connectionId) return;
      const text = hexToText(event.data) || event.data;
      set((s) => ({
        rxBuffers: {
          ...s.rxBuffers,
          [connectionId]: `${new Date(
            event.timestamp * 1000,
          ).toLocaleTimeString()} ${text}\n${s.rxBuffers[connectionId] ?? ""}`.slice(
            0,
            20000,
          ),
        },
      }));
    },

    streamError: (event) => {
      set((s) => ({
        streaming: { ...s.streaming, [event.connection_id]: false },
        workbenchError: event.message,
      }));
    },

    streamClosed: (deviceId, connectionId) => {
      void deviceId;
      set((s) => ({ streaming: { ...s.streaming, [connectionId]: false } }));
    },

    clearRx: (connectionId) => {
      set((s) => ({ rxBuffers: { ...s.rxBuffers, [connectionId]: "" } }));
    },

    sendRemoteData: async (connectionId, data) => {
      const { activeDeviceId } = get();
      if (!activeDeviceId) return;
      try {
        await tauriApi.remoteSendData(activeDeviceId, connectionId, data);
      } catch (err) {
        set({ workbenchError: String(err) });
      }
    },

    recvRemoteData: async (connectionId, timeoutMs) => {
      const { activeDeviceId } = get();
      if (!activeDeviceId) return null;
      try {
        return await tauriApi.remoteRecvData(
          activeDeviceId,
          connectionId,
          timeoutMs,
        );
      } catch (err) {
        set({ workbenchError: String(err) });
        return null;
      }
    },

    setError: (error) => set({ error }),
    setWorkbenchError: (workbenchError) => set({ workbenchError }),
  }),
);

// Setup Tauri event listeners for remote data streaming
// Returns an unsubscribe function.
export function setupRemoteDataListener(): Promise<() => void> {
  return Promise.all([
    listen<RemoteDataEvent>("remote-data-received", (event) => {
      useRemoteStore.getState().appendStreamData(event.payload);
    }),
    listen<RemoteStreamErrorEvent>("remote-stream-error", (event) => {
      useRemoteStore.getState().streamError(event.payload);
    }),
    listen<{ device_id: string; connection_id: string }>(
      "remote-stream-closed",
      (event) => {
        const { device_id, connection_id } = event.payload;
        useRemoteStore.getState().streamClosed(device_id, connection_id);
      },
    ),
  ]).then((unlisteners) => () => unlisteners.forEach((un) => un()));
}
