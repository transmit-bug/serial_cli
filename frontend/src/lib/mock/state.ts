import type {
  CapturedPacket,
  ConfigData,
  ConnectionPreset,
  PortInfo,
  PortStatus,
  RemoteDevice,
  Script,
  ScriptStatus,
  SerialConfig,
  ServerStatus,
  UserScriptInfo,
  VirtualPortInfo,
  VirtualPortStats,
} from "@/types";
import { mockEmit } from "./events";

/** Simulated modem signal state for a port (mock mode only). */
export interface MockSignalState {
  dtr: boolean;
  rts: boolean;
  cts: boolean;
  dsr: boolean;
}

export function defaultSignals(): MockSignalState {
  return { dtr: true, rts: true, cts: false, dsr: false };
}

interface PortEntry {
  info: PortInfo;
  status: "open" | "closed";
  config: SerialConfig | null;
  bytesSent: number;
  bytesReceived: number;
  packetsSent: number;
  packetsReceived: number;
  lastActivity: number | null;
  sniffing: boolean;
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
  /** Full content of saved user scripts, so the editor can restore them. */
  userScriptContents = new Map<string, string>();
  attachedScripts = new Map<string, string>(); // portId -> scriptSource
  config: ConfigData;
  presets: ConnectionPreset[] = [];
  server: ServerStatus;
  virtualPorts = new Map<string, VirtualPortEntry>();
  logs: string[] = [];
  hotReloadEnabled = false;
  /** Ports currently "sniffing" (simulated receive loop active). */
  sniffingPorts = new Set<string>();
  /** Interval handles for the simulated receive loops. */
  sniffTimers = new Map<string, ReturnType<typeof setInterval>>();
  /** Simulated modem signal state per port id. */
  signals = new Map<string, MockSignalState>();

  // Remote devices (LAN daemon simulation)
  remoteDevices: RemoteDevice[] = [];
  remoteConnections = new Map<string, string>(); // connection_id -> port
  private remoteConnCounter = 0;
  private remoteStreamTimers = new Map<
    string,
    ReturnType<typeof setInterval>
  >();

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
      sniffing: false,
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
      sniffing: false,
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

    // Seed remote devices
    this.remoteDevices = [
      {
        id: "dev-rpi",
        name: "RPi Lab Board",
        host: "192.168.1.50",
        port: 23333,
        created_at: Math.floor(Date.now() / 1000) - 86400,
      },
      {
        id: "dev-bench",
        name: "Test Bench",
        host: "192.168.1.60",
        port: 23333,
        created_at: Math.floor(Date.now() / 1000) - 3600,
      },
    ];
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
        sniffing: false,
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
  createVirtualPort(id: string, backend: string, monitor = false): void {
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
      monitoring: monitor,
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

  /** Append a captured packet to a virtual port pair (monitoring enabled). */
  captureVirtualPacket(
    id: string,
    direction: "AtoB" | "BtoA",
    data: number[],
  ): void {
    const entry = this.virtualPorts.get(id);
    if (!entry || data.length === 0) return;
    entry.capturedPackets.push({
      direction,
      data,
      timestamp_millis: Date.now(),
    });
    entry.stats.capture_packets += 1;
    entry.stats.capture_bytes += data.length;
  }

  // Simulated serial receive

  /**
   * Simulate incoming data on a port: updates RX stats and emits the
   * `data-received` event exactly like the real backend sniffer does.
   */
  simulateReceive(portId: string, data: number[]): void {
    if (data.length === 0) return;
    const entry = this.ports.get(portId);
    if (entry) {
      entry.bytesReceived += data.length;
      entry.packetsReceived += 1;
      entry.lastActivity = Date.now();
    }
    mockEmit("data-received", {
      port_id: portId,
      data,
      timestamp: Date.now(),
      direction: "rx",
    });
  }

  // Signal control simulation

  setSignal(
    portId: string,
    signal: "dtr" | "rts",
    enable: boolean,
  ): MockSignalState {
    const current = this.signals.get(portId) ?? defaultSignals();
    const next = { ...current, [signal]: enable };
    this.signals.set(portId, next);
    return next;
  }

  getSignals(portId: string): MockSignalState {
    return this.signals.get(portId) ?? defaultSignals();
  }

  // Remote device operations (LAN daemon simulation)

  addRemoteDevice(name: string, host: string, port: number): RemoteDevice {
    const device: RemoteDevice = {
      id: `dev-${Math.random().toString(36).slice(2, 8)}`,
      name,
      host,
      port,
      created_at: Math.floor(Date.now() / 1000),
    };
    this.remoteDevices.push(device);
    return device;
  }

  updateRemoteDevice(
    id: string,
    name: string,
    host: string,
    port: number,
  ): void {
    const device = this.remoteDevices.find((d) => d.id === id);
    if (device) {
      device.name = name;
      device.host = host;
      device.port = port;
    }
  }

  deleteRemoteDevice(id: string): void {
    this.remoteDevices = this.remoteDevices.filter((d) => d.id !== id);
  }

  openRemoteConnection(port: string): string {
    this.remoteConnCounter += 1;
    const connectionId = `mock-conn-${this.remoteConnCounter}`;
    this.remoteConnections.set(connectionId, port);
    return connectionId;
  }

  closeRemoteConnection(connectionId: string): void {
    this.remoteConnections.delete(connectionId);
    this.stopRemoteStream(connectionId);
  }

  /** Simulated streaming: emit remote-data-received every 2s. */
  startRemoteStream(deviceId: string, connectionId: string): void {
    if (this.remoteStreamTimers.has(connectionId)) return;
    const timer = setInterval(() => {
      const data = `OK 000${Math.floor(Math.random() * 10)}\r\n`;
      mockEmit("remote-data-received", {
        device_id: deviceId,
        connection_id: connectionId,
        data: Array.from(new TextEncoder().encode(data))
          .map((b) => b.toString(16).padStart(2, "0"))
          .join(""),
        bytes_read: data.length,
        timestamp: Math.floor(Date.now() / 1000),
      });
    }, 2000);
    this.remoteStreamTimers.set(connectionId, timer);
  }

  stopRemoteStream(connectionId: string): void {
    const timer = this.remoteStreamTimers.get(connectionId);
    if (timer) {
      clearInterval(timer);
      this.remoteStreamTimers.delete(connectionId);
    }
  }

  // User script content

  saveScriptContent(name: string, content: string): void {
    this.userScriptContents.set(name, content);
  }

  getScriptContent(name: string): string | null {
    return this.userScriptContents.get(name) ?? null;
  }
}

export const state = new MockState();
