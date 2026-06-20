import { beforeEach, describe, expect, it, vi } from "vitest";
import { tauriApi } from "@/lib/tauri-api";
import { useConnectionStore } from "@/stores/connection";

vi.mock("@/lib/tauri-api", () => ({
  tauriApi: {
    listPorts: vi.fn().mockResolvedValue([]),
    openPort: vi.fn().mockResolvedValue("port-1"),
    closePort: vi.fn().mockResolvedValue(undefined),
    startSniffing: vi.fn().mockResolvedValue(undefined),
    stopSniffing: vi.fn().mockResolvedValue(undefined),
    getPortStatus: vi.fn().mockResolvedValue({
      id: "port-1",
      port_name: "/dev/ttyUSB0",
      is_open: true,
      config: null,
      stats: {
        bytes_sent: 0,
        bytes_received: 0,
        packets_sent: 0,
        packets_received: 0,
        last_activity: null,
      },
    }),
  },
}));

function resetStore() {
  useConnectionStore.setState({
    availablePorts: [],
    connections: [],
    activePortId: null,
    pendingPort: null,
    defaultConfig: {
      baudrate: 115200,
      databits: 8,
      stopbits: 1,
      parity: "None",
      timeout_ms: 1000,
      flow_control: "None",
    },
  });
}

