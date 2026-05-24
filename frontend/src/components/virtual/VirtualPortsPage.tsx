import { useCallback, useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { toast } from "sonner";
import { formatBytes, formatDuration } from "@/lib/utils";
import { useVirtualPortStore } from "@/stores/virtualPort";
import type { VirtualPortStats } from "@/types";

type DirectionFilter = "all" | "a2b" | "b2a";

export function VirtualPortsPage() {
  const { t } = useTranslation();
  const {
    ports,
    selectedPort,
    capturedPackets,
    refreshPorts,
    createPort,
    stopPort,
    getStats,
    loadCapturedPackets,
    setSelectedPort,
  } = useVirtualPortStore();

  const [backend, setBackend] = useState("pty");
  const [statsMap, setStatsMap] = useState<Record<string, VirtualPortStats>>(
    {},
  );
  const [prevBytes, setPrevBytes] = useState<Record<string, number>>({});
  const [throughput, setThroughput] = useState<Record<string, number>>({});
  const [directionFilter, setDirectionFilter] =
    useState<DirectionFilter>("all");

  useEffect(() => {
    refreshPorts();
    const interval = setInterval(refreshPorts, 3000);
    return () => clearInterval(interval);
  }, [refreshPorts]);

  // Load stats and compute throughput
  useEffect(() => {
    ports.forEach(async (p) => {
      const stats = await getStats(p.id);
      if (stats) {
        setStatsMap((prev) => ({ ...prev, [p.id]: stats }));
        const prev = prevBytes[p.id] ?? stats.bytes_bridged;
        const delta = stats.bytes_bridged - prev;
        setThroughput((prev) => ({ ...prev, [p.id]: delta }));
        setPrevBytes((prev) => ({ ...prev, [p.id]: stats.bytes_bridged }));
      }
    });
  }, [ports, getStats, prevBytes]);

  const handleCreate = useCallback(async () => {
    try {
      await createPort({ backend });
      toast.success("Virtual port pair created");
    } catch (e) {
      toast.error(String(e));
    }
  }, [backend, createPort]);

  const handleStop = useCallback(
    async (id: string) => {
      try {
        await stopPort(id);
        toast.success("Virtual port stopped");
      } catch (e) {
        toast.error(String(e));
      }
    },
    [stopPort],
  );

  const handleCapture = useCallback(
    async (id: string) => {
      setSelectedPort(id);
      await loadCapturedPackets(id);
    },
    [setSelectedPort, loadCapturedPackets],
  );

  const handleExportCapture = useCallback(() => {
    if (capturedPackets.length === 0) return;
    const header = "timestamp,direction,data\n";
    const rows = capturedPackets
      .map(
        (p) =>
          `${p.timestamp_millis},${p.direction},"${p.data.map((b) => b.toString(16).padStart(2, "0")).join(" ")}"`,
      )
      .join("\n");
    const blob = new Blob([header + rows], { type: "text/csv" });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = `capture-${selectedPort}-${Date.now()}.csv`;
    a.click();
    URL.revokeObjectURL(url);
  }, [capturedPackets, selectedPort]);

  const filteredPackets = capturedPackets.filter((p) => {
    if (directionFilter === "all") return true;
    return directionFilter === "a2b"
      ? p.direction === "a2b"
      : p.direction === "b2a";
  });

  return (
    <div className="flex flex-col h-full p-4 overflow-y-auto">
      <div className="flex items-center justify-between mb-4">
        <h1 className="text-lg font-semibold">{t("virtual.title")}</h1>
        <div className="flex items-center gap-2">
          <select
            value={backend}
            onChange={(e) => setBackend(e.target.value)}
            className="text-xs rounded border border-border bg-transparent px-2 py-1"
          >
            <option value="pty">PTY</option>
            <option value="socat">socat</option>
            <option value="namedpipe">Named Pipe</option>
          </select>
          <button
            onClick={handleCreate}
            className="px-3 py-1.5 rounded text-xs bg-accent/20 text-accent hover:bg-accent/30"
          >
            + {t("virtual.createPair")}
          </button>
        </div>
      </div>

      {ports.length === 0 ? (
        <div className="flex items-center justify-center flex-1">
          <div className="text-center space-y-3 p-8 rounded-lg border border-dashed border-border">
            <div className="text-text-muted text-sm">
              {t("virtual.emptyState")}
            </div>
            <div className="text-xs text-text-muted max-w-xs">
              {t("virtual.emptyStateDesc")}
            </div>
            <button
              onClick={handleCreate}
              className="px-4 py-2 rounded text-xs bg-accent/20 text-accent hover:bg-accent/30"
            >
              + {t("virtual.createPair")}
            </button>
          </div>
        </div>
      ) : (
        <div className="space-y-3">
          {ports.map((port) => {
            const stats = statsMap[port.id];
            const tp = throughput[port.id] ?? 0;
            return (
              <div
                key={port.id}
                className="p-3 bg-base-deep rounded border border-border"
              >
                <div className="flex items-center justify-between mb-2">
                  <div className="flex items-center gap-2">
                    <span
                      className={`w-2 h-2 rounded-full ${port.running ? "bg-success" : "bg-text-muted"}`}
                    />
                    <span className="font-medium text-sm">{port.id}</span>
                    <span className="text-text-muted text-xs">
                      {t("virtual.backend")}: {port.backend}
                    </span>
                    {tp > 0 && (
                      <span className="text-xs text-accent font-mono">
                        {formatBytes(tp)}/s
                      </span>
                    )}
                  </div>
                  <div className="flex gap-2">
                    <button
                      onClick={() => handleCapture(port.id)}
                      className="px-2 py-0.5 rounded text-xs text-text-muted hover:text-text"
                    >
                      {t("virtual.capture")}
                    </button>
                    <button
                      onClick={() => handleStop(port.id)}
                      className="px-2 py-0.5 rounded text-xs text-danger hover:bg-danger/20"
                    >
                      {t("common.stop")}
                    </button>
                  </div>
                </div>
                <div className="grid grid-cols-2 gap-x-6 gap-y-1 text-xs">
                  <span className="text-text-muted">{t("virtual.portA")}:</span>
                  <span className="font-mono">{port.port_a}</span>
                  <span className="text-text-muted">{t("virtual.portB")}:</span>
                  <span className="font-mono">{port.port_b}</span>
                  <span className="text-text-muted">
                    {t("virtual.uptime")}:
                  </span>
                  <span>{formatDuration(port.uptime_secs * 1000)}</span>
                  {stats && (
                    <>
                      <span className="text-text-muted">
                        {t("virtual.bridged")}:
                      </span>
                      <span>{formatBytes(stats.bytes_bridged)}</span>
                      <span className="text-text-muted">
                        {t("virtual.packetsBridged")}:
                      </span>
                      <span>{stats.packets_bridged}</span>
                    </>
                  )}
                </div>
              </div>
            );
          })}
        </div>
      )}

      {/* Captured packets */}
      {selectedPort && capturedPackets.length > 0 && (
        <div className="mt-4">
          <div className="flex items-center justify-between mb-2">
            <h2 className="text-sm font-medium">
              {t("virtual.capturedPackets")}
            </h2>
            <div className="flex items-center gap-2">
              {/* Direction filter */}
              <div className="flex rounded border border-border overflow-hidden text-xs">
                {(["all", "a2b", "b2a"] as const).map((f) => (
                  <button
                    key={f}
                    className={`px-2 py-0.5 ${directionFilter === f ? "bg-accent/20 text-accent" : "text-text-muted hover:bg-surface"}`}
                    onClick={() => setDirectionFilter(f)}
                  >
                    {f === "all"
                      ? t("virtual.filterAll")
                      : f === "a2b"
                        ? "A→B"
                        : "B→A"}
                  </button>
                ))}
              </div>
              {/* Export */}
              <button
                onClick={handleExportCapture}
                className="px-2 py-0.5 rounded text-xs text-text-muted hover:text-text"
              >
                {t("common.export")} CSV
              </button>
            </div>
          </div>
          <div className="bg-base-deep rounded border border-border overflow-auto max-h-64">
            <table className="w-full text-xs font-mono">
              <thead>
                <tr className="border-b border-border text-text-muted">
                  <th className="px-2 py-1 text-left">#</th>
                  <th className="px-2 py-1 text-left">Dir</th>
                  <th className="px-2 py-1 text-left">Time</th>
                  <th className="px-2 py-1 text-left">Data</th>
                </tr>
              </thead>
              <tbody>
                {filteredPackets.map((pkt, i) => (
                  <tr
                    key={i}
                    className="border-b border-border/50 hover:bg-surface/30"
                  >
                    <td className="px-2 py-1 text-text-muted">{i + 1}</td>
                    <td className="px-2 py-1 text-accent">{pkt.direction}</td>
                    <td className="px-2 py-1 text-text-secondary">
                      {new Date(pkt.timestamp_millis).toLocaleTimeString()}
                    </td>
                    <td className="px-2 py-1 break-all">
                      {pkt.data
                        .map((b) => b.toString(16).padStart(2, "0"))
                        .join(" ")
                        .toUpperCase()}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </div>
      )}
    </div>
  );
}
