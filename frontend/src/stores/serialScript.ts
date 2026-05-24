import { create } from "zustand";
import { tauriApi } from "@/lib/tauri-api";
import type { ScriptStatus, UiAction } from "@/types";

interface SerialScriptStore {
  attachedScript: string | null;
  scriptStatus: ScriptStatus | null;
  actions: UiAction[];
  loading: boolean;

  attachScript: (portId: string, scriptSource: string) => Promise<void>;
  detachScript: (portId: string) => Promise<void>;
  refreshStatus: (portId: string) => Promise<void>;
  loadActions: (portId: string) => Promise<void>;
  callAction: (portId: string, functionName: string) => Promise<string>;
}

export const useSerialScriptStore = create<SerialScriptStore>()((set) => ({
  attachedScript: null,
  scriptStatus: null,
  actions: [],
  loading: false,

  attachScript: async (portId, scriptSource) => {
    set({ loading: true });
    try {
      await tauriApi.attachScript(portId, scriptSource);
      set({ attachedScript: scriptSource, loading: false });
    } catch {
      set({ loading: false });
    }
  },

  detachScript: async (portId) => {
    set({ loading: true });
    try {
      await tauriApi.detachScript(portId);
      set({
        attachedScript: null,
        scriptStatus: null,
        actions: [],
        loading: false,
      });
    } catch {
      set({ loading: false });
    }
  },

  refreshStatus: async (portId) => {
    try {
      const status = await tauriApi.getScriptStatus(portId);
      set({ scriptStatus: status });
    } catch {
      // ignore
    }
  },

  loadActions: async (portId) => {
    try {
      const actions = await tauriApi.listScriptActions(portId);
      set({ actions });
    } catch {
      set({ actions: [] });
    }
  },

  callAction: async (portId, functionName) => {
    return await tauriApi.callScriptFunction(portId, functionName);
  },
}));
