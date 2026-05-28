import { Pencil, Trash2 } from "lucide-react";
import { useCallback, useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { useCommandStore } from "@/stores/commands";
import type { QuickCommand } from "@/types";

export interface CommandSenderProps {
  enabled?: boolean;
  sendFn: (data: string, format: "hex" | "ascii") => Promise<void>;
  onSent?: (data: string, format: "hex" | "ascii") => void;
}

export function CommandSender({
  enabled = true,
  sendFn,
  onSent,
}: CommandSenderProps) {
  const { t } = useTranslation();
  const commands = useCommandStore((s) => s.commands);
  const updateCommand = useCommandStore((s) => s.updateCommand);
  const addCommand = useCommandStore((s) => s.addCommand);
  const deleteCommand = useCommandStore((s) => s.deleteCommand);
  const [editingIndex, setEditingIndex] = useState<number | null>(null);
  const [editForm, setEditForm] = useState<QuickCommand>({
    label: "",
    data: "",
    format: "ascii",
  });

  const handleSend = useCallback(
    async (index: number) => {
      if (!enabled) return;
      const cmd = commands[index];
      if (!cmd) return;
      try {
        await sendFn(cmd.data, cmd.format);
        onSent?.(cmd.data, cmd.format);
      } catch {
        // error handled by caller
      }
    },
    [enabled, commands, sendFn, onSent],
  );

  const startEdit = (index: number) => {
    setEditingIndex(index);
    setEditForm({ ...commands[index] });
  };

  const saveEdit = () => {
    if (editingIndex === null) return;
    updateCommand(editingIndex, editForm);
    setEditingIndex(null);
  };

  const handleAddCommand = () => {
    const newCmd: QuickCommand = { label: "New", data: "", format: "ascii" };
    addCommand(newCmd);
    startEdit(commands.length);
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
        const idx = commands.findIndex((c) => c.hotkey?.toUpperCase() === key);
        if (idx >= 0) {
          e.preventDefault();
          handleSend(idx);
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
              disabled={!enabled}
              onClick={() => handleSend(index)}
              title={cmd.data ?? ""}
            >
              <span className="truncate">{cmd.label ?? ""}</span>
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
                className="flex items-center justify-center w-4 h-4 rounded bg-zinc-300 dark:bg-zinc-600 hover:bg-zinc-400 dark:hover:bg-zinc-500"
                onClick={() => startEdit(index)}
                title={t("quickSend.edit")}
              >
                <Pencil size={10} />
              </button>
              <button
                className="flex items-center justify-center w-4 h-4 rounded bg-zinc-300 dark:bg-zinc-600 hover:bg-danger/30 dark:hover:bg-danger/30"
                onClick={() => deleteCommand(index)}
                title={t("quickSend.delete")}
              >
                <Trash2 size={10} />
              </button>
            </div>
          </div>
        ))}
      </div>
      <button
        className="mt-1 w-full rounded px-2 py-1 text-xs text-text-muted hover:bg-zinc-100 hover:text-text dark:hover:bg-zinc-800"
        onClick={handleAddCommand}
      >
        + {t("quickSend.addCommand")}
      </button>
    </div>
  );
}
