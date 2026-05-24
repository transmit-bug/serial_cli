import { create } from "zustand";
import { tauriApi } from "@/lib/tauri-api";
import type { ScriptInfo, ValidationError } from "@/types";

interface ScriptStore {
  scripts: ScriptInfo[];
  currentScript: { name: string; content: string } | null;
  isDirty: boolean;
  output: string[];
  loading: boolean;

  loadScriptList: () => Promise<void>;
  openScript: (name: string) => Promise<void>;
  saveScript: (name: string, content: string) => Promise<void>;
  deleteScript: (name: string) => Promise<void>;
  executeScript: (content: string) => Promise<void>;
  validateScript: (content: string) => Promise<ValidationError[]>;
  newScript: () => void;
  updateContent: (content: string) => void;
  clearOutput: () => void;
}

export const useScriptStore = create<ScriptStore>()((set, get) => ({
  scripts: [],
  currentScript: null,
  isDirty: false,
  output: [],
  loading: false,

  loadScriptList: async () => {
    set({ loading: true });
    try {
      const scripts = await tauriApi.listScripts();
      set({ scripts, loading: false });
    } catch {
      set({ loading: false });
    }
  },

  openScript: async (name) => {
    // Scripts are stored as files; we need to read them.
    // Since there's no read_script command, we use save_script path to find the file
    // and read it via the backend. For now, load by name.
    const { scripts } = get();
    const info = scripts.find((s) => s.name === name);
    if (!info) return;

    try {
      // Read file content - the backend stores scripts at ~/.serial-cli/scripts/{name}.lua
      const response = await fetch(`file://${info.path}`);
      const content = await response.text();
      set({ currentScript: { name, content }, isDirty: false });
    } catch {
      // Fallback: try to get from backend via execute (not ideal but works for now)
      set({ currentScript: { name, content: "" }, isDirty: false });
    }
  },

  saveScript: async (name, content) => {
    await tauriApi.saveScript(name, content);
    set({ isDirty: false });
    await get().loadScriptList();
  },

  deleteScript: async (name) => {
    await tauriApi.deleteScript(name);
    const { currentScript } = get();
    if (currentScript?.name === name) {
      set({ currentScript: null, isDirty: false });
    }
    await get().loadScriptList();
  },

  executeScript: async (content) => {
    try {
      const result = await tauriApi.executeScript(content);
      set((s) => ({ output: [...s.output, `> ${result}`] }));
    } catch (e) {
      set((s) => ({ output: [...s.output, `Error: ${e}`] }));
    }
  },

  validateScript: async (content) => {
    return await tauriApi.validateScript(content);
  },

  newScript: () => {
    set({ currentScript: { name: "", content: "" }, isDirty: false });
  },

  updateContent: (content) => {
    set((s) => ({
      currentScript: s.currentScript ? { ...s.currentScript, content } : null,
      isDirty: true,
    }));
  },

  clearOutput: () => set({ output: [] }),
}));
