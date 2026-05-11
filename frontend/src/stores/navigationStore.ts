import { create } from 'zustand'

export type View = 'ports' | 'data' | 'scripts' | 'protocols' | 'settings' | 'virtual' | 'terminal'

interface NavigationState {
  currentView: View
  previousView: View | null
  sidebarCollapsed: boolean

  // Actions
  navigateTo: (view: View) => void
  goBack: () => void
  toggleSidebar: () => void
  setSidebarCollapsed: (collapsed: boolean) => void
}

export const useNavigationStore = create<NavigationState>((set, get) => ({
  // Initial state
  currentView: 'terminal',
  previousView: null,
  sidebarCollapsed: false,

  // Actions
  navigateTo: (view) =>
    set((state) => ({
      previousView: state.currentView,
      currentView: view,
    })),

  goBack: () =>
    set((state) => ({
      currentView: state.previousView || 'terminal',
      previousView: null,
    })),

  toggleSidebar: () =>
    set((state) => ({ sidebarCollapsed: !state.sidebarCollapsed })),

  setSidebarCollapsed: (collapsed) => set({ sidebarCollapsed: collapsed }),
}))
