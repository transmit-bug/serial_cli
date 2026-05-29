import { useCallback, useState } from "react";
import { useTranslation } from "react-i18next";
import type { CapturedPacket } from "@/types";

type DirectionFilter = "all" | "a2b" | "b2a";

interface VirtualPacketTableProps {
  packets: CapturedPacket[];
  portId: string;
  collapsed?: boolean;
  onToggleCollapse?: () => void;
}

export function VirtualPacketTable({
  packets,
  portId,
  collapsed = false,
  onToggleCollapse,
}: VirtualPacketTableProps) {
  const { t } = useTranslation();
  const [directionFilter, setDirectionFilter] =
    useState<DirectionFilter>("all");

  const filteredPackets = packets.filter((p) => {
    if (directionFilter === "all") return true;
    return directionFilter === "a2b"
      ? p.direction === "AtoB"
      : p.direction === "BtoA";
  });

  const a2bCount = packets.filter((p) => p.direction === "AtoB").length;
  const b2aCount = packets.filter((p) => p.direction === "BtoA").length;

  const handleExport = useCallback(() => {
    if (filteredPackets.length === 0) return;
    const header = "timestamp,direction,data\n";
    const rows = filteredPackets
      .map(
        (p) =>
          `${p.timestamp_millis},${p.direction},"${p.data.map((b) => b.toString(16).padStart(2, "0")).join(" ")}"`,
      )
      .join("\n");
    const blob = new Blob([header + rows], { type: "text/csv" });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = `capture-${portId}-${Date.now()}.csv`;
    a.click();
    URL.revokeObjectURL(url);
  }, [filteredPackets, portId]);

  return (
    <div className="bg-base-deep rounded border border-border">
      {/* Header */}
      <div className="flex items-center justify-between px-3 py-1.5">
        <button
          className="flex items-center gap-1.5 text-xs font-medium hover:text-text"
          onClick={onToggleCollapse}
        >
          <span
            className={`transition-transform ${collapsed ? "" : "rotate-90"}`}
          >
            ▶
          </span>
          {t("virtual.capturedPackets")}
          <span className="text-text-muted font-normal">
            ({packets.length} {t("virtual.packetsBridged")}: A→B {a2bCount}, B→A{" "}
            {b2aCount})
          </span>
        </button>
        {!collapsed && (
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
              onClick={handleExport}
              disabled={filteredPackets.length === 0}
              className="px-2 py-0.5 rounded text-xs text-text-muted hover:text-text disabled:opacity-40"
            >
              {t("common.export")} CSV
            </button>
          </div>
        )}
      </div>

      {/* Table */}
      {!collapsed && (
        <div className="overflow-auto max-h-64 border-t border-border">
          <table className="w-full text-xs font-mono">
            <thead>
              <tr className="border-b border-border text-text-muted sticky top-0 bg-base-deep">
                <th className="px-2 py-1 text-left w-10">#</th>
                <th className="px-2 py-1 text-left w-14">Dir</th>
                <th className="px-2 py-1 text-left w-20">Time</th>
                <th className="px-2 py-1 text-left">Data</th>
              </tr>
            </thead>
            <tbody>
              {filteredPackets.length === 0 ? (
                <tr>
                  <td
                    colSpan={4}
                    className="px-2 py-4 text-center text-text-muted"
                  >
                    {t("virtual.noPackets")}
                  </td>
                </tr>
              ) : (
                filteredPackets.map((pkt, i) => (
                  <tr
                    key={`${pkt.timestamp_millis}-${i}`}
                    className="border-b border-border/50 hover:bg-surface/30"
                  >
                    <td className="px-2 py-1 text-text-muted">{i + 1}</td>
                    <td
                      className={`px-2 py-1 ${pkt.direction === "AtoB" ? "text-accent" : "text-blue-400"}`}
                    >
                      {pkt.direction === "AtoB" ? "A→B" : "B→A"}
                    </td>
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
                ))
              )}
            </tbody>
          </table>
        </div>
      )}
    </div>
  );
}
