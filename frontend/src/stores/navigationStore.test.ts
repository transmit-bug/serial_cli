import { describe, it, expect, beforeEach } from 'vitest';
import { useNavigationStore } from '@/stores/navigationStore';

describe('navigationStore', () => {
  beforeEach(() => {
    useNavigationStore.setState({
      currentView: 'terminal',
      previousView: null,
      sidebarCollapsed: false,
    });
  });

  describe('initial state', () => {
    it('starts on terminal view', () => {
      const state = useNavigationStore.getState();
      expect(state.currentView).toBe('terminal');
      expect(state.previousView).toBeNull();
    });

    it('has sidebar expanded by default', () => {
      const state = useNavigationStore.getState();
      expect(state.sidebarCollapsed).toBe(false);
    });
  });

  describe('navigateTo', () => {
    it('changes the current view', () => {
      const state = useNavigationStore.getState();
      state.navigateTo('settings');
      expect(useNavigationStore.getState().currentView).toBe('settings');
    });

    it('stores the previous view', () => {
      const state = useNavigationStore.getState();
      state.navigateTo('data');

      const updated = useNavigationStore.getState();
      expect(updated.currentView).toBe('data');
      expect(updated.previousView).toBe('terminal');
    });

    it('supports all view types', () => {
      const views = ['ports', 'data', 'scripts', 'protocols', 'settings', 'virtual', 'terminal'] as const;
      const state = useNavigationStore.getState();

      for (const view of views) {
        state.navigateTo(view);
        expect(useNavigationStore.getState().currentView).toBe(view);
      }
    });
  });

  describe('goBack', () => {
    it('returns to the previous view', () => {
      const state = useNavigationStore.getState();
      state.navigateTo('settings');
      state.goBack();

      const updated = useNavigationStore.getState();
      expect(updated.currentView).toBe('terminal');
      expect(updated.previousView).toBeNull();
    });

    it('falls back to terminal if no previous view', () => {
      const state = useNavigationStore.getState();
      // Don't navigate anywhere, previousView is null
      state.goBack();

      const updated = useNavigationStore.getState();
      expect(updated.currentView).toBe('terminal');
      expect(updated.previousView).toBeNull();
    });

    it('clears the previous view after going back', () => {
      const state = useNavigationStore.getState();
      state.navigateTo('data');
      state.navigateTo('settings');
      state.goBack();

      const updated = useNavigationStore.getState();
      expect(updated.currentView).toBe('data');
      expect(updated.previousView).toBeNull();
    });
  });

  describe('toggleSidebar', () => {
    it('collapses the sidebar if expanded', () => {
      const state = useNavigationStore.getState();
      expect(state.sidebarCollapsed).toBe(false);

      state.toggleSidebar();
      expect(useNavigationStore.getState().sidebarCollapsed).toBe(true);
    });

    it('expands the sidebar if collapsed', () => {
      const state = useNavigationStore.getState();
      state.setSidebarCollapsed(true);

      state.toggleSidebar();
      expect(useNavigationStore.getState().sidebarCollapsed).toBe(false);
    });
  });

  describe('setSidebarCollapsed', () => {
    it('sets the sidebar collapse state directly', () => {
      const state = useNavigationStore.getState();
      state.setSidebarCollapsed(true);
      expect(useNavigationStore.getState().sidebarCollapsed).toBe(true);

      state.setSidebarCollapsed(false);
      expect(useNavigationStore.getState().sidebarCollapsed).toBe(false);
    });
  });
});
