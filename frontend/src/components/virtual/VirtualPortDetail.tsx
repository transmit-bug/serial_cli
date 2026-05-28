import { useCallback, useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { toast } from "sonner";
import { CommandSender } from "@/components/shared/CommandSender";
import { tauriApi } from "@/lib/tauri-api";
import { useVirtualPortStore } from "@/stores/virtualPort";
import type { VirtualPortInfo } from "@/types";
import { BridgeVisualization } from "./BridgeVisualization";
import { VirtualPacketTable } from "./VirtualPacketTable";

function hexToBytes(data: string): number[] {
  return data
    .trim()
    .split(/\s+/)
    .map((b) => parseInt(b, 16))
    .filter((n) => !Number.isNaN(n));
}

function asciiToBytes(data: string): number[] {
  return data.split("").map((c) => c.charCodeAt(0));
}

export function VirtualPortDetail({ port }: { port: VirtualPortInfo }) {
  const { t } = useTranslation();
  const {
    statsMap,
    throughputMap,
    healthMap,
    capturedPackets,
    getStats,
    checkHealth,
    loadCapturedPackets,
  } = useVirtualPortStore();
  const [packetsCollapsed, setPacketsCollapsed] = useState(false);
  const [sendTarget, setSendTarget] = useState<"a" | "b">("b");

  const stats = statsMap[port.id];
  const throughput = throughputMap[port.id] ?? 0;
  const healthy = healthMap[port.id];

  // Poll stats for selected port
  useEffect(() => {
    getStats(port.id);
    const interval = setInterval(() => {
      getStats(port.id);
    }, 1000);
    return () => clearInterval(interval);
  }, [port.id, getStats]);

  // Periodic health check
  useEffect(() => {
    checkHealth(port.id);
    const interval = setInterval(() => {
      checkHealth(port.id);
    }, 5000);
    return () => clearInterval(interval);
  }, [port.id, checkHealth]);

  // Load captured packets
  useEffect(() => {
    loadCapturedPackets(port.id);
    const interval = setInterval(() => {
      loadCapturedPackets(port.id);
    }, 2000);
    return () => clearInterval(interval);
  }, [port.id, loadCapturedPackets]);

  const sendFn = useCallback(
    async (data: string, format: "hex" | "ascii") => {
      const bytes = format === "hex" ? hexToBytes(data) : asciiToBytes(data);
      try {
        const written = await tauriApi.sendToVirtualPort(
          port.id,
          sendTarget,
          bytes,
        );
        toast.success(
          `${written} bytes sent to port ${sendTarget.toUpperCase()}`,
        );
      } catch (e) {
        toast.error(String(e));
      }
    },
    [port.id, sendTarget],
  );

  return (
    <div className="flex flex-col h-full overflow-y-auto p-4 gap-4">
      {/* Bridge visualization */}
      <BridgeVisualization
        port={port}
        stats={stats}
        throughput={throughput}
        healthy={healthy}
      />

      {/* Port info */}
      <div className="p-3 bg-base-deep rounded border border-border">
        <h3 className="text-xs font-medium text-text-muted mb-2">
          {t("virtual.portInfo")}
        </h3>
        <div className="grid grid-cols-[auto_1fr] gap-x-4 gap-y-1 text-xs">
          <span className="text-text-muted">ID</span>
          <span className="font-mono">{port.id}</span>
          <span className="text-text-muted">{t("virtual.backend")}</span>
          <span>{port.backend}</span>
          <span className="text-text-muted">{t("virtual.portA")}</span>
          <span className="font-mono">{port.port_a}</span>
          <span className="text-text-muted">{t("virtual.portB")}</span>
          <span className="font-mono">{port.port_b}</span>
          {stats && (
            <>
              <span className="text-text-muted">{t("virtual.bridged")}</span>
              <span>{stats.bytes_bridged} bytes</span>
              <span className="text-text-muted">
                {t("virtual.packetsBridged")}
              </span>
              <span>{stats.packets_bridged}</span>
            </>
          )}
        </div>
      </div>

      {/* Quick send */}
      <div className="bg-base-deep rounded border border-border">
        <div className="flex items-center justify-between px-3 py-1.5 border-b border-border">
          <h3 className="text-xs font-medium">{t("virtual.quickSend")}</h3>
          <div className="flex rounded border border-border overflow-hidden text-xs">
            <button
              className={`px-2 py-0.5 ${sendTarget === "a" ? "bg-accent/20 text-accent" : "text-text-muted hover:bg-surface"}`}
              onClick={() => setSendTarget("a")}
            >
              {t("virtual.sendToA")}
            </button>
            <button
              className={`px-2 py-0.5 ${sendTarget === "b" ? "bg-accent/20 text-accent" : "text-text-muted hover:bg-surface"}`}
              onClick={() => setSendTarget("b")}
            >
              {t("virtual.sendToB")}
            </button>
          </div>
        </div>
        <CommandSender enabled={port.running} sendFn={sendFn} />
      </div>

      {/* Captured packets */}
      <VirtualPacketTable
        packets={capturedPackets}
        portId={port.id}
        collapsed={packetsCollapsed}
        onToggleCollapse={() => setPacketsCollapsed(!packetsCollapsed)}
      />
    </div>
  );
}
