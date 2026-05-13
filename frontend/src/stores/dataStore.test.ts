import { describe, it, expect, beforeEach } from 'vitest';
import { useDataStore } from '@/stores/dataStore';
import type { DataPacket } from '@/stores/dataStore';

describe('dataStore', () => {
  beforeEach(() => {
    useDataStore.setState({
      rxPackets: [],
      txPackets: [],
      maxPackets: 10000,
      displayFormat: 'hex',
      showTimestamp: true,
    });
  });

  describe('initial state', () => {
    it('starts with empty packets', () => {
      const state = useDataStore.getState();
      expect(state.rxPackets).toEqual([]);
      expect(state.txPackets).toEqual([]);
    });

    it('has correct default settings', () => {
      const state = useDataStore.getState();
      expect(state.maxPackets).toBe(10000);
      expect(state.displayFormat).toBe('hex');
      expect(state.showTimestamp).toBe(true);
    });
  });

  describe('addRxPacket', () => {
    it('adds a received packet with generated id', () => {
      const state = useDataStore.getState();
      state.addRxPacket({
        portId: 'test-port',
        direction: 'rx',
        data: [1, 2, 3],
        timestamp: Date.now(),
      });

      const updated = useDataStore.getState();
      expect(updated.rxPackets).toHaveLength(1);
      expect(updated.rxPackets[0].portId).toBe('test-port');
      expect(updated.rxPackets[0].data).toEqual([1, 2, 3]);
      expect(updated.rxPackets[0].direction).toBe('rx');
      expect(updated.rxPackets[0].id).toBeDefined();
    });

    it('applies current display format to new packets', () => {
      const state = useDataStore.getState();
      state.setDisplayFormat('ascii');
      state.addRxPacket({
        portId: 'test-port',
        direction: 'rx',
        data: [65],
        timestamp: Date.now(),
      });

      const updated = useDataStore.getState();
      expect(updated.rxPackets[0].displayFormat).toBe('ascii');
    });
  });

  describe('addTxPacket', () => {
    it('adds a transmitted packet with generated id', () => {
      const state = useDataStore.getState();
      state.addTxPacket({
        portId: 'test-port',
        direction: 'tx',
        data: [4, 5, 6],
        timestamp: Date.now(),
      });

      const updated = useDataStore.getState();
      expect(updated.txPackets).toHaveLength(1);
      expect(updated.txPackets[0].portId).toBe('test-port');
      expect(updated.txPackets[0].data).toEqual([4, 5, 6]);
      expect(updated.txPackets[0].direction).toBe('tx');
    });
  });

  describe('clearPackets', () => {
    it('clears both rx and tx packets', () => {
      const state = useDataStore.getState();
      state.addRxPacket({
        portId: 'p1',
        direction: 'rx',
        data: [1],
        timestamp: Date.now(),
      });
      state.addTxPacket({
        portId: 'p1',
        direction: 'tx',
        data: [2],
        timestamp: Date.now(),
      });

      state.clearPackets();

      const updated = useDataStore.getState();
      expect(updated.rxPackets).toEqual([]);
      expect(updated.txPackets).toEqual([]);
    });
  });

  describe('setDisplayFormat', () => {
    it('changes the display format', () => {
      const state = useDataStore.getState();
      state.setDisplayFormat('mixed');
      expect(useDataStore.getState().displayFormat).toBe('mixed');
    });

    it('clears existing packets when format changes', () => {
      const state = useDataStore.getState();
      state.addRxPacket({
        portId: 'p1',
        direction: 'rx',
        data: [1],
        timestamp: Date.now(),
      });

      state.setDisplayFormat('ascii');

      const updated = useDataStore.getState();
      expect(updated.rxPackets).toEqual([]);
      expect(updated.txPackets).toEqual([]);
    });
  });

  describe('setShowTimestamp', () => {
    it('toggles the timestamp visibility', () => {
      const state = useDataStore.getState();
      expect(state.showTimestamp).toBe(true);

      state.setShowTimestamp(false);
      expect(useDataStore.getState().showTimestamp).toBe(false);
    });
  });

  describe('setMaxPackets', () => {
    it('changes the max packets limit', () => {
      const state = useDataStore.getState();
      state.setMaxPackets(5000);
      expect(useDataStore.getState().maxPackets).toBe(5000);
    });

    it('trims existing packets when max is reduced below current count', () => {
      const state = useDataStore.getState();

      // Add 5 packets
      for (let i = 0; i < 5; i++) {
        state.addRxPacket({
          portId: `p${i}`,
          direction: 'rx',
          data: [i],
          timestamp: Date.now(),
        });
      }

      // Reduce max to 3
      state.setMaxPackets(3);

      const updated = useDataStore.getState();
      expect(updated.rxPackets).toHaveLength(3);
      // Should keep the most recent 3 (indices 2, 3, 4)
      expect(updated.rxPackets[0].data).toEqual([2]);
      expect(updated.rxPackets[2].data).toEqual([4]);
    });

    it('enforces FIFO cleanup on add when at capacity', () => {
      const state = useDataStore.getState();
      state.setMaxPackets(3);

      // Add 4 packets
      for (let i = 0; i < 4; i++) {
        state.addRxPacket({
          portId: `p${i}`,
          direction: 'rx',
          data: [i],
          timestamp: Date.now(),
        });
      }

      const updated = useDataStore.getState();
      expect(updated.rxPackets).toHaveLength(3);
      // Should keep most recent 3 (1, 2, 3)
      expect(updated.rxPackets[0].data).toEqual([1]);
      expect(updated.rxPackets[2].data).toEqual([3]);
    });
  });
});
