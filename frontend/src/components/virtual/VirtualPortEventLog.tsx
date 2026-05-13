import { useState, useEffect } from 'react'
import { useEvents, VirtualPortEventData } from '@/hooks/useEvents'
import { Panel } from '@/components/ui/panel'
import { Activity } from 'lucide-react'
import { cn } from '@/lib/utils'

interface EventLogEntry {
  id: string
  type: 'created' | 'stopped' | 'stats-updated'
  portId: string
  timestamp: number
  details?: string
}

/**
 * VirtualPortEventLog - Real-time event log for virtual ports
 *
 * Shows a live feed of virtual port events with filtering
 */
export function VirtualPortEventLog({ filterPortId }: { filterPortId?: string }) {
  const { onVirtualPortCreated, onVirtualPortStopped, onVirtualPortStatsUpdated } = useEvents()
  const [events, setEvents] = useState<EventLogEntry[]>([])

  const addEvent = (
    type: EventLogEntry['type'],
    data: VirtualPortEventData,
    details?: string
  ) => {
    if (filterPortId && data.port_id !== filterPortId) return

    const entry: EventLogEntry = {
      id: `${type}-${data.timestamp}-${Math.random()}`,
      type,
      portId: data.port_id,
      timestamp: data.timestamp,
      details,
    }

    setEvents((prev) => {
      const newEvents = [entry, ...prev]
      // Keep only last 50 events
      return newEvents.slice(0, 50)
    })
  }

  useEffect(() => {
    const cleanupCreated = onVirtualPortCreated((data) => {
      addEvent('created', data, `Virtual port created: ${data.port_id}`)
    })

    const cleanupStopped = onVirtualPortStopped((data) => {
      addEvent('stopped', data, `Virtual port stopped: ${data.port_id}`)
    })

    const cleanupStats = onVirtualPortStatsUpdated((data) => {
      const packetsBridged = data.stats?.packets_bridged ?? 0
      const bytesBridged = data.stats?.bytes_bridged ?? 0
      addEvent('stats-updated', data, `Stats: ${packetsBridged} packets, ${bytesBridged} bytes`)
    })

    return () => {
      cleanupCreated()
      cleanupStopped()
      cleanupStats()
    }
  }, [onVirtualPortCreated, onVirtualPortStopped, onVirtualPortStatsUpdated, filterPortId])

  const clearEvents = () => setEvents([])

  const getEventColor = (type: EventLogEntry['type']) => {
    switch (type) {
      case 'created':
        return 'text-signal'
      case 'stopped':
        return 'text-alert'
      case 'stats-updated':
        return 'text-info'
      default:
        return 'text-text-secondary'
    }
  }

  const getEventIcon = (type: EventLogEntry['type']) => {
    switch (type) {
      case 'created':
        return '🟢'
      case 'stopped':
        return '🔴'
      case 'stats-updated':
        return '📊'
      default:
        return '•'
    }
  }

  return (
    <Panel
      title="Virtual Port Events"
      variant="default"
      actions={
        <button
          onClick={clearEvents}
          className="text-xs text-text-tertiary hover:text-text-primary transition-colors"
        >
          Clear
        </button>
      }
    >
      <div className="space-y-2 max-h-96 overflow-y-auto">
        {events.length === 0 ? (
          <div className="text-center py-8 text-text-tertiary text-sm">
            <Activity className="w-8 h-8 mx-auto mb-2 opacity-50" />
            <p>No events yet</p>
            <p className="text-xs mt-1">Virtual port events will appear here</p>
          </div>
        ) : (
          events.map((event) => (
            <div
              key={event.id}
              className="flex items-start gap-2 text-xs p-2 rounded bg-bg-base hover:bg-bg-elevated transition-colors"
            >
              <span className="text-base">{getEventIcon(event.type)}</span>
              <div className="flex-1 min-w-0">
                <div className="flex items-center justify-between gap-2">
                  <span className={cn('font-medium', getEventColor(event.type))}>
                    {event.type.replace('-', ' ')}
                  </span>
                  <span className="text-text-tertiary text-[10px]">
                    {new Date(event.timestamp).toLocaleTimeString()}
                  </span>
                </div>
                <div className="text-text-secondary mt-0.5 truncate">{event.details}</div>
                <div className="text-text-tertiary text-[10px] font-mono">{event.portId}</div>
              </div>
            </div>
          ))
        )}
      </div>
    </Panel>
  )
}
