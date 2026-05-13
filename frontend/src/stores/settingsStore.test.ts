import { describe, it, expect, beforeEach, vi } from 'vitest';
import { useSettingsStore } from '@/stores/settingsStore';
import type { AppConfig } from '@/stores/settingsStore';
import { invoke } from '@tauri-apps/api/core';

vi.mock('@tauri-apps/api/core');

const mockConfig: AppConfig = {
  serial: { defaultBaudrate: 9600, timeoutMs: 1000 },
  protocols: { hotReload: false, customDir: '' },
  virtualPorts: { backend: 'pty', monitor: true },
  display: { theme: 'dark', maxPackets: 10000, format: 'hex', showTimestamp: true },
};

describe('settingsStore', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    useSettingsStore.setState({
      config: null,
      loading: false,
      saving: false,
      error: null,
    });
  });

  describe('initial state', () => {
    it('starts with null config', () => {
      const state = useSettingsStore.getState();
      expect(state.config).toBeNull();
    });

    it('is not loading initially', () => {
      const state = useSettingsStore.getState();
      expect(state.loading).toBe(false);
      expect(state.saving).toBe(false);
      expect(state.error).toBeNull();
    });
  });

  describe('updateConfig', () => {
    it('merges updates into existing config', () => {
      // First set config via loadConfig mock
      vi.mocked(invoke).mockResolvedValue(mockConfig);

      // Set config directly
      useSettingsStore.setState({ config: mockConfig });
      const state = useSettingsStore.getState();
      state.updateConfig({
        serial: { defaultBaudrate: 115200, timeoutMs: 2000 },
      });

      const updated = useSettingsStore.getState();
      expect(updated.config?.serial.defaultBaudrate).toBe(115200);
      // Other fields should be preserved
      expect(updated.config?.protocols.hotReload).toBe(false);
      expect(updated.config?.display.theme).toBe('dark');
    });

    it('does nothing when config is null', () => {
      useSettingsStore.setState({ config: null });

      const state = useSettingsStore.getState();
      state.updateConfig({ serial: { defaultBaudrate: 9600, timeoutMs: 1000 } });

      const updated = useSettingsStore.getState();
      expect(updated.config).toBeNull();
    });

    it('only updates specified fields', () => {
      useSettingsStore.setState({ config: mockConfig });
      const state = useSettingsStore.getState();
      state.updateConfig({
        display: { theme: 'light', maxPackets: 5000, format: 'ascii', showTimestamp: false },
      });

      const updated = useSettingsStore.getState();
      expect(updated.config?.display.theme).toBe('light');
      // serial should be unchanged
      expect(updated.config?.serial.defaultBaudrate).toBe(9600);
    });
  });

  describe('clearError', () => {
    it('clears the error state', () => {
      useSettingsStore.setState({ error: 'test error' });

      const state = useSettingsStore.getState();
      state.clearError();
      expect(useSettingsStore.getState().error).toBeNull();
    });
  });

  describe('loadConfig', () => {
    it('sets loading and calls invoke', async () => {
      vi.mocked(invoke).mockResolvedValue(mockConfig);

      const state = useSettingsStore.getState();
      await state.loadConfig();

      expect(invoke).toHaveBeenCalledWith('get_config');
      expect(useSettingsStore.getState().config).toEqual(mockConfig);
      expect(useSettingsStore.getState().loading).toBe(false);
      expect(useSettingsStore.getState().error).toBeNull();
    });

    it('sets error on failure', async () => {
      vi.mocked(invoke).mockRejectedValue(new Error('Permission denied'));

      const state = useSettingsStore.getState();
      await state.loadConfig();

      expect(useSettingsStore.getState().loading).toBe(false);
      expect(useSettingsStore.getState().error).toBe('Permission denied');
    });
  });

  describe('saveConfig', () => {
    it('calls invoke with config when config exists', async () => {
      vi.mocked(invoke).mockResolvedValue(undefined);
      useSettingsStore.setState({ config: mockConfig });

      const state = useSettingsStore.getState();
      await state.saveConfig();

      expect(invoke).toHaveBeenCalledWith('update_config', { config: mockConfig });
      expect(useSettingsStore.getState().saving).toBe(false);
    });

    it('does nothing when config is null', async () => {
      useSettingsStore.setState({ config: null });

      const state = useSettingsStore.getState();
      await state.saveConfig();

      expect(invoke).not.toHaveBeenCalled();
    });

    it('sets error on failure and throws', async () => {
      useSettingsStore.setState({ config: mockConfig });
      vi.mocked(invoke).mockRejectedValue(new Error('Permission denied'));

      const state = useSettingsStore.getState();
      await expect(state.saveConfig()).rejects.toThrow('Permission denied');

      expect(useSettingsStore.getState().saving).toBe(false);
      expect(useSettingsStore.getState().error).toBe('Permission denied');
    });
  });
});
