import type { Handler } from "../interceptor";

/** Counter for the synthetic RX frames emitted by the mock sniffer. */
let rxCounter = 0;

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

  /**
   * Start the simulated receive loop for a port. Emits a small synthetic
   * frame on `data-received` every few seconds so the Terminal receive area
   * is testable in mock mode. Manual injection is also available via the
   * `simulate_receive` command or `__MOCK_EMIT__("data-received", ...)`.
   */
  start_sniffing: ({ portId }, state) => {
    const id = portId as string;
    const entry = state.ports.get(id);
    if (!entry) return;
    if (state.sniffTimers.has(id)) return; // already sniffing

    entry.sniffing = true;
    state.sniffingPorts.add(id);
    const timer = setInterval(() => {
      if (!state.ports.get(id) || state.ports.get(id)?.status !== "open") {
        return;
      }
      const text = `MOCK RX ${++rxCounter}`;
      const bytes = Array.from(text).map((c) => c.charCodeAt(0));
      state.simulateReceive(id, bytes);
    }, 3000);
    state.sniffTimers.set(id, timer);
  },

  stop_sniffing: ({ portId }, state) => {
    const id = portId as string;
    const timer = state.sniffTimers.get(id);
    if (timer) {
      clearInterval(timer);
      state.sniffTimers.delete(id);
    }
    state.sniffingPorts.delete(id);
    const entry = state.ports.get(id);
    if (entry) entry.sniffing = false;
  },

  /** Mock-only command: inject one incoming frame on a port. */
  simulate_receive: ({ portId, data }, state) => {
    state.simulateReceive(portId as string, (data as number[]) ?? []);
  },

  // ── Modem signal control simulation (issue #54) ──

  set_dtr: ({ portId, enable }, state) => {
    return state.setSignal(portId as string, "dtr", enable as boolean);
  },

  set_rts: ({ portId, enable }, state) => {
    return state.setSignal(portId as string, "rts", enable as boolean);
  },

  get_signals: ({ portId }, state) => {
    const sig = state.getSignals(portId as string);
    return { ...sig, platform: "mock" };
  },
};
