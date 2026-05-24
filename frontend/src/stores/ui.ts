import { create } from "zustand";
import type { Locale, PageName } from "@/types";

interface UIStore {
  currentPage: PageName;
  sidebarCollapsed: boolean;
  rightPanelCollapsed: boolean;
  locale: Locale;

  navigateTo: (page: PageName) => void;
  toggleSidebar: () => void;
  toggleRightPanel: () => void;
  setLocale: (locale: Locale) => void;
}

export const useUIStore = create<UIStore>()((set) => ({
  currentPage: "terminal",
  sidebarCollapsed: false,
  rightPanelCollapsed: false,
  locale: (localStorage.getItem("serial-cli-locale") as Locale) || "zh",

  navigateTo: (page) => set({ currentPage: page }),
  toggleSidebar: () => set((s) => ({ sidebarCollapsed: !s.sidebarCollapsed })),
  toggleRightPanel: () =>
    set((s) => ({ rightPanelCollapsed: !s.rightPanelCollapsed })),
  setLocale: (locale) => {
    localStorage.setItem("serial-cli-locale", locale);
    set({ locale });
  },
}));
