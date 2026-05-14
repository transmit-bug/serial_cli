// Tauri command interfaces
export interface SerialPort {
  port_name: string
  port_type: string
  /** Whether this is a virtual port (PTY/named-pipe/socat) */
  is_virtual?: boolean
  /** Virtual port pair ID (only set when is_virtual == true) */
  virtual_id?: string
}

export interface PortConfig {
  baudrate: number
  databits: number
  stopbits: number
  parity: string
  timeout_ms: number
  flow_control: string
}

export interface PortStatus {
  port_id: string
  port_name: string
  is_open: boolean
  config: PortConfig
  stats: PortStats
}

export interface PortStats {
  bytes_sent: number
  bytes_received: number
  packets_sent: number
  packets_received: number
  last_activity: number | null
}

export interface DataPacket {
  port_id: string
  direction: 'rx' | 'tx'
  data: number[]
  timestamp: number
}

export interface DataEvent {
  port_id: string
  data: number[]
  timestamp: number
}

export interface ProtocolInfo {
  name: string
  version: string
  description: string
  author: string
}

export interface ScriptResult {
  success: boolean
  output: string
  error?: string
}

// Virtual serial port types
export interface VirtualPortConfig {
  name?: string
  backend: 'pty' | 'named_pipe' | 'socat'
  buffer_size?: number
  monitor?: boolean
}

export interface VirtualPortInfo {
  id: string
  port_a: string
  port_b: string
  backend: string
  created_at: string
  uptime_secs: number
  running: boolean
}

export interface VirtualPortStats {
  id: string
  port_a: string
  port_b: string
  backend: string
  running: boolean
  uptime_secs: number
  bytes_bridged: number
  packets_bridged: number
  bridge_errors: number
  last_error: string | null
  capture_packets: number
  capture_bytes: number
  monitoring: boolean
}

export interface CapturedPacket {
  direction: string
  data: number[]
  timestamp_millis: number
}

export interface ScriptStatus {
  has_script: boolean
  timer_interval_ms: number
}