describe("useConnectionStore", () => {
  beforeEach(() => {
    resetStore();
    vi.clearAllMocks();
  });

  // --- refreshPorts ---

  describe("refreshPorts", () => {
    it("fetches and sets available ports", async () => {
      const ports = [
        {
          port_name: "/dev/ttyUSB0",
          port_type: "usb",
          is_virtual: false,
          virtual_id: null,
        },
      ];
      vi.mocked(tauriApi.listPorts).mockResolvedValueOnce(ports);

      await useConnectionStore.getState().refreshPorts();

      expect(useConnectionStore.getState().availablePorts).toEqual(ports);
    });

    it("falls back to empty array on error", async () => {
      vi.mocked(tauriApi.listPorts).mockRejectedValueOnce(new Error("failed"));

      await useConnectionStore.getState().refreshPorts();

      expect(useConnectionStore.getState().availablePorts).toEqual([]);
    });
  });

  // --- connect ---

  describe("connect", () => {
    it("creates connecting entry then transitions to connected", async () => {
      useConnectionStore.setState({ pendingPort: "/dev/ttyUSB0" });
      vi.mocked(tauriApi.openPort).mockResolvedValueOnce("port-1");

      await useConnectionStore.getState().connect("/dev/ttyUSB0");

      const conn = useConnectionStore.getState().connections[0];
      expect(conn.portName).toBe("/dev/ttyUSB0");
      expect(conn.status).toBe("connected");
      expect(conn.portId).toBe("port-1");
      expect(conn.connectedAt).toBeTypeOf("number");
      expect(useConnectionStore.getState().activePortId).toBe("port-1");

      expect(tauriApi.openPort).toHaveBeenCalledWith(
        "/dev/ttyUSB0",
        expect.any(Object),
        undefined,
      );
      expect(tauriApi.startSniffing).toHaveBeenCalledWith("port-1");
    });

    it("does nothing if no pending port", async () => {
      useConnectionStore.setState({ pendingPort: null });

      await useConnectionStore.getState().connect("/dev/ttyUSB0");

      expect(useConnectionStore.getState().connections).toHaveLength(0);
      expect(tauriApi.openPort).not.toHaveBeenCalled();
    });

    it("does nothing if port is already connected", async () => {
      useConnectionStore.setState({
        pendingPort: "/dev/ttyUSB0",
        connections: [
          {
            portId: "port-1",
            portName: "/dev/ttyUSB0",
            status: "connected",
            config: useConnectionStore.getState().defaultConfig,
            portStatus: null,
            connectedAt: Date.now(),
            error: null,
          },
        ],
      });

      await useConnectionStore.getState().connect("/dev/ttyUSB0");

      expect(tauriApi.openPort).not.toHaveBeenCalled();
    });

    it("sets error status on open failure", async () => {
      useConnectionStore.setState({ pendingPort: "/dev/ttyUSB0" });
      vi.mocked(tauriApi.openPort).mockRejectedValueOnce(
        new Error("port busy"),
      );

      await useConnectionStore.getState().connect("/dev/ttyUSB0");

      const conn = useConnectionStore.getState().connections[0];
      expect(conn.status).toBe("error");
      expect(conn.error).toBe("Error: port busy");
    });

    it("merges config override with default config", async () => {
      useConnectionStore.setState({ pendingPort: "/dev/ttyUSB0" });
      vi.mocked(tauriApi.openPort).mockResolvedValueOnce("port-1");

      await useConnectionStore
        .getState()
        .connect("/dev/ttyUSB0", { baudrate: 9600 });

      expect(tauriApi.openPort).toHaveBeenCalledWith(
        "/dev/ttyUSB0",
        expect.objectContaining({ baudrate: 9600 }),
        undefined,
      );
    });
  });

  // --- disconnect ---

  describe("disconnect", () => {
    it("stops sniffing, closes port, sets disconnected", async () => {
      useConnectionStore.setState({
        connections: [
          {
            portId: "port-1",
            portName: "/dev/ttyUSB0",
            status: "connected",
            config: useConnectionStore.getState().defaultConfig,
            portStatus: null,
            connectedAt: Date.now(),
            error: null,
          },
        ],
        activePortId: "port-1",
      });

      await useConnectionStore.getState().disconnect("port-1");

      expect(tauriApi.stopSniffing).toHaveBeenCalledWith("port-1");
      expect(tauriApi.closePort).toHaveBeenCalledWith("port-1");
      expect(useConnectionStore.getState().connections[0].status).toBe(
        "disconnected",
      );
    });

    it("auto-selects next connected port as active", async () => {
      useConnectionStore.setState({
        connections: [
          {
            portId: "port-1",
            portName: "/dev/ttyUSB0",
            status: "connected",
            config: useConnectionStore.getState().defaultConfig,
            portStatus: null,
            connectedAt: Date.now(),
            error: null,
          },
          {
            portId: "port-2",
            portName: "/dev/ttyUSB1",
            status: "connected",
            config: useConnectionStore.getState().defaultConfig,
            portStatus: null,
            connectedAt: Date.now(),
            error: null,
          },
        ],
        activePortId: "port-1",
      });

      await useConnectionStore.getState().disconnect("port-1");

      expect(useConnectionStore.getState().activePortId).toBe("port-2");
    });

    it("sets activePortId to null when no connected ports remain", async () => {
      useConnectionStore.setState({
        connections: [
          {
            portId: "port-1",
            portName: "/dev/ttyUSB0",
            status: "connected",
            config: useConnectionStore.getState().defaultConfig,
            portStatus: null,
            connectedAt: Date.now(),
            error: null,
          },
        ],
        activePortId: "port-1",
      });

      await useConnectionStore.getState().disconnect("port-1");

      expect(useConnectionStore.getState().activePortId).toBeNull();
    });

    it("handles already-closed port gracefully", async () => {
      useConnectionStore.setState({
        connections: [
          {
            portId: "port-1",
            portName: "/dev/ttyUSB0",
            status: "connected",
            config: useConnectionStore.getState().defaultConfig,
            portStatus: null,
            connectedAt: Date.now(),
            error: null,
          },
        ],
        activePortId: "port-1",
      });
      vi.mocked(tauriApi.stopSniffing).mockRejectedValueOnce(
        new Error("already stopped"),
      );

      await useConnectionStore.getState().disconnect("port-1");

      expect(useConnectionStore.getState().connections[0].status).toBe(
        "disconnected",
      );
    });
  });

  // --- disconnectAll ---

  describe("disconnectAll", () => {
    it("disconnects all connected ports", async () => {
      useConnectionStore.setState({
        connections: [
          {
            portId: "port-1",
            portName: "/dev/ttyUSB0",
            status: "connected",
            config: useConnectionStore.getState().defaultConfig,
            portStatus: null,
            connectedAt: Date.now(),
            error: null,
          },
          {
            portId: "port-2",
            portName: "/dev/ttyUSB1",
            status: "connected",
            config: useConnectionStore.getState().defaultConfig,
            portStatus: null,
            connectedAt: Date.now(),
            error: null,
          },
          {
            portId: "port-3",
            portName: "/dev/ttyUSB2",
            status: "disconnected",
            config: useConnectionStore.getState().defaultConfig,
            portStatus: null,
            connectedAt: null,
            error: null,
          },
        ],
        activePortId: "port-1",
      });

      await useConnectionStore.getState().disconnectAll();

      expect(tauriApi.closePort).toHaveBeenCalledTimes(2);
      const statuses = useConnectionStore
        .getState()
        .connections.map((c) => c.status);
      expect(statuses).toEqual([
        "disconnected",
        "disconnected",
        "disconnected",
      ]);
    });
  });

  // --- simple setters ---

  describe("setActivePort", () => {
    it("sets activePortId", () => {
      useConnectionStore.getState().setActivePort("port-1");
      expect(useConnectionStore.getState().activePortId).toBe("port-1");
    });
  });

  describe("setPendingPort", () => {
    it("sets pendingPort", () => {
      useConnectionStore.getState().setPendingPort("/dev/ttyUSB0");
      expect(useConnectionStore.getState().pendingPort).toBe("/dev/ttyUSB0");
    });
  });

  describe("removePort", () => {
    it("removes entry and auto-selects next active", () => {
      useConnectionStore.setState({
        connections: [
          {
            portId: "port-1",
            portName: "/dev/ttyUSB0",
            status: "connected",
            config: useConnectionStore.getState().defaultConfig,
            portStatus: null,
            connectedAt: Date.now(),
            error: null,
          },
          {
            portId: "port-2",
            portName: "/dev/ttyUSB1",
            status: "connected",
            config: useConnectionStore.getState().defaultConfig,
            portStatus: null,
            connectedAt: Date.now(),
            error: null,
          },
        ],
        activePortId: "port-1",
      });

      useConnectionStore.getState().removePort("port-1");

      expect(useConnectionStore.getState().connections).toHaveLength(1);
      expect(useConnectionStore.getState().activePortId).toBe("port-2");
    });

    it("sets activePortId to null when removing the only port", () => {
      useConnectionStore.setState({
        connections: [
          {
            portId: "port-1",
            portName: "/dev/ttyUSB0",
            status: "connected",
            config: useConnectionStore.getState().defaultConfig,
            portStatus: null,
            connectedAt: Date.now(),
            error: null,
          },
        ],
        activePortId: "port-1",
      });

      useConnectionStore.getState().removePort("port-1");

      expect(useConnectionStore.getState().connections).toHaveLength(0);
      expect(useConnectionStore.getState().activePortId).toBeNull();
    });
  });

  describe("setDefaultConfig", () => {
    it("merges partial config into defaultConfig", () => {
      useConnectionStore.getState().setDefaultConfig({ baudrate: 9600 });
      expect(useConnectionStore.getState().defaultConfig.baudrate).toBe(9600);
      expect(useConnectionStore.getState().defaultConfig.databits).toBe(8);
    });
  });

  describe("setPortError", () => {
    it("sets error on specified port", () => {
      useConnectionStore.setState({
        connections: [
          {
            portId: "port-1",
            portName: "/dev/ttyUSB0",
            status: "connected",
            config: useConnectionStore.getState().defaultConfig,
            portStatus: null,
            connectedAt: Date.now(),
            error: null,
          },
        ],
      });

      useConnectionStore.getState().setPortError("port-1", "timeout");

      expect(useConnectionStore.getState().connections[0].error).toBe(
        "timeout",
      );
    });

    it("clears error when set to null", () => {
      useConnectionStore.setState({
        connections: [
          {
            portId: "port-1",
            portName: "/dev/ttyUSB0",
            status: "error",
            config: useConnectionStore.getState().defaultConfig,
            portStatus: null,
            connectedAt: null,
            error: "old error",
          },
        ],
      });

      useConnectionStore.getState().setPortError("port-1", null);

      expect(useConnectionStore.getState().connections[0].error).toBeNull();
    });
  });
});
