import { beforeEach, describe, expect, it, vi } from "vitest";
import { tauriApi } from "@/lib/tauri-api";
import { useSettingsStore } from "@/stores/settings";

vi.mock("@/lib/tauri-api", () => ({
  tauriApi: {
    getConfig: vi.fn().mockResolvedValue(null),
    updateConfig: vi.fn().mockResolvedValue(undefined),
    resetConfig: vi.fn().mockResolvedValue(undefined),
  },
}));

const sampleConfig = {
  serial: {
    defaultBaudrate: 115200,
    databits: 8,
    stopbits: 1,
    parity: "None",
    timeoutMs: 1000,
  },
  logging: { level: "info", format: "text", file: "serial-cli.log" },
  lua: { memory_limit_mb: 64, timeout_seconds: 30, enable_sandbox: true },
  task: { max_concurrent: 4, default_timeout_seconds: 60 },
  output: { json_pretty: true, show_timestamp: true },
  protocols: { hotReload: true, customDir: "" },
  virtual_ports: { backend: "pty", monitor: false },
  display: {
    theme: "dark",
    maxPackets: 10000,
    format: "mixed",
    showTimestamp: true,
  },
};

describe("useSettingsStore", () => {
  beforeEach(() => {
    useSettingsStore.setState({ config: null, loading: false });
    vi.clearAllMocks();
  });

  // --- loadConfig ---

  describe("loadConfig", () => {
    it("fetches config from backend", async () => {
      vi.mocked(tauriApi.getConfig).mockResolvedValueOnce(sampleConfig);

      await useSettingsStore.getState().loadConfig();

      expect(useSettingsStore.getState().config).toEqual(sampleConfig);
      expect(useSettingsStore.getState().loading).toBe(false);
    });

    it("sets loading to true then false", async () => {
      vi.mocked(tauriApi.getConfig).mockResolvedValueOnce(sampleConfig);

      const promise = useSettingsStore.getState().loadConfig();
      expect(useSettingsStore.getState().loading).toBe(true);

      await promise;
      expect(useSettingsStore.getState().loading).toBe(false);
    });

    it("sets loading false on error", async () => {
      vi.mocked(tauriApi.getConfig).mockRejectedValueOnce(new Error("fail"));

      await useSettingsStore.getState().loadConfig();

      expect(useSettingsStore.getState().loading).toBe(false);
      expect(useSettingsStore.getState().config).toBeNull();
    });
  });

  // --- updateConfig ---

  describe("updateConfig", () => {
    it("persists config to backend and updates state", async () => {
      await useSettingsStore.getState().updateConfig(sampleConfig);

      expect(tauriApi.updateConfig).toHaveBeenCalledWith(sampleConfig);
      expect(useSettingsStore.getState().config).toEqual(sampleConfig);
    });
  });

  // --- resetConfig ---

  describe("resetConfig", () => {
    it("calls resetConfig then reloads config", async () => {
      vi.mocked(tauriApi.getConfig).mockResolvedValueOnce(sampleConfig);

      await useSettingsStore.getState().resetConfig();

      expect(tauriApi.resetConfig).toHaveBeenCalled();
      expect(tauriApi.getConfig).toHaveBeenCalled();
      expect(useSettingsStore.getState().config).toEqual(sampleConfig);
    });
  });
});
