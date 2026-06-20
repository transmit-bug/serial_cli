import { describe, it, expect, beforeEach, vi } from "vitest";
import { useScriptStore } from "./script";
import { tauriApi } from "@/lib/tauri-api";

// Mock tauriApi
vi.mock("@/lib/tauri-api", () => ({
  tauriApi: {
    listScripts: vi.fn(),
    loadScript: vi.fn(),
    unloadScript: vi.fn(),
    reloadScript: vi.fn(),
    bindScript: vi.fn(),
    listUserScripts: vi.fn(),
    saveUserScript: vi.fn(),
    deleteUserScript: vi.fn(),
  },
}));

// Mock global fetch
const mockFetch = vi.fn();
global.fetch = mockFetch;

const mockTauriApi = vi.mocked(tauriApi);

describe("useScriptStore", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    useScriptStore.setState({
      scripts: [],
      scriptsLoading: false,
      userScripts: [],
      userScriptsLoading: false,
      currentScript: null,
      isDirty: false,
      activeScript: null,
    });
  });

  describe("initial state", () => {
    it("should have correct initial state", () => {
      const state = useScriptStore.getState();
      expect(state.scripts).toEqual([]);
      expect(state.scriptsLoading).toBe(false);
      expect(state.userScripts).toEqual([]);
      expect(state.userScriptsLoading).toBe(false);
      expect(state.currentScript).toBeNull();
      expect(state.isDirty).toBe(false);
      expect(state.activeScript).toBeNull();
    });
  });

  describe("loadScripts", () => {
    it("should set loading while fetching", async () => {
      mockTauriApi.listScripts.mockImplementation(
        () => new Promise((resolve) => setTimeout(() => resolve([]), 10)),
      );

      const promise = useScriptStore.getState().loadScripts();
      expect(useScriptStore.getState().scriptsLoading).toBe(true);
      await promise;
    });

    it("should update scripts on success", async () => {
      const mockScripts = [
        { name: "line", description: "Line protocol", built_in: true },
        { name: "custom", description: "Custom script", built_in: false },
      ];
      mockTauriApi.listScripts.mockResolvedValueOnce(mockScripts as any);

      await useScriptStore.getState().loadScripts();

      expect(useScriptStore.getState().scripts).toEqual(mockScripts);
      expect(useScriptStore.getState().scriptsLoading).toBe(false);
    });

    it("should set loading to false on error", async () => {
      mockTauriApi.listScripts.mockRejectedValueOnce(new Error("Failed"));

      await useScriptStore.getState().loadScripts();

      expect(useScriptStore.getState().scriptsLoading).toBe(false);
      expect(useScriptStore.getState().scripts).toEqual([]);
    });
  });

  describe("loadCustomScript", () => {
    it("should call API and reload scripts list", async () => {
      mockTauriApi.loadScript.mockResolvedValueOnce({
        name: "new_script",
        description: "New script",
        built_in: false,
      });
      mockTauriApi.listScripts.mockResolvedValueOnce([
        { name: "new_script", description: "New script", built_in: false },
      ]);

      await useScriptStore.getState().loadCustomScript("/path/to/script.lua");

      expect(mockTauriApi.loadScript).toHaveBeenCalledWith(
        "/path/to/script.lua",
      );
      expect(useScriptStore.getState().scripts).toHaveLength(1);
    });
  });

  describe("unloadScript", () => {
    it("should remove script from list", async () => {
      useScriptStore.setState({
        scripts: [
          { name: "script1", description: "Script 1", built_in: false },
          { name: "script2", description: "Script 2", built_in: false },
        ],
      });
      mockTauriApi.unloadScript.mockResolvedValueOnce(undefined);

      await useScriptStore.getState().unloadScript("script1");

      expect(useScriptStore.getState().scripts).toHaveLength(1);
      expect(useScriptStore.getState().scripts[0].name).toBe("script2");
    });

    it("should clear activeScript if it was unloaded", async () => {
      useScriptStore.setState({
        scripts: [
          { name: "script1", description: "Script 1", built_in: false },
        ],
        activeScript: "script1",
      });
      mockTauriApi.unloadScript.mockResolvedValueOnce(undefined);

      await useScriptStore.getState().unloadScript("script1");

      expect(useScriptStore.getState().activeScript).toBeNull();
    });

    it("should keep activeScript if different script was unloaded", async () => {
      useScriptStore.setState({
        scripts: [
          { name: "script1", description: "Script 1", built_in: false },
          { name: "script2", description: "Script 2", built_in: false },
        ],
        activeScript: "script2",
      });
      mockTauriApi.unloadScript.mockResolvedValueOnce(undefined);

      await useScriptStore.getState().unloadScript("script1");

      expect(useScriptStore.getState().activeScript).toBe("script2");
    });
  });

  describe("reloadScript", () => {
    it("should call API to reload script", async () => {
      mockTauriApi.reloadScript.mockResolvedValueOnce(undefined);

      await useScriptStore.getState().reloadScript("my_script");

      expect(mockTauriApi.reloadScript).toHaveBeenCalledWith("my_script");
    });
  });

  describe("setActiveScript", () => {
    it("should bind script and update state", async () => {
      mockTauriApi.bindScript.mockResolvedValueOnce(undefined);

      await useScriptStore.getState().setActiveScript("port1", "line");

      expect(useScriptStore.getState().activeScript).toBe("line");
      expect(mockTauriApi.bindScript).toHaveBeenCalledWith("port1", "line");
    });

    it("should clear activeScript when scriptName is null", async () => {
      useScriptStore.setState({ activeScript: "line" });

      await useScriptStore.getState().setActiveScript("port1", null);

      expect(useScriptStore.getState().activeScript).toBeNull();
      expect(mockTauriApi.bindScript).not.toHaveBeenCalled();
    });
  });

  describe("loadUserScripts", () => {
    it("should set loading while fetching", async () => {
      mockTauriApi.listUserScripts.mockImplementation(
        () => new Promise((resolve) => setTimeout(() => resolve([]), 10)),
      );

      const promise = useScriptStore.getState().loadUserScripts();
      expect(useScriptStore.getState().userScriptsLoading).toBe(true);
      await promise;
    });

    it("should update userScripts on success", async () => {
      const mockUserScripts = [
        {
          name: "script1",
          path: "/path/script1.lua",
          size: 100,
          modified: 1234567890,
        },
      ];
      mockTauriApi.listUserScripts.mockResolvedValueOnce(mockUserScripts as any);

      await useScriptStore.getState().loadUserScripts();

      expect(useScriptStore.getState().userScripts).toEqual(mockUserScripts);
      expect(useScriptStore.getState().userScriptsLoading).toBe(false);
    });

    it("should set loading to false on error", async () => {
      mockTauriApi.listUserScripts.mockRejectedValueOnce(new Error("Failed"));

      await useScriptStore.getState().loadUserScripts();

      expect(useScriptStore.getState().userScriptsLoading).toBe(false);
    });
  });

  describe("openUserScript", () => {
    it("should fetch script content and set currentScript", async () => {
      useScriptStore.setState({
        userScripts: [
          {
            name: "my_script",
            path: "/path/my_script.lua",
            size: 100,
            modified: 1234567890,
          },
        ],
      });
      mockFetch.mockResolvedValueOnce({
        text: () => Promise.resolve("function on_recv(data) return data end"),
      });

      await useScriptStore.getState().openUserScript("my_script");

      expect(useScriptStore.getState().currentScript).toEqual({
        name: "my_script",
        content: "function on_recv(data) return data end",
      });
      expect(useScriptStore.getState().isDirty).toBe(false);
    });

    it("should do nothing if script not found", async () => {
      useScriptStore.setState({ userScripts: [] });

      await useScriptStore.getState().openUserScript("nonexistent");

      expect(useScriptStore.getState().currentScript).toBeNull();
    });

    it("should set empty content on fetch error", async () => {
      useScriptStore.setState({
        userScripts: [
          {
            name: "my_script",
            path: "/path/my_script.lua",
            size: 100,
            modified: 1234567890,
          },
        ],
      });
      mockFetch.mockRejectedValueOnce(new Error("File not found"));

      await useScriptStore.getState().openUserScript("my_script");

      expect(useScriptStore.getState().currentScript).toEqual({
        name: "my_script",
        content: "",
      });
      expect(useScriptStore.getState().isDirty).toBe(false);
    });
  });

  describe("saveUserScript", () => {
    it("should call API and reload user scripts", async () => {
      mockTauriApi.saveUserScript.mockResolvedValueOnce(undefined);
      mockTauriApi.listUserScripts.mockResolvedValueOnce([]);

      await useScriptStore.getState().saveUserScript("my_script", "content");

      expect(mockTauriApi.saveUserScript).toHaveBeenCalledWith(
        "my_script",
        "content",
      );
      expect(useScriptStore.getState().isDirty).toBe(false);
    });
  });

  describe("deleteUserScript", () => {
    it("should call API and reload user scripts", async () => {
      mockTauriApi.deleteUserScript.mockResolvedValueOnce(undefined);
      mockTauriApi.listUserScripts.mockResolvedValueOnce([]);

      await useScriptStore.getState().deleteUserScript("my_script");

      expect(mockTauriApi.deleteUserScript).toHaveBeenCalledWith("my_script");
    });

    it("should clear currentScript if deleted script was open", async () => {
      useScriptStore.setState({
        currentScript: { name: "my_script", content: "content" },
        isDirty: true,
      });
      mockTauriApi.deleteUserScript.mockResolvedValueOnce(undefined);
      mockTauriApi.listUserScripts.mockResolvedValueOnce([]);

      await useScriptStore.getState().deleteUserScript("my_script");

      expect(useScriptStore.getState().currentScript).toBeNull();
      expect(useScriptStore.getState().isDirty).toBe(false);
    });

    it("should keep currentScript if different script was deleted", async () => {
      useScriptStore.setState({
        currentScript: { name: "other_script", content: "content" },
        isDirty: true,
      });
      mockTauriApi.deleteUserScript.mockResolvedValueOnce(undefined);
      mockTauriApi.listUserScripts.mockResolvedValueOnce([]);

      await useScriptStore.getState().deleteUserScript("my_script");

      expect(useScriptStore.getState().currentScript).toEqual({
        name: "other_script",
        content: "content",
      });
      expect(useScriptStore.getState().isDirty).toBe(true);
    });
  });

  describe("newUserScript", () => {
    it("should set currentScript to empty and reset isDirty", async () => {
      useScriptStore.setState({
        currentScript: { name: "old", content: "old content" },
        isDirty: true,
      });

      useScriptStore.getState().newUserScript();

      expect(useScriptStore.getState().currentScript).toEqual({
        name: "",
        content: "",
      });
      expect(useScriptStore.getState().isDirty).toBe(false);
    });
  });

  describe("updateContent", () => {
    it("should update currentScript content and set isDirty", async () => {
      useScriptStore.setState({
        currentScript: { name: "my_script", content: "old" },
        isDirty: false,
      });

      useScriptStore.getState().updateContent("new content");

      expect(useScriptStore.getState().currentScript).toEqual({
        name: "my_script",
        content: "new content",
      });
      expect(useScriptStore.getState().isDirty).toBe(true);
    });

    it("should do nothing if currentScript is null", () => {
      useScriptStore.setState({ currentScript: null });

      useScriptStore.getState().updateContent("new content");

      expect(useScriptStore.getState().currentScript).toBeNull();
    });
  });
});
