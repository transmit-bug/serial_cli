export interface PortInfo {
  port_name: string;
  port_type: string;
  is_virtual: boolean;
  virtual_id: string | null;
}

export interface SerialConfig {
  baudrate: number;
  databits: number;
  stopbits: number;
  parity: string;
  timeout_ms: number;
  flow_control: string;
}

export interface PortStatus {
  id: string;
  port_name: string;
  is_open: boolean;
  config: SerialConfig | null;
  stats: PortStats;
}

export interface PortStats {
  bytes_sent: number;
  bytes_received: number;
  packets_sent: number;
  packets_received: number;
  last_activity: number | null;
}

export interface Script {
  name: string;
  description: string;
  built_in: boolean;
}

export interface UserScriptInfo {
  name: string;
  path: string;
  size: number;
  modified: number;
}

export interface ValidationError {
  line: number;
  column: number;
  message: string;
}

export interface UiAction {
  function_name: string;
  label: string;
  icon: string | null;
  group: string | null;
  confirm: boolean;
}

export interface VirtualPortInfo {
  id: string;
  port_a: string;
  port_b: string;
  backend: string;
  created_at: string;
  uptime_secs: number;
  running: boolean;
}

export interface VirtualPortStats {
  id: string;
  port_a: string;
  port_b: string;
  backend: string;
  running: boolean;
  uptime_secs: number;
  bytes_bridged: number;
  packets_bridged: number;
  bridge_errors: number;
  last_error: string | null;
  capture_packets: number;
  capture_bytes: number;
  monitoring: boolean;
}

export interface CapturedPacket {
  direction: string;
  data: number[];
  timestamp_millis: number;
}

export interface CreateVirtualPortConfig {
  name?: string;
  backend: string;
  buffer_size?: number;
  monitor?: boolean;
}

export interface ScriptStatus {
  has_script: boolean;
  timer_interval_ms: number;
}

export interface ConfigData {
  serial: SerialConfigData;
  logging: LoggingConfigData;
  lua: LuaConfigData;
  output: OutputConfigData;
  protocols: ProtocolsConfigData;
  virtual_ports: VirtualPortsConfigData;
  display: DisplayConfigData;
}

export interface SerialConfigData {
  defaultBaudrate: number;
  databits: number;
  stopbits: number;
  parity: string;
  timeoutMs: number;
}

export interface LoggingConfigData {
  level: string;
  format: string;
  file: string;
}

export interface LuaConfigData {
  memory_limit_mb: number;
  timeout_seconds: number;
  enable_sandbox: boolean;
}

export interface OutputConfigData {
  json_pretty: boolean;
  show_timestamp: boolean;
}

export interface ProtocolsConfigData {
  hotReload: boolean;
  customDir: string;
}

export interface VirtualPortsConfigData {
  backend: string;
  monitor: boolean;
}

export interface DisplayConfigData {
  theme: string;
  maxPackets: number;
  format: string;
  showTimestamp: boolean;
}

export interface DataPacket {
  id: number;
  portId: string;
  direction: "rx" | "tx";
  timestamp: number;
  data: number[];
  decoded?: string;
}

export type DisplayFormat = "hex" | "ascii" | "mixed";

export type PageName =
  | "terminal"
  | "virtual"
  | "editor"
  | "server"
  | "settings";

export interface ScriptValidationResult {
  warnings: string[];
}

export interface ServerConnectionInfo {
  connection_id: string;
  port_id: string | null;
  protocol: string | null;
  created_at: number;
  subscribed: boolean;
}

export interface ServerStatus {
  running: boolean;
  socket_path: string;
  started_at: number;
  active_connections: number;
  total_requests: number;
  total_errors: number;
  connections: ServerConnectionInfo[];
}

export type ConnectionStatus =
  | "disconnected"
  | "connecting"
  | "connected"
  | "error";

export type Locale = "en" | "zh";

export interface QuickCommand {
  label: string;
  data: string;
  format: "hex" | "ascii";
  hotkey?: string;
}

export interface OutputLine {
  text: string;
  timestamp: number;
  type: "info" | "error" | "success";
}

export interface SequenceStep {
  label: string;
  data: string;
  format: "hex" | "ascii";
  delay: number;
  waitFor?: string;
  waitTimeout?: number;
  loopCount?: number;
}

export interface CommandSequence {
  id: string;
  name: string;
  steps: SequenceStep[];
}

export interface SequenceExecutionState {
  sequenceId: string | null;
  sequenceName: string | null;
  stepIndex: number;
  loopIteration: number;
  status: "idle" | "running" | "paused" | "completed" | "error";
  error?: string;
}

export interface ConnectionPreset {
  name: string;
  port_name: string;
  baudrate: number;
  databits: number;
  stopbits: number;
  parity: string;
  flow_control: string;
  timeout_ms: number;
}
