import { useCallback, useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { toast } from "sonner";
import { useConnectionStore } from "@/stores/connection";
import { usePresetsStore } from "@/stores/presets";
import { useSettingsStore } from "@/stores/settings";
import { tauriApi } from "@/lib/tauri-api";
import type { ConfigData, ConnectionPreset } from "@/types";

const SETTINGS_TABS = [
  "serial",
  "logging",
  "luaEngine",
  "output",
  "protocolsTab",
  "display",
  "general",
  "about",
] as const;
type SettingsTab = (typeof SETTINGS_TABS)[number];

export function SettingsPage() {
  const { t, i18n } = useTranslation();
  const { config, loading, loadConfig, updateConfig, resetConfig } =
    useSettingsStore();
  const {
    presets,
    loading: presetsLoading,
    loadPresets,
    addPreset,
    updatePreset,
    deletePreset,
    movePresetUp,
    movePresetDown,
  } = usePresetsStore();
  const [activeTab, setActiveTab] = useState<SettingsTab>("serial");
  const [newPresetName, setNewPresetName] = useState("");
  const [editingPreset, setEditingPreset] = useState<ConnectionPreset | null>(
    null,
  );
  const [editingOriginalName, setEditingOriginalName] = useState("");
  const [hotReloadEnabled, setHotReloadEnabled] = useState(false);

  useEffect(() => {
    loadConfig();
    loadPresets();
  }, [loadConfig, loadPresets]);

  useEffect(() => {
    if (activeTab === "luaEngine") {
      tauriApi.getHotReloadStatus().then(setHotReloadEnabled).catch(() => {});
    }
  }, [activeTab]);

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

  const savePreset = useCallback(() => {
    if (!newPresetName.trim() || !config) return;
    const preset: ConnectionPreset = {
      name: newPresetName,
      port_name: "",
      baudrate: config.serial.defaultBaudrate,
      databits: config.serial.databits,
      stopbits: config.serial.stopbits,
      parity: config.serial.parity,
      flow_control: "None",
      timeout_ms: config.serial.timeoutMs,
    };
    addPreset(preset);
    setNewPresetName("");
  }, [newPresetName, config, addPreset]);

  const handleApplyPreset = useCallback(
    (preset: ConnectionPreset) => {
      if (!config) return;
      updateConfig({
        ...config,
        serial: {
          ...config.serial,
          defaultBaudrate: preset.baudrate,
          databits: preset.databits,
          stopbits: preset.stopbits,
          parity: preset.parity,
          timeoutMs: preset.timeout_ms,
        },
      });
      useConnectionStore.setState({ pendingPort: preset.port_name || null });
      toast.success(`Preset "${preset.name}" loaded`);
    },
    [config, updateConfig],
  );

  const handleDeletePreset = useCallback(
    (name: string) => {
      deletePreset(name);
    },
    [deletePreset],
  );

  const handleMoveUp = useCallback(
    (index: number) => {
      movePresetUp(index);
    },
    [movePresetUp],
  );

  const handleMoveDown = useCallback(
    (index: number) => {
      movePresetDown(index);
    },
    [movePresetDown],
  );

  const startEdit = useCallback((preset: ConnectionPreset) => {
    setEditingPreset({ ...preset });
    setEditingOriginalName(preset.name);
  }, []);

  const cancelEdit = useCallback(() => {
    setEditingPreset(null);
    setEditingOriginalName("");
  }, []);

  const saveEdit = useCallback(() => {
    if (!editingPreset) return;
    updatePreset(editingOriginalName, editingPreset);
    setEditingPreset(null);
    setEditingOriginalName("");
  }, [editingPreset, editingOriginalName, updatePreset]);

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
            <FieldRow
              label={t("common.baudRate")}
              desc={t("settings.baudRateDesc")}
            >
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
            <FieldRow
              label={t("common.dataBits")}
              desc={t("settings.dataBitsDesc")}
            >
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
            <FieldRow
              label={t("common.stopBits")}
              desc={t("settings.stopBitsDesc")}
            >
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
            <FieldRow
              label={t("common.parity")}
              desc={t("settings.parityDesc")}
            >
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
            <FieldRow
              label={t("common.timeout")}
              desc={t("settings.timeoutDesc")}
            >
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

            {/* Connection Presets */}
            <div className="pt-4 border-t border-border">
              <h3 className="text-xs font-medium text-text-secondary mb-2">
                {t("settings.connectionPresets")}
              </h3>
              {presetsLoading ? (
                <p className="text-xs text-text-muted">{t("common.loading")}</p>
              ) : presets.length === 0 ? (
                <p className="text-xs text-text-muted">
                  {t("presets.noPresets")}
                </p>
              ) : (
                <div className="space-y-1 mb-2">
                  {presets.map((preset, i) => (
                    <div
                      key={preset.name}
                      className="flex items-center gap-1 px-2 py-1 rounded bg-surface text-xs"
                    >
                      {editingPreset?.name === preset.name ? (
                        <>
                          <input
                            className="w-20 h-5 text-xs rounded border border-border bg-transparent px-1"
                            value={editingPreset.name}
                            onChange={(e) =>
                              setEditingPreset({
                                ...editingPreset,
                                name: e.target.value,
                              })
                            }
                          />
                          <span className="text-text-muted">@</span>
                          <input
                            className="w-20 h-5 text-xs rounded border border-border bg-transparent px-1"
                            value={editingPreset.port_name}
                            onChange={(e) =>
                              setEditingPreset({
                                ...editingPreset,
                                port_name: e.target.value,
                              })
                            }
                            placeholder={t("common.port")}
                          />
                          <select
                            value={editingPreset.baudrate}
                            onChange={(e) =>
                              setEditingPreset({
                                ...editingPreset,
                                baudrate: Number(e.target.value),
                              })
                            }
                            className="w-18 h-5"
                          >
                            {[
                              9600, 19200, 38400, 57600, 115200, 230400, 460800,
                              921600,
                            ].map((b) => (
                              <option key={b} value={b}>
                                {b}
                              </option>
                            ))}
                          </select>
                          <button
                            className="text-success hover:text-success/80"
                            onClick={saveEdit}
                          >
                            {t("common.save")}
                          </button>
                          <button
                            className="text-text-muted hover:text-text"
                            onClick={cancelEdit}
                          >
                            {t("common.cancel")}
                          </button>
                        </>
                      ) : (
                        <>
                          <button
                            className="text-text-muted hover:text-text disabled:opacity-30"
                            disabled={i === 0}
                            onClick={() => handleMoveUp(i)}
                          >
                            ↑
                          </button>
                          <button
                            className="text-text-muted hover:text-text disabled:opacity-30"
                            disabled={i === presets.length - 1}
                            onClick={() => handleMoveDown(i)}
                          >
                            ↓
                          </button>
                          <span className="flex-1 cursor-default">
                            {preset.name}
                          </span>
                          <span className="text-text-muted">
                            {preset.baudrate}
                          </span>
                          <button
                            className="text-accent hover:text-accent-hover"
                            onClick={() => handleApplyPreset(preset)}
                          >
                            {t("common.apply")}
                          </button>
                          <button
                            className="text-text-muted hover:text-text"
                            onClick={() => startEdit(preset)}
                          >
                            {t("common.edit")}
                          </button>
                          <button
                            className="text-danger hover:text-danger/80"
                            onClick={() => handleDeletePreset(preset.name)}
                          >
                            ×
                          </button>
                        </>
                      )}
                    </div>
                  ))}
                </div>
              )}
              <div className="flex gap-2">
                <input
                  className="flex-1 h-6 text-xs rounded border border-border bg-transparent px-2"
                  placeholder={t("settings.presetName")}
                  value={newPresetName}
                  onChange={(e) => setNewPresetName(e.target.value)}
                />
                <button
                  className="px-2 py-1 rounded text-xs bg-accent/20 text-accent hover:bg-accent/30 disabled:opacity-50"
                  disabled={!newPresetName.trim()}
                  onClick={savePreset}
                >
                  {t("common.add")}
                </button>
              </div>
            </div>
          </div>
        )}

        {activeTab === "logging" && (
          <div className="space-y-4 max-w-md">
            <h2 className="text-sm font-semibold mb-4">
              {t("settings.logging")}
            </h2>
            <FieldRow
              label={t("settings.logLevel")}
              desc={t("settings.logLevelDesc")}
            >
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
            <FieldRow
              label={t("settings.logFormat")}
              desc={t("settings.logFormatDesc")}
            >
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
            <FieldRow
              label={t("settings.logFile")}
              desc={t("settings.logFileDesc")}
            >
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
            <FieldRow
              label={t("settings.memoryLimit")}
              desc={t("settings.memoryLimitDesc")}
            >
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
            <FieldRow
              label={t("settings.timeoutSeconds")}
              desc={t("settings.timeoutSecondsDesc")}
            >
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
            <FieldRow
              label={t("settings.sandbox")}
              desc={t("settings.sandboxDesc")}
            >
              <input
                type="checkbox"
                checked={config.lua.enable_sandbox}
                onChange={(e) =>
                  update("lua", "enable_sandbox", e.target.checked)
                }
              />
            </FieldRow>
            <FieldRow
              label={t("settings.hotReload")}
              desc={t("settings.hotReloadDesc")}
            >
              <input
                type="checkbox"
                checked={hotReloadEnabled}
                onChange={async (e) => {
                  const enabled = e.target.checked;
                  try {
                    await tauriApi.setHotReloadEnabled(enabled);
                    setHotReloadEnabled(enabled);
                    toast.success(
                      enabled
                        ? t("settings.hotReloadEnabled")
                        : t("settings.hotReloadDisabled"),
                    );
                  } catch (err) {
                    toast.error(String(err));
                  }
                }}
              />
            </FieldRow>
          </div>
        )}

        {activeTab === "output" && (
          <div className="space-y-4 max-w-md">
            <h2 className="text-sm font-semibold mb-4">
              {t("settings.output")}
            </h2>
            <FieldRow
              label={t("settings.jsonPretty")}
              desc={t("settings.jsonPrettyDesc")}
            >
              <input
                type="checkbox"
                checked={config.output.json_pretty}
                onChange={(e) =>
                  update("output", "json_pretty", e.target.checked)
                }
              />
            </FieldRow>
            <FieldRow
              label={t("settings.showTimestamp")}
              desc={t("settings.showTimestampDesc")}
            >
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
            <FieldRow
              label={t("settings.hotReload")}
              desc={t("settings.hotReloadDesc")}
            >
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
            <FieldRow
              label={t("settings.theme")}
              desc={t("settings.themeDesc")}
            >
              <select
                value={config.display.theme}
                onChange={(e) => update("display", "theme", e.target.value)}
              >
                <option value="dark">Dark</option>
                <option value="light">Light</option>
              </select>
            </FieldRow>
            <FieldRow
              label={t("settings.maxPackets")}
              desc={t("settings.maxPacketsDesc")}
            >
              <input
                type="number"
                value={config.display.maxPackets}
                onChange={(e) =>
                  update("display", "maxPackets", Number(e.target.value))
                }
                className="w-24"
              />
            </FieldRow>
            <FieldRow
              label={t("settings.showTimestamp")}
              desc={t("settings.showTimestampDesc")}
            >
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

        {activeTab === "general" && (
          <div className="space-y-4 max-w-md">
            <h2 className="text-sm font-semibold mb-4">
              {t("settings.general")}
            </h2>
            <FieldRow
              label={t("settings.language")}
              desc={t("settings.languageDesc")}
            >
              <select
                value={i18n.language}
                onChange={(e) => {
                  i18n.changeLanguage(e.target.value);
                }}
              >
                <option value="zh">中文</option>
                <option value="en">English</option>
              </select>
            </FieldRow>
          </div>
        )}

        {activeTab === "about" && (
          <div className="space-y-4 max-w-md">
            <h2 className="text-sm font-semibold mb-4">
              {t("settings.about")}
            </h2>
            <div className="space-y-2 text-sm">
              <div className="flex justify-between">
                <span className="text-text-muted">{t("settings.appName")}</span>
                <span>Serial CLI</span>
              </div>
              <div className="flex justify-between">
                <span className="text-text-muted">{t("settings.version")}</span>
                <span>{import.meta.env.VITE_APP_VERSION ?? "dev"}</span>
              </div>
              <div className="flex justify-between">
                <span className="text-text-muted">
                  {t("settings.description")}
                </span>
                <span className="text-right text-xs text-text-muted">
                  {t("settings.appDescription")}
                </span>
              </div>
            </div>
            <div className="flex gap-3 pt-2">
              <a
                href="https://github.com/pony/serial_cli"
                target="_blank"
                rel="noopener noreferrer"
                className="text-xs text-accent hover:text-accent-hover"
              >
                GitHub
              </a>
              <a
                href="https://github.com/pony/serial_cli/issues"
                target="_blank"
                rel="noopener noreferrer"
                className="text-xs text-accent hover:text-accent-hover"
              >
                {t("settings.reportIssue")}
              </a>
            </div>
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
  desc,
  children,
}: {
  label: string;
  desc?: string;
  children: React.ReactNode;
}) {
  return (
    <div className="flex flex-col gap-1">
      <div className="flex items-center gap-3">
        <label className="text-xs text-text-secondary w-28 shrink-0">
          {label}
        </label>
        <div className="flex items-center gap-1">{children}</div>
      </div>
      {desc && <p className="text-[10px] text-text-muted ml-28">{desc}</p>}
    </div>
  );
}
