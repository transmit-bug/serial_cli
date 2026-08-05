/**
 * Mock entry point — replaces src/lib/tauri-api.ts in mock mode.
 * Exports tauriApi with the exact same shape as the real implementation.
 *
 * Switched via Vite alias: when TAURI_PLATFORM is absent (pnpm dev),
 * all imports of "@/lib/tauri-api" resolve to this file.
 */

import type {
  CapturedPacket,
  ConfigData,
  ConnectionPreset,
  CreateVirtualPortConfig,
  PortInfo,
  PortStatus,
  Script,
  ScriptStatus,
  ScriptValidationResult,
  SerialConfig,
  ServerStatus,
  SignalStatus,
  UiAction,
  UserScriptInfo,
  ValidationError,
  VirtualPortInfo,
  VirtualPortStats,
} from "@/types";
import { configHandlers } from "./handlers/config";
import { exportHandlers } from "./handlers/export";
import { logHandlers } from "./handlers/log";
import { portHandlers } from "./handlers/port";
import { scriptHandlers } from "./handlers/script";
import { serialHandlers } from "./handlers/serial";
import { serialScriptHandlers } from "./handlers/serial-script";
import { serverHandlers } from "./handlers/server";
import { virtualPortHandlers } from "./handlers/virtual-port";
import { dispatch, registerHandlers } from "./interceptor";
import { state } from "./state";

// Register all handlers
registerHandlers(portHandlers);
registerHandlers(serialHandlers);
registerHandlers(scriptHandlers);
registerHandlers(serialScriptHandlers);
registerHandlers(configHandlers);
registerHandlers(serverHandlers);
registerHandlers(virtualPortHandlers);
registerHandlers(exportHandlers);
registerHandlers(logHandlers);

// Helper to call dispatch with proper typing
function call<T>(
  command: string,
  args: Record<string, unknown> = {},
): Promise<T> {
  return dispatch(command, args, state) as Promise<T>;
}

/**
 * Mock tauriApi — same shape as src/lib/tauri-api.ts
 */
