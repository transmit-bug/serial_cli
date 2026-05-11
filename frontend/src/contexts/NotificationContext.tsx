import React, { createContext, useContext, useState, useCallback, useEffect } from 'react'
import { toast } from 'sonner'
import { isTauri } from '@/lib/utils'
import { settingsStorage } from '@/lib/storage'

export type ToastType = 'success' | 'error' | 'warning' | 'info'

export interface SystemNotificationOptions {
  title: string
  body: string
  type?: ToastType
  duration?: number
  sound?: boolean
}

interface NotificationContextType {
  // System-level notifications (OS notification)
  sendSystemNotification: (options: SystemNotificationOptions) => Promise<void>
  requestNotificationPermission: () => Promise<boolean>

  // Enhanced in-app notifications
  notify: {
    success: (message: string, title?: string) => void
    error: (message: string, title?: string) => void
    warning: (message: string, title?: string) => void
    info: (message: string, title?: string) => void
  }

  // Settings
  settings: NotificationSettings
  updateSettings: (settings: Partial<NotificationSettings>) => void
}

export interface NotificationSettings {
  enabled: boolean
  sound: boolean
  portEvents: boolean
  errors: boolean
  scriptComplete: boolean
  duration: number
}

const NotificationContext = createContext<NotificationContextType | undefined>(undefined)

const DEFAULT_SETTINGS: NotificationSettings = {
  enabled: true,
  sound: true,
  portEvents: true,
  errors: true,
  scriptComplete: true,
  duration: 3000,
}

export function NotificationProvider({ children }: { children: React.ReactNode }) {
  const [settings, setSettings] = useState<NotificationSettings>(DEFAULT_SETTINGS)
  const [permission, setPermission] = useState<NotificationPermission>('default')

  // Request notification permission on mount
  useEffect(() => {
    if ('Notification' in window && Notification.permission === 'default') {
      // Don't request automatically, let user trigger it
      setPermission(Notification.permission)
    }
  }, [])

  const requestNotificationPermission = useCallback(async (): Promise<boolean> => {
    if (!('Notification' in window)) {
      console.warn('This browser does not support desktop notification')
      return false
    }

    const result = await Notification.requestPermission()
    setPermission(result)
    return result === 'granted'
  }, [])

  const sendSystemNotification = useCallback(async (options: SystemNotificationOptions) => {
    if (!settings.enabled) return

    // Try Tauri notification first (desktop app)
    if (isTauri()) {
      try {
        // For Tauri, we'll use the browser notification API which works in desktop apps too
        // In production, you might want to use Tauri's notification plugin
        if ('Notification' in window) {
          if (Notification.permission !== 'granted') {
            const granted = await requestNotificationPermission()
            if (!granted) return
          }

          // Play sound if enabled
          if (settings.sound && options.sound !== false) {
            playNotificationSound(options.type || 'info')
          }

          const notification = new Notification(options.title, {
            body: options.body,
            icon: '/icons/32x32.png',
            tag: 'serial-cli-notification',
            requireInteraction: options.type === 'error',
          })

          if (options.type !== 'error') {
            setTimeout(() => notification.close(), options.duration || settings.duration)
          }

          notification.onclick = () => {
            window.focus()
            notification.close()
          }
        }
      } catch (e) {
        console.warn('Tauri notification failed, falling back to in-app:', e)
      }
    } else {
      // Browser-based notification
      if (permission !== 'granted') {
        const granted = await requestNotificationPermission()
        if (!granted) return
      }

      // Play sound if enabled
      if (settings.sound && options.sound !== false) {
        playNotificationSound(options.type || 'info')
      }

      // Show browser notification
      if ('Notification' in window && permission === 'granted') {
        const notification = new Notification(options.title, {
          body: options.body,
          icon: '/icons/32x32.png',
          tag: 'serial-cli-notification',
          requireInteraction: options.type === 'error',
        })

        if (options.type !== 'error') {
          setTimeout(() => notification.close(), options.duration || settings.duration)
        }

        notification.onclick = () => {
          window.focus()
          notification.close()
        }
      }
    }

    // Also show in-app toast for all platforms
    const toastFn = {
      success: toast.success,
      error: toast.error,
      warning: toast.warning,
      info: toast.info,
    }[options.type || 'info']
    toastFn(`${options.title}: ${options.body}`)
  }, [settings, permission, requestNotificationPermission])

  const notify = {
    success: (message: string, title = 'Success') =>
      sendSystemNotification({ title, body: message, type: 'success' }),
    error: (message: string, title = 'Error') =>
      sendSystemNotification({ title, body: message, type: 'error' }),
    warning: (message: string, title = 'Warning') =>
      sendSystemNotification({ title, body: message, type: 'warning' }),
    info: (message: string, title = 'Info') =>
      sendSystemNotification({ title, body: message, type: 'info' }),
  }

  const updateSettings = useCallback((newSettings: Partial<NotificationSettings>) => {
    setSettings(prev => {
      const updated = { ...prev, ...newSettings }
      settingsStorage.update({ notifications: updated })
      return updated
    })
  }, [])

  return (
    <NotificationContext.Provider value={{
      sendSystemNotification,
      requestNotificationPermission,
      notify,
      settings,
      updateSettings,
    }}>
      {children}
    </NotificationContext.Provider>
  )
}

export function useNotification() {
  const context = useContext(NotificationContext)
  if (!context) {
    throw new Error('useNotification must be used within NotificationProvider')
  }
  return context
}

// Simple notification sound using Web Audio API
function playNotificationSound(type: ToastType) {
  try {
    const audioContext = new AudioContext()
    const oscillator = audioContext.createOscillator()
    const gainNode = audioContext.createGain()

    oscillator.connect(gainNode)
    gainNode.connect(audioContext.destination)

    // Different sounds for different types
    const frequencies = {
      success: [800, 1000],
      error: [300, 200],
      warning: [600, 400],
      info: [500, 600],
    }

    const [startFreq, endFreq] = frequencies[type] || frequencies.info

    oscillator.type = 'sine'
    oscillator.frequency.setValueAtTime(startFreq, audioContext.currentTime)
    oscillator.frequency.exponentialRampToValueAtTime(endFreq, audioContext.currentTime + 0.1)

    gainNode.gain.setValueAtTime(0.1, audioContext.currentTime)
    gainNode.gain.exponentialRampToValueAtTime(0.01, audioContext.currentTime + 0.2)

    oscillator.start(audioContext.currentTime)
    oscillator.stop(audioContext.currentTime + 0.2)
  } catch (e) {
    console.warn('Failed to play notification sound:', e)
  }
}
