import { invoke } from "@tauri-apps/api/core";
import type {
  CapturedPacket,
  ConfigData,
  ConnectionPreset,
  CreateVirtualPortConfig,
  PortInfo,
  PortStatus,
  ProtocolInfo,
  ScriptInfo,
  ScriptStatus,
  SerialConfig,
  UiAction,
  ValidationError,
  VirtualPortInfo,
  VirtualPortStats,
} from "@/types";

export const tauriApi = {
  // Port commands
  listPorts: () => invoke<PortInfo[]>("list_ports"),
  openPort: (portName: string, config: SerialConfig, isVirtual?: boolean) =>
    invoke<string>("open_port", { portName, config, isVirtual }),
  closePort: (portId: string) => invoke<void>("close_port", { portId }),
  getPortStatus: (portId: string) =>
    invoke<PortStatus>("get_port_status", { portId }),
  checkPortHealth: (portId: string) =>
    invoke<boolean>("check_port_health", { portId }),

  // Serial commands
  sendData: (portId: string, data: number[]) =>
    invoke<number>("send_data", { portId, data }),
  startSniffing: (portId: string) => invoke<void>("start_sniffing", { portId }),
  stopSniffing: (portId: string) => invoke<void>("stop_sniffing", { portId }),

  // Protocol commands
  listProtocols: () => invoke<ProtocolInfo[]>("list_protocols"),
  loadProtocol: (path: string) =>
    invoke<ProtocolInfo>("load_protocol", { path }),
  unloadProtocol: (name: string) => invoke<void>("unload_protocol", { name }),
  reloadProtocol: (name: string) => invoke<void>("reload_protocol", { name }),
  validateProtocol: (path: string) =>
    invoke<void>("validate_protocol", { path }),
  protocolEncode: (protocol: string, data: number[]) =>
    invoke<number[]>("protocol_encode", { protocol, data }),
  protocolDecode: (protocol: string, data: number[]) =>
    invoke<number[]>("protocol_decode", { protocol, data }),
  setPortProtocol: (portId: string, protocolName: string) =>
    invoke<void>("set_port_protocol", { portId, protocolName }),
  saveProtocolFile: (name: string, content: string) =>
    invoke<string>("save_protocol_file", { name, content }),
  getProtocolInfo: (name: string) =>
    invoke<ProtocolInfo>("get_protocol_info", { name }),

  // Script commands
  executeScript: (script: string) =>
    invoke<string>("execute_script", { script }),
  validateScript: (script: string) =>
    invoke<ValidationError[]>("validate_script", { script }),
  listScripts: () => invoke<ScriptInfo[]>("list_scripts"),
  saveScript: (name: string, content: string) =>
    invoke<void>("save_script", { name, content }),
  deleteScript: (name: string) => invoke<void>("delete_script", { name }),

  // Serial script commands
  attachScript: (portId: string, scriptSource: string) =>
    invoke<void>("attach_script", { portId, scriptSource }),
  detachScript: (portId: string) => invoke<void>("detach_script", { portId }),
  hasScript: (portId: string) => invoke<boolean>("has_script", { portId }),
  getScriptStatus: (portId: string) =>
    invoke<ScriptStatus>("get_script_status", { portId }),
  listScriptActions: (portId: string) =>
    invoke<UiAction[]>("list_script_actions", { portId }),
  callScriptFunction: (portId: string, functionName: string) =>
    invoke<string>("call_script_function", { portId, functionName }),

  // Standalone script UI actions
  listStandaloneScriptActions: (scriptSource: string) =>
    invoke<UiAction[]>("list_standalone_script_actions", { scriptSource }),
  callStandaloneScriptFunction: (scriptSource: string, functionName: string) =>
    invoke<string>("call_standalone_script_function", {
      scriptSource,
      functionName,
    }),

  // Virtual port commands
  createVirtualPort: (config: CreateVirtualPortConfig) =>
    invoke<string>("create_virtual_port", { config }),
  listVirtualPorts: () => invoke<VirtualPortInfo[]>("list_virtual_ports"),
  stopVirtualPort: (id: string) => invoke<void>("stop_virtual_port", { id }),
  getVirtualPortStats: (id: string) =>
    invoke<VirtualPortStats>("get_virtual_port_stats", { id }),
  checkVirtualPortHealth: (id: string) =>
    invoke<boolean>("check_virtual_port_health", { id }),
  getCapturedPackets: (id: string) =>
    invoke<CapturedPacket[]>("get_captured_packets", { id }),

  // Config commands
  getConfig: () => invoke<ConfigData>("get_config"),
  updateConfig: (config: ConfigData) =>
    invoke<void>("update_config", { config }),
  resetConfig: () => invoke<void>("reset_config"),

  // Preset commands
  getConnectionPresets: () =>
    invoke<ConnectionPreset[]>("get_connection_presets"),
  saveConnectionPresets: (presets: ConnectionPreset[]) =>
    invoke<void>("save_connection_presets", { presets }),
  deleteConnectionPreset: (name: string) =>
    invoke<void>("delete_connection_preset", { name }),
};
