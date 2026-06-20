import { useTranslation } from "react-i18next";
import { formatBytes } from "@/lib/utils";
import { useConnectionStore } from "@/stores/connection";
import { useDataStore } from "@/stores/data";
import { useScriptStore } from "@/stores/script";

export function StatusBar() {
  const { t } = useTranslation();
  const activeEntry = useConnectionStore((s) =>
    s.connections.find((c) => c.portId === s.activePortId),
  );
  const status = activeEntry?.status ?? "disconnected";
  const portName = activeEntry?.portName;
  const config = activeEntry?.config;
  const packets = useDataStore((s) => s.packets);
  const activeScript = useScriptStore((s) => s.activeScript);

  const rxBytes = packets.reduce(
    (sum, p) =>
      p.direction === "rx"
        ? sum + (Array.isArray(p.data) ? p.data.length : 0)
        : sum,
    0,
  );
  const txBytes = packets.reduce(
    (sum, p) =>
      p.direction === "tx"
        ? sum + (Array.isArray(p.data) ? p.data.length : 0)
        : sum,
    0,
  );

  return (
    <footer className="flex items-center h-8 px-3 bg-base-deep border-t border-border text-text-muted text-xs shrink-0">
      <div className="flex items-center gap-1.5 flex-1">
        <span
          className={`w-2 h-2 rounded-full ${
            status === "connected"
              ? "bg-success"
              : status === "error"
                ? "bg-danger"
                : "bg-text-muted"
          }`}
        />
        <span>
          {status === "connected" && portName
            ? t("statusbar.connectedInfo", {
                port: portName,
                baud: config?.baudrate,
              })
            : t("statusbar.noConnection")}
        </span>
      </div>
      {activeScript && (
        <div className="flex items-center gap-2 px-4">
          <span className="text-accent">{activeScript}</span>
        </div>
      )}
      <div className="flex items-center gap-3">
        <span>RX: {formatBytes(rxBytes)}</span>
        <span>TX: {formatBytes(txBytes)}</span>
      </div>
    </footer>
  );
}
