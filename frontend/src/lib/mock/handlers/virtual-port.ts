import type { Handler } from "../interceptor";

export const virtualPortHandlers: Record<string, Handler> = {
  create_virtual_port: ({ config }, state) => {
    // biome-ignore lint/suspicious/noExplicitAny: mock handler receives generic Record
    const cfg = config as any;
    const id = cfg?.name ?? `vp-${Date.now()}`;
    const backend = cfg?.backend ?? "pty";
    state.createVirtualPort(id, backend);
    return id;
  },

  list_virtual_ports: (_args, state) => {
    return Array.from(state.virtualPorts.values()).map((e) => e.info);
  },

  stop_virtual_port: ({ id }, state) => {
    state.stopVirtualPort(id as string);
  },

  get_virtual_port_stats: ({ id }, state) => {
    const entry = state.virtualPorts.get(id as string);
    if (!entry) {
      return {
        id: id as string,
        port_a: "",
        port_b: "",
        backend: "unknown",
        running: false,
        uptime_secs: 0,
        bytes_bridged: 0,
        packets_bridged: 0,
        bridge_errors: 0,
        last_error: null,
        capture_packets: 0,
        capture_bytes: 0,
        monitoring: false,
      };
    }
    return entry.stats;
  },

  check_virtual_port_health: ({ id }, state) => {
    const entry = state.virtualPorts.get(id as string);
    return entry?.info.running ?? false;
  },

  get_captured_packets: ({ id }, state) => {
    const entry = state.virtualPorts.get(id as string);
    return entry?.capturedPackets ?? [];
  },

  send_to_virtual_port: ({ id, data }, state) => {
    const bytes = (data as number[]) || [];
    const entry = state.virtualPorts.get(id as string);
    if (entry) {
      entry.stats.bytes_bridged += bytes.length;
      entry.stats.packets_bridged += 1;
    }
    return bytes.length;
  },
};
