import type { Handler } from "../interceptor";

export const configHandlers: Record<string, Handler> = {
  get_config: (_args, state) => state.config,

  update_config: ({ config }, state) => {
    // biome-ignore lint/suspicious/noExplicitAny: mock handler receives generic Record
    state.config = { ...state.config, ...(config as any) };
  },

  reset_config: (_args, state) => {
    // Re-create default config by resetting individual fields
    state.config = {
      serial: {
        defaultBaudrate: 115200,
        databits: 8,
        stopbits: 1,
        parity: "none",
        timeoutMs: 1000,
      },
      logging: { level: "info", format: "json", file: "" },
      lua: { memory_limit_mb: 64, timeout_seconds: 5, enable_sandbox: true },
      output: { json_pretty: true, show_timestamp: true },
      protocols: { hotReload: false, customDir: "" },
      virtual_ports: { backend: "pty", monitor: false },
      display: {
        theme: "dark",
        maxPackets: 10000,
        format: "hex",
        showTimestamp: true,
      },
    };
  },

  get_connection_presets: (_args, state) => state.presets,

  save_connection_presets: ({ presets }, state) => {
    state.presets = presets as typeof state.presets;
  },

  delete_connection_preset: ({ name }, state) => {
    state.presets = state.presets.filter((p) => p.name !== name);
  },
};
