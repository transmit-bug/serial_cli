import { useCallback, useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { RefreshCw } from "lucide-react";
import { useLogStore } from "@/stores/log";

export function RightPanelLog() {
  const { t } = useTranslation();
  const lines = useLogStore((s) => s.lines);
  const loading = useLogStore((s) => s.loading);
  const autoRefresh = useLogStore((s) => s.autoRefresh);
  const filter = useLogStore((s) => s.filter);
  const loadLogs = useLogStore((s) => s.loadLogs);
  const setAutoRefresh = useLogStore((s) => s.setAutoRefresh);
  const setFilter = useLogStore((s) => s.setFilter);
  const clearLogs = useLogStore((s) => s.clearLogs);

  const scrollRef = useRef<HTMLDivElement>(null);
  const [autoScroll, setAutoScroll] = useState(true);

  // Initial load
  useEffect(() => {
    loadLogs();
  }, []);

  // Auto-refresh interval
  useEffect(() => {
    if (!autoRefresh) return;
    const timer = setInterval(loadLogs, 2000);
    return () => clearInterval(timer);
  }, [autoRefresh]);

  // Auto-scroll on new logs
  useEffect(() => {
    if (autoScroll && scrollRef.current) {
      scrollRef.current.scrollTop = scrollRef.current.scrollHeight;
    }
  }, [lines, autoScroll]);

  const handleScroll = useCallback(() => {
    const el = scrollRef.current;
    if (!el) return;
    const atBottom = el.scrollHeight - el.scrollTop - el.clientHeight < 30;
    setAutoScroll(atBottom);
  }, []);

  const filtered = filter
    ? lines.filter((l) => l.toLowerCase().includes(filter.toLowerCase()))
    : lines;

  if (loading && lines.length === 0) {
    return (
      <div className="flex flex-col h-full items-center justify-center text-text-muted text-xs">
        {t("log.loading")}
      </div>
    );
  }

  return (
    <div className="flex flex-col h-full">
      {/* Controls */}
      <div className="flex items-center gap-2 p-2 border-b border-border shrink-0">
        <input
          className="flex-1 h-6 text-xs rounded border border-border bg-transparent px-2"
          placeholder={t("log.search")}
          value={filter}
          onChange={(e) => setFilter(e.target.value)}
        />
        <button
          className={`px-2 py-0.5 text-[10px] rounded transition-colors ${
            autoRefresh
              ? "bg-accent/20 text-accent"
              : "text-text-muted hover:text-text"
          }`}
          onClick={() => setAutoRefresh(!autoRefresh)}
          title={t("log.autoRefresh")}
        >
          {autoRefresh ? "⏸" : "▶"}
        </button>
        <button
          className="px-2 py-0.5 text-[10px] rounded text-text-muted hover:text-text transition-colors"
          onClick={loadLogs}
          title={t("log.refresh")}
        >
          <RefreshCw size={10} />
        </button>
        <button
          className="px-2 py-0.5 text-[10px] rounded text-text-muted hover:text-danger transition-colors"
          onClick={clearLogs}
          title={t("log.clear")}
        >
          ✕
        </button>
      </div>

      {/* Log lines */}
      <div
        ref={scrollRef}
        className="flex-1 overflow-y-auto font-mono text-[11px] leading-tight"
        onScroll={handleScroll}
      >
        {filtered.length === 0 ? (
          <div className="p-3 text-center text-text-muted text-xs">
            {lines.length === 0 ? t("log.empty") : t("log.noMatch")}
          </div>
        ) : (
          filtered.map((line, i) => (
            <div
              key={i}
              className={`px-2 py-0.5 ${
                line.includes("ERROR") || line.includes("error")
                  ? "text-danger/80"
                  : line.includes("WARN") || line.includes("warn")
                    ? "text-warning/80"
                    : line.includes("INFO")
                      ? "text-text"
                      : "text-text-muted"
              }`}
            >
              {line}
            </div>
          ))
        )}
      </div>

      {/* Footer */}
      <div className="flex items-center justify-between px-2 py-1 border-t border-border text-[10px] text-text-muted shrink-0">
        <span>
          {filtered.length} / {lines.length} {t("log.lines")}
        </span>
        <span>{autoScroll ? t("log.autoScrollOn") : ""}</span>
      </div>
    </div>
  );
}
