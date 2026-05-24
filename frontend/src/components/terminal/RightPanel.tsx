import { useCallback, useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { toast } from "sonner";
import { formatBytes, formatDuration } from "@/lib/utils";
import { useConnectionStore } from "@/stores/connection";
import { useDataStore } from "@/stores/data";
import { useSerialScriptStore } from "@/stores/serialScript";

function useConnectionDuration(): string {
  const connectedAt = useConnectionStore((s) => s.connectedAt);
  const [, setTick] = useState(0);
  useEffect(() => {
    const timer = setInterval(() => setTick((t) => t + 1), 1000);
    return () => clearInterval(timer);
  }, []);
  if (!connectedAt) return "00:00:00";
  return formatDuration(Date.now() - connectedAt);
}

export function RightPanel() {
  const { t } = useTranslation();
  const portId = useConnectionStore((s) => s.portId);
  const isConnected = useConnectionStore((s) => s.status === "connected");
  const portStatus = useConnectionStore((s) => s.portStatus);
  const packets = useDataStore((s) => s.packets);
  const actions = useSerialScriptStore((s) => s.actions);
  const callAction = useSerialScriptStore((s) => s.callAction);

  const duration = useConnectionDuration();

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

  const exportData = useCallback(
    (format: "txt" | "csv" | "json") => {
      if (packets.length === 0) return;

      let content: string;
      let mimeType = "text/plain";
      const extension = format;

      if (format === "json") {
        content = JSON.stringify(packets, null, 2);
        mimeType = "application/json";
      } else if (format === "csv") {
        const header = "timestamp,direction,data\n";
        const rows = packets
          .map((p) => `${p.timestamp},${p.direction},"${p.data.join(" ")}"`)
          .join("\n");
        content = header + rows;
        mimeType = "text/csv";
      } else {
        content = packets
          .map(
            (p) =>
              `[${new Date(p.timestamp).toISOString()}] ${p.direction.toUpperCase()} ${p.data.join(" ")}`,
          )
          .join("\n");
      }

      const blob = new Blob([content], { type: mimeType });
      const url = URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = url;
      a.download = `serial-data-${Date.now()}.${extension}`;
      a.click();
      URL.revokeObjectURL(url);
    },
    [packets],
  );

  return (
    <div className="flex flex-col h-full overflow-y-auto">
      {/* Stats */}
      <section className="p-3 border-b border-border">
        <h3 className="text-xs font-medium text-text-secondary mb-2">
          {t("terminal.stats")}
        </h3>

        {/* Connection info */}
        {isConnected && (
          <div className="mb-2 space-y-1 text-xs">
            <div className="flex justify-between">
              <span className="text-text-muted">
                {t("terminal.connectedSince")}
              </span>
              <span className="font-mono">{duration}</span>
            </div>
            <div className="flex justify-between">
              <span className="text-text-muted">
                {t("terminal.lastActivity")}
              </span>
              <span className="font-mono text-[10px]">
                {packets.length > 0
                  ? new Date(
                      packets[packets.length - 1].timestamp,
                    ).toLocaleTimeString()
                  : "—"}
              </span>
            </div>
          </div>
        )}

        {/* Port stats from backend */}
        {portStatus && (
          <div className="grid grid-cols-2 gap-1 text-xs">
            <span className="text-text-muted">RX:</span>
            <span>
              {formatBytes(portStatus.bytes_received)} (
              {portStatus.packets_received} pkts)
            </span>
            <span className="text-text-muted">TX:</span>
            <span>
              {formatBytes(portStatus.bytes_sent)} ({portStatus.packets_sent}{" "}
              pkts)
            </span>
          </div>
        )}

        {/* Fallback: compute from packets */}
        {!portStatus && isConnected && (
          <div className="grid grid-cols-2 gap-1 text-xs">
            {(() => {
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
              return (
                <>
                  <span className="text-text-muted">RX:</span>
                  <span>
                    {formatBytes(rxBytes)} ({rxPkts})
                  </span>
                  <span className="text-text-muted">TX:</span>
                  <span>
                    {formatBytes(txBytes)} ({txPkts})
                  </span>
                </>
              );
            })()}
          </div>
        )}
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

      {/* Export Data */}
      {isConnected && packets.length > 0 && (
        <section className="p-3 border-b border-border">
          <h3 className="text-xs font-medium text-text-secondary mb-2">
            {t("terminal.exportData")}
          </h3>
          <div className="flex gap-1">
            {(["txt", "csv", "json"] as const).map((fmt) => (
              <button
                key={fmt}
                onClick={() => exportData(fmt)}
                className="flex-1 rounded px-2 py-1 text-xs uppercase bg-surface text-text-muted hover:bg-surface-hover hover:text-text transition"
              >
                .{fmt}
              </button>
            ))}
          </div>
        </section>
      )}

      {/* Empty state */}
      {!isConnected && (
        <div className="flex flex-1 items-center justify-center p-4 text-center text-xs text-text-muted">
          <p>{t("terminal.connectToStart")}</p>
        </div>
      )}
    </div>
  );
}
