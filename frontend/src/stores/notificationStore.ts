import { create } from 'zustand'

export type NotificationType = 'info' | 'success' | 'warning' | 'error'

export interface Notification {
  id: string
  type: NotificationType
  title: string
  message: string
  duration?: number
  timestamp: number
}

interface NotificationState {
  notifications: Notification[]
  maxNotifications: number

  // Actions
  addNotification: (notification: Omit<Notification, 'id' | 'timestamp'>) => void
  removeNotification: (id: string) => void
  clearNotifications: () => void
}

export const useNotificationStore = create<NotificationState>((set, get) => ({
  // Initial state
  notifications: [],
  maxNotifications: 5,

  // Actions
  addNotification: (notification) => {
    const newNotification: Notification = {
      ...notification,
      id: `${Date.now()}-${Math.random()}`,
      timestamp: Date.now(),
    }

    set((state) => {
      const notifications = [...state.notifications, newNotification]

      // Auto-remove after duration
      if (newNotification.duration) {
        setTimeout(() => {
          get().removeNotification(newNotification.id)
        }, newNotification.duration)
      }

      // Limit to maxNotifications
      if (notifications.length > state.maxNotifications) {
        return { notifications: notifications.slice(-state.maxNotifications) }
      }

      return { notifications }
    })
  },

  removeNotification: (id) =>
    set((state) => ({
      notifications: state.notifications.filter((n) => n.id !== id),
    })),

  clearNotifications: () => set({ notifications: [] }),
}))
