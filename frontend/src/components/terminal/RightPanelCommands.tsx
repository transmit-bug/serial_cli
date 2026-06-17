import { useCallback, useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { useCommandStore } from "@/stores/commands";
import { useConnectionStore } from "@/stores/connection";
import { useDataStore } from "@/stores/data";
import type { QuickCommand } from "@/types";

export function RightPanelCommands() {
  const { t } = useTranslation();
  const activePortId = useConnectionStore((s) => s.activePortId);
  const connections = useConnectionStore((s) => s.connections);
  const addPacket = useDataStore((s) => s.addPacket);
  const commands = useCommandStore((s) => s.commands);
  const updateCommand = useCommandStore((s) => s.updateCommand);
  const addCommand = useCommandStore((s) => s.addCommand);
  const deleteCommand = useCommandStore((s) => s.deleteCommand);
  const reorderCommand = useCommandStore((s) => s.reorderCommand);
  const sendCommand = useCommandStore((s) => s.sendCommand);

  const portId = activePortId;
  const isConnected =
    !!activePortId &&
    connections.find((c) => c.portId === activePortId)?.status === "connected";
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
        <Input
          placeholder={t("commands.search")}
          value={searchQuery}
          onChange={(e) => setSearchQuery(e.target.value)}
          className="flex-1"
        />
        <Button variant="default" size="sm" onClick={handleAddCommand}>
          +
        </Button>
      </div>

      {/* Edit form */}
      {editingIndex !== null && (
        <div className="p-2 border-b border-border space-y-1.5">
          <div className="text-xs font-medium text-text-secondary">
            {t("quickSend.editCommand")}
          </div>
          <Input
            placeholder={t("quickSend.label")}
            value={editForm.label}
            onChange={(e) =>
              setEditForm({ ...editForm, label: e.target.value })
            }
          />
          <Input
            placeholder={t("quickSend.data")}
            value={editForm.data}
            onChange={(e) =>
              setEditForm({ ...editForm, data: e.target.value })
            }
            className="font-mono"
          />
          <div className="flex gap-2">
            <Select
              value={editForm.format}
              onValueChange={(value) =>
                setEditForm({
                  ...editForm,
                  format: value as "hex" | "ascii",
                })
              }
            >
              <SelectTrigger className="w-24">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="ascii">ASCII</SelectItem>
                <SelectItem value="hex">HEX</SelectItem>
              </SelectContent>
            </Select>
            <Input
              placeholder="F1"
              value={editForm.hotkey || ""}
              onChange={(e) =>
                setEditForm({ ...editForm, hotkey: e.target.value })
              }
              className="w-16"
            />
          </div>
          <div className="flex gap-2">
            <Button variant="default" size="sm" onClick={saveEdit}>
              {t("common.save")}
            </Button>
            <Button
              variant="ghost"
              size="sm"
              onClick={() => setEditingIndex(null)}
            >
              {t("common.cancel")}
            </Button>
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
                    <Button
                      variant="ghost"
                      size="icon-xs"
                      onClick={() => moveUp(realIndex)}
                      disabled={realIndex === 0}
                    >
                      ▲
                    </Button>
                    <Button
                      variant="ghost"
                      size="icon-xs"
                      onClick={() => moveDown(realIndex)}
                      disabled={realIndex === commands.length - 1}
                    >
                      ▼
                    </Button>
                  </div>

                  {/* Send button / label */}
                  <Button
                    variant="ghost"
                    className="flex-1 flex items-center justify-between text-left min-w-0 h-auto py-1"
                    disabled={!isConnected}
                    onClick={() => handleSend(realIndex)}
                    title={cmd.data ?? ""}
                  >
                    <span className="text-xs font-mono truncate">
                      {cmd.label ?? ""}
                    </span>
                    <div className="flex items-center gap-1 ml-2 shrink-0">
                      {cmd.hotkey && (
                        <Badge variant="secondary" className="text-[9px]">
                          {cmd.hotkey}
                        </Badge>
                      )}
                      <Badge variant="outline" className="text-[9px] uppercase">
                        {cmd.format}
                      </Badge>
                    </div>
                  </Button>

                  {/* Edit / Delete */}
                  <div className="hidden group-hover:flex items-center gap-0.5 ml-1 shrink-0">
                    <Button
                      variant="ghost"
                      size="sm"
                      onClick={() => startEdit(realIndex)}
                    >
                      {t("common.edit")}
                    </Button>
                    <Button
                      variant="ghost"
                      size="sm"
                      onClick={() => deleteCommand(realIndex)}
                    >
                      {t("common.delete")}
                    </Button>
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
