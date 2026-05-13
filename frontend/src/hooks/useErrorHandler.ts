import { useCallback } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { toast } from 'sonner'
import { useNotificationStore } from '@/stores/notificationStore'

/**
 * Global error handler hook for Tauri invoke calls.
 * Wraps `invoke()` to automatically route errors through toast + notificationStore.
 */
export function useErrorHandler() {
  const addNotification = useNotificationStore((s) => s.addNotification)

  /**
   * Invoke a Tauri command with automatic error handling.
   * Returns the result on success, or undefined on failure.
   */
  const invokeSafe = useCallback(
    async <T>(command: string, args?: Record<string, unknown>): Promise<T | undefined> => {
      try {
        const result = await invoke<T>(command, args)
        return result
      } catch (err) {
        const message = err instanceof Error ? err.message : String(err)

        // Show toast for immediate feedback
        toast.error(message)

        // Add to notification store for persistent visibility
        addNotification({
          type: 'error',
          title: `${command} failed`,
          message,
          duration: 10000,
        })

        return undefined
      }
    },
    [addNotification],
  )

  /**
   * Log a message to the notification store (for non-error logs).
   */
  const log = useCallback(
    (type: 'info' | 'success' | 'warning', title: string, message: string) => {
      addNotification({ type, title, message, duration: 5000 })
    },
    [addNotification],
  )

  return { invokeSafe, log }
}
