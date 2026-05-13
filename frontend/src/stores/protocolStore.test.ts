import { describe, it, expect, beforeEach, vi } from 'vitest';
import { useProtocolStore } from '@/stores/protocolStore';
import { invoke } from '@tauri-apps/api/core';
import type { ProtocolInfo } from '@/types/tauri';

vi.mock('@tauri-apps/api/core');

const mockProtocols: ProtocolInfo[] = [
  { name: 'modbus-rtu', version: '1.0', description: 'Modbus RTU', author: 'serial-cli' },
  { name: 'modbus-ascii', version: '1.0', description: 'Modbus ASCII', author: 'serial-cli' },
];

describe('protocolStore', () => {
  beforeEach(() => {
    vi.resetAllMocks();
    useProtocolStore.setState({
      protocols: [],
      activeProtocol: null,
      loading: false,
      error: null,
    });
  });

  describe('initial state', () => {
    it('starts with empty protocols', () => {
      const state = useProtocolStore.getState();
      expect(state.protocols).toEqual([]);
      expect(state.activeProtocol).toBeNull();
    });

    it('is not loading initially', () => {
      const state = useProtocolStore.getState();
      expect(state.loading).toBe(false);
      expect(state.error).toBeNull();
    });
  });

  describe('loadProtocols', () => {
    it('loads protocols and sets loading false', async () => {
      vi.mocked(invoke).mockResolvedValue(mockProtocols);

      const state = useProtocolStore.getState();
      await state.loadProtocols();

      expect(invoke).toHaveBeenCalledWith('list_protocols');
      expect(useProtocolStore.getState().protocols).toEqual(mockProtocols);
      expect(useProtocolStore.getState().loading).toBe(false);
      expect(useProtocolStore.getState().error).toBeNull();
    });

    it('sets error on failure', async () => {
      vi.mocked(invoke).mockRejectedValue(new Error('Network error'));

      const state = useProtocolStore.getState();
      await state.loadProtocols();

      expect(useProtocolStore.getState().loading).toBe(false);
      expect(useProtocolStore.getState().error).toBe('Network error');
    });

    it('sets loading true during the call', async () => {
      let capturedLoading: boolean | null = null;
      vi.mocked(invoke).mockImplementation(async () => {
        capturedLoading = useProtocolStore.getState().loading;
        return mockProtocols;
      });

      const state = useProtocolStore.getState();
      await state.loadProtocols();

      expect(capturedLoading).toBe(true);
    });
  });

  describe('setActiveProtocol', () => {
    it('sets the active protocol', () => {
      const state = useProtocolStore.getState();
      state.setActiveProtocol('modbus-rtu');
      expect(useProtocolStore.getState().activeProtocol).toBe('modbus-rtu');
    });

    it('can set null', () => {
      const state = useProtocolStore.getState();
      state.setActiveProtocol('modbus-rtu');
      state.setActiveProtocol(null as any);
      expect(useProtocolStore.getState().activeProtocol).toBeNull();
    });
  });

  describe('enableProtocol', () => {
    it('refreshes the protocol list', async () => {
      vi.mocked(invoke).mockResolvedValue(mockProtocols);

      const state = useProtocolStore.getState();
      await state.enableProtocol('modbus-rtu');

      // enableProtocol delegates to loadProtocols which calls list_protocols
      expect(invoke).toHaveBeenCalledWith('list_protocols');
      expect(useProtocolStore.getState().loading).toBe(false);
    });

    it('sets error on failure', async () => {
      vi.mocked(invoke).mockRejectedValue(new Error('Load failed'));

      const state = useProtocolStore.getState();
      await state.enableProtocol('broken');

      expect(useProtocolStore.getState().error).toBe('Load failed');
    });
  });

  describe('disableProtocol', () => {
    it('calls unload_protocol and refreshes list', async () => {
      vi.mocked(invoke).mockResolvedValue(undefined);

      const state = useProtocolStore.getState();
      await state.disableProtocol('custom-protocol');

      expect(invoke).toHaveBeenCalledWith('unload_protocol', { name: 'custom-protocol' });
      expect(invoke).toHaveBeenCalledWith('list_protocols');
    });

    it('sets error on failure', async () => {
      vi.mocked(invoke).mockRejectedValue(new Error('Protocol not loaded'));

      const state = useProtocolStore.getState();
      await expect(state.disableProtocol('nonexistent')).rejects.toThrow('Protocol not loaded');

      expect(useProtocolStore.getState().error).toBe('Protocol not loaded');
    });
  });

  describe('reloadProtocol', () => {
    it('calls reload_protocol and refreshes list', async () => {
      vi.mocked(invoke).mockResolvedValue(undefined);

      const state = useProtocolStore.getState();
      await state.reloadProtocol('modbus-rtu');

      expect(invoke).toHaveBeenCalledWith('reload_protocol', { name: 'modbus-rtu' });
      expect(invoke).toHaveBeenCalledWith('list_protocols');
    });

    it('sets error on failure', async () => {
      vi.mocked(invoke).mockRejectedValue(new Error('Reload failed'));

      const state = useProtocolStore.getState();
      await expect(state.reloadProtocol('modbus-rtu')).rejects.toThrow('Reload failed');

      expect(useProtocolStore.getState().error).toBe('Reload failed');
    });
  });

  describe('clearError', () => {
    it('clears the error state', () => {
      useProtocolStore.setState({ error: 'test error' });

      const state = useProtocolStore.getState();
      state.clearError();
      expect(useProtocolStore.getState().error).toBeNull();
    });
  });
});
