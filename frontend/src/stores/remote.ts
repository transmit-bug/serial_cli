import { create } from "zustand";
import { tauriApi } from "@/lib/tauri-api";
import type {
  RemoteConnectionInfo,
  RemoteDevice,
  RemoteOpenResult,
  RemotePortInfo,
  RemoteRecvResult,
} from "@/types";

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
        await tauriApi.remoteCloseConnection(activeDeviceId, connectionId);
        await get().refreshConnections();
      } catch (err) {
        set({ workbenchError: String(err) });
      }
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
