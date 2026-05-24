import { useCallback, useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { tauriApi } from "@/lib/tauri-api";
import { useConnectionStore } from "@/stores/connection";
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

interface QuickSendPanelProps {
  onSent?: (data: string, format: "hex" | "ascii") => void;
}

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

function saveCommands(cmds: QuickCommand[]) {
  localStorage.setItem(STORAGE_KEY, JSON.stringify(cmds));
}

export function QuickSendPanel({ onSent }: QuickSendPanelProps) {
  const { t } = useTranslation();
  const { portId, status } = useConnectionStore();
  const [commands, setCommands] = useState<QuickCommand[]>(loadCommands);
  const [editingIndex, setEditingIndex] = useState<number | null>(null);
  const [editForm, setEditForm] = useState<QuickCommand>({
    label: "",
    data: "",
    format: "ascii",
  });

  const isConnected = status === "connected";

  const handleSend = useCallback(
    async (cmd: QuickCommand) => {
      if (!portId || !isConnected) return;
      try {
        const data =
          cmd.format === "hex"
            ? cmd.data.split(/\s+/).map((b) => Number.parseInt(b, 16))
            : Array.from(new TextEncoder().encode(cmd.data));
        await tauriApi.sendData(portId, data);
        onSent?.(cmd.data, cmd.format);
      } catch {
        // error handled by caller
      }
    },
    [portId, isConnected, onSent],
  );

  const startEdit = (index: number) => {
    setEditingIndex(index);
    setEditForm({ ...commands[index] });
  };

  const saveEdit = () => {
    if (editingIndex === null) return;
    const updated = [...commands];
    updated[editingIndex] = editForm;
    setCommands(updated);
    saveCommands(updated);
    setEditingIndex(null);
  };

  const addCommand = () => {
    const newCmd: QuickCommand = { label: "New", data: "", format: "ascii" };
    const updated = [...commands, newCmd];
    setCommands(updated);
    saveCommands(updated);
    startEdit(commands.length);
  };

  const deleteCommand = (index: number) => {
    const updated = commands.filter((_, i) => i !== index);
    setCommands(updated);
    saveCommands(updated);
    if (editingIndex === index) setEditingIndex(null);
  };

  // Hotkey listener
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      const target = e.target as HTMLElement;
      if (
        target.tagName === "INPUT" ||
        target.tagName === "TEXTAREA" ||
        target.isContentEditable
      )
        return;

      const key = e.key.toUpperCase();
      if (key.startsWith("F") && key.length <= 3) {
        const cmd = commands.find((c) => c.hotkey?.toUpperCase() === key);
        if (cmd) {
          e.preventDefault();
          handleSend(cmd);
        }
      }
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [commands, handleSend]);

  if (editingIndex !== null) {
    return (
      <div className="flex flex-col gap-2 p-2">
        <div className="text-xs font-medium text-zinc-500">
          {t("quickSend.editCommand")}
        </div>
        <input
          className="w-full h-6 text-xs rounded border border-border bg-transparent px-2"
          placeholder={t("quickSend.label")}
          value={editForm.label}
          onChange={(e) => setEditForm({ ...editForm, label: e.target.value })}
        />
        <input
          className="w-full h-6 text-xs rounded border border-border bg-transparent px-2 font-mono"
          placeholder={t("quickSend.data")}
          value={editForm.data}
          onChange={(e) => setEditForm({ ...editForm, data: e.target.value })}
        />
        <div className="flex gap-2">
          <select
            className="h-6 text-xs rounded border border-border bg-transparent px-2"
            value={editForm.format}
            onChange={(e) =>
              setEditForm({
                ...editForm,
                format: e.target.value as "hex" | "ascii",
              })
            }
          >
            <option value="ascii">ASCII</option>
            <option value="hex">HEX</option>
          </select>
          <input
            className="w-16 h-6 text-xs rounded border border-border bg-transparent px-2"
            placeholder="F1"
            value={editForm.hotkey || ""}
            onChange={(e) =>
              setEditForm({ ...editForm, hotkey: e.target.value })
            }
          />
        </div>
        <div className="flex gap-2">
          <button
            className="px-2 py-1 text-xs rounded bg-accent/20 text-accent hover:bg-accent/30"
            onClick={saveEdit}
          >
            {t("common.save")}
          </button>
          <button
            className="px-2 py-1 text-xs rounded text-text-muted hover:text-text"
            onClick={() => setEditingIndex(null)}
          >
            {t("common.cancel")}
          </button>
        </div>
      </div>
    );
  }

  return (
    <div className="flex flex-col gap-1 p-1">
      <div className="grid grid-cols-2 gap-1">
        {commands.map((cmd, index) => (
          <div key={index} className="group relative flex flex-col gap-0.5">
            <button
              className="flex items-center justify-between rounded-md bg-zinc-100 px-2 py-1.5 text-xs font-mono transition hover:bg-zinc-200 disabled:opacity-40 dark:bg-zinc-800 dark:hover:bg-zinc-700"
              disabled={!isConnected}
              onClick={() => handleSend(cmd)}
              title={cmd.data}
            >
              <span className="truncate">{cmd.label}</span>
              <span className="ml-1 shrink-0 rounded bg-zinc-200 px-1 py-0.5 text-[10px] text-zinc-500 dark:bg-zinc-700">
                {cmd.format}
              </span>
            </button>
            {cmd.hotkey && (
              <span className="absolute right-1 top-1 text-[9px] text-zinc-400 opacity-0 group-hover:opacity-100">
                {cmd.hotkey}
              </span>
            )}
            <div className="absolute -right-1 -top-1 hidden gap-0.5 group-hover:flex">
              <button
                className="rounded bg-zinc-300 px-1 text-[9px] dark:bg-zinc-600"
                onClick={() => startEdit(index)}
              >
                ✎
              </button>
              <button
                className="rounded bg-zinc-300 px-1 text-[9px] dark:bg-zinc-600"
                onClick={() => deleteCommand(index)}
              >
                ×
              </button>
            </div>
          </div>
        ))}
      </div>
      <button
        className="mt-1 w-full rounded px-2 py-1 text-xs text-text-muted hover:bg-zinc-100 hover:text-text dark:hover:bg-zinc-800"
        onClick={addCommand}
      >
        + {t("quickSend.addCommand")}
      </button>
    </div>
  );
}
