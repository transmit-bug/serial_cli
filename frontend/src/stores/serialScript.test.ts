import { describe, it, expect, beforeEach, vi } from "vitest";
import { useSerialScriptStore, useStandaloneScriptStore } from "./serialScript";
import { tauriApi } from "@/lib/tauri-api";

// Mock tauriApi
vi.mock("@/lib/tauri-api", () => ({
  tauriApi: {
    attachScript: vi.fn(),
    detachScript: vi.fn(),
    getScriptStatus: vi.fn(),
    listScriptActions: vi.fn(),
    callScriptFunction: vi.fn(),
    listStandaloneScriptActions: vi.fn(),
    callStandaloneScriptFunction: vi.fn(),
  },
}));

const mockTauriApi = vi.mocked(tauriApi);

describe("useSerialScriptStore", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    useSerialScriptStore.setState({
      attachedScript: null,
      scriptStatus: null,
      actions: [],
      loading: false,
    });
  });

  describe("initial state", () => {
    it("should have correct initial state", () => {
      const state = useSerialScriptStore.getState();
      expect(state.attachedScript).toBeNull();
      expect(state.scriptStatus).toBeNull();
      expect(state.actions).toEqual([]);
      expect(state.loading).toBe(false);
    });
  });

  describe("attachScript", () => {
    it("should set loading to true while attaching", async () => {
      mockTauriApi.attachScript.mockResolvedValueOnce(undefined);

      const promise = useSerialScriptStore
        .getState()
        .attachScript("port1", "script source");

      expect(useSerialScriptStore.getState().loading).toBe(true);
      await promise;
    });

    it("should set attachedScript on success", async () => {
      mockTauriApi.attachScript.mockResolvedValueOnce(undefined);

      await useSerialScriptStore
        .getState()
        .attachScript("port1", "script source");

      expect(useSerialScriptStore.getState().attachedScript).toBe(
        "script source",
      );
      expect(useSerialScriptStore.getState().loading).toBe(false);
      expect(mockTauriApi.attachScript).toHaveBeenCalledWith(
        "port1",
        "script source",
      );
    });

    it("should set loading to false on error", async () => {
      mockTauriApi.attachScript.mockRejectedValueOnce(new Error("Failed"));

      await useSerialScriptStore
        .getState()
        .attachScript("port1", "script source")
        .catch(() => {});

      expect(useSerialScriptStore.getState().loading).toBe(false);
      expect(useSerialScriptStore.getState().attachedScript).toBeNull();
    });
  });

  describe("detachScript", () => {
    it("should clear state on successful detach", async () => {
      useSerialScriptStore.setState({
        attachedScript: "some script",
        scriptStatus: { has_script: true, timer_interval_ms: 0 },
        actions: [{ label: "Action", function_name: "action", icon: null, group: null, confirm: false, params: [] }],
      });
      mockTauriApi.detachScript.mockResolvedValueOnce(undefined);

      await useSerialScriptStore.getState().detachScript("port1");

      expect(useSerialScriptStore.getState().attachedScript).toBeNull();
      expect(useSerialScriptStore.getState().scriptStatus).toBeNull();
      expect(useSerialScriptStore.getState().actions).toEqual([]);
      expect(useSerialScriptStore.getState().loading).toBe(false);
    });

    it("should set loading to false on error", async () => {
      useSerialScriptStore.setState({ attachedScript: "some script" });
      mockTauriApi.detachScript.mockRejectedValueOnce(new Error("Failed"));

      await useSerialScriptStore
        .getState()
        .detachScript("port1")
        .catch(() => {});

      expect(useSerialScriptStore.getState().loading).toBe(false);
      expect(useSerialScriptStore.getState().attachedScript).toBe("some script");
    });
  });

  describe("refreshStatus", () => {
    it("should update scriptStatus from API", async () => {
      const mockStatus = { status: "attached" };
      mockTauriApi.getScriptStatus.mockResolvedValueOnce(mockStatus as any);

      await useSerialScriptStore.getState().refreshStatus("port1");

      expect(useSerialScriptStore.getState().scriptStatus).toEqual(mockStatus);
      expect(mockTauriApi.getScriptStatus).toHaveBeenCalledWith("port1");
    });

    it("should ignore errors", async () => {
      mockTauriApi.getScriptStatus.mockRejectedValueOnce(new Error("Failed"));

      await useSerialScriptStore.getState().refreshStatus("port1");

      expect(useSerialScriptStore.getState().scriptStatus).toBeNull();
    });
  });

  describe("loadActions", () => {
    it("should update actions from API", async () => {
      const mockActions = [
        { label: "Action 1", function_name: "action1", icon: null, group: null, confirm: false, params: [] },
        { label: "Action 2", function_name: "action2", icon: null, group: null, confirm: false, params: [] },
      ];
      mockTauriApi.listScriptActions.mockResolvedValueOnce(mockActions as any);

      await useSerialScriptStore.getState().loadActions("port1");

      expect(useSerialScriptStore.getState().actions).toEqual(mockActions);
    });

    it("should set empty actions on error", async () => {
      mockTauriApi.listScriptActions.mockRejectedValueOnce(new Error("Failed"));

      await useSerialScriptStore.getState().loadActions("port1");

      expect(useSerialScriptStore.getState().actions).toEqual([]);
    });
  });

  describe("callAction", () => {
    it("should call API and return result", async () => {
      mockTauriApi.callScriptFunction.mockResolvedValueOnce("result");

      const result = await useSerialScriptStore
        .getState()
        .callAction("port1", "myFunc");

      expect(result).toBe("result");
      expect(mockTauriApi.callScriptFunction).toHaveBeenCalledWith(
        "port1",
        "myFunc",
        undefined,
      );
    });

    it("should propagate errors", async () => {
      mockTauriApi.callScriptFunction.mockRejectedValueOnce(
        new Error("Function failed"),
      );

      await expect(
        useSerialScriptStore.getState().callAction("port1", "myFunc"),
      ).rejects.toThrow("Function failed");
    });
  });
});

