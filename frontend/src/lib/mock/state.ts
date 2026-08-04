import type {
  CapturedPacket,
  ConfigData,
  ConnectionPreset,
  PortInfo,
  PortStatus,
  Script,
  ScriptStatus,
  SerialConfig,
  ServerStatus,
  UserScriptInfo,
  VirtualPortInfo,
  VirtualPortStats,
} from "@/types";

interface PortEntry {
  info: PortInfo;
  status: "open" | "closed";
  config: SerialConfig | null;
  bytesSent: number;
  bytesReceived: number;
  packetsSent: number;
  packetsReceived: number;
  lastActivity: number | null;
}

interface VirtualPortEntry {
  info: VirtualPortInfo;
  stats: VirtualPortStats;
  capturedPackets: CapturedPacket[];
}

export class MockState {
  ports = new Map<string, PortEntry>();
  scripts: Script[] = [];
  userScripts = new Map<string, UserScriptInfo>();
  attachedScripts = new Map<string, string>(); // portId -> scriptSource
  config: ConfigData;
  presets: ConnectionPreset[] = [];
  server: ServerStatus;
  virtualPorts = new Map<string, VirtualPortEntry>();
  logs: string[] = [];
  hotReloadEnabled = false;

  constructor() {
    // Seed fake ports
    this.ports.set("/dev/ttyUSB0", {
      info: {
        port_name: "/dev/ttyUSB0",
        port_type: "usb-serial",
        is_virtual: false,
        virtual_id: null,
      },
      status: "closed",
      config: null,
      bytesSent: 0,
      bytesReceived: 0,
      packetsSent: 0,
      packetsReceived: 0,
      lastActivity: null,
    });
    this.ports.set("/dev/ttyUSB1", {
      info: {
        port_name: "/dev/ttyUSB1",
        port_type: "usb-serial",
        is_virtual: false,
        virtual_id: null,
      },
      status: "closed",
      config: null,
      bytesSent: 0,
      bytesReceived: 0,
      packetsSent: 0,
      packetsReceived: 0,
      lastActivity: null,
    });

    // Seed built-in scripts
    this.scripts = [
      { name: "line", description: "Line-based protocol", built_in: true },
      {
        name: "at_command",
        description: "AT command protocol",
        built_in: true,
      },
      {
        name: "modbus_rtu",
        description: "Modbus RTU protocol",
        built_in: true,
      },
    ];

    // Default config
    this.config = {
      serial: {
        defaultBaudrate: 115200,
        databits: 8,
        stopbits: 1,
        parity: "none",
        timeoutMs: 1000,
      },
      logging: {
        level: "info",
        format: "json",
        file: "",
      },
      lua: {
        memory_limit_mb: 64,
        timeout_seconds: 5,
        enable_sandbox: true,
      },
      output: {
        json_pretty: true,
        show_timestamp: true,
      },
      protocols: {
        hotReload: false,
        customDir: "",
      },
      virtual_ports: {
        backend: "pty",
        monitor: false,
      },
      display: {
        theme: "dark",
        maxPackets: 10000,
        format: "hex",
        showTimestamp: true,
      },
    };

    // Default server status
    this.server = {
      running: false,
      socket_path: "",
      started_at: 0,
      active_connections: 0,
      total_requests: 0,
      total_errors: 0,
      connections: [],
    };
  }

  // Port operations
  openPort(portName: string, config: SerialConfig): void {
    const entry = this.ports.get(portName);
    if (entry) {
      entry.status = "open";
      entry.config = config;
    } else {
      this.ports.set(portName, {
        info: {
          port_name: portName,
          port_type: "unknown",
          is_virtual: false,
          virtual_id: null,
        },
        status: "open",
        config,
        bytesSent: 0,
        bytesReceived: 0,
        packetsSent: 0,
        packetsReceived: 0,
        lastActivity: null,
      });
    }
  }

  closePort(portId: string): void {
    const entry = this.ports.get(portId);
    if (entry) {
      entry.status = "closed";
      entry.config = null;
    }
  }

  getPortStatus(portId: string): PortStatus {
    const entry = this.ports.get(portId);
    if (!entry) {
      return {
        id: portId,
        port_name: portId,
        is_open: false,
        config: null,
        stats: {
          bytes_sent: 0,
          bytes_received: 0,
          packets_sent: 0,
          packets_received: 0,
          last_activity: null,
        },
      };
    }
    return {
      id: portId,
      port_name: entry.info.port_name,
      is_open: entry.status === "open",
      config: entry.config,
      stats: {
        bytes_sent: entry.bytesSent,
        bytes_received: entry.bytesReceived,
        packets_sent: entry.packetsSent,
        packets_received: entry.packetsReceived,
        last_activity: entry.lastActivity,
      },
    };
  }

  // Script operations
  attachScript(portId: string, scriptSource: string): void {
    this.attachedScripts.set(portId, scriptSource);
  }

  detachScript(portId: string): void {
    this.attachedScripts.delete(portId);
  }

  getScriptStatus(portId: string): ScriptStatus {
    const source = this.attachedScripts.get(portId);
    return {
      has_script: !!source,
      timer_interval_ms: source ? 1000 : 0,
    };
  }

  // Virtual port operations
  createVirtualPort(id: string, backend: string): void {
    const now = Date.now();
    const info: VirtualPortInfo = {
      id,
      port_a: `${id}_a`,
      port_b: `${id}_b`,
      backend,
      created_at: new Date(now).toISOString(),
      uptime_secs: 0,
      running: true,
    };
    const stats: VirtualPortStats = {
      id,
      port_a: info.port_a,
      port_b: info.port_b,
      backend,
      running: true,
      uptime_secs: 0,
      bytes_bridged: 0,
      packets_bridged: 0,
      bridge_errors: 0,
      last_error: null,
      capture_packets: 0,
      capture_bytes: 0,
      monitoring: false,
    };
    this.virtualPorts.set(id, { info, stats, capturedPackets: [] });
  }

  stopVirtualPort(id: string): void {
    const entry = this.virtualPorts.get(id);
    if (entry) {
      entry.info.running = false;
      entry.stats.running = false;
    }
  }
}

export const state = new MockState();
