import { useCallback, useEffect } from "react";
import { useTranslation } from "react-i18next";
import { Button } from "@/components/ui/button";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { useConnectionStore } from "@/stores/connection";
import { usePresetsStore } from "@/stores/presets";
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
    serverOccupiedPorts,
    refreshPorts,
    connect,
    disconnect,
    disconnectAll,
    setPendingPort,
    setDefaultConfig,
  } = useConnectionStore();
  const { scripts, loadScripts } = useScriptStore();
  const { presets, loadPresets } = usePresetsStore();

  useEffect(() => {
    refreshPorts();
    loadScripts();
    loadPresets();
  }, [refreshPorts, loadScripts, loadPresets]);

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
        <Select
          onValueChange={(value) => {
            if (value) {
              const idx = Number(value);
              handleApplyPreset(presets[idx]);
            }
          }}
        >
          <SelectTrigger className="w-36">
            <SelectValue placeholder={`${t("presets.apply")}...`} />
          </SelectTrigger>
          <SelectContent>
            {presets.map((p, i) => (
              <SelectItem key={p.name} value={String(i)}>
                {p.name} ({p.baudrate})
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
      )}

      {/* Port selector */}
      {!isConnected && (
        <Select
          value={pendingPort ?? ""}
          onValueChange={(value) => setPendingPort(value || null)}
        >
          <SelectTrigger className="w-40" onFocus={() => refreshPorts()}>
            <SelectValue placeholder={`${t("common.port")}...`} />
          </SelectTrigger>
          <SelectContent>
            {availablePorts
              .filter(
                (p) =>
                  !connections.some(
                    (c) =>
                      c.portName === p.port_name && c.status === "connected",
                  ),
              )
              .map((p) => {
                const isServerOccupied = serverOccupiedPorts.has(p.port_name);
                return (
                  <SelectItem
                    key={p.port_name}
                    value={p.port_name}
                    disabled={isServerOccupied}
                    className={isServerOccupied ? "opacity-50" : ""}
                  >
                    {p.port_name}
                    {p.is_virtual ? " (virtual)" : ""}
                    {isServerOccupied ? " 🔒" : ""}
                  </SelectItem>
                );
              })}
          </SelectContent>
        </Select>
      )}

      {/* Config controls */}
      {!isConnected && (
        <>
          <Select
            value={String(defaultConfig.baudrate)}
            onValueChange={(value) =>
              setDefaultConfig({ baudrate: Number(value) })
            }
          >
            <SelectTrigger className="w-24">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              {BAUD_RATES.map((b) => (
                <SelectItem key={b} value={String(b)}>
                  {b}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>

          <Select
            value={String(defaultConfig.databits)}
            onValueChange={(value) =>
              setDefaultConfig({ databits: Number(value) })
            }
          >
            <SelectTrigger className="w-16">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              {DATA_BITS.map((d) => (
                <SelectItem key={d} value={String(d)}>
                  {d}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>

          <Select
            value={String(defaultConfig.stopbits)}
            onValueChange={(value) =>
              setDefaultConfig({ stopbits: Number(value) })
            }
          >
            <SelectTrigger className="w-16">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              {STOP_BITS.map((s) => (
                <SelectItem key={s} value={String(s)}>
                  {s}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>

          <Select
            value={defaultConfig.parity}
            onValueChange={(value) => setDefaultConfig({ parity: value })}
          >
            <SelectTrigger className="w-20">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              {PARITY_OPTIONS.map((p) => (
                <SelectItem key={p} value={p}>
                  {p}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>

          <Select
            value={defaultConfig.flow_control}
            onValueChange={(value) => setDefaultConfig({ flow_control: value })}
          >
            <SelectTrigger className="w-24">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              {FLOW_OPTIONS.map((f) => (
                <SelectItem key={f} value={f}>
                  {f}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </>
      )}

      {/* Connect/Disconnect */}
      <Button
        variant={isConnected ? "destructive" : "default"}
        onClick={handleConnect}
        disabled={!pendingPort && !isConnected}
      >
        {isConnected ? t("common.disconnect") : t("common.connect")}
      </Button>

      {/* Disconnect All */}
      {hasAnyConnected && connections.length > 1 && (
        <Button variant="ghost" onClick={disconnectAll}>
          {t("common.disconnect")} All
        </Button>
      )}

      <div className="mx-2 w-px h-5 bg-border" />

      {/* Script/Protocol selector */}
      <Select
        disabled={!isConnected}
        value={
          activeEntry ? (useScriptStore.getState().activeScript ?? "") : ""
        }
        onValueChange={async (value) => {
          if (activePortId) {
            await useScriptStore
              .getState()
              .setActiveScript(activePortId, value || null);
          }
        }}
      >
        <SelectTrigger className="w-32">
          <SelectValue
            placeholder={`${t("common.script")}: ${t("common.none")}`}
          />
        </SelectTrigger>
        <SelectContent>
          {scripts.map((s) => (
            <SelectItem key={s.name} value={s.name}>
              {s.name}
            </SelectItem>
          ))}
        </SelectContent>
      </Select>
    </div>
  );
}
