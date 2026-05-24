import { useCallback, useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { toast } from "sonner";
import { formatBytes } from "@/lib/utils";
import { useVirtualPortStore } from "@/stores/virtualPort";
import type { VirtualPortStats } from "@/types";

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

  useEffect(() => {
    refreshPorts();
    const interval = setInterval(refreshPorts, 3000);
    return () => clearInterval(interval);
  }, [refreshPorts]);

  // Load stats for all ports
  useEffect(() => {
    ports.forEach(async (p) => {
      const stats = await getStats(p.id);
      if (stats) {
        setStatsMap((prev) => ({ ...prev, [p.id]: stats }));
      }
    });
  }, [ports, getStats]);

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

  return (
    <div className="flex flex-col h-full p-4 overflow-y-auto">
      <div className="flex items-center justify-between mb-4">
        <h1 className="text-lg font-semibold">{t("virtual.title")}</h1>
        <div className="flex items-center gap-2">
          <select
            value={backend}
            onChange={(e) => setBackend(e.target.value)}
            className="text-xs"
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
        <div className="flex items-center justify-center flex-1 text-text-muted">
          {t("virtual.noPorts")}
        </div>
      ) : (
        <div className="space-y-3">
          {ports.map((port) => {
            const stats = statsMap[port.id];
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
                  <span>{port.uptime_secs}s</span>
                  {stats && (
                    <>
                      <span className="text-text-muted">
                        {t("virtual.bridged")}:
                      </span>
                      <span>{formatBytes(stats.bytes_bridged)}</span>
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
          <h2 className="text-sm font-medium mb-2">
            {t("virtual.capturedPackets")}
          </h2>
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
                {capturedPackets.map((pkt, i) => (
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
