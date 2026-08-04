import type { Handler } from "../interceptor";

export const serialHandlers: Record<string, Handler> = {
  send_data: ({ portId, data }, state) => {
    const entry = state.ports.get(portId as string);
    if (entry) {
      const bytes = (data as number[]) || [];
      entry.bytesSent += bytes.length;
      entry.packetsSent += 1;
      entry.lastActivity = Date.now();
    }
    return (data as number[])?.length ?? 0;
  },

  start_sniffing: () => {
    // No-op in mock — use __MOCK_EMIT__("data-received", ...) to simulate
  },

  stop_sniffing: () => {
    // No-op in mock
  },
};
