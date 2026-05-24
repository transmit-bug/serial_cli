import { useMemo, useState } from "react";
import { useTranslation } from "react-i18next";
import { useDataStore } from "@/stores/data";
import { ExportControls } from "./ExportControls";

type DirectionFilter = "all" | "rx" | "tx";

export function RightPanelHistory() {
  const { t } = useTranslation();
  const packets = useDataStore((s) => s.packets);
  const clearBuffer = useDataStore((s) => s.clearBuffer);

  const [searchQuery, setSearchQuery] = useState("");
  const [directionFilter, setDirectionFilter] =
    useState<DirectionFilter>("all");

  const filtered = useMemo(() => {
    return packets.filter((p) => {
      if (directionFilter !== "all" && p.direction !== directionFilter)
        return false;
      if (searchQuery) {
        const hex = p.data
          .map((b) => b.toString(16).padStart(2, "0"))
          .join(" ")
          .toLowerCase();
        const ascii = p.data
          .map((b) => (b >= 32 && b <= 126 ? String.fromCharCode(b) : ""))
          .join("")
          .toLowerCase();
        return (
          hex.includes(searchQuery.toLowerCase()) ||
          ascii.includes(searchQuery.toLowerCase())
        );
      }
      return true;
    });
  }, [packets, searchQuery, directionFilter]);

  return (
    <div className="flex flex-col h-full">
      {/* Controls */}
      <div className="p-2 border-b border-border space-y-2">
        <input
          className="w-full h-6 text-xs rounded border border-border bg-transparent px-2"
          placeholder={t("history.search")}
          value={searchQuery}
          onChange={(e) => setSearchQuery(e.target.value)}
        />
        <div className="flex items-center justify-between">
          <div className="flex rounded border border-border overflow-hidden text-xs">
            {(["all", "rx", "tx"] as const).map((f) => (
              <button
                key={f}
                className={`px-2 py-0.5 ${
                  directionFilter === f
                    ? "bg-accent/20 text-accent"
                    : "text-text-muted hover:bg-surface"
                }`}
                onClick={() => setDirectionFilter(f)}
              >
                {f === "all" ? t("history.all") : f.toUpperCase()}
              </button>
            ))}
          </div>
          <div className="flex items-center gap-1">
            <span className="text-[10px] text-text-muted">
              {filtered.length}/{packets.length}
            </span>
            {packets.length > 0 && (
              <button
                className="text-[10px] text-danger hover:text-danger-hover"
                onClick={clearBuffer}
              >
                {t("common.clear")}
              </button>
            )}
          </div>
        </div>
      </div>

      {/* Packet list */}
      <div className="flex-1 overflow-y-auto">
        {filtered.length === 0 ? (
          <div className="p-4 text-center text-xs text-text-muted">
            {packets.length === 0
              ? t("history.emptyState")
              : t("history.noMatch")}
          </div>
        ) : (
          <div className="divide-y divide-border/50">
            {filtered.map((p) => (
              <div
                key={p.id}
                className="px-2 py-1.5 text-xs font-mono hover:bg-surface/30"
              >
                <div className="flex items-center gap-2">
                  <span
                    className={
                      p.direction === "rx"
                        ? "text-blue-400 shrink-0"
                        : "text-green-400 shrink-0"
                    }
                  >
                    {p.direction.toUpperCase()}
                  </span>
                  <span className="text-[9px] text-text-muted shrink-0">
                    {new Date(p.timestamp).toLocaleTimeString()}
                  </span>
                  <span className="text-[9px] text-text-muted shrink-0">
                    {p.data.length}B
                  </span>
                  <span className="text-text truncate">
                    {p.data
                      .map((b) => b.toString(16).padStart(2, "0"))
                      .join(" ")
                      .toUpperCase()}
                  </span>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>

      {/* Export */}
      <ExportControls />
    </div>
  );
}
