import { create } from "zustand";
import { tauriApi } from "@/lib/tauri-api";
import { useConnectionStore } from "@/stores/connection";
import type { ConnectionPreset } from "@/types";

interface PresetsStore {
  presets: ConnectionPreset[];
  loading: boolean;

  loadPresets: () => Promise<void>;
  addPreset: (preset: ConnectionPreset) => Promise<void>;
  updatePreset: (oldName: string, preset: ConnectionPreset) => Promise<void>;
  deletePreset: (name: string) => Promise<void>;
  movePresetUp: (index: number) => Promise<void>;
  movePresetDown: (index: number) => Promise<void>;
  applyPreset: (preset: ConnectionPreset) => void;
}

export const usePresetsStore = create<PresetsStore>()((set, get) => ({
  presets: [],
  loading: false,

  loadPresets: async () => {
    set({ loading: true });
    try {
      const presets = await tauriApi.getConnectionPresets();
      set({ presets, loading: false });
    } catch {
      set({ loading: false });
    }
  },

  addPreset: async (preset) => {
    const { presets } = get();
    const updated = [...presets, preset];
    await tauriApi.saveConnectionPresets(updated);
    set({ presets: updated });
  },

  updatePreset: async (oldName, preset) => {
    const { presets } = get();
    const updated = presets.map((p) => (p.name === oldName ? preset : p));
    await tauriApi.saveConnectionPresets(updated);
    set({ presets: updated });
  },

  deletePreset: async (name) => {
    const { presets } = get();
    await tauriApi.deleteConnectionPreset(name);
    set({ presets: presets.filter((p) => p.name !== name) });
  },

  movePresetUp: async (index) => {
    if (index === 0) return;
    const { presets } = get();
    const updated = [...presets];
    [updated[index - 1], updated[index]] = [updated[index], updated[index - 1]];
    await tauriApi.saveConnectionPresets(updated);
    set({ presets: updated });
  },

  movePresetDown: async (index) => {
    const { presets } = get();
    if (index >= presets.length - 1) return;
    const updated = [...presets];
    [updated[index], updated[index + 1]] = [updated[index + 1], updated[index]];
    await tauriApi.saveConnectionPresets(updated);
    set({ presets: updated });
  },

  applyPreset: (preset) => {
    const { setDefaultConfig } = useConnectionStore.getState();
    setDefaultConfig({
      baudrate: preset.baudrate,
      databits: preset.databits,
      stopbits: preset.stopbits,
      parity: preset.parity,
      flow_control: preset.flow_control,
      timeout_ms: preset.timeout_ms,
    });
    if (preset.port_name) {
      useConnectionStore.setState({ pendingPort: preset.port_name });
    }
  },
}));
