import { describe, it, expect, beforeEach, vi } from "vitest";
import { useServerStore } from "./server";
import { tauriApi } from "@/lib/tauri-api";

// Mock tauriApi
vi.mock("@/lib/tauri-api", () => ({
  tauriApi: {
    startServer: vi.fn(),
    stopServer: vi.fn(),
    getServerStatus: vi.fn(),
  },
}));

const mockTauriApi = vi.mocked(tauriApi);

describe("useServerStore", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    useServerStore.setState({
      status: {
        running: false,
        socket_path: "",
        started_at: 0,
        active_connections: 0,
        total_requests: 0,
        total_errors: 0,
        connections: [],
      },
      loading: false,
      error: null,
    });
  });

  describe("initial state", () => {
    it("should have correct initial state", () => {
      const state = useServerStore.getState();
      expect(state.status.running).toBe(false);
      expect(state.status.socket_path).toBe("");
      expect(state.status.started_at).toBe(0);
      expect(state.status.active_connections).toBe(0);
      expect(state.status.total_requests).toBe(0);
      expect(state.status.total_errors).toBe(0);
      expect(state.status.connections).toEqual([]);
      expect(state.loading).toBe(false);
      expect(state.error).toBeNull();
    });
  });

  describe("startServer", () => {
    it("should set loading and clear error while starting", async () => {
      mockTauriApi.startServer.mockImplementation(
        () => new Promise((resolve) => setTimeout(() => resolve({} as any), 10)),
      );

      useServerStore.setState({ error: "previous error" });
      const promise = useServerStore.getState().startServer();

      expect(useServerStore.getState().loading).toBe(true);
      expect(useServerStore.getState().error).toBeNull();
      await promise;
    });

    it("should update status on success", async () => {
      const mockStatus = {
        running: true,
        socket_path: "/tmp/serial-cli.sock",
        started_at: 1234567890,
        active_connections: 0,
        total_requests: 0,
        total_errors: 0,
        connections: [],
      };
      mockTauriApi.startServer.mockResolvedValueOnce(mockStatus as any);

      await useServerStore.getState().startServer();

      expect(useServerStore.getState().status).toEqual(mockStatus);
      expect(useServerStore.getState().loading).toBe(false);
      expect(useServerStore.getState().error).toBeNull();
    });

    it("should set error and stop loading on failure", async () => {
      mockTauriApi.startServer.mockRejectedValueOnce(new Error("Start failed"));

      await useServerStore.getState().startServer();

      expect(useServerStore.getState().loading).toBe(false);
      expect(useServerStore.getState().error).toContain("Start failed");
      expect(useServerStore.getState().status.running).toBe(false);
    });
  });

  describe("stopServer", () => {
    it("should set loading and clear error while stopping", async () => {
      mockTauriApi.stopServer.mockImplementation(
        () => new Promise((resolve) => setTimeout(() => resolve(undefined), 10)),
      );

      useServerStore.setState({ error: "previous error" });
      const promise = useServerStore.getState().stopServer();

      expect(useServerStore.getState().loading).toBe(true);
      expect(useServerStore.getState().error).toBeNull();
      await promise;
    });

    it("should reset status on success", async () => {
      useServerStore.setState({
        status: {
          running: true,
          socket_path: "/tmp/serial-cli.sock",
          started_at: 1234567890,
          active_connections: 2,
          total_requests: 100,
          total_errors: 5,
          connections: [{ id: "1" }, { id: "2" }] as any,
        },
      });
      mockTauriApi.stopServer.mockResolvedValueOnce(undefined);

      await useServerStore.getState().stopServer();

      expect(useServerStore.getState().status).toEqual({
        running: false,
        socket_path: "",
        started_at: 0,
        active_connections: 0,
        total_requests: 0,
        total_errors: 0,
        connections: [],
      });
      expect(useServerStore.getState().loading).toBe(false);
    });

    it("should set error and stop loading on failure", async () => {
      mockTauriApi.stopServer.mockRejectedValueOnce(new Error("Stop failed"));

      await useServerStore.getState().stopServer();

      expect(useServerStore.getState().loading).toBe(false);
      expect(useServerStore.getState().error).toContain("Stop failed");
    });
  });

  describe("refreshStatus", () => {
    it("should update status from API", async () => {
      const mockStatus = {
        running: true,
        socket_path: "/tmp/serial-cli.sock",
        started_at: 1234567890,
        active_connections: 1,
        total_requests: 50,
        total_errors: 2,
        connections: [{ id: "conn1" }] as any,
      };
      mockTauriApi.getServerStatus.mockResolvedValueOnce(mockStatus as any);

      await useServerStore.getState().refreshStatus();

      expect(useServerStore.getState().status).toEqual(mockStatus);
      expect(useServerStore.getState().error).toBeNull();
    });

    it("should set error on failure", async () => {
      mockTauriApi.getServerStatus.mockRejectedValueOnce(
        new Error("Status check failed"),
      );

      await useServerStore.getState().refreshStatus();

      expect(useServerStore.getState().error).toContain("Status check failed");
    });
  });

  describe("setError", () => {
    it("should set error state", () => {
      useServerStore.getState().setError("Custom error message");

      expect(useServerStore.getState().error).toBe("Custom error message");
    });

    it("should clear error when null is passed", () => {
      useServerStore.setState({ error: "previous error" });

      useServerStore.getState().setError(null);

      expect(useServerStore.getState().error).toBeNull();
    });
  });

  describe("status updates via setState", () => {
    it("should allow partial status updates", () => {
      useServerStore.setState((state) => ({
        status: {
          ...state.status,
          active_connections: 5,
          total_requests: 100,
        },
      }));

      expect(useServerStore.getState().status.active_connections).toBe(5);
      expect(useServerStore.getState().status.total_requests).toBe(100);
      expect(useServerStore.getState().status.running).toBe(false);
    });
  });
});
