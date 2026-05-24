import { useCallback, useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { useCommandStore } from "@/stores/commands";
import { useConnectionStore } from "@/stores/connection";
import { useDataStore } from "@/stores/data";
import type { QuickCommand } from "@/types";

export function RightPanelCommands() {
  const { t } = useTranslation();
  const { portId, status } = useConnectionStore();
  const addPacket = useDataStore((s) => s.addPacket);
  const commands = useCommandStore((s) => s.commands);
  const updateCommand = useCommandStore((s) => s.updateCommand);
  const addCommand = useCommandStore((s) => s.addCommand);
  const deleteCommand = useCommandStore((s) => s.deleteCommand);
  const reorderCommand = useCommandStore((s) => s.reorderCommand);
  const sendCommand = useCommandStore((s) => s.sendCommand);

  const isConnected = status === "connected";
  const [editingIndex, setEditingIndex] = useState<number | null>(null);
  const [editForm, setEditForm] = useState<QuickCommand>({
    label: "",
    data: "",
    format: "ascii",
  });
  const [searchQuery, setSearchQuery] = useState("");

  const handleSend = useCallback(
    async (index: number) => {
      if (!portId || !isConnected) return;
      try {
        await sendCommand(index, portId, addPacket);
      } catch {
        // error handled by caller
      }
    },
    [portId, isConnected, sendCommand, addPacket],
  );

  const startEdit = (index: number) => {
    const cmd = commands[index];
    if (!cmd) return;
    setEditingIndex(index);
    setEditForm({ ...cmd });
  };

  const saveEdit = () => {
    if (editingIndex === null) return;
    updateCommand(editingIndex, editForm);
    setEditingIndex(null);
  };

  const handleAddCommand = () => {
    const newCmd: QuickCommand = {
      label: t("quickSend.newCommand"),
      data: "",
      format: "ascii",
    };
    addCommand(newCmd);
    startEdit(commands.length);
  };

  const moveUp = useCallback(
    (index: number) => {
      if (index > 0) reorderCommand(index, index - 1);
    },
    [reorderCommand],
  );

  const moveDown = useCallback(
    (index: number) => {
      if (index < commands.length - 1) reorderCommand(index, index + 1);
    },
    [reorderCommand, commands.length],
  );

  const filtered = commands.filter((c) => {
    const label = c.label ?? "";
    const data = c.data ?? "";
    return (
      label.toLowerCase().includes(searchQuery.toLowerCase()) ||
      data.toLowerCase().includes(searchQuery.toLowerCase())
    );
  });

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

  return (
    <div className="flex flex-col h-full">
      {/* Search + Add */}
      <div className="flex items-center gap-2 p-2 border-b border-border">
        <input
          className="flex-1 h-6 text-xs rounded border border-border bg-transparent px-2"
          placeholder={t("commands.search")}
          value={searchQuery}
          onChange={(e) => setSearchQuery(e.target.value)}
        />
        <button
          className="px-2 py-0.5 rounded text-xs bg-accent/20 text-accent hover:bg-accent/30"
          onClick={handleAddCommand}
        >
          +
        </button>
      </div>

      {/* Edit form */}
      {editingIndex !== null && (
        <div className="p-2 border-b border-border space-y-1.5">
          <div className="text-xs font-medium text-text-secondary">
            {t("quickSend.editCommand")}
          </div>
          <input
            className="w-full h-6 text-xs rounded border border-border bg-transparent px-2"
            placeholder={t("quickSend.label")}
            value={editForm.label}
            onChange={(e) =>
              setEditForm({ ...editForm, label: e.target.value })
            }
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
              className="px-2 py-0.5 text-xs rounded bg-accent/20 text-accent hover:bg-accent/30"
              onClick={saveEdit}
            >
              {t("common.save")}
            </button>
            <button
              className="px-2 py-0.5 text-xs rounded text-text-muted hover:text-text"
              onClick={() => setEditingIndex(null)}
            >
              {t("common.cancel")}
            </button>
          </div>
        </div>
      )}

      {/* Command list */}
      <div className="flex-1 overflow-y-auto">
        {filtered.length === 0 ? (
          <div className="p-4 text-center text-xs text-text-muted">
            {searchQuery ? t("commands.noMatch") : t("commands.emptyState")}
          </div>
        ) : (
          <div className="divide-y divide-border/50">
            {filtered.map((cmd) => {
              const realIndex = commands.indexOf(cmd);
              return (
                <div
                  key={realIndex}
                  className="flex items-center gap-1 px-2 py-1.5 hover:bg-surface/30 group"
                >
                  {/* Reorder arrows */}
                  <div className="flex flex-col gap-0.5 mr-1 opacity-0 group-hover:opacity-100 transition-opacity">
                    <button
                      className="text-[8px] text-text-muted leading-none"
                      onClick={() => moveUp(realIndex)}
                      disabled={realIndex === 0}
                    >
                      ▲
                    </button>
                    <button
                      className="text-[8px] text-text-muted leading-none"
                      onClick={() => moveDown(realIndex)}
                      disabled={realIndex === commands.length - 1}
                    >
                      ▼
                    </button>
                  </div>

                  {/* Send button / label */}
                  <button
                    className="flex-1 flex items-center justify-between text-left min-w-0"
                    disabled={!isConnected}
                    onClick={() => handleSend(realIndex)}
                    title={cmd.data ?? ""}
                  >
                    <span className="text-xs font-mono truncate">
                      {cmd.label ?? ""}
                    </span>
                    <div className="flex items-center gap-1 ml-2 shrink-0">
                      {cmd.hotkey && (
                        <span className="text-[9px] text-text-muted bg-surface rounded px-1">
                          {cmd.hotkey}
                        </span>
                      )}
                      <span className="text-[9px] text-text-muted uppercase">
                        {cmd.format}
                      </span>
                    </div>
                  </button>

                  {/* Edit / Delete */}
                  <div className="hidden group-hover:flex items-center gap-0.5 ml-1 shrink-0">
                    <button
                      className="text-[10px] text-text-muted hover:text-text px-1"
                      onClick={() => startEdit(realIndex)}
                    >
                      {t("common.edit")}
                    </button>
                    <button
                      className="text-[10px] text-danger hover:text-danger-hover px-1"
                      onClick={() => deleteCommand(realIndex)}
                    >
                      {t("common.delete")}
                    </button>
                  </div>
                </div>
              );
            })}
          </div>
        )}
      </div>
    </div>
  );
}
