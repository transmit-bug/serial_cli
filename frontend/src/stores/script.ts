import { create } from "zustand";
import { tauriApi } from "@/lib/tauri-api";
import type { Script, UserScriptInfo } from "@/types";

interface ScriptStore {
  // Registered scripts (built-in + custom)
  scripts: Script[];
  scriptsLoading: boolean;

  // User script files
  userScripts: UserScriptInfo[];
  userScriptsLoading: boolean;
  currentScript: { name: string; content: string } | null;
  isDirty: boolean;

  // Active script attached to port
  activeScript: string | null;

  // Registered scripts actions
  loadScripts: () => Promise<void>;
  loadCustomScript: (path: string) => Promise<void>;
  unloadScript: (name: string) => Promise<void>;
  reloadScript: (name: string) => Promise<void>;
  setActiveScript: (portId: string, scriptName: string | null) => Promise<void>;

  // User script files actions
  loadUserScripts: () => Promise<void>;
  openUserScript: (name: string) => Promise<void>;
  saveUserScript: (name: string, content: string) => Promise<void>;
  deleteUserScript: (name: string) => Promise<void>;
  newUserScript: () => void;
  updateContent: (content: string) => void;
}

export const useScriptStore = create<ScriptStore>()((set, get) => ({
  // Registered scripts
  scripts: [],
  scriptsLoading: false,

  // User script files
  userScripts: [],
  userScriptsLoading: false,
  currentScript: null,
  isDirty: false,

  // Active script
  activeScript: null,

  // ─── Registered Scripts Actions ───

  loadScripts: async () => {
    set({ scriptsLoading: true });
    try {
      const scripts = await tauriApi.listScripts();
      set({ scripts, scriptsLoading: false });
    } catch {
      set({ scriptsLoading: false });
    }
  },

  loadCustomScript: async (path) => {
    await tauriApi.loadScript(path);
    await get().loadScripts();
  },

  unloadScript: async (name) => {
    await tauriApi.unloadScript(name);
    set((s) => ({
      scripts: s.scripts.filter((p) => p.name !== name),
      activeScript: s.activeScript === name ? null : s.activeScript,
    }));
  },

  reloadScript: async (name) => {
    await tauriApi.reloadScript(name);
  },

  setActiveScript: async (portId, scriptName) => {
    if (!scriptName) {
      set({ activeScript: null });
      return;
    }
    await tauriApi.bindScript(portId, scriptName);
    set({ activeScript: scriptName });
  },

  // ─── User Script Files Actions ───

  loadUserScripts: async () => {
    set({ userScriptsLoading: true });
    try {
      const userScripts = await tauriApi.listUserScripts();
      set({ userScripts, userScriptsLoading: false });
    } catch {
      set({ userScriptsLoading: false });
    }
  },

  openUserScript: async (name) => {
    const { userScripts } = get();
    const info = userScripts.find((s) => s.name === name);
    if (!info) return;

    try {
      const response = await fetch(`file://${info.path}`);
      const content = await response.text();
      set({ currentScript: { name, content }, isDirty: false });
    } catch {
      set({ currentScript: { name, content: "" }, isDirty: false });
    }
  },

  saveUserScript: async (name, content) => {
    await tauriApi.saveUserScript(name, content);
    set({ isDirty: false });
    await get().loadUserScripts();
  },

  deleteUserScript: async (name) => {
    await tauriApi.deleteUserScript(name);
    const { currentScript } = get();
    if (currentScript?.name === name) {
      set({ currentScript: null, isDirty: false });
    }
    await get().loadUserScripts();
  },

  newUserScript: () => {
    set({ currentScript: { name: "", content: "" }, isDirty: false });
  },

  updateContent: (content) => {
    set((s) => ({
      currentScript: s.currentScript ? { ...s.currentScript, content } : null,
      isDirty: true,
    }));
  },
}));
