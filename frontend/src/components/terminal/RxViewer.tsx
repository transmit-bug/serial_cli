import { useVirtualizer } from "@tanstack/react-virtual";
import { useCallback, useEffect, useRef } from "react";
import { useTranslation } from "react-i18next";
import { bytesToAscii, bytesToHex, formatTimestamp } from "@/lib/utils";
import { useDataStore } from "@/stores/data";
import type { DataPacket, DisplayFormat } from "@/types";

export function RxViewer() {
  const { t } = useTranslation();
  const packets = useDataStore((s) => s.packets);
  const displayFormat = useDataStore((s) => s.displayFormat);
  const autoScroll = useDataStore((s) => s.autoScroll);
  const searchQuery = useDataStore((s) => s.searchQuery);
  const clearBuffer = useDataStore((s) => s.clearBuffer);
  const setDisplayFormat = useDataStore((s) => s.setDisplayFormat);
  const toggleAutoScroll = useDataStore((s) => s.toggleAutoScroll);
  const setSearchQuery = useDataStore((s) => s.setSearchQuery);

  const parentRef = useRef<HTMLDivElement>(null);

  const filteredPackets = searchQuery
    ? packets.filter((p) => {
        const hex = bytesToHex(p.data).toLowerCase();
        const ascii = bytesToAscii(p.data).toLowerCase();
        return (
          hex.includes(searchQuery.toLowerCase()) ||
          ascii.includes(searchQuery.toLowerCase())
        );
      })
    : packets;

  const virtualizer = useVirtualizer({
    count: filteredPackets.length,
    getScrollElement: () => parentRef.current,
    estimateSize: () => 24,
    overscan: 50,
  });

  // Auto-scroll to bottom
  useEffect(() => {
    if (autoScroll && filteredPackets.length > 0) {
      virtualizer.scrollToIndex(filteredPackets.length - 1, { align: "end" });
    }
  }, [filteredPackets.length, autoScroll, virtualizer]);

  const renderRow = useCallback(
    (packet: DataPacket) => {
      if (displayFormat === "hex") {
        return (
          <div className="flex items-center gap-2 px-2 py-0.5 hover:bg-surface/50 font-mono text-xs">
            <span className="text-text-muted w-8 shrink-0 text-right">
              {packet.id}
            </span>
            <span className="text-text-secondary w-20 shrink-0">
              {formatTimestamp(packet.timestamp)}
            </span>
            <span
              className={`w-6 shrink-0 ${packet.direction === "rx" ? "text-accent" : "text-success"}`}
            >
              {packet.direction === "rx" ? "RX" : "TX"}
            </span>
            <span className="text-text break-all">
              {bytesToHex(packet.data)}
            </span>
          </div>
        );
      }
      if (displayFormat === "ascii") {
        return (
          <div className="flex items-center gap-2 px-2 py-0.5 hover:bg-surface/50 font-mono text-xs">
            <span className="text-text-muted w-8 shrink-0 text-right">
              {packet.id}
            </span>
            <span className="text-text-secondary w-20 shrink-0">
              {formatTimestamp(packet.timestamp)}
            </span>
            <span
              className={`w-6 shrink-0 ${packet.direction === "rx" ? "text-accent" : "text-success"}`}
            >
              {packet.direction === "rx" ? "RX" : "TX"}
            </span>
            <span className="text-text break-all">
              {bytesToAscii(packet.data)}
            </span>
          </div>
        );
      }
      // Mixed mode — table layout
      return (
        <div className="flex items-center gap-2 px-2 py-0.5 hover:bg-surface/50 font-mono text-xs">
          <span className="text-text-muted w-8 shrink-0 text-right">
            {packet.id}
          </span>
          <span className="text-text-secondary w-20 shrink-0">
            {formatTimestamp(packet.timestamp)}
          </span>
          <span
            className={`w-6 shrink-0 ${packet.direction === "rx" ? "text-accent" : "text-success"}`}
          >
            {packet.direction === "rx" ? "RX" : "TX"}
          </span>
          <span className="text-warning w-52 shrink-0 truncate">
            {bytesToHex(packet.data)}
          </span>
          <span className="text-text break-all">
            {bytesToAscii(packet.data)}
          </span>
        </div>
      );
    },
    [displayFormat],
  );

  const formatBtn = (fmt: DisplayFormat, label: string) => (
    <button
      onClick={() => setDisplayFormat(fmt)}
      className={`px-2 py-0.5 rounded text-xs transition-colors ${
        displayFormat === fmt
          ? "bg-accent/20 text-accent"
          : "text-text-muted hover:text-text"
      }`}
    >
      {label}
    </button>
  );

  return (
    <div className="flex flex-col h-full">
      {/* Toolbar */}
      <div className="flex items-center gap-2 px-3 py-1.5 border-b border-border shrink-0">
        {formatBtn("hex", t("terminal.hexMode"))}
        {formatBtn("ascii", t("terminal.asciiMode"))}
        {formatBtn("mixed", t("terminal.mixedMode"))}
        <div className="mx-1 w-px h-4 bg-border" />
        <input
          type="text"
          placeholder={t("common.search")}
          value={searchQuery}
          onChange={(e) => setSearchQuery(e.target.value)}
          className="w-40 h-6 text-xs"
        />
        <button
          onClick={toggleAutoScroll}
          className={`px-2 py-0.5 rounded text-xs ${
            autoScroll ? "text-accent" : "text-text-muted"
          }`}
        >
          {t("terminal.autoScroll")}
        </button>
        <button
          onClick={clearBuffer}
          className="px-2 py-0.5 rounded text-xs text-text-muted hover:text-text"
        >
          {t("terminal.clearBuffer")}
        </button>
        <span className="text-text-muted text-xs ml-auto">
          {t("terminal.packetCount", { count: packets.length })}
        </span>
      </div>

      {/* Data area */}
      <div ref={parentRef} className="flex-1 overflow-auto">
        {filteredPackets.length === 0 ? (
          <div className="flex items-center justify-center h-full text-text-muted text-sm">
            {t("terminal.noData")}
          </div>
        ) : (
          <div
            style={{
              height: `${virtualizer.getTotalSize()}px`,
              width: "100%",
              position: "relative",
            }}
          >
            {virtualizer.getVirtualItems().map((virtualRow) => {
              const packet = filteredPackets[virtualRow.index];
              return (
                <div
                  key={virtualRow.key}
                  style={{
                    position: "absolute",
                    top: 0,
                    left: 0,
                    width: "100%",
                    transform: `translateY(${virtualRow.start}px)`,
                    height: `${virtualRow.size}px`,
                  }}
                >
                  {renderRow(packet)}
                </div>
              );
            })}
          </div>
        )}
      </div>
    </div>
  );
}
