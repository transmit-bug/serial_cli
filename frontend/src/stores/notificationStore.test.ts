import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import { useNotificationStore } from '@/stores/notificationStore';

describe('notificationStore', () => {
  beforeEach(() => {
    vi.useFakeTimers();
    useNotificationStore.setState({
      notifications: [],
      maxNotifications: 5,
    });
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  describe('initial state', () => {
    it('starts with empty notifications', () => {
      const state = useNotificationStore.getState();
      expect(state.notifications).toEqual([]);
      expect(state.maxNotifications).toBe(5);
    });
  });

  describe('addNotification', () => {
    it('adds a notification with generated id and timestamp', () => {
      const state = useNotificationStore.getState();
      state.addNotification({
        type: 'info',
        title: 'Test',
        message: 'Test message',
      });

      const updated = useNotificationStore.getState();
      expect(updated.notifications).toHaveLength(1);
      expect(updated.notifications[0].title).toBe('Test');
      expect(updated.notifications[0].message).toBe('Test message');
      expect(updated.notifications[0].type).toBe('info');
      expect(updated.notifications[0].id).toBeDefined();
      expect(updated.notifications[0].timestamp).toBeDefined();
    });

    it('adds multiple notifications', () => {
      const state = useNotificationStore.getState();
      state.addNotification({ type: 'info', title: 'One', message: 'First' });
      state.addNotification({ type: 'success', title: 'Two', message: 'Second' });

      expect(useNotificationStore.getState().notifications).toHaveLength(2);
    });

    it('enforces maxNotifications limit', () => {
      const state = useNotificationStore.getState();

      for (let i = 0; i < 7; i++) {
        state.addNotification({
          type: 'info',
          title: `Notification ${i}`,
          message: `Message ${i}`,
        });
      }

      const updated = useNotificationStore.getState();
      expect(updated.notifications).toHaveLength(5);
      // Should keep the most recent 5 (indices 2-6)
      expect(updated.notifications[0].title).toBe('Notification 2');
      expect(updated.notifications[4].title).toBe('Notification 6');
    });

    it('schedules auto-removal when duration is set', () => {
      const state = useNotificationStore.getState();
      state.addNotification({
        type: 'success',
        title: 'Timeout',
        message: 'Gone soon',
        duration: 3000,
      });

      expect(useNotificationStore.getState().notifications).toHaveLength(1);

      // Advance past the duration
      vi.advanceTimersByTime(3001);

      expect(useNotificationStore.getState().notifications).toHaveLength(0);
    });

    it('does not auto-remove when duration is not set', () => {
      const state = useNotificationStore.getState();
      state.addNotification({
        type: 'info',
        title: 'Persistent',
        message: 'Stays forever',
      });

      vi.advanceTimersByTime(999999);

      expect(useNotificationStore.getState().notifications).toHaveLength(1);
    });
  });

  describe('removeNotification', () => {
    it('removes a notification by id', () => {
      const state = useNotificationStore.getState();
      state.addNotification({ type: 'info', title: 'Remove me', message: 'test' });

      const { id } = useNotificationStore.getState().notifications[0];
      state.removeNotification(id);

      expect(useNotificationStore.getState().notifications).toHaveLength(0);
    });

    it('does nothing when id not found', () => {
      const state = useNotificationStore.getState();
      state.addNotification({ type: 'info', title: 'Test', message: 'test' });

      state.removeNotification('nonexistent-id');

      expect(useNotificationStore.getState().notifications).toHaveLength(1);
    });
  });

  describe('clearNotifications', () => {
    it('removes all notifications', () => {
      const state = useNotificationStore.getState();
      state.addNotification({ type: 'info', title: 'One', message: 'test' });
      state.addNotification({ type: 'error', title: 'Two', message: 'test' });

      state.clearNotifications();

      expect(useNotificationStore.getState().notifications).toHaveLength(0);
    });
  });

  describe('notification types', () => {
    it('supports all notification types', () => {
      const state = useNotificationStore.getState();
      const types = ['info', 'success', 'warning', 'error'] as const;

      for (const type of types) {
        state.addNotification({ type, title: type, message: 'test' });
      }

      expect(useNotificationStore.getState().notifications).toHaveLength(4);
    });
  });
});
