import { useEffect, useState } from 'react'
import { useEvents, ErrorEventData } from '@/hooks/useEvents'
import { toast } from 'sonner'
import { AlertCircle } from 'lucide-react'

/**
 * ErrorEventToast - Global error event handler
 *
 * Listens to error events and displays them as toast notifications
 */
export function ErrorEventToast() {
  const { onError } = useEvents()
  const [errorCount, setErrorCount] = useState(0)

  useEffect(() => {
    const cleanup = onError((data: ErrorEventData) => {
      setErrorCount((prev) => prev + 1)

      // Show error toast
      toast.error(`Error: ${data.error}`, {
        icon: <AlertCircle className="w-4 h-4" />,
        description: new Date(data.timestamp).toLocaleTimeString(),
      })
    })

    return cleanup
  }, [onError])

  // This component doesn't render anything visible
  // It only handles error events in the background
  return null
}

/**
 * useErrorCount - Hook to track error count
 *
 * @returns Object containing error count and reset function
 */
export function useErrorCount() {
  const { onError } = useEvents()
  const [errorCount, setErrorCount] = useState(0)
  const [recentErrors, setRecentErrors] = useState<ErrorEventData[]>([])

  useEffect(() => {
    const cleanup = onError((data: ErrorEventData) => {
      setErrorCount((prev) => prev + 1)
      setRecentErrors((prev) => {
        const newErrors = [data, ...prev]
        // Keep only last 10 errors
        return newErrors.slice(0, 10)
      })
    })

    return cleanup
  }, [onError])

  const resetErrorCount = () => setErrorCount(0)

  const clearRecentErrors = () => setRecentErrors([])

  return {
    errorCount,
    recentErrors,
    resetErrorCount,
    clearRecentErrors,
  }
}
