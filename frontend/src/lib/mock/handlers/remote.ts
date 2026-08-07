import type { Handler } from "../interceptor";

/**
 * Mock handlers for the remote device commands (LAN daemon simulation).
 * Simulates a target device running a Daemon: a device registry, fake
 * serial ports, open/close connections, echo responses, and periodic
 * `remote-data-received` streaming events.
 */

const FAKE_PORTS = [
  { port_name: "/dev/ttyUSB0", port_type: "UsbPort" },
  { port_name: "/dev/ttyUSB1", port_type: "UsbPort" },
  { port_name: "/dev/ttyACM0", port_type: "AcmPort" },
];

export const remoteHandlers: Record<string, Handler> = {
  get_remote_devices: (_args, state) => {
    return state.remoteDevices.map((d) => ({ ...d }));
  },

  add_remote_device: (args, state) => {
    const name = String(args.name ?? "").trim();
    const host = String(args.host ?? "").trim();
    const port = Number(args.port);
    if (!name || !host || port <= 0) {
      throw new Error("Name and host are required");
    }
    state.addRemoteDevice(name, host, port);
    return state.remoteDevices.map((d) => ({ ...d }));
  },

  update_remote_device: (args, state) => {
    state.updateRemoteDevice(
      String(args.id),
      String(args.name).trim(),
      String(args.host).trim(),
      Number(args.port),
    );
    return state.remoteDevices.map((d) => ({ ...d }));
  },

  delete_remote_device: (args, state) => {
    state.deleteRemoteDevice(String(args.id));
    return state.remoteDevices.map((d) => ({ ...d }));
  },

  test_remote_device: (_args, _state) => {
    return {
      connections: { active: 1, max: 10 },
      max_connections: 10,
      total_requests: 42,
      total_errors: 0,
      started_at: Math.floor(Date.now() / 1000) - 600,
    };
  },

  remote_port_list: (_args, _state) => {
    return FAKE_PORTS;
  },

  remote_open_port: (args, state) => {
    const port = String(args.port ?? "");
    const connectionId = state.openRemoteConnection(port);
    return { connection_id: connectionId, port, protocol: null };
  },

  remote_close_connection: (args, state) => {
    state.closeRemoteConnection(String(args.connectionId));
    return null;
  },

  remote_send_data: (args, _state) => {
    const data = Array.isArray(args.data) ? (args.data as number[]) : [];
    return data.length;
  },

  remote_recv_data: (_args, _state) => {
    // Simulated echo response
    const text = "OK\r\n";
    return {
      data: Array.from(new TextEncoder().encode(text))
        .map((b) => b.toString(16).padStart(2, "0"))
        .join(""),
      bytes_read: text.length,
      timeout: false,
    };
  },

  remote_connection_list: (_args, state) => {
    return Array.from(state.remoteConnections.entries()).map(
      ([connection_id, port]) => ({
        connection_id,
        port_id: port,
        protocol: null,
      }),
    );
  },

  start_remote_subscribe: (args, state) => {
    const deviceId = String(args.deviceId);
    const connectionId = String(args.connectionId);
    if (state.remoteConnections.has(connectionId)) {
      state.startRemoteStream(deviceId, connectionId);
    } else {
      // Simulate an unknown-connection error path
      throw new Error("Connection not found");
    }
    return null;
  },

  stop_remote_subscribe: (args, state) => {
    state.stopRemoteStream(String(args.connectionId));
    return null;
  },
};
