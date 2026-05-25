import { describe, it, expect, beforeEach } from "vitest";
import { useUIStore } from "@/stores/ui";

describe("useUIStore", () => {
  beforeEach(() => {
    useUIStore.setState({
      currentPage: "terminal",
      sidebarCollapsed: true,
      rightPanelCollapsed: false,
      locale: "zh",
    });
  });

  it("navigates between pages", () => {
    const { navigateTo } = useUIStore.getState();
    navigateTo("editor");
    expect(useUIStore.getState().currentPage).toBe("editor");

    navigateTo("settings");
    expect(useUIStore.getState().currentPage).toBe("settings");
  });

  it("toggles sidebar", () => {
    expect(useUIStore.getState().sidebarCollapsed).toBe(true);
    useUIStore.getState().toggleSidebar();
    expect(useUIStore.getState().sidebarCollapsed).toBe(false);
    useUIStore.getState().toggleSidebar();
    expect(useUIStore.getState().sidebarCollapsed).toBe(true);
  });

  it("toggles right panel", () => {
    expect(useUIStore.getState().rightPanelCollapsed).toBe(false);
    useUIStore.getState().toggleRightPanel();
    expect(useUIStore.getState().rightPanelCollapsed).toBe(true);
  });

  it("sets locale and persists to localStorage", () => {
    useUIStore.getState().setLocale("en");
    expect(useUIStore.getState().locale).toBe("en");
    expect(localStorage.getItem("serial-cli-locale")).toBe("en");
  });
});
