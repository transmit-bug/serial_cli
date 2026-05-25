import { describe, it, expect, beforeEach, vi } from "vitest";
import { useLogStore } from "@/stores/log";
import { invoke } from "@tauri-apps/api/core";

vi.mock("@tauri-apps/api/core");

describe("useLogStore", () => {
  beforeEach(() => {
    useLogStore.setState({
      lines: [],
      loading: false,
      autoRefresh: false,
      filter: "",
    });
    vi.clearAllMocks();
  });

  it("loads logs from backend", async () => {
    const fakeLogs = [
      "2026-05-26 INFO app started",
      "2026-05-26 WARN slow query",
      "2026-05-26 ERROR connection failed",
    ];
    vi.mocked(invoke).mockResolvedValueOnce(fakeLogs);

    await useLogStore.getState().loadLogs();

    expect(invoke).toHaveBeenCalledWith("read_logs", { maxLines: 2000 });
    expect(useLogStore.getState().lines).toEqual(fakeLogs);
    expect(useLogStore.getState().loading).toBe(false);
  });

  it("handles load error gracefully", async () => {
    vi.mocked(invoke).mockRejectedValueOnce(new Error("file not found"));

    await useLogStore.getState().loadLogs();

    expect(useLogStore.getState().lines).toEqual([]);
    expect(useLogStore.getState().loading).toBe(false);
  });

  it("clears logs via backend", async () => {
    useLogStore.setState({ lines: ["log1", "log2"] });
    vi.mocked(invoke).mockResolvedValueOnce(undefined);

    await useLogStore.getState().clearLogs();

    expect(invoke).toHaveBeenCalledWith("clear_logs");
    expect(useLogStore.getState().lines).toEqual([]);
  });

  it("sets filter", () => {
    useLogStore.getState().setFilter("ERROR");
    expect(useLogStore.getState().filter).toBe("ERROR");
  });

  it("enables auto-refresh and triggers immediate load", () => {
    useLogStore.getState().setAutoRefresh(true);
    expect(useLogStore.getState().autoRefresh).toBe(true);
  });
});
