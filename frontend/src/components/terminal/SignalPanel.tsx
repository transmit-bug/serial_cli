import { useCallback, useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { tauriApi } from "@/lib/tauri-api";
import type { SignalStatus } from "@/types";

function SignalChip({
  label,
  value,
  togglable,
  onToggle,
}: {
  label: string;
  value: boolean | null;
  togglable?: boolean;
  onToggle?: () => void;
}) {
  return (
    <div className="flex items-center gap-1.5">
      <span
        className={`w-2 h-2 rounded-full ${
          value === null
            ? "bg-text-muted/40"
            : value
              ? "bg-success"
              : "bg-danger/70"
        }`}
      />
      <span className="font-mono text-[10px] text-text-muted">{label}</span>
      <span className="font-mono text-[10px] w-6">
        {value === null ? "—" : value ? "ON" : "OFF"}
      </span>
      {togglable && (
        <button
          onClick={onToggle}
          className="px-1.5 py-0.5 rounded text-[10px] font-mono border border-border hover:bg-surface transition-colors"
          title={`Toggle ${label}`}
        >
          Toggle
        </button>
      )}
    </div>
  );
}

/**
 * Modem signal control panel (RTS/DTR toggle + CTS/DSR readback).
 * Shown in the Terminal workbench for the active port.
 */
export function SignalPanel({ portId }: { portId: string | null }) {
  const { t } = useTranslation();
  const [signals, setSignals] = useState<SignalStatus | null>(null);

  useEffect(() => {
    if (!portId) {
      setSignals(null);
      return;
    }
    let cancelled = false;
    const poll = async () => {
      try {
        const s = await tauriApi.getSignals(portId);
        if (!cancelled) setSignals(s);
      } catch {
        // Port closed or signal control unavailable — leave last state
      }
    };
    poll();
    const timer = setInterval(poll, 1000);
    return () => {
      cancelled = true;
      clearInterval(timer);
    };
  }, [portId]);

  const toggle = useCallback(
    async (signal: "dtr" | "rts", current: boolean) => {
      if (!portId) return;
      try {
        const next =
          signal === "dtr"
            ? await tauriApi.setDtr(portId, !current)
            : await tauriApi.setRts(portId, !current);
        setSignals(next);
      } catch {
        // Ignore — polling will refresh
      }
    },
    [portId],
  );

  if (!portId || !signals) return null;

  return (
    <div className="flex items-center gap-4 px-3 py-1 border-b border-border bg-base-deep text-xs flex-wrap">
      <span className="text-[10px] uppercase tracking-wider text-text-muted">
        {t("terminal.signals")}
      </span>
      <SignalChip
        label="DTR"
        value={signals.dtr}
        togglable
        onToggle={() => toggle("dtr", signals.dtr)}
      />
      <SignalChip
        label="RTS"
        value={signals.rts}
        togglable
        onToggle={() => toggle("rts", signals.rts)}
      />
      <SignalChip label="CTS" value={signals.cts} />
      <SignalChip label="DSR" value={signals.dsr} />
      {signals.platform && (
        <span className="ml-auto text-[10px] text-text-muted/60">
          {signals.platform}
        </span>
      )}
    </div>
  );
}
