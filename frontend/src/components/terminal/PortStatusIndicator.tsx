import { useEffect, useState } from 'react'
import { useEvents } from '@/hooks/useEvents'
import { useConnectionStore } from '@/stores'
import { Circle } from 'lucide-react'
import { cn } from '@/lib/utils'
import { useTranslation } from 'react-i18next'

/**
 * PortStatusIndicator - Real-time port status indicator
 *
 * Shows live connection status with event-driven updates
 */
export function PortStatusIndicator() {
  const { portId, portName } = useConnectionStore()
  const { onPortStatusChanged } = useEvents()
  const [lastActivity, setLastActivity] = useState<number | null>(null)
  const [isActive, setIsActive] = useState(false)
  const { t } = useTranslation()

  useEffect(() => {
    if (!portId) return

    const cleanup = onPortStatusChanged((event) => {
      if (event.port_id === portId) {
        setLastActivity(event.timestamp)
        setIsActive(true)

        // Reset active state after 100ms
        setTimeout(() => setIsActive(false), 100)
      }
    })

    return cleanup
  }, [portId, onPortStatusChanged])

  if (!portId || !portName) {
    return (
      <div className="flex items-center gap-2 text-xs text-text-tertiary">
        <Circle className="w-2 h-2 fill-current" />
        <span>{t('connection.disconnected')}</span>
      </div>
    )
  }

  return (
    <div className="flex items-center gap-2 text-xs">
      <Circle
        className={cn(
          'w-2 h-2 fill-current transition-colors',
          isActive ? 'text-signal animate-pulse' : 'text-green-500'
        )}
      />
      <span className="text-text-primary">{portName}</span>
      {lastActivity && (
        <span className="text-text-tertiary">
          {new Date(lastActivity).toLocaleTimeString()}
        </span>
      )}
    </div>
  )
}