describe("useStandaloneScriptStore", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    useStandaloneScriptStore.setState({
      actions: [],
      currentScriptSource: "",
      loading: false,
      output: null,
    });
  });

  describe("initial state", () => {
    it("should have correct initial state", () => {
      const state = useStandaloneScriptStore.getState();
      expect(state.actions).toEqual([]);
      expect(state.currentScriptSource).toBe("");
      expect(state.loading).toBe(false);
      expect(state.output).toBeNull();
    });
  });

  describe("loadActions", () => {
    it("should set loading and currentScriptSource", async () => {
      mockTauriApi.listStandaloneScriptActions.mockResolvedValueOnce([]);

      await useStandaloneScriptStore.getState().loadActions("script source");

      expect(useStandaloneScriptStore.getState().currentScriptSource).toBe(
        "script source",
      );
      expect(useStandaloneScriptStore.getState().loading).toBe(false);
    });

    it("should update actions from API", async () => {
      const mockActions = [{ label: "Action", function_name: "action", icon: null, group: null, confirm: false, params: [] }];
      mockTauriApi.listStandaloneScriptActions.mockResolvedValueOnce(
        mockActions as any,
      );

      await useStandaloneScriptStore.getState().loadActions("script source");

      expect(useStandaloneScriptStore.getState().actions).toEqual(mockActions);
    });

    it("should set empty actions on error", async () => {
      useStandaloneScriptStore.setState({
        actions: [{ label: "Old", function_name: "old", icon: null, group: null, confirm: false, params: [] }],
      });
      mockTauriApi.listStandaloneScriptActions.mockRejectedValueOnce(
        new Error("Failed"),
      );

      await useStandaloneScriptStore
        .getState()
        .loadActions("script source");

      expect(useStandaloneScriptStore.getState().actions).toEqual([]);
      expect(useStandaloneScriptStore.getState().loading).toBe(false);
    });
  });

  describe("callAction", () => {
    it("should call API with current script source and return result", async () => {
      useStandaloneScriptStore.setState({ currentScriptSource: "my script" });
      mockTauriApi.callStandaloneScriptFunction.mockResolvedValueOnce("output");

      const result =
        await useStandaloneScriptStore.getState().callAction("myFunc");

      expect(result).toBe("output");
      expect(mockTauriApi.callStandaloneScriptFunction).toHaveBeenCalledWith(
        "my script",
        "myFunc",
        undefined,
      );
    });

    it("should update output state", async () => {
      useStandaloneScriptStore.setState({ currentScriptSource: "my script" });
      mockTauriApi.callStandaloneScriptFunction.mockResolvedValueOnce("output");

      await useStandaloneScriptStore.getState().callAction("myFunc");

      expect(useStandaloneScriptStore.getState().output).toBe("output");
    });

    it("should propagate errors", async () => {
      useStandaloneScriptStore.setState({ currentScriptSource: "my script" });
      mockTauriApi.callStandaloneScriptFunction.mockRejectedValueOnce(
        new Error("Failed"),
      );

      await expect(
        useStandaloneScriptStore.getState().callAction("myFunc"),
      ).rejects.toThrow("Failed");
    });
  });
});
