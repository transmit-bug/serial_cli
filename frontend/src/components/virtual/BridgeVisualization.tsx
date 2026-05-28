import { useTranslation } from "react-i18next";
import { formatBytes, formatDuration } from "@/lib/utils";
import type { VirtualPortInfo, VirtualPortStats } from "@/types";

interface BridgeVisualizationProps {
  port: VirtualPortInfo;
  stats?: VirtualPortStats;
  throughput: number;
  healthy?: boolean;
}

export function BridgeVisualization({
  port,
  stats,
  throughput,
  healthy,
}: BridgeVisualizationProps) {
  const { t } = useTranslation();

  const isHealthy = healthy !== false;
  const errorCount = stats?.bridge_errors ?? 0;
  const lineColor = !isHealthy
    ? "border-danger"
    : errorCount > 0
      ? "border-warning"
      : "border-accent/60";
  const arrowColor = !isHealthy ? "border-l-danger" : "border-l-accent/60";
  const arrowColorReverse = !isHealthy
    ? "border-r-danger"
    : "border-r-accent/60";

  return (
    <div className="p-3 bg-base-deep rounded border border-border">
      <div className="flex items-center justify-between gap-4">
        {/* Port A */}
        <div className="flex flex-col items-center gap-1 min-w-0 flex-1">
          <span className="text-[10px] text-text-muted uppercase tracking-wider">
            {t("virtual.portA")}
          </span>
          <div className="w-full px-2 py-1.5 rounded bg-surface text-center">
            <span
              className="text-xs font-mono truncate block"
              title={port.port_a}
            >
              {port.port_a}
            </span>
          </div>
        </div>

        {/* Bridge line */}
        <div className="flex flex-col items-center gap-1 flex-shrink-0 px-2">
          <div className="flex items-center gap-1">
            <div className={`w-20 h-0.5 ${lineColor} relative`}>
              <div
                className={`absolute top-1/2 -translate-y-1/2 right-0 w-0 h-0 border-t-[4px] border-b-[4px] border-l-[6px] border-transparent ${arrowColor}`}
              />
            </div>
            <div className={`w-20 h-0.5 ${lineColor} relative`}>
              <div
                className={`absolute top-1/2 -translate-y-1/2 left-0 w-0 h-0 border-t-[4px] border-b-[4px] border-r-[6px] border-transparent ${arrowColorReverse}`}
              />
            </div>
          </div>
          <span className="text-[10px] font-mono text-accent">
            {throughput > 0 ? `${formatBytes(throughput)}/s` : "—"}
          </span>
        </div>

        {/* Port B */}
        <div className="flex flex-col items-center gap-1 min-w-0 flex-1">
          <span className="text-[10px] text-text-muted uppercase tracking-wider">
            {t("virtual.portB")}
          </span>
          <div className="w-full px-2 py-1.5 rounded bg-surface text-center">
            <span
              className="text-xs font-mono truncate block"
              title={port.port_b}
            >
              {port.port_b}
            </span>
          </div>
        </div>
      </div>

      {/* Stats row */}
      <div className="flex items-center justify-center gap-4 mt-2 text-[10px] text-text-muted">
        <span>
          {t("virtual.uptime")}: {formatDuration(port.uptime_secs * 1000)}
        </span>
        {stats && (
          <>
            <span className="text-text-tertiary">·</span>
            <span>
              {t("virtual.bridged")}: {formatBytes(stats.bytes_bridged)}
            </span>
            <span className="text-text-tertiary">·</span>
            <span>
              {stats.packets_bridged} {t("virtual.packetsBridged")}
            </span>
            {errorCount > 0 && (
              <>
                <span className="text-text-tertiary">·</span>
                <span className="text-danger">
                  {errorCount} {t("virtual.errors")}
                </span>
              </>
            )}
          </>
        )}
      </div>

      {/* Error detail */}
      {stats?.last_error && (
        <div
          className="mt-1 text-[10px] text-danger text-center truncate"
          title={stats.last_error}
        >
          {stats.last_error}
        </div>
      )}

      {/* Health warning */}
      {!isHealthy && (
        <div className="mt-1 text-[10px] text-danger text-center">
          {t("virtual.healthFailed")}
        </div>
      )}
    </div>
  );
}
