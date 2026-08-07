import { beforeEach, describe, expect, it, vi } from "vitest";
import { tauriApi } from "@/lib/tauri-api";
import { useRemoteStore } from "@/stores/remote";
import type { RemoteDataEvent } from "@/types";

vi.mock("@/lib/tauri-api", () => ({
  tauriApi: {
    getRemoteDevices: vi.fn().mockResolvedValue([]),
    addRemoteDevice: vi.fn().mockResolvedValue([]),
    updateRemoteDevice: vi.fn().mockResolvedValue([]),
    deleteRemoteDevice: vi.fn().mockResolvedValue([]),
    testRemoteDevice: vi.fn().mockResolvedValue({
      connections: { active: 0, max: 10 },
      max_connections: 10,
      total_requests: 0,
      total_errors: 0,
      started_at: 0,
    }),
    remotePortList: vi.fn().mockResolvedValue([]),
    remoteOpenPort: vi.fn().mockResolvedValue({
      connection_id: "conn-1",
      port: "/dev/ttyUSB0",
      protocol: null,
    }),
    remoteCloseConnection: vi.fn().mockResolvedValue(undefined),
    remoteSendData: vi.fn().mockResolvedValue(0),
    remoteRecvData: vi.fn().mockResolvedValue({
      data: "",
      bytes_read: 0,
      timeout: true,
    }),
    remoteConnectionList: vi.fn().mockResolvedValue([]),
    startRemoteSubscribe: vi.fn().mockResolvedValue(undefined),
    stopRemoteSubscribe: vi.fn().mockResolvedValue(undefined),
  },
}));

function resetStore() {
  useRemoteStore.setState({
    devices: [],
    loading: false,
    error: null,
    activeDeviceId: null,
    ports: [],
    connections: [],
    workbenchLoading: false,
    workbenchError: null,
    streaming: {},
    rxBuffers: {},
  });
}

describe("remote store streaming", () => {
  beforeEach(resetStore);

  it("appendStreamData routes data by connection_id (snake_case contract)", () => {
    const event: RemoteDataEvent = {
      device_id: "dev-1",
      connection_id: "conn-7",
      data: "4f4b0d0a", // "OK\r\n"
      bytes_read: 4,
      timestamp: 1_700_000_000,
    };

    useRemoteStore.getState().appendStreamData(event);

    const buffer = useRemoteStore.getState().rxBuffers["conn-7"];
    expect(buffer).toBeDefined();
    expect(buffer).toContain("OK");
    // Must NOT be keyed under "undefined" (the pre-fix bug)
    expect(useRemoteStore.getState().rxBuffers["undefined"]).toBeUndefined();
  });

  it("ignores events without a connection_id", () => {
    useRemoteStore.getState().appendStreamData({
      device_id: "dev-1",
      connection_id: "",
      data: "4f4b",
      bytes_read: 2,
      timestamp: 1,
    });
    expect(Object.keys(useRemoteStore.getState().rxBuffers)).toHaveLength(0);
  });

  it("startStream toggles streaming state via the API", async () => {
    useRemoteStore.setState({
      activeDeviceId: "dev-1",
      connections: [{ connection_id: "conn-1", port_id: null, protocol: null }],
    });
    await useRemoteStore.getState().startStream("conn-1");
    expect(useRemoteStore.getState().streaming["conn-1"]).toBe(true);
    expect(tauriApi.startRemoteSubscribe).toHaveBeenCalledWith(
      "dev-1",
      "conn-1",
    );

    await useRemoteStore.getState().stopStream("conn-1");
    expect(useRemoteStore.getState().streaming["conn-1"]).toBe(false);
    expect(tauriApi.stopRemoteSubscribe).toHaveBeenCalledWith(
      "dev-1",
      "conn-1",
    );
  });
});
