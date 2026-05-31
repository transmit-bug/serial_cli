import { Check, Copy, Plus, X } from "lucide-react";
import { useCallback, useState } from "react";
import { useTranslation } from "react-i18next";
import { toast } from "sonner";
import { formatBytes } from "@/lib/utils";
import { useVirtualPortStore } from "@/stores/virtualPort";
import type { VirtualPortInfo } from "@/types";

const AVAILABLE_BACKENDS = [
  { value: "pty", label: "PTY" },
  { value: "socat", label: "socat" },
  { value: "namedpipe", label: "Named Pipe" },
];

export function VirtualPortList() {
  const { t } = useTranslation();
  const {
    ports,
    selectedPort,
    throughputMap,
    healthMap,
    createFormOpen,
    stopPort,
    setSelectedPort,
    setCreateFormOpen,
  } = useVirtualPortStore();

  return (
    <div className="flex flex-col h-full">
      {/* Header */}
      <div className="flex items-center justify-between px-3 py-2 border-b border-border">
        <h2 className="text-xs font-semibold">{t("virtual.title")}</h2>
        <button
          onClick={() => setCreateFormOpen(!createFormOpen)}
          className="p-1 rounded text-text-muted hover:text-text hover:bg-surface"
        >
          <Plus size={14} />
        </button>
      </div>

      {/* Create form */}
      {createFormOpen && <CreateForm />}

      {/* Port list */}
      <div className="flex-1 overflow-y-auto">
        {ports.length === 0 && !createFormOpen ? (
          <EmptyState />
        ) : (
          <div className="p-2 space-y-2">
            {ports.map((port) => (
              <PortCard
                key={port.id}
                port={port}
                selected={selectedPort === port.id}
                throughput={throughputMap[port.id] ?? 0}
                healthy={healthMap[port.id]}
                onSelect={() => setSelectedPort(port.id)}
                onStop={() => stopPort(port.id)}
              />
            ))}
          </div>
        )}
      </div>
    </div>
  );
}

function CreateForm() {
  const { t } = useTranslation();
  const { createPort, setCreateFormOpen } = useVirtualPortStore();
  const [backend, setBackend] = useState("pty");
  const [monitor, setMonitor] = useState(true);
  const [bufferSize, setBufferSize] = useState(8192);

  const handleCreate = useCallback(async () => {
    try {
      await createPort({ backend, monitor, buffer_size: bufferSize });
      toast.success(t("virtual.createSuccess"));
      setCreateFormOpen(false);
    } catch (e) {
      toast.error(String(e));
    }
  }, [backend, monitor, bufferSize, createPort, setCreateFormOpen, t]);

  return (
    <div className="p-3 border-b border-border bg-surface/50 space-y-2">
      <div className="flex items-center justify-between">
        <span className="text-xs font-medium">{t("virtual.createPair")}</span>
        <button
          onClick={() => setCreateFormOpen(false)}
          className="p-0.5 rounded text-text-muted hover:text-text"
        >
          <X size={12} />
        </button>
      </div>

      {/* Backend */}
      <div className="space-y-1">
        <label className="text-[10px] text-text-muted">
          {t("virtual.backend")}
        </label>
        <select
          value={backend}
          onChange={(e) => setBackend(e.target.value)}
          className="w-full h-6 text-xs rounded border border-border bg-transparent px-2"
        >
          {AVAILABLE_BACKENDS.map((b) => (
            <option key={b.value} value={b.value}>
              {b.label}
            </option>
          ))}
        </select>
      </div>

      {/* Monitor toggle */}
      <label className="flex items-center gap-2 text-xs">
        <input
          type="checkbox"
          checked={monitor}
          onChange={(e) => setMonitor(e.target.checked)}
          className="rounded border-border"
        />
        {t("virtual.enableMonitor")}
      </label>

      {/* Buffer size */}
      <div className="space-y-1">
        <label className="text-[10px] text-text-muted">
          {t("virtual.bufferSize")}
        </label>
        <input
          type="number"
          value={bufferSize}
          onChange={(e) => setBufferSize(Number(e.target.value) || 8192)}
          className="w-full h-6 text-xs rounded border border-border bg-transparent px-2 font-mono"
          min={512}
          step={1024}
        />
      </div>

      <button
        onClick={handleCreate}
        className="w-full px-3 py-1.5 rounded text-xs bg-accent/20 text-accent hover:bg-accent/30"
      >
        {t("virtual.createPair")}
      </button>
    </div>
  );
}

