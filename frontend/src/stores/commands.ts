import { create } from "zustand";
import { tauriApi } from "@/lib/tauri-api";
import { useDataStore } from "@/stores/data";
import type {
  CommandSequence,
  QuickCommand,
  SequenceExecutionState,
  SequenceStep,
} from "@/types";

const STORAGE_KEY = "serial-cli-quick-commands";
const SEQUENCE_STORAGE_KEY = "serial-cli-command-sequences";

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

function loadSequences(): CommandSequence[] {
  try {
    const raw = localStorage.getItem(SEQUENCE_STORAGE_KEY);
    if (raw) return JSON.parse(raw) as CommandSequence[];
  } catch {
    // ignore
  }
  return [];
}

function persistSequences(seq: CommandSequence[]) {
  localStorage.setItem(SEQUENCE_STORAGE_KEY, JSON.stringify(seq));
}

const defaultExecutionState: SequenceExecutionState = {
  sequenceId: null,
  sequenceName: null,
  stepIndex: -1,
  loopIteration: 0,
  status: "idle",
};

function stepToBytes(step: SequenceStep): number[] {
  if (step.format === "hex") {
    return step.data.split(/\s+/).map((b) => Number.parseInt(b, 16));
  }
  return Array.from(new TextEncoder().encode(step.data));
}

function bytesToAsciiString(data: number[]): string {
  return data
    .map((b) => (b >= 32 && b <= 126 ? String.fromCharCode(b) : ""))
    .join("");
}

function bytesToHexString(data: number[]): string {
  return data
    .map((b) => b.toString(16).padStart(2, "0").toUpperCase())
    .join(" ");
}

interface CommandStore {
  commands: QuickCommand[];

  loadCommands: () => void;
  addCommand: (cmd: QuickCommand) => void;
  updateCommand: (index: number, cmd: QuickCommand) => void;
  deleteCommand: (index: number) => void;
  reorderCommand: (from: number, to: number) => void;
  sendCommand: (
    index: number,
    portId: string,
    addPacket: (portId: string, direction: "rx" | "tx", data: number[], timestamp: number, decoded?: string) => void,
  ) => Promise<void>;

  sequences: CommandSequence[];
  executionState: SequenceExecutionState;

  addSequence: (seq: CommandSequence) => void;
  updateSequence: (id: string, seq: CommandSequence) => void;
  deleteSequence: (id: string) => void;
  executeSequence: (sequenceId: string, portId: string) => Promise<void>;
  stopSequence: () => void;
}

let abortController: AbortController | null = null;

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
    addPacket(portId, "tx", data, Date.now());
  },

  sequences: loadSequences(),
  executionState: defaultExecutionState,

  addSequence: (seq) =>
    set((s) => {
      const updated = [...s.sequences, seq];
      persistSequences(updated);
      return { sequences: updated };
    }),

  updateSequence: (id, seq) =>
    set((s) => {
      const updated = s.sequences.map((s2) => (s2.id === id ? seq : s2));
      persistSequences(updated);
      return { sequences: updated };
    }),

  deleteSequence: (id) =>
    set((s) => {
      const updated = s.sequences.filter((s2) => s2.id !== id);
      persistSequences(updated);
      return { sequences: updated };
    }),

  executeSequence: async (sequenceId, portId) => {
    const sequence = get().sequences.find((s) => s.id === sequenceId);
    if (!sequence || sequence.steps.length === 0) return;

    abortController?.abort();
    abortController = new AbortController();
    const { signal } = abortController;

    const addPacket = useDataStore.getState().addPacket;

    set({
      executionState: {
        sequenceId,
        sequenceName: sequence.name,
        stepIndex: 0,
        loopIteration: 0,
        status: "running",
      },
    });

    try {
      for (let stepIdx = 0; stepIdx < sequence.steps.length; stepIdx++) {
        if (signal.aborted) break;

        const step = sequence.steps[stepIdx];
        if (!step) continue;

        const loopN = step.loopCount ?? 1;
        for (let loopI = 0; loopI < loopN; loopI++) {
          if (signal.aborted) break;

          set({
            executionState: {
              sequenceId,
              sequenceName: sequence.name,
              stepIndex: stepIdx,
              loopIteration: loopN > 1 ? loopI : -1,
              status: "running",
            },
          });

          const bytes = stepToBytes(step);
          await tauriApi.sendData(portId, bytes);
          addPacket(portId, "tx", bytes, Date.now());

          if (step.delay > 0) {
            await new Promise<void>((resolve) => {
              const timer = setTimeout(resolve, step.delay);
              const onAbort = () => {
                clearTimeout(timer);
                resolve();
              };
              signal.addEventListener("abort", onAbort, { once: true });
            });
          }

          if (signal.aborted) break;

          if (step.waitFor && stepIdx < sequence.steps.length - 1) {
            const timeout = step.waitTimeout ?? 5000;
            const startTime = Date.now();
            const snapshotLen = useDataStore.getState().packets.length;
            let found = false;

            while (Date.now() - startTime < timeout && !signal.aborted) {
              const currentPackets = useDataStore.getState().packets;
              for (let p = snapshotLen; p < currentPackets.length; p++) {
                const pkt = currentPackets[p];
                if (pkt && pkt.direction === "rx") {
                  const ascii = bytesToAsciiString(pkt.data);
                  const hex = bytesToHexString(pkt.data);
                  if (
                    ascii.includes(step.waitFor) ||
                    hex.includes(step.waitFor)
                  ) {
                    found = true;
                    break;
                  }
                }
              }
              if (found) break;
              await new Promise<void>((resolve) => {
                const timer = setTimeout(resolve, 50);
                signal.addEventListener(
                  "abort",
                  () => {
                    clearTimeout(timer);
                    resolve();
                  },
                  { once: true },
                );
              });
            }

            if (!found && !signal.aborted) {
              set({
                executionState: {
                  sequenceId,
                  sequenceName: sequence.name,
                  stepIndex: stepIdx,
                  loopIteration: loopI,
                  status: "error",
                  error: `Wait for "${step.waitFor}" timed out after ${timeout}ms at step ${stepIdx + 1}`,
                },
              });
              return;
            }
          }
        }
      }

      if (!signal.aborted) {
        set({
          executionState: {
            sequenceId,
            sequenceName: sequence.name,
            stepIndex: sequence.steps.length - 1,
            loopIteration: -1,
            status: "completed",
          },
        });
      }
    } catch (e) {
      set({
        executionState: {
          sequenceId,
          sequenceName: sequence.name,
          stepIndex: get().executionState.stepIndex,
          loopIteration: -1,
          status: "error",
          error: String(e),
        },
      });
    }
  },

  stopSequence: () => {
    abortController?.abort();
    abortController = null;
    set({
      executionState: {
        sequenceId: null,
        sequenceName: null,
        stepIndex: -1,
        loopIteration: 0,
        status: "idle",
      },
    });
  },
}));
