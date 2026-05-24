import { useCallback, useEffect } from "react";
import { useTranslation } from "react-i18next";
import { tauriApi } from "@/lib/tauri-api";
import { useConnectionStore } from "@/stores/connection";
import { useProtocolStore } from "@/stores/protocol";
import { useScriptStore } from "@/stores/script";

const BAUD_RATES = [9600, 19200, 38400, 57600, 115200, 230400, 460800, 921600];
const DATA_BITS = [5, 6, 7, 8];
const STOP_BITS = [1, 2];
const PARITY_OPTIONS = ["None", "Odd", "Even"];
const FLOW_OPTIONS = ["None", "Software", "Hardware"];

export function ConnectionBar() {
  const { t } = useTranslation();
  const {
    status,
    config,
    availablePorts,
    portName,
    error,
    refreshPorts,
    connect,
    disconnect,
    setConfig,
  } = useConnectionStore();
  const { protocols, loadProtocols } = useProtocolStore();
  const { scripts, loadScriptList } = useScriptStore();

  const isConnected = status === "connected";

  useEffect(() => {
    refreshPorts();
    loadProtocols();
    loadScriptList();
  }, [refreshPorts, loadProtocols, loadScriptList]);

  const handleConnect = useCallback(() => {
    if (isConnected) {
      disconnect();
    } else if (portName) {
      connect(portName);
    }
  }, [isConnected, portName, connect, disconnect]);

  return (
    <div className="flex items-center gap-2 px-3 py-2 bg-base-deep border-b border-border flex-wrap">
      {/* Port */}
      <select
        disabled={isConnected}
        className="w-40"
        value={portName ?? ""}
        onChange={(e) => {
          useConnectionStore.setState({ portName: e.target.value });
        }}
        onFocus={() => !isConnected && refreshPorts()}
      >
        <option value="">{t("common.port")}...</option>
        {availablePorts.map((p) => (
          <option key={p.port_name} value={p.port_name}>
            {p.port_name} {p.is_virtual ? "(virtual)" : ""}
          </option>
        ))}
      </select>

      {/* Baud Rate */}
      <select
        disabled={isConnected}
        value={config.baudrate}
        onChange={(e) => setConfig({ baudrate: Number(e.target.value) })}
      >
        {BAUD_RATES.map((b) => (
          <option key={b} value={b}>
            {b}
          </option>
        ))}
      </select>

      {/* Data Bits */}
      <select
        disabled={isConnected}
        value={config.databits}
        onChange={(e) => setConfig({ databits: Number(e.target.value) })}
      >
        {DATA_BITS.map((d) => (
          <option key={d} value={d}>
            {d}
          </option>
        ))}
      </select>

      {/* Stop Bits */}
      <select
        disabled={isConnected}
        value={config.stopbits}
        onChange={(e) => setConfig({ stopbits: Number(e.target.value) })}
      >
        {STOP_BITS.map((s) => (
          <option key={s} value={s}>
            {s}
          </option>
        ))}
      </select>

      {/* Parity */}
      <select
        disabled={isConnected}
        value={config.parity}
        onChange={(e) => setConfig({ parity: e.target.value })}
      >
        {PARITY_OPTIONS.map((p) => (
          <option key={p} value={p}>
            {p}
          </option>
        ))}
      </select>

      {/* Flow Control */}
      <select
        disabled={isConnected}
        value={config.flow_control}
        onChange={(e) => setConfig({ flow_control: e.target.value })}
      >
        {FLOW_OPTIONS.map((f) => (
          <option key={f} value={f}>
            {f}
          </option>
        ))}
      </select>

      {/* Connect/Disconnect */}
      <button
        onClick={handleConnect}
        disabled={status === "connecting"}
        className={`px-3 py-1 rounded text-xs font-medium transition-colors ${
          isConnected
            ? "bg-danger/20 text-danger hover:bg-danger/30"
            : "bg-success/20 text-success hover:bg-success/30"
        }`}
      >
        {isConnected ? t("common.disconnect") : t("common.connect")}
      </button>

      <div className="mx-2 w-px h-5 bg-border" />

      {/* Protocol */}
      <select
        disabled={!isConnected}
        value={useProtocolStore.getState().activeProtocol ?? ""}
        onChange={async (e) => {
          const portId = useConnectionStore.getState().portId;
          if (portId) {
            await useProtocolStore
              .getState()
              .setActiveProtocol(portId, e.target.value || null);
          }
        }}
      >
        <option value="">
          {t("common.protocol")}: {t("common.none")}
        </option>
        {protocols.map((p) => (
          <option key={p.name} value={p.name}>
            {p.name}
          </option>
        ))}
      </select>

      {/* Script */}
      <select
        disabled={!isConnected}
        onChange={async (e) => {
          const portId = useConnectionStore.getState().portId;
          if (!portId) return;
          if (e.target.value) {
            const script = scripts.find((s) => s.name === e.target.value);
            if (script) {
              await useConnectionStore.getState().setError(null);
              // Note: we'd need the script content to attach
            }
          } else {
            await tauriApi.detachScript(portId);
          }
        }}
      >
        <option value="">
          {t("common.script")}: {t("common.none")}
        </option>
        {scripts.map((s) => (
          <option key={s.name} value={s.name}>
            {s.name}
          </option>
        ))}
      </select>

      {/* Error */}
      {error && <span className="text-danger text-xs ml-auto">{error}</span>}
    </div>
  );
}
