import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { formatBytes, formatDuration } from "@/lib/utils";
import { useConnectionStore } from "@/stores/connection";
import { useDataStore } from "@/stores/data";

interface ThroughputSample {
  time: number;
  rx: number;
  tx: number;
}

export function RightPanelMonitor() {
  const { t } = useTranslation();
  const activePortId = useConnectionStore((s) => s.activePortId);
  const activeEntry = useConnectionStore((s) =>
    s.connections.find((c) => c.portId === s.activePortId),
  );
  const isConnected = activeEntry?.status === "connected";
  const portStatus = activeEntry?.portStatus;
  const connectedAt = activeEntry?.connectedAt;
  const packets = useDataStore((s) =>
    activePortId ? s.packets.filter((p) => p.portId === activePortId) : [],
  );

  const [throughput, setThroughput] = useState<ThroughputSample[]>([]);

  useEffect(() => {
    if (!portStatus) return;
    setThroughput((prev) => {
      const sample: ThroughputSample = {
        time: Date.now(),
        rx: portStatus.bytes_received,
        tx: portStatus.bytes_sent,
      };
      const updated = [...prev, sample];
      // Keep last 60 samples (60 seconds)
      return updated.slice(-60);
    });
  }, [portStatus]);

  if (!isConnected) {
    return (
      <div className="flex flex-1 items-center justify-center text-xs text-text-muted">
        {t("terminal.connectToStart")}
      </div>
    );
  }

  const latestRxRate =
    throughput.length >= 2
      ? throughput[throughput.length - 1].rx -
        throughput[throughput.length - 2].rx
      : 0;
  const latestTxRate =
    throughput.length >= 2
      ? throughput[throughput.length - 1].tx -
        throughput[throughput.length - 2].tx
      : 0;

  const duration = connectedAt
    ? formatDuration(Date.now() - connectedAt)
    : "00:00:00";
  const rxPkts = packets.filter((p) => p.direction === "rx").length;
  const txPkts = packets.filter((p) => p.direction === "tx").length;

  return (
    <div className="flex flex-col h-full overflow-y-auto p-3">
      {/* Summary */}
      <div className="space-y-2 text-xs mb-4">
        <div className="flex justify-between">
          <span className="text-text-muted">{t("monitor.duration")}</span>
          <span className="font-mono">{duration}</span>
        </div>
        <div className="flex justify-between">
          <span className="text-text-muted">{t("monitor.totalRx")}</span>
          <span className="font-mono">
            {portStatus ? formatBytes(portStatus.bytes_received) : "—"} (
            {portStatus?.packets_received ?? rxPkts} pkts)
          </span>
        </div>
        <div className="flex justify-between">
          <span className="text-text-muted">{t("monitor.totalTx")}</span>
          <span className="font-mono">
            {portStatus ? formatBytes(portStatus.bytes_sent) : "—"} (
            {portStatus?.packets_sent ?? txPkts} pkts)
          </span>
        </div>
        <div className="border-t border-border pt-2">
          <div className="flex justify-between">
            <span className="text-text-muted">{t("monitor.rxRate")}</span>
            <span className="font-mono text-accent">
              {formatBytes(latestRxRate)}/s
            </span>
          </div>
          <div className="flex justify-between">
            <span className="text-text-muted">{t("monitor.txRate")}</span>
            <span className="font-mono text-accent">
              {formatBytes(latestTxRate)}/s
            </span>
          </div>
        </div>
      </div>

      {/* Mini sparkline */}
      {throughput.length > 2 && (
        <div className="mb-3">
          <div className="text-[10px] text-text-muted mb-1 uppercase">
            {t("monitor.throughput")}
          </div>
          <svg
            viewBox={`0 0 ${throughput.length} 40`}
            className="w-full h-10"
            role="img"
            aria-label={t("monitor.throughput")}
          >
            {/* RX line */}
            <polyline
              fill="none"
              stroke="hsl(200, 80%, 60%)"
              strokeWidth="0.5"
              points={throughput
                .map((s, i) => {
                  const delta = i > 0 ? s.rx - throughput[i - 1].rx : 0;
                  const max = Math.max(
                    ...throughput
                      .slice(1)
                      .map((ss, j) => (j > 0 ? ss.rx - throughput[j].rx : 0)),
                    1,
                  );
                  const y = 40 - (delta / max) * 36 - 2;
                  return `${i},${y}`;
                })
                .join(" ")}
            />
            {/* TX line */}
            <polyline
              fill="none"
              stroke="hsl(160, 70%, 50%)"
              strokeWidth="0.5"
              points={throughput
                .map((s, i) => {
                  const delta = i > 0 ? s.tx - throughput[i - 1].tx : 0;
                  const max = Math.max(
                    ...throughput
                      .slice(1)
                      .map((ss, j) => (j > 0 ? ss.tx - throughput[j].tx : 0)),
                    1,
                  );
                  const y = 40 - (delta / max) * 36 - 2;
                  return `${i},${y}`;
                })
                .join(" ")}
            />
          </svg>
          <div className="flex gap-3 text-[9px] text-text-muted mt-0.5">
            <span className="flex items-center gap-1">
              <span className="w-2 h-0.5 bg-[hsl(200,80%,60%)]" /> RX
            </span>
            <span className="flex items-center gap-1">
              <span className="w-2 h-0.5 bg-[hsl(160,70%,50%)]" /> TX
            </span>
          </div>
        </div>
      )}

      {/* Recent packets */}
      <div>
        <div className="text-[10px] text-text-muted mb-1 uppercase">
          {t("monitor.recentPackets")}
        </div>
        <div className="space-y-0.5 max-h-40 overflow-y-auto">
          {packets
            .slice(-20)
            .reverse()
            .map((p, i) => (
              <div
                key={packets.length - 1 - i}
                className="flex items-center gap-2 text-[10px] font-mono"
              >
                <span
                  className={
                    p.direction === "rx" ? "text-blue-400" : "text-green-400"
                  }
                >
                  {p.direction.toUpperCase()}
                </span>
                <span className="text-text-muted">{p.data.length}B</span>
                <span className="text-text-secondary truncate">
                  {p.data
                    .slice(0, 8)
                    .map((b) => b.toString(16).padStart(2, "0"))
                    .join(" ")
                    .toUpperCase()}
                </span>
              </div>
            ))}
        </div>
      </div>
    </div>
  );
}