export const tauriApi = {
  // Port commands
  listPorts: () => call<PortInfo[]>("list_ports"),
  openPort: (portName: string, config: SerialConfig, isVirtual?: boolean) =>
    call<string>("open_port", { portName, config, isVirtual }),
  closePort: (portId: string) => call<void>("close_port", { portId }),
  getPortStatus: (portId: string) =>
    call<PortStatus>("get_port_status", { portId }),
  checkPortHealth: (portId: string) =>
    call<boolean>("check_port_health", { portId }),

  // Serial commands
  sendData: (portId: string, data: number[]) =>
    call<number>("send_data", { portId, data }),
  startSniffing: (portId: string) => call<void>("start_sniffing", { portId }),
  stopSniffing: (portId: string) => call<void>("stop_sniffing", { portId }),

  // Signal commands
  setDtr: (portId: string, enable: boolean) =>
    call<SignalStatus>("set_dtr", { portId, enable }),
  setRts: (portId: string, enable: boolean) =>
    call<SignalStatus>("set_rts", { portId, enable }),
  getSignals: (portId: string) => call<SignalStatus>("get_signals", { portId }),

  // Mock-only: inject one incoming frame
  simulateReceive: (portId: string, data: number[]) =>
    call<void>("simulate_receive", { portId, data }),

  // Script commands
  listScripts: () => call<Script[]>("list_scripts"),
  loadScript: (path: string) => call<Script>("load_script", { path }),
  unloadScript: (name: string) => call<void>("unload_script", { name }),
  reloadScript: (name: string) => call<void>("reload_script", { name }),
  getScriptInfo: (name: string) => call<Script>("get_script_info", { name }),
  executeScript: (script: string) => call<string>("execute_script", { script }),
  validateScript: (script: string) =>
    call<ValidationError[]>("validate_script", { script }),
  validateScriptDetailed: (script: string) =>
    call<ScriptValidationResult>("validate_script_detailed", { script }),
  validateScriptFile: (path: string) =>
    call<void>("validate_script_file", { path }),
  listUserScripts: () => call<UserScriptInfo[]>("list_user_scripts"),
  readUserScriptContent: (name: string) =>
    call<string>("read_user_script", { name }),
  saveUserScript: (name: string, content: string) =>
    call<void>("save_user_script", { name, content }),
  deleteUserScript: (name: string) =>
    call<void>("delete_user_script", { name }),
  bindScript: (portId: string, scriptName: string) =>
    call<void>("bind_script", { portId, scriptName }),
  scriptEncode: (script: string, data: number[]) =>
    call<number[]>("script_encode", { script, data }),
  scriptDecode: (script: string, data: number[]) =>
    call<number[]>("script_decode", { script, data }),
  saveScriptFile: (name: string, content: string) =>
    call<string>("save_script_file", { name, content }),

  // Hot reload commands
  getHotReloadStatus: () => call<boolean>("get_hot_reload_status"),
  setHotReloadEnabled: (enabled: boolean) =>
    call<void>("set_hot_reload_enabled", { enabled }),

  // Serial script commands
  attachScript: (portId: string, scriptSource: string) =>
    call<void>("attach_script", { portId, scriptSource }),
  detachScript: (portId: string) => call<void>("detach_script", { portId }),
  getScriptStatus: (portId: string) =>
    call<ScriptStatus>("get_script_status", { portId }),
  listScriptActions: (portId: string) =>
    call<UiAction[]>("list_script_actions", { portId }),
  callScriptFunction: (portId: string, functionName: string, args?: string) =>
    call<string>("call_script_function", {
      portId,
      functionName,
      args: args ?? null,
    }),

  // Standalone script UI actions
  listStandaloneScriptActions: (scriptSource: string) =>
    call<UiAction[]>("list_standalone_script_actions", { scriptSource }),
  callStandaloneScriptFunction: (
    scriptSource: string,
    functionName: string,
    args?: string,
  ) =>
    call<string>("call_standalone_script_function", {
      scriptSource,
      functionName,
      args: args ?? null,
    }),

  // Virtual port commands
  createVirtualPort: (config: CreateVirtualPortConfig) =>
    call<string>("create_virtual_port", { config }),
  listVirtualPorts: () => call<VirtualPortInfo[]>("list_virtual_ports"),
  stopVirtualPort: (id: string) => call<void>("stop_virtual_port", { id }),
  getVirtualPortStats: (id: string) =>
    call<VirtualPortStats>("get_virtual_port_stats", { id }),
  checkVirtualPortHealth: (id: string) =>
    call<boolean>("check_virtual_port_health", { id }),
  getCapturedPackets: (id: string) =>
    call<CapturedPacket[]>("get_captured_packets", { id }),
  sendToVirtualPort: (id: string, portEnd: string, data: number[]) =>
    call<number>("send_to_virtual_port", { id, portEnd, data }),

  // Server commands
  startServer: () => call<ServerStatus>("start_server"),
  stopServer: () => call<void>("stop_server"),
  getServerStatus: () => call<ServerStatus>("get_server_status"),

  // Config commands
  getConfig: () => call<ConfigData>("get_config"),
  updateConfig: (config: ConfigData) => call<void>("update_config", { config }),
  resetConfig: () => call<void>("reset_config"),

  // Data export
  exportData: (path: string, format: string, data: unknown[]) =>
    call<void>("export_data", { path, format, data }),

  // Preset commands
  getConnectionPresets: () =>
    call<ConnectionPreset[]>("get_connection_presets"),
  saveConnectionPresets: (presets: ConnectionPreset[]) =>
    call<void>("save_connection_presets", { presets }),
  deleteConnectionPreset: (name: string) =>
    call<void>("delete_connection_preset", { name }),

  // Log commands
  readLogs: (maxLines?: number) => call<string[]>("read_logs", { maxLines }),
  clearLogs: () => call<void>("clear_logs"),
};
