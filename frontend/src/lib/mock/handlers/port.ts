import type { Handler } from "../interceptor";

export const portHandlers: Record<string, Handler> = {
  list_ports: (_args, state) => {
    return Array.from(state.ports.values()).map((e) => e.info);
  },

  open_port: ({ portName, config }, state) => {
    // biome-ignore lint/suspicious/noExplicitAny: mock handler receives generic Record
    state.openPort(portName as string, config as any);
    return portName;
  },

  close_port: ({ portId }, state) => {
    state.closePort(portId as string);
  },

  get_port_status: ({ portId }, state) => {
    return state.getPortStatus(portId as string);
  },

  check_port_health: ({ portId }, state) => {
    const entry = state.ports.get(portId as string);
    return entry?.status === "open";
  },
};
