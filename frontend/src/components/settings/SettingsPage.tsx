import { useCallback, useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { toast } from "sonner";
import { useSettingsStore } from "@/stores/settings";
import type { ConfigData } from "@/types";

const SETTINGS_TABS = [
  "serial",
  "logging",
  "luaEngine",
  "output",
  "protocolsTab",
  "display",
] as const;
type SettingsTab = (typeof SETTINGS_TABS)[number];

export function SettingsPage() {
  const { t } = useTranslation();
  const { config, loading, loadConfig, updateConfig, resetConfig } =
    useSettingsStore();
  const [activeTab, setActiveTab] = useState<SettingsTab>("serial");

  useEffect(() => {
    loadConfig();
  }, [loadConfig]);

  const handleSave = useCallback(async () => {
    if (!config) return;
    try {
      await updateConfig(config);
      toast.success(t("settings.save"));
    } catch (e) {
      toast.error(String(e));
    }
  }, [config, updateConfig, t]);

  const handleReset = useCallback(async () => {
    try {
      await resetConfig();
      toast.success(t("settings.reset"));
    } catch (e) {
      toast.error(String(e));
    }
  }, [resetConfig, t]);

  const update = useCallback(
    <K extends keyof ConfigData>(
      section: K,
      field: keyof ConfigData[K],
      value: ConfigData[K][typeof field],
    ) => {
      if (!config) return;
      updateConfig({
        ...config,
        [section]: { ...config[section], [field]: value },
      });
    },
    [config, updateConfig],
  );

  if (loading || !config) {
    return (
      <div className="flex items-center justify-center h-full text-text-muted">
        {t("common.loading")}
      </div>
    );
  }

  return (
    <div className="flex h-full">
      {/* Tab navigation */}
      <nav className="w-40 border-r border-border py-2 shrink-0">
        {SETTINGS_TABS.map((tab) => (
          <button
            key={tab}
            onClick={() => setActiveTab(tab)}
            className={`w-full text-left px-4 py-2 text-xs ${
              activeTab === tab
                ? "bg-accent/10 text-accent"
                : "text-text-muted hover:text-text hover:bg-surface/30"
            }`}
          >
            {t(`settings.${tab}`)}
          </button>
        ))}
      </nav>

      {/* Content */}
      <div className="flex-1 p-6 overflow-y-auto">
        {activeTab === "serial" && (
          <div className="space-y-4 max-w-md">
            <h2 className="text-sm font-semibold mb-4">
              {t("settings.serialDefaults")}
            </h2>
            <FieldRow label={t("common.baudRate")}>
              <select
                value={config.serial.defaultBaudrate}
                onChange={(e) =>
                  update("serial", "defaultBaudrate", Number(e.target.value))
                }
              >
                {[
                  9600, 19200, 38400, 57600, 115200, 230400, 460800, 921600,
                ].map((b) => (
                  <option key={b} value={b}>
                    {b}
                  </option>
                ))}
              </select>
            </FieldRow>
            <FieldRow label={t("common.dataBits")}>
              <select
                value={config.serial.databits}
                onChange={(e) =>
                  update("serial", "databits", Number(e.target.value))
                }
              >
                {[5, 6, 7, 8].map((d) => (
                  <option key={d} value={d}>
                    {d}
                  </option>
                ))}
              </select>
            </FieldRow>
            <FieldRow label={t("common.stopBits")}>
              <select
                value={config.serial.stopbits}
                onChange={(e) =>
                  update("serial", "stopbits", Number(e.target.value))
                }
              >
                {[1, 2].map((s) => (
                  <option key={s} value={s}>
                    {s}
                  </option>
                ))}
              </select>
            </FieldRow>
            <FieldRow label={t("common.parity")}>
              <select
                value={config.serial.parity}
                onChange={(e) => update("serial", "parity", e.target.value)}
              >
                {["None", "Odd", "Even"].map((p) => (
                  <option key={p} value={p}>
                    {p}
                  </option>
                ))}
              </select>
            </FieldRow>
            <FieldRow label={t("common.timeout")}>
              <input
                type="number"
                value={config.serial.timeoutMs}
                onChange={(e) =>
                  update("serial", "timeoutMs", Number(e.target.value))
                }
                className="w-24"
              />
              <span className="text-xs text-text-muted ml-1">ms</span>
            </FieldRow>
          </div>
        )}

        {activeTab === "logging" && (
          <div className="space-y-4 max-w-md">
            <h2 className="text-sm font-semibold mb-4">
              {t("settings.logging")}
            </h2>
            <FieldRow label={t("settings.logLevel")}>
              <select
                value={config.logging.level}
                onChange={(e) => update("logging", "level", e.target.value)}
              >
                {["trace", "debug", "info", "warn", "error"].map((l) => (
                  <option key={l} value={l}>
                    {l}
                  </option>
                ))}
              </select>
            </FieldRow>
            <FieldRow label={t("settings.logFormat")}>
              <select
                value={config.logging.format}
                onChange={(e) => update("logging", "format", e.target.value)}
              >
                {["text", "json"].map((f) => (
                  <option key={f} value={f}>
                    {f}
                  </option>
                ))}
              </select>
            </FieldRow>
            <FieldRow label={t("settings.logFile")}>
              <input
                type="text"
                value={config.logging.file}
                onChange={(e) => update("logging", "file", e.target.value)}
                className="w-64"
              />
            </FieldRow>
          </div>
        )}

        {activeTab === "luaEngine" && (
          <div className="space-y-4 max-w-md">
            <h2 className="text-sm font-semibold mb-4">
              {t("settings.luaEngine")}
            </h2>
            <FieldRow label={t("settings.memoryLimit")}>
              <input
                type="number"
                value={config.lua.memory_limit_mb}
                onChange={(e) =>
                  update("lua", "memory_limit_mb", Number(e.target.value))
                }
                className="w-24"
              />
              <span className="text-xs text-text-muted ml-1">MB</span>
            </FieldRow>
            <FieldRow label={t("settings.timeoutSeconds")}>
              <input
                type="number"
                value={config.lua.timeout_seconds}
                onChange={(e) =>
                  update("lua", "timeout_seconds", Number(e.target.value))
                }
                className="w-24"
              />
              <span className="text-xs text-text-muted ml-1">s</span>
            </FieldRow>
            <FieldRow label={t("settings.sandbox")}>
              <input
                type="checkbox"
                checked={config.lua.enable_sandbox}
                onChange={(e) =>
                  update("lua", "enable_sandbox", e.target.checked)
                }
              />
            </FieldRow>
          </div>
        )}

        {activeTab === "output" && (
          <div className="space-y-4 max-w-md">
            <h2 className="text-sm font-semibold mb-4">
              {t("settings.output")}
            </h2>
            <FieldRow label={t("settings.jsonPretty")}>
              <input
                type="checkbox"
                checked={config.output.json_pretty}
                onChange={(e) =>
                  update("output", "json_pretty", e.target.checked)
                }
              />
            </FieldRow>
            <FieldRow label={t("settings.showTimestamp")}>
              <input
                type="checkbox"
                checked={config.output.show_timestamp}
                onChange={(e) =>
                  update("output", "show_timestamp", e.target.checked)
                }
              />
            </FieldRow>
          </div>
        )}

        {activeTab === "protocolsTab" && (
          <div className="space-y-4 max-w-md">
            <h2 className="text-sm font-semibold mb-4">
              {t("settings.protocolsTab")}
            </h2>
            <FieldRow label={t("settings.hotReload")}>
              <input
                type="checkbox"
                checked={config.protocols.hotReload}
                onChange={(e) =>
                  update("protocols", "hotReload", e.target.checked)
                }
              />
            </FieldRow>
          </div>
        )}

        {activeTab === "display" && (
          <div className="space-y-4 max-w-md">
            <h2 className="text-sm font-semibold mb-4">
              {t("settings.display")}
            </h2>
            <FieldRow label={t("settings.maxPackets")}>
              <input
                type="number"
                value={config.display.maxPackets}
                onChange={(e) =>
                  update("display", "maxPackets", Number(e.target.value))
                }
                className="w-24"
              />
            </FieldRow>
            <FieldRow label={t("settings.showTimestamp")}>
              <input
                type="checkbox"
                checked={config.display.showTimestamp}
                onChange={(e) =>
                  update("display", "showTimestamp", e.target.checked)
                }
              />
            </FieldRow>
          </div>
        )}

        <div className="flex gap-2 mt-8 pt-4 border-t border-border">
          <button
            onClick={handleSave}
            className="px-4 py-1.5 rounded text-xs bg-accent/20 text-accent hover:bg-accent/30"
          >
            {t("settings.save")}
          </button>
          <button
            onClick={handleReset}
            className="px-4 py-1.5 rounded text-xs text-text-muted hover:text-text"
          >
            {t("settings.reset")}
          </button>
        </div>
      </div>
    </div>
  );
}

function FieldRow({
  label,
  children,
}: {
  label: string;
  children: React.ReactNode;
}) {
  return (
    <div className="flex items-center gap-3">
      <label className="text-xs text-text-secondary w-28 shrink-0">
        {label}
      </label>
      <div className="flex items-center gap-1">{children}</div>
    </div>
  );
}
