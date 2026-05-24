import { create } from "zustand";
import { tauriApi } from "@/lib/tauri-api";
import type { QuickCommand } from "@/types";

const STORAGE_KEY = "serial-cli-quick-commands";

const DEFAULT_COMMANDS: QuickCommand[] = [
  { label: "AT", data: "AT\r\n", format: "ascii", hotkey: "F1" },
  { label: "ATE0", data: "ATE0\r\n", format: "ascii", hotkey: "F2" },
  { label: "AT+GMR", data: "AT+GMR\r\n", format: "ascii", hotkey: "F3" },
  { label: "AT+CSQ", data: "AT+CSQ\r\n", format: "ascii", hotkey: "F4" },
  {
    label: "7E 03 01 02 03 7F",
    data: "7E 03 01 02 03 7F",
    format: "hex",
    hotkey: "F5",
  },
  {
    label: "01 03 00 00 00 01 84 0A",
    data: "01 03 00 00 00 01 84 0A",
    format: "hex",
    hotkey: "F6",
  },
  { label: "Hello", data: "Hello\r\n", format: "ascii", hotkey: "F7" },
  { label: "RESET", data: "AT+RST\r\n", format: "ascii", hotkey: "F8" },
];

function loadCommands(): QuickCommand[] {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (raw) {
      const parsed = JSON.parse(raw) as QuickCommand[];
      return parsed.length > 0 ? parsed : DEFAULT_COMMANDS;
    }
  } catch {
    // ignore
  }
  return DEFAULT_COMMANDS;
}

function persistCommands(cmds: QuickCommand[]) {
  localStorage.setItem(STORAGE_KEY, JSON.stringify(cmds));
}

interface CommandStore {
  commands: QuickCommand[];

  /** Load commands from localStorage */
  loadCommands: () => void;
  /** Add a new command */
  addCommand: (cmd: QuickCommand) => void;
  /** Update a command by index */
  updateCommand: (index: number, cmd: QuickCommand) => void;
  /** Delete a command by index */
  deleteCommand: (index: number) => void;
  /** Reorder: move command from `from` to `to` */
  reorderCommand: (from: number, to: number) => void;
  /** Send a command to the given port */
  sendCommand: (
    index: number,
    portId: string,
    addPacket: (direction: "tx", data: number[], ts: number) => void,
  ) => Promise<void>;
}

export const useCommandStore = create<CommandStore>()((set, get) => ({
  commands: loadCommands(),

  loadCommands: () => set({ commands: loadCommands() }),

  addCommand: (cmd) =>
    set((s) => {
      const updated = [...s.commands, cmd];
      persistCommands(updated);
      return { commands: updated };
    }),

  updateCommand: (index, cmd) =>
    set((s) => {
      const updated = [...s.commands];
      updated[index] = cmd;
      persistCommands(updated);
      return { commands: updated };
    }),

  deleteCommand: (index) =>
    set((s) => {
      const updated = s.commands.filter((_, i) => i !== index);
      persistCommands(updated);
      return { commands: updated };
    }),

  reorderCommand: (from, to) =>
    set((s) => {
      const updated = [...s.commands];
      const [item] = updated.splice(from, 1);
      updated.splice(to, 0, item);
      persistCommands(updated);
      return { commands: updated };
    }),

  sendCommand: async (index, portId, addPacket) => {
    const cmd = get().commands[index];
    if (!cmd) return;
    const data =
      cmd.format === "hex"
        ? cmd.data.split(/\s+/).map((b) => Number.parseInt(b, 16))
        : Array.from(new TextEncoder().encode(cmd.data));
    await tauriApi.sendData(portId, data);
    addPacket("tx", data, Date.now());
  },
}));