function PortCard({
  port,
  selected,
  throughput,
  healthy,
  onSelect,
  onStop,
}: {
  port: VirtualPortInfo;
  selected: boolean;
  throughput: number;
  healthy?: boolean;
  onSelect: () => void;
  onStop: () => void;
}) {
  const { t } = useTranslation();
  const [copied, setCopied] = useState<string | null>(null);

  const copyPath = useCallback((path: string, label: string) => {
    navigator.clipboard.writeText(path);
    setCopied(label);
    setTimeout(() => setCopied(null), 1500);
  }, []);

  return (
    // biome-ignore lint/a11y/noStaticElementInteractions: Card is a selection surface; inner buttons use stopPropagation
    // biome-ignore lint/a11y/useKeyWithClickEvents: Card is mouse-click selection; keyboard nav handled by inner focusable controls
    <div
      className={`w-full text-left p-2.5 rounded border cursor-pointer transition-colors ${
        selected
          ? "bg-accent/10 border-accent/30"
          : "bg-base-deep border-border hover:border-accent/20"
      }`}
      onClick={onSelect}
    >
      {/* Header row */}
      <div className="flex items-center justify-between mb-1.5">
        <div className="flex items-center gap-1.5 min-w-0">
          <span
            className={`w-1.5 h-1.5 rounded-full flex-shrink-0 ${
              port.running
                ? healthy === false
                  ? "bg-danger"
                  : "bg-success"
                : "bg-text-muted"
            }`}
          />
          <span className="text-xs font-medium truncate">
            {port.id.slice(0, 8)}
          </span>
          <span className="text-[10px] text-text-muted flex-shrink-0">
            {port.backend}
          </span>
        </div>
        <div className="flex items-center gap-1 flex-shrink-0">
          {throughput > 0 && (
            <span className="text-[10px] text-accent font-mono">
              {formatBytes(throughput)}/s
            </span>
          )}
          <button
            type="button"
            onClick={(e) => {
              e.stopPropagation();
              onStop();
            }}
            className="p-0.5 rounded text-text-muted hover:text-danger hover:bg-danger/10"
            title={t("common.stop")}
          >
            <X size={12} />
          </button>
        </div>
      </div>

      {/* Port paths */}
      <div className="space-y-0.5 text-[10px]">
        <div className="flex items-center gap-1">
          <span className="text-text-muted w-3">A:</span>
          <span className="font-mono truncate flex-1" title={port.port_a}>
            {port.port_a}
          </span>
          <button
            type="button"
            onClick={(e) => {
              e.stopPropagation();
              copyPath(port.port_a, "a");
            }}
            className="p-0.5 rounded text-text-muted hover:text-text flex-shrink-0"
            title={t("virtual.copyPath")}
          >
            {copied === "a" ? <Check size={10} /> : <Copy size={10} />}
          </button>
        </div>
        <div className="flex items-center gap-1">
          <span className="text-text-muted w-3">B:</span>
          <span className="font-mono truncate flex-1" title={port.port_b}>
            {port.port_b}
          </span>
          <button
            type="button"
            onClick={(e) => {
              e.stopPropagation();
              copyPath(port.port_b, "b");
            }}
            className="p-0.5 rounded text-text-muted hover:text-text flex-shrink-0"
            title={t("virtual.copyPath")}
          >
            {copied === "b" ? <Check size={10} /> : <Copy size={10} />}
          </button>
        </div>
      </div>
    </div>
  );
}

function EmptyState() {
  const { t } = useTranslation();
  const { setCreateFormOpen } = useVirtualPortStore();

  return (
    <div className="flex items-center justify-center flex-1 p-6">
      <div className="text-center space-y-3">
        <div className="text-text-muted text-xs leading-relaxed">
          {t("virtual.emptyState")}
        </div>
        <div className="text-[10px] text-text-muted max-w-[200px] mx-auto">
          {t("virtual.emptyStateDesc")}
        </div>
        {/* Simple bridge diagram */}
        <div className="text-[10px] font-mono text-text-muted py-1">
          {t("virtual.bridgeDiagram")}
        </div>
        <button
          onClick={() => setCreateFormOpen(true)}
          className="px-3 py-1.5 rounded text-xs bg-accent/20 text-accent hover:bg-accent/30"
        >
          + {t("virtual.createPair")}
        </button>
      </div>
    </div>
  );
}
