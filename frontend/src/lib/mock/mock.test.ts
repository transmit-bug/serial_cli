import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { portHandlers } from "./handlers/port";
import { scriptHandlers } from "./handlers/script";
import { serialHandlers } from "./handlers/serial";
import { virtualPortHandlers } from "./handlers/virtual-port";
import { dispatch, registerHandlers } from "./interceptor";
import { state } from "./state";

// The global setup mocks "@tauri-apps/api/event" (aliased to ./events in
// mock mode); these tests exercise the real event bus, so restore it.
vi.mock("@tauri-apps/api/event", async () => {
  return await vi.importActual("@tauri-apps/api/event");
});

describe("mock serial handlers", () => {
  beforeEach(() => {
    vi.useFakeTimers();
    registerHandlers(portHandlers);
    registerHandlers(serialHandlers);
    registerHandlers(scriptHandlers);
    registerHandlers(virtualPortHandlers);
    state.sniffTimers.forEach((t) => clearInterval(t));
    state.sniffTimers.clear();
    state.sniffingPorts.clear();
    state.ports.delete("/dev/ttyUSB0");
    state.openPort("/dev/ttyUSB0", {
      baudrate: 115200,
      databits: 8,
      stopbits: 1,
      parity: "None",
      timeout_ms: 1000,
      flow_control: "None",
    });
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it("start_sniffing emits simulated data-received frames (issue #49)", async () => {
    const received: unknown[] = [];
    const unlisten = await import("./events").then(({ listen }) =>
      listen("data-received", (e) => received.push(e.payload)),
    );

    await dispatch("start_sniffing", { portId: "/dev/ttyUSB0" }, state);
    // First simulated frame fires after the 3s interval
    vi.advanceTimersByTime(3100);

    expect(received.length).toBe(1);
    const payload = received[0] as {
      port_id: string;
      data: number[];
      direction: string;
    };
    expect(payload.port_id).toBe("/dev/ttyUSB0");
    expect(payload.direction).toBe("rx");
    expect(payload.data.length).toBeGreaterThan(0);
    // RX stats updated
    const status = state.getPortStatus("/dev/ttyUSB0");
    expect(status.stats.bytes_received).toBe(payload.data.length);
    expect(status.stats.packets_received).toBe(1);

    await dispatch("stop_sniffing", { portId: "/dev/ttyUSB0" }, state);
    const count = received.length;
    vi.advanceTimersByTime(7000);
    expect(received.length).toBe(count); // no more frames after stop
    unlisten();
  });

  it("simulate_receive injects one frame manually", async () => {
    const received: unknown[] = [];
    const unlisten = await import("./events").then(({ listen }) =>
      listen("data-received", (e) => received.push(e.payload)),
    );

    await dispatch(
      "simulate_receive",
      { portId: "/dev/ttyUSB0", data: [0x01, 0x02, 0x03] },
      state,
    );

    expect(received.length).toBe(1);
    expect(state.getPortStatus("/dev/ttyUSB0").stats.bytes_received).toBe(3);
    unlisten();
  });

  it("signal control commands update and read back DTR/RTS (issue #54)", async () => {
    const initial = (await dispatch(
      "get_signals",
      { portId: "/dev/ttyUSB0" },
      state,
    )) as { dtr: boolean; rts: boolean; cts: boolean; dsr: boolean };

    expect(initial.dtr).toBe(true);
    expect(initial.rts).toBe(true);

    const after = (await dispatch(
      "set_dtr",
      { portId: "/dev/ttyUSB0", enable: false },
      state,
    )) as { dtr: boolean };

    expect(after.dtr).toBe(false);

    const readBack = (await dispatch(
      "get_signals",
      { portId: "/dev/ttyUSB0" },
      state,
    )) as { dtr: boolean; rts: boolean };
    expect(readBack.dtr).toBe(false);
    expect(readBack.rts).toBe(true);
  });
});

describe("mock virtual port handlers", () => {
  beforeEach(() => {
    registerHandlers(virtualPortHandlers);
  });

  it("captures packets only when monitoring is enabled (issues #45/#48)", async () => {
    await dispatch(
      "create_virtual_port",
      { config: { name: "vp1", backend: "pty", monitor: true } },
      state,
    );

    const written = (await dispatch(
      "send_to_virtual_port",
      { id: "vp1", portEnd: "b", data: [0x01, 0x03, 0x00] },
      state,
    )) as number;
    expect(written).toBe(3);

    const packets = (await dispatch(
      "get_captured_packets",
      { id: "vp1" },
      state,
    )) as { direction: string; data: number[]; timestamp_millis: number }[];
    expect(packets.length).toBe(1);
    expect(packets[0].direction).toBe("AtoB");
    expect(packets[0].data).toEqual([0x01, 0x03, 0x00]);

    const stats = (await dispatch(
      "get_virtual_port_stats",
      { id: "vp1" },
      state,
    )) as {
      monitoring: boolean;
      capture_packets: number;
      capture_bytes: number;
    };
    expect(stats.monitoring).toBe(true);
    expect(stats.capture_packets).toBe(1);
    expect(stats.capture_bytes).toBe(3);

    // B→A direction
    await dispatch(
      "send_to_virtual_port",
      { id: "vp1", portEnd: "a", data: [0xaa, 0xbb] },
      state,
    );
    const packets2 = (await dispatch(
      "get_captured_packets",
      { id: "vp1" },
      state,
    )) as { direction: string }[];
    expect(packets2[1].direction).toBe("BtoA");
  });

  it("does not capture when monitoring is off", async () => {
    await dispatch(
      "create_virtual_port",
      { config: { name: "vp2", backend: "pty", monitor: false } },
      state,
    );
    await dispatch(
      "send_to_virtual_port",
      { id: "vp2", portEnd: "b", data: [0x01] },
      state,
    );
    const packets = (await dispatch(
      "get_captured_packets",
      { id: "vp2" },
      state,
    )) as unknown[];
    expect(packets.length).toBe(0);
  });
});

describe("mock script handlers", () => {
  beforeEach(() => {
    registerHandlers(scriptHandlers);
  });

  it("saves and reads back user script content (issue #52)", async () => {
    await dispatch(
      "save_user_script",
      { name: "my_script", content: "function on_data(d) return d end" },
      state,
    );
    const content = (await dispatch(
      "read_user_script",
      { name: "my_script" },
      state,
    )) as string;
    expect(content).toBe("function on_data(d) return d end");

    // list_user_scripts still returns metadata
    const list = (await dispatch("list_user_scripts", {}, state)) as {
      name: string;
      size: number;
    }[];
    expect(list).toHaveLength(1);
    expect(list[0].name).toBe("my_script");
  });

  it("encodes Modbus RTU frames with CRC16 (issue #53)", async () => {
    const encoded = (await dispatch(
      "script_encode",
      { script: "modbus_rtu", data: [0x01, 0x03, 0x00, 0x00, 0x00, 0x01] },
      state,
    )) as number[];
    // CRC16-IBM(A001) of 01 03 00 00 00 01 = 84 0A (little-endian)
    expect(encoded).toEqual([0x01, 0x03, 0x00, 0x00, 0x00, 0x01, 0x84, 0x0a]);

    const decoded = (await dispatch(
      "script_decode",
      { script: "modbus_rtu", data: encoded },
      state,
    )) as number[];
    expect(decoded).toEqual([0x01, 0x03, 0x00, 0x00, 0x00, 0x01]);
  });

  it("encodes Modbus ASCII frames with LRC (issue #53)", async () => {
    const encoded = (await dispatch(
      "script_encode",
      { script: "modbus_ascii", data: [0x01, 0x03, 0x00, 0x00, 0x00, 0x01] },
      state,
    )) as number[];
    const ascii = String.fromCharCode(...encoded);
    // LRC of 01 03 00 00 00 01 = 0xFB
    expect(ascii).toBe(":010300000001FB\r\n");

    const decoded = (await dispatch(
      "script_decode",
      { script: "modbus_ascii", data: encoded },
      state,
    )) as number[];
    expect(decoded).toEqual([0x01, 0x03, 0x00, 0x00, 0x00, 0x01]);
  });
});
