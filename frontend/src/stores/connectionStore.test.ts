import { describe, it, expect, beforeEach, vi } from 'vitest';
import { useConnectionStore } from '@/stores/connectionStore';
import { invoke } from '@tauri-apps/api/core';

vi.mock('@tauri-apps/api/core');

describe('connectionStore', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    useConnectionStore.setState({
      status: 'disconnected',
      portId: null,
      portName: null,
      config: null,
      error: null,
      portStatus: null,
    });
  });

  describe('initial state', () => {
    it('starts disconnected', () => {
      const state = useConnectionStore.getState();
      expect(state.status).toBe('disconnected');
      expect(state.portId).toBeNull();
      expect(state.portName).toBeNull();
    });
  });

  describe('connect', () => {
    const mockConfig = {
      baudrate: 9600,
      databits: 8,
      stopbits: 1,
      parity: 'none',
      timeout_ms: 1000,
      flow_control: 'none',
      dtr_enable: false,
      rts_enable: false,
    };

    it('transitions to connecting then connected', async () => {
      vi.mocked(invoke).mockImplementation(async (cmd: string) => {
        if (cmd === 'open_port') return 'port-123';
        if (cmd === 'start_sniffing') return undefined;
      });

      const state = useConnectionStore.getState();
      await state.connect('/dev/ttyUSB0', mockConfig);

      const updated = useConnectionStore.getState();
      expect(updated.status).toBe('connected');
      expect(updated.portId).toBe('port-123');
      expect(updated.portName).toBe('/dev/ttyUSB0');
      expect(updated.config).toEqual(mockConfig);
      expect(updated.error).toBeNull();
    });

    it('starts port sniffing after opening', async () => {
      vi.mocked(invoke).mockImplementation(async (cmd: string) => {
        if (cmd === 'open_port') return 'port-456';
        if (cmd === 'start_sniffing') return undefined;
      });

      const state = useConnectionStore.getState();
      await state.connect('/dev/ttyUSB0', mockConfig);

      expect(invoke).toHaveBeenCalledWith('open_port', {
        portName: '/dev/ttyUSB0',
        config: mockConfig,
      });
      expect(invoke).toHaveBeenCalledWith('start_sniffing', { portId: 'port-456' });
    });

    it('sets portStatus with initial zero stats', async () => {
      vi.mocked(invoke).mockImplementation(async (cmd: string) => {
        if (cmd === 'open_port') return 'port-789';
        if (cmd === 'start_sniffing') return undefined;
      });

      const state = useConnectionStore.getState();
      await state.connect('/dev/ttyUSB0', mockConfig);

      const updated = useConnectionStore.getState();
      expect(updated.portStatus).not.toBeNull();
      expect(updated.portStatus?.stats.bytes_sent).toBe(0);
      expect(updated.portStatus?.stats.bytes_received).toBe(0);
    });

    it('sets error status on open failure', async () => {
      vi.mocked(invoke).mockRejectedValue(new Error('Port not found'));

      const state = useConnectionStore.getState();
      await expect(state.connect('/dev/nonexistent', mockConfig)).rejects.toThrow('Port not found');

      const updated = useConnectionStore.getState();
      expect(updated.status).toBe('error');
      expect(updated.error).toBe('Port not found');
    });

    it('still connects if sniffing fails (non-fatal)', async () => {
      let callCount = 0;
      vi.mocked(invoke).mockImplementation(async (cmd: string) => {
        callCount++;
        if (cmd === 'open_port') return 'port-ok';
        if (cmd === 'start_sniffing') throw new Error('Sniffing failed');
      });

      const state = useConnectionStore.getState();
      // Should not throw - sniffing failure is swallowed
      await state.connect('/dev/ttyUSB0', mockConfig);

      const updated = useConnectionStore.getState();
      expect(updated.status).toBe('connected');
    });
  });

  describe('disconnect', () => {
    it('closes port and resets state', async () => {
      // First connect manually
      useConnectionStore.setState({
        status: 'connected',
        portId: 'port-123',
        portName: '/dev/ttyUSB0',
        config: {
          baudrate: 9600,
          databits: 8,
          stopbits: 1,
          parity: 'none',
          timeout_ms: 1000,
          flow_control: 'none',
          dtr_enable: false,
          rts_enable: false,
        },
        portStatus: {
          port_id: 'port-123',
          port_name: '/dev/ttyUSB0',
          is_open: true,
          config: {
            baudrate: 9600,
            databits: 8,
            stopbits: 1,
            parity: 'none',
            timeout_ms: 1000,
            flow_control: 'none',
            dtr_enable: false,
            rts_enable: false,
          },
          stats: {
            bytes_sent: 100,
            bytes_received: 200,
            packets_sent: 5,
            packets_received: 10,
            last_activity: Date.now(),
          },
        },
      });

      vi.mocked(invoke).mockResolvedValue(undefined);

      const state = useConnectionStore.getState();
      await state.disconnect();

      expect(invoke).toHaveBeenCalledWith('close_port', { portId: 'port-123' });
      const updated = useConnectionStore.getState();
      expect(updated.status).toBe('disconnected');
      expect(updated.portId).toBeNull();
      expect(updated.portStatus).toBeNull();
    });

    it('does nothing when not connected', async () => {
      const state = useConnectionStore.getState();
      await state.disconnect();

      expect(invoke).not.toHaveBeenCalled();
      expect(useConnectionStore.getState().status).toBe('disconnected');
    });

    it('sets error on close failure', async () => {
      useConnectionStore.setState({
        status: 'connected',
        portId: 'port-123',
        portName: '/dev/ttyUSB0',
        config: null,
      });

      vi.mocked(invoke).mockRejectedValue(new Error('Close failed'));

      const state = useConnectionStore.getState();
      await expect(state.disconnect()).rejects.toThrow('Close failed');

      const updated = useConnectionStore.getState();
      expect(updated.error).toBe('Close failed');
    });
  });

  describe('checkHealth', () => {
    it('returns true when port is healthy', async () => {
      useConnectionStore.setState({ portId: 'port-123' });
      vi.mocked(invoke).mockResolvedValue(true);

      const state = useConnectionStore.getState();
      const result = await state.checkHealth();

      expect(result).toBe(true);
      expect(invoke).toHaveBeenCalledWith('check_port_health', { portId: 'port-123' });
    });

    it('resets state when port is unhealthy', async () => {
      useConnectionStore.setState({
        portId: 'port-123',
        status: 'connected',
        portName: '/dev/ttyUSB0',
        config: {
          baudrate: 9600,
          databits: 8,
          stopbits: 1,
          parity: 'none',
          timeout_ms: 1000,
          flow_control: 'none',
          dtr_enable: false,
          rts_enable: false,
        },
        portStatus: {
          port_id: 'port-123',
          port_name: '/dev/ttyUSB0',
          is_open: true,
          config: {
            baudrate: 9600,
            databits: 8,
            stopbits: 1,
            parity: 'none',
            timeout_ms: 1000,
            flow_control: 'none',
            dtr_enable: false,
            rts_enable: false,
          },
          stats: {
            bytes_sent: 100,
            bytes_received: 200,
            packets_sent: 5,
            packets_received: 10,
            last_activity: Date.now(),
          },
        },
      });
      vi.mocked(invoke).mockResolvedValue(false);

      const state = useConnectionStore.getState();
      const result = await state.checkHealth();

      expect(result).toBe(false);
      const updated = useConnectionStore.getState();
      expect(updated.status).toBe('disconnected');
      expect(updated.portId).toBeNull();
      expect(updated.portStatus).toBeNull();
    });

    it('returns false when no port is connected', async () => {
      const state = useConnectionStore.getState();
      const result = await state.checkHealth();

      expect(result).toBe(false);
      expect(invoke).not.toHaveBeenCalled();
    });

    it('returns false on check failure', async () => {
      useConnectionStore.setState({ portId: 'port-123' });
      vi.mocked(invoke).mockRejectedValue(new Error('Health check failed'));

      const state = useConnectionStore.getState();
      const result = await state.checkHealth();

      expect(result).toBe(false);
    });
  });

  describe('setStatus', () => {
    it('sets the connection status directly', () => {
      const state = useConnectionStore.getState();
      state.setStatus('connecting');
      expect(useConnectionStore.getState().status).toBe('connecting');
    });
  });

  describe('setError', () => {
    it('sets the error message', () => {
      const state = useConnectionStore.getState();
      state.setError('Test error');
      expect(useConnectionStore.getState().error).toBe('Test error');
    });

    it('clears the error when passed null', () => {
      useConnectionStore.setState({ error: 'existing error' });
      const state = useConnectionStore.getState();
      state.setError(null);
      expect(useConnectionStore.getState().error).toBeNull();
    });
  });
});
