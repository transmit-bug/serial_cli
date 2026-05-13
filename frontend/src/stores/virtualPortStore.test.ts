import { describe, it, expect, beforeEach, vi } from 'vitest';
import { useVirtualPortStore } from '@/stores/virtualPortStore';
import { invoke } from '@tauri-apps/api/core';
import type { VirtualPortConfig, VirtualPortInfo, VirtualPortStats, CapturedPacket } from '@/types/tauri';

vi.mock('@tauri-apps/api/core');

const mockPort: VirtualPortInfo = {
  id: 'vp-001',
  port_a: '/dev/ttys000',
  port_b: '/dev/ttys001',
  backend: 'pty',
  created_at: '2026-05-13T12:00:00Z',
  uptime_secs: 120,
  running: true,
};

const mockStats: VirtualPortStats = {
  id: 'vp-001',
  port_a: '/dev/ttys000',
  port_b: '/dev/ttys001',
  backend: 'pty',
  running: true,
  uptime_secs: 120,
  bytes_bridged: 1024,
  packets_bridged: 50,
  bridge_errors: 0,
  last_error: null,
  capture_packets: 10,
  capture_bytes: 512,
  monitoring: true,
};

describe('virtualPortStore', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    useVirtualPortStore.setState({
      ports: new Map(),
      portStats: new Map(),
      capturedPackets: new Map(),
      selectedPort: null,
      loading: false,
      error: null,
    });
  });

  describe('initial state', () => {
    it('starts with empty ports', () => {
      const state = useVirtualPortStore.getState();
      expect(state.ports.size).toBe(0);
      expect(state.portStats.size).toBe(0);
      expect(state.capturedPackets.size).toBe(0);
    });

    it('has no selected port initially', () => {
      const state = useVirtualPortStore.getState();
      expect(state.selectedPort).toBeNull();
    });
  });

  describe('createPort', () => {
    const mockConfig: VirtualPortConfig = {
      name: 'test-port',
      backend: 'pty',
      monitor: true,
    };

    it('creates port and refreshes list', async () => {
      vi.mocked(invoke).mockImplementation(async (cmd: string) => {
        if (cmd === 'create_virtual_port') return 'vp-001';
        if (cmd === 'list_virtual_ports') return [mockPort];
      });

      const state = useVirtualPortStore.getState();
      const id = await state.createPort(mockConfig);

      expect(id).toBe('vp-001');
      expect(invoke).toHaveBeenCalledWith('create_virtual_port', { config: mockConfig });
      expect(invoke).toHaveBeenCalledWith('list_virtual_ports');
    });

    it('sets error on failure', async () => {
      vi.mocked(invoke).mockRejectedValue(new Error('PTY allocation failed'));

      const state = useVirtualPortStore.getState();
      await expect(state.createPort(mockConfig)).rejects.toThrow('PTY allocation failed');

      expect(useVirtualPortStore.getState().error).toBe('PTY allocation failed');
    });
  });

  describe('stopPort', () => {
    it('stops port and removes from local state', async () => {
      // Add port to state first
      useVirtualPortStore.setState({
        ports: new Map([['vp-001', mockPort]]),
        portStats: new Map([['vp-001', mockStats]]),
        capturedPackets: new Map([['vp-001', [{ data: [1, 2], direction: 'rx', timestamp_millis: Date.now() }]]]),
      });

      vi.mocked(invoke).mockResolvedValue(undefined);

      const state = useVirtualPortStore.getState();
      await state.stopPort('vp-001');

      expect(invoke).toHaveBeenCalledWith('stop_virtual_port', { id: 'vp-001' });

      const updated = useVirtualPortStore.getState();
      expect(updated.ports.has('vp-001')).toBe(false);
      expect(updated.portStats.has('vp-001')).toBe(false);
      expect(updated.capturedPackets.has('vp-001')).toBe(false);
    });

    it('sets error on failure', async () => {
      vi.mocked(invoke).mockRejectedValue(new Error('Port not found'));

      const state = useVirtualPortStore.getState();
      await expect(state.stopPort('nonexistent')).rejects.toThrow('Port not found');

      expect(useVirtualPortStore.getState().error).toBe('Port not found');
    });
  });

  describe('listPorts', () => {
    it('loads ports into Map keyed by id', async () => {
      vi.mocked(invoke).mockResolvedValue([mockPort]);

      const state = useVirtualPortStore.getState();
      await state.listPorts();

      expect(invoke).toHaveBeenCalledWith('list_virtual_ports');

      const updated = useVirtualPortStore.getState();
      expect(updated.ports.size).toBe(1);
      expect(updated.ports.has('vp-001')).toBe(true);
      expect(updated.ports.get('vp-001')?.port_a).toBe('/dev/ttys000');
    });

    it('clears error on success', async () => {
      useVirtualPortStore.setState({ error: 'previous error' });
      vi.mocked(invoke).mockResolvedValue([mockPort]);

      const state = useVirtualPortStore.getState();
      await state.listPorts();

      expect(useVirtualPortStore.getState().error).toBeNull();
    });

    it('sets error on failure', async () => {
      vi.mocked(invoke).mockRejectedValue(new Error('Failed to list virtual ports'));

      const state = useVirtualPortStore.getState();
      await state.listPorts();

      expect(useVirtualPortStore.getState().error).toBe('Failed to list virtual ports');
    });
  });

  describe('getPortStats', () => {
    it('fetches and stores stats', async () => {
      vi.mocked(invoke).mockResolvedValue(mockStats);

      const state = useVirtualPortStore.getState();
      const result = await state.getPortStats('vp-001');

      expect(invoke).toHaveBeenCalledWith('get_virtual_port_stats', { id: 'vp-001' });
      expect(result).toEqual(mockStats);

      const updated = useVirtualPortStore.getState();
      expect(updated.portStats.get('vp-001')).toEqual(mockStats);
    });

    it('throws on failure', async () => {
      vi.mocked(invoke).mockRejectedValue(new Error('Stats unavailable'));

      const state = useVirtualPortStore.getState();
      await expect(state.getPortStats('vp-001')).rejects.toThrow('Stats unavailable');
    });
  });

  describe('getCapturedPackets', () => {
    it('fetches and stores packets', async () => {
      const mockPackets: CapturedPacket[] = [
        { data: [1, 2, 3], direction: 'rx', timestamp_millis: 1000 },
        { data: [4, 5, 6], direction: 'tx', timestamp_millis: 2000 },
      ];
      vi.mocked(invoke).mockResolvedValue(mockPackets);

      const state = useVirtualPortStore.getState();
      const result = await state.getCapturedPackets('vp-001');

      expect(invoke).toHaveBeenCalledWith('get_captured_packets', { id: 'vp-001' });
      expect(result).toEqual(mockPackets);

      const updated = useVirtualPortStore.getState();
      expect(updated.capturedPackets.get('vp-001')).toEqual(mockPackets);
    });
  });

  describe('selectPort', () => {
    it('sets the selected port id', () => {
      const state = useVirtualPortStore.getState();
      state.selectPort('vp-001');
      expect(useVirtualPortStore.getState().selectedPort).toBe('vp-001');
    });

    it('can clear the selection', () => {
      useVirtualPortStore.setState({ selectedPort: 'vp-001' });

      const state = useVirtualPortStore.getState();
      state.selectPort(null);
      expect(useVirtualPortStore.getState().selectedPort).toBeNull();
    });
  });

  describe('clearError', () => {
    it('clears the error state', () => {
      useVirtualPortStore.setState({ error: 'test error' });

      const state = useVirtualPortStore.getState();
      state.clearError();
      expect(useVirtualPortStore.getState().error).toBeNull();
    });
  });
});
