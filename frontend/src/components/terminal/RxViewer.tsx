import { useVirtualizer } from "@tanstack/react-virtual";
import { useEffect, useMemo, useRef } from "react";
import { useTranslation } from "react-i18next";
import { splitHighlights } from "@/lib/highlight";
import { bytesToAscii, bytesToHex, formatTimestamp } from "@/lib/utils";
import { type SearchOptions, useDataStore } from "@/stores/data";
import type { DataPacket, DisplayFormat } from "@/types";

function HighlightedText({
  text,
  query,
  options,
  className,
}: {
  text: string;
  query: string;
  options: SearchOptions;
  className?: string;
}) {
  const segments = useMemo(
    () => splitHighlights(text, query, options),
    [text, query, options],
  );

  return (
    <span className={className}>
      {query
        ? segments.map((seg, i) =>
            seg.match ? (
              <mark
                key={i}
                className="bg-warning/30 text-warning rounded-sm px-px"
              >
                {seg.text}
              </mark>
            ) : (
              <span key={i}>{seg.text}</span>
            ),
          )
        : text}
    </span>
  );
}

function PacketRow({
  packet,
  displayFormat,
  showTimestamp,
  searchQuery,
  searchOptions,
}: {
  packet: DataPacket;
  displayFormat: DisplayFormat;
  showTimestamp: boolean;
  searchQuery: string;
  searchOptions: SearchOptions;
}) {
  const hexStr = bytesToHex(packet.data);
  const asciiStr = bytesToAscii(packet.data);
  const dirClass = packet.direction === "rx" ? "text-accent" : "text-success";
  const dirLabel = packet.direction === "rx" ? "RX" : "TX";

  const dataCols = useMemo(() => {
    const q = searchQuery;
    const opts = searchOptions;

    if (displayFormat === "hex") {
      return (
        <HighlightedText
          text={hexStr}
          query={q}
          options={opts}
          className="text-text break-all"
        />
      );
    }
    if (displayFormat === "ascii") {
      return (
        <HighlightedText
          text={asciiStr}
          query={q}
          options={opts}
          className="text-text break-all"
        />
      );
    }
    // Mixed
    return (
      <>
        <HighlightedText
          text={hexStr}
          query={q}
          options={opts}
          className="text-warning w-52 shrink-0 truncate"
        />
        <HighlightedText
          text={asciiStr}
          query={q}
          options={opts}
          className="text-text break-all"
        />
      </>
    );
  }, [displayFormat, hexStr, asciiStr, searchQuery, searchOptions]);

  return (
    <div className="flex items-center gap-2 px-2 py-0.5 hover:bg-surface/50 font-mono text-xs">
      <span className="text-text-muted w-8 shrink-0 text-right">
        {packet.id}
      </span>
      {showTimestamp && (
        <span className="text-text-secondary w-20 shrink-0">
          {formatTimestamp(packet.timestamp)}
        </span>
      )}
      <span className={`w-6 shrink-0 ${dirClass}`}>{dirLabel}</span>
      {dataCols}
    </div>
  );
}

export function RxViewer({ portId }: { portId?: string }) {
  const { t } = useTranslation();
  const packets = useDataStore((s) => s.packets);
  const displayFormat = useDataStore((s) => s.displayFormat);
  const showTimestamp = useDataStore((s) => s.showTimestamp);
  const autoScroll = useDataStore((s) => s.autoScroll);
  const searchQuery = useDataStore((s) => s.searchQuery);
  const searchOptions = useDataStore((s) => s.searchOptions);
  const clearBuffer = useDataStore((s) => s.clearBuffer);
  const setDisplayFormat = useDataStore((s) => s.setDisplayFormat);
  const toggleAutoScroll = useDataStore((s) => s.toggleAutoScroll);
  const setSearchQuery = useDataStore((s) => s.setSearchQuery);
  const setSearchOptions = useDataStore((s) => s.setSearchOptions);
  const getFilteredPackets = useDataStore((s) => s.getFilteredPackets);

  const parentRef = useRef<HTMLDivElement>(null);
  const searchInputRef = useRef<HTMLInputElement>(null);

  const filteredPackets = useMemo(() => {
    return getFilteredPackets(portId);
  }, [getFilteredPackets, portId]);

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

  // Ctrl+F / Cmd+F → focus search input
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && e.key === "f") {
        e.preventDefault();
        searchInputRef.current?.focus();
      }
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, []);

  // Escape → blur and clear search
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if (
        e.key === "Escape" &&
        document.activeElement === searchInputRef.current
      ) {
        setSearchQuery("");
        searchInputRef.current?.blur();
      }
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [setSearchQuery]);

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

        {/* Search input + toggles */}
        <div className="flex items-center gap-1 flex-1 min-w-0">
          <input
            ref={searchInputRef}
            type="text"
            placeholder={t("common.search")}
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="w-40 h-6 text-xs shrink-0"
          />
          <button
            onClick={() =>
              setSearchOptions({ caseSensitive: !searchOptions.caseSensitive })
            }
            className={`px-1.5 py-0.5 rounded text-[10px] font-mono font-bold transition-colors ${
              searchOptions.caseSensitive
                ? "bg-accent/20 text-accent"
                : "text-text-muted hover:text-text"
            }`}
            title={t("terminal.searchCaseSensitive")}
          >
            Aa
          </button>
          <button
            onClick={() =>
              setSearchOptions({ useRegex: !searchOptions.useRegex })
            }
            className={`px-1.5 py-0.5 rounded text-[10px] font-mono transition-colors ${
              searchOptions.useRegex
                ? "bg-accent/20 text-accent"
                : "text-text-muted hover:text-text"
            }`}
            title={t("terminal.searchRegex")}
          >
            .*
          </button>
          {searchQuery && (
            <span className="text-text-muted text-[10px] shrink-0">
              {t("terminal.searchMatchCount", {
                matched: filteredPackets.length,
                total: packets.length,
              })}
            </span>
          )}
        </div>

        <div className="mx-1 w-px h-4 bg-border" />
        <button
          onClick={toggleAutoScroll}
          className={`px-2 py-0.5 rounded text-xs ${
            autoScroll ? "text-accent" : "text-text-muted"
          }`}
        >
          {t("terminal.autoScroll")}
        </button>
        <button
          onClick={() => clearBuffer(portId)}
          className="px-2 py-0.5 rounded text-xs text-text-muted hover:text-text"
        >
          {t("terminal.clearBuffer")}
        </button>
        {portId && (
          <span className="text-text-muted text-xs shrink-0">
            {t("terminal.packetCount", {
              count: filteredPackets.length,
            })}
          </span>
        )}
      </div>

      {/* Data area */}
      <div ref={parentRef} className="flex-1 overflow-auto">
        {filteredPackets.length === 0 ? (
          <div className="flex items-center justify-center h-full text-text-muted text-sm">
            {searchQuery && packets.length > 0
              ? t("terminal.noSearchResults")
              : t("terminal.noData")}
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
                  <PacketRow
                    packet={packet}
                    displayFormat={displayFormat}
                    showTimestamp={showTimestamp}
                    searchQuery={searchQuery}
                    searchOptions={searchOptions}
                  />
                </div>
              );
            })}
          </div>
        )}
      </div>
    </div>
  );
}
