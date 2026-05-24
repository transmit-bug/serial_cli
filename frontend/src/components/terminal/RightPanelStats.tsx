import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { formatBytes, formatDuration } from "@/lib/utils";
import { useConnectionStore } from "@/stores/connection";
import { useDataStore } from "@/stores/data";

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

export function RightPanelStats() {
  const { t } = useTranslation();
  const isConnected = useConnectionStore((s) => s.status === "connected");
  const portStatus = useConnectionStore((s) => s.portStatus);
  const packets = useDataStore((s) => s.packets);

  const duration = useConnectionDuration();

  return (
    <div className="flex flex-col h-full overflow-y-auto p-3">
      {/* Connection info */}
      {isConnected && (
        <div className="mb-3 space-y-1 text-xs">
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

      {!isConnected && (
        <div className="flex flex-1 items-center justify-center text-xs text-text-muted">
          <p>{t("terminal.connectToStart")}</p>
        </div>
      )}
    </div>
  );
}
