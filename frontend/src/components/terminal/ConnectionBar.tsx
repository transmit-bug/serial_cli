import { useCallback, useEffect } from "react";
import { useTranslation } from "react-i18next";
import { tauriApi } from "@/lib/tauri-api";
import { useConnectionStore } from "@/stores/connection";
import { usePresetsStore } from "@/stores/presets";
import { useProtocolStore } from "@/stores/protocol";
import { useScriptStore } from "@/stores/script";
import type { ConnectionPreset } from "@/types";

const BAUD_RATES = [9600, 19200, 38400, 57600, 115200, 230400, 460800, 921600];
const DATA_BITS = [5, 6, 7, 8];
const STOP_BITS = [1, 2];
const PARITY_OPTIONS = ["None", "Odd", "Even"];
const FLOW_OPTIONS = ["None", "Software", "Hardware"];

export function ConnectionBar() {
  const { t } = useTranslation();
  const {
    availablePorts,
    connections,
    activePortId,
    pendingPort,
    defaultConfig,
    refreshPorts,
    connect,
    disconnect,
    disconnectAll,
    setPendingPort,
    setDefaultConfig,
  } = useConnectionStore();
  const { protocols, loadProtocols } = useProtocolStore();
  const { scripts, loadScriptList } = useScriptStore();
  const { presets, loadPresets } = usePresetsStore();

  useEffect(() => {
    refreshPorts();
    loadProtocols();
    loadScriptList();
    loadPresets();
  }, [refreshPorts, loadProtocols, loadScriptList, loadPresets]);

  const activeEntry = connections.find((c) => c.portId === activePortId);
  const isConnected = activeEntry?.status === "connected";
  const hasAnyConnected = connections.some((c) => c.status === "connected");

  const handleConnect = useCallback(() => {
    if (isConnected && activePortId) {
      disconnect(activePortId);
    } else if (pendingPort) {
      connect(pendingPort);
    }
  }, [isConnected, activePortId, pendingPort, connect, disconnect]);

  const handleApplyPreset = useCallback(
    (preset: ConnectionPreset) => {
      setDefaultConfig({
        baudrate: preset.baudrate,
        databits: preset.databits,
        stopbits: preset.stopbits,
        parity: preset.parity,
        flow_control: preset.flow_control,
        timeout_ms: preset.timeout_ms,
      });
      if (preset.port_name) {
        setPendingPort(preset.port_name);
      }
    },
    [setDefaultConfig, setPendingPort],
  );

  return (
    <div className="flex items-center gap-2 px-3 py-2 bg-base-deep border-b border-border flex-wrap">
      {/* Preset Quick Apply */}
      {presets.length > 0 && !hasAnyConnected && (
        <select
          className="w-36"
          value=""
          onChange={(e) => {
            if (e.target.value) {
              const idx = Number(e.target.value);
              handleApplyPreset(presets[idx]);
              e.target.value = "";
            }
          }}
        >
          <option value="">{t("presets.apply")}...</option>
          {presets.map((p, i) => (
            <option key={p.name} value={i}>
              {p.name} ({p.baudrate})
            </option>
          ))}
        </select>
      )}

      {/* Port selector */}
      {!isConnected && (
        <select
          className="w-40"
          value={pendingPort ?? ""}
          onChange={(e) => setPendingPort(e.target.value || null)}
          onFocus={() => refreshPorts()}
        >
          <option value="">{t("common.port")}...</option>
          {availablePorts
            .filter(
              (p) =>
                !connections.some(
                  (c) => c.portName === p.port_name && c.status === "connected",
                ),
            )
            .map((p) => (
              <option key={p.port_name} value={p.port_name}>
                {p.port_name} {p.is_virtual ? "(virtual)" : ""}
              </option>
            ))}
        </select>
      )}

      {/* Config controls */}
      {!isConnected && (
        <>
          <select
            value={defaultConfig.baudrate}
            onChange={(e) =>
              setDefaultConfig({ baudrate: Number(e.target.value) })
            }
          >
            {BAUD_RATES.map((b) => (
              <option key={b} value={b}>
                {b}
              </option>
            ))}
          </select>

          <select
            value={defaultConfig.databits}
            onChange={(e) =>
              setDefaultConfig({ databits: Number(e.target.value) })
            }
          >
            {DATA_BITS.map((d) => (
              <option key={d} value={d}>
                {d}
              </option>
            ))}
          </select>

          <select
            value={defaultConfig.stopbits}
            onChange={(e) =>
              setDefaultConfig({ stopbits: Number(e.target.value) })
            }
          >
            {STOP_BITS.map((s) => (
              <option key={s} value={s}>
                {s}
              </option>
            ))}
          </select>

          <select
            value={defaultConfig.parity}
            onChange={(e) => setDefaultConfig({ parity: e.target.value })}
          >
            {PARITY_OPTIONS.map((p) => (
              <option key={p} value={p}>
                {p}
              </option>
            ))}
          </select>

          <select
            value={defaultConfig.flow_control}
            onChange={(e) => setDefaultConfig({ flow_control: e.target.value })}
          >
            {FLOW_OPTIONS.map((f) => (
              <option key={f} value={f}>
                {f}
              </option>
            ))}
          </select>
        </>
      )}

      {/* Connect/Disconnect */}
      <button
        onClick={handleConnect}
        disabled={!pendingPort && !isConnected}
        className={`px-3 py-1 rounded text-xs font-medium transition-colors ${
          isConnected
            ? "bg-danger/20 text-danger hover:bg-danger/30"
            : "bg-success/20 text-success hover:bg-success/30 disabled:opacity-50"
        }`}
      >
        {isConnected ? t("common.disconnect") : t("common.connect")}
      </button>

      {/* Disconnect All */}
      {hasAnyConnected && connections.length > 1 && (
        <button
          onClick={disconnectAll}
          className="px-2 py-1 rounded text-xs text-danger/70 hover:text-danger transition-colors"
        >
          {t("common.disconnect")} All
        </button>
      )}

      <div className="mx-2 w-px h-5 bg-border" />

      {/* Protocol */}
      <select
        disabled={!isConnected}
        value={
          activeEntry ? (useProtocolStore.getState().activeProtocol ?? "") : ""
        }
        onChange={async (e) => {
          if (activePortId) {
            await useProtocolStore
              .getState()
              .setActiveProtocol(activePortId, e.target.value || null);
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
          if (!activePortId) return;
          if (e.target.value) {
            const script = scripts.find((s) => s.name === e.target.value);
            if (script) {
              await useConnectionStore
                .getState()
                .setPortError(activePortId, null);
            }
          } else {
            await tauriApi.detachScript(activePortId);
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

      {/* Status */}
      {isConnected && activeEntry && (
        <span className="text-success text-xs ml-auto">
          {t("statusbar.connectedInfo", {
            port: activeEntry.portName,
            baud: activeEntry.config.baudrate,
          })}
        </span>
      )}
      {!isConnected && !hasAnyConnected && (
        <span className="text-text-muted text-xs ml-auto">
          {t("statusbar.noConnection")}
        </span>
      )}
    </div>
  );
}
