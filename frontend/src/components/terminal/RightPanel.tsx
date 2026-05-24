import { useCallback, useState } from "react";
import { useTranslation } from "react-i18next";
import { toast } from "sonner";
import { tauriApi } from "@/lib/tauri-api";
import { formatBytes } from "@/lib/utils";
import { useConnectionStore } from "@/stores/connection";
import { useDataStore } from "@/stores/data";
import { useSerialScriptStore } from "@/stores/serialScript";

interface QuickCommand {
  label: string;
  data: string;
  format: "hex" | "ascii";
}

const STORAGE_KEY = "serial-cli-quick-commands";

function loadQuickCommands(): QuickCommand[] {
  try {
    const stored = localStorage.getItem(STORAGE_KEY);
    return stored ? JSON.parse(stored) : [];
  } catch {
    return [];
  }
}

function saveQuickCommands(cmds: QuickCommand[]) {
  localStorage.setItem(STORAGE_KEY, JSON.stringify(cmds));
}

export function RightPanel() {
  const { t } = useTranslation();
  const portId = useConnectionStore((s) => s.portId);
  const isConnected = useConnectionStore((s) => s.status === "connected");
  const packets = useDataStore((s) => s.packets);
  const actions = useSerialScriptStore((s) => s.actions);
  const callAction = useSerialScriptStore((s) => s.callAction);

  const [quickCommands, setQuickCommands] =
    useState<QuickCommand[]>(loadQuickCommands);
  const [editingIdx, setEditingIdx] = useState<number | null>(null);
  const [editLabel, setEditLabel] = useState("");
  const [editData, setEditData] = useState("");
  const [editFormat, setEditFormat] = useState<"hex" | "ascii">("ascii");

  // Stats
  const rxBytes = packets.reduce(
    (sum, p) => (p.direction === "rx" ? sum + p.data.length : sum),
    0,
  );
  const txBytes = packets.reduce(
    (sum, p) => (p.direction === "tx" ? sum + p.data.length : sum),
    0,
  );
  const rxPkts = packets.filter((p) => p.direction === "rx").length;
  const txPkts = packets.filter((p) => p.direction === "tx").length;

  const handleCallAction = useCallback(
    async (fn: string) => {
      if (!portId) return;
      try {
        const result = await callAction(portId, fn);
        toast.success(result);
      } catch (e) {
        toast.error(String(e));
      }
    },
    [portId, callAction],
  );

  const handleQuickSend = useCallback(
    async (cmd: QuickCommand) => {
      if (!portId) return;
      try {
        const data =
          cmd.format === "hex"
            ? cmd.data.split(/\s+/).map((b) => Number.parseInt(b, 16))
            : Array.from(new TextEncoder().encode(cmd.data));
        await tauriApi.sendData(portId, data);
      } catch (e) {
        toast.error(String(e));
      }
    },
    [portId],
  );

  const saveCommand = useCallback(() => {
    if (!editLabel.trim() || !editData.trim()) return;
    const cmd: QuickCommand = {
      label: editLabel,
      data: editData,
      format: editFormat,
    };
    const updated = [...quickCommands];
    if (editingIdx !== null) {
      updated[editingIdx] = cmd;
    } else {
      updated.push(cmd);
    }
    setQuickCommands(updated);
    saveQuickCommands(updated);
    setEditingIdx(null);
    setEditLabel("");
    setEditData("");
  }, [editLabel, editData, editFormat, editingIdx, quickCommands]);

  const deleteCommand = useCallback(
    (idx: number) => {
      const updated = quickCommands.filter((_, i) => i !== idx);
      setQuickCommands(updated);
      saveQuickCommands(updated);
    },
    [quickCommands],
  );

  return (
    <div className="flex flex-col h-full overflow-y-auto">
      {/* Stats */}
      <section className="p-3 border-b border-border">
        <h3 className="text-xs font-medium text-text-secondary mb-2">
          {t("terminal.stats")}
        </h3>
        <div className="grid grid-cols-2 gap-1 text-xs">
          <span className="text-text-muted">RX:</span>
          <span>
            {formatBytes(rxBytes)} ({rxPkts})
          </span>
          <span className="text-text-muted">TX:</span>
          <span>
            {formatBytes(txBytes)} ({txPkts})
          </span>
        </div>
      </section>

      {/* Script Actions */}
      {isConnected && actions.length > 0 && (
        <section className="p-3 border-b border-border">
          <h3 className="text-xs font-medium text-text-secondary mb-2">
            {t("terminal.actions")}
          </h3>
          <div className="flex flex-col gap-1">
            {actions.map((action) => (
              <button
                key={action.function_name}
                onClick={() => handleCallAction(action.function_name)}
                className="px-2 py-1 rounded text-xs bg-surface hover:bg-surface-hover text-text text-left"
              >
                {action.label}
              </button>
            ))}
          </div>
        </section>
      )}

      {/* Quick Commands */}
      <section className="p-3 flex-1">
        <div className="flex items-center justify-between mb-2">
          <h3 className="text-xs font-medium text-text-secondary">
            {t("terminal.quickCommands")}
          </h3>
          <button
            onClick={() => {
              setEditingIdx(null);
              setEditLabel("");
              setEditData("");
              setEditFormat("ascii");
            }}
            className="text-xs text-accent hover:text-accent-hover"
          >
            + {t("terminal.addQuickCommand")}
          </button>
        </div>

        {/* Edit form */}
        <div className="mb-2 p-2 bg-base rounded border border-border space-y-1">
          <input
            value={editLabel}
            onChange={(e) => setEditLabel(e.target.value)}
            placeholder={t("terminal.commandLabel")}
            className="w-full h-6 text-xs"
          />
          <input
            value={editData}
            onChange={(e) => setEditData(e.target.value)}
            placeholder={t("terminal.commandData")}
            className="w-full h-6 text-xs font-mono"
          />
          <div className="flex gap-1">
            <button
              onClick={() => setEditFormat("ascii")}
              className={`px-2 py-0.5 text-xs rounded ${editFormat === "ascii" ? "bg-accent/20 text-accent" : "text-text-muted"}`}
            >
              ASCII
            </button>
            <button
              onClick={() => setEditFormat("hex")}
              className={`px-2 py-0.5 text-xs rounded ${editFormat === "hex" ? "bg-accent/20 text-accent" : "text-text-muted"}`}
            >
              HEX
            </button>
            <button
              onClick={saveCommand}
              className="ml-auto px-2 py-0.5 text-xs bg-accent/20 text-accent rounded"
            >
              {t("common.save")}
            </button>
          </div>
        </div>

        {/* Command buttons */}
        <div className="flex flex-col gap-1">
          {quickCommands.map((cmd, idx) => (
            <div key={idx} className="flex items-center gap-1">
              <button
                onClick={() => handleQuickSend(cmd)}
                disabled={!isConnected}
                className="flex-1 px-2 py-1 rounded text-xs bg-surface hover:bg-surface-hover text-left disabled:opacity-50"
              >
                {cmd.label}
              </button>
              <button
                onClick={() => {
                  setEditingIdx(idx);
                  setEditLabel(cmd.label);
                  setEditData(cmd.data);
                  setEditFormat(cmd.format);
                }}
                className="text-text-muted hover:text-text"
              >
                <span className="text-xs">✎</span>
              </button>
              <button
                onClick={() => deleteCommand(idx)}
                className="text-text-muted hover:text-danger"
              >
                <span className="text-xs">✕</span>
              </button>
            </div>
          ))}
        </div>
      </section>
    </div>
  );
}
