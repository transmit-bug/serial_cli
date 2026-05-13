import { useEffect, useRef, useCallback } from 'react'
import { listen, UnlistenFn } from '@tauri-apps/api/event'
import type { PortStatus, PortStats, VirtualPortInfo, VirtualPortStats } from '@/types/tauri'

export interface SerialEventData {
  port_id: string
  data: number[]
  timestamp: number
  direction: 'rx' | 'tx'
}

export interface PortStatusEventData {
  port_id: string
  status: PortStatus
  timestamp: number
}

export interface VirtualPortEventData {
  port_id: string
  port_info?: VirtualPortInfo
  stats?: VirtualPortStats
  timestamp: number
}

export interface ErrorEventData {
  error: string
  timestamp: number
}

type EventCallback<T> = (event: T) => void

interface EventFilter<T> {
  callback: EventCallback<T>
  filter?: (event: T) => boolean
}

/**
 * useEvents - Hook for managing Tauri events with filtering and subscription management
 */
export function useEvents() {
  const listenersRef = useRef<Map<string, UnlistenFn>>(new Map())

  /**
   * Listen to data received events
   */
  const onDataReceived = useCallback(
    (callback: EventCallback<SerialEventData>, filter?: (event: SerialEventData) => boolean) => {
      const unlisten = listen<SerialEventData>('data-received', (event) => {
        if (!filter || filter(event.payload)) {
          callback(event.payload)
        }
      })

      unlisten.then((fn) => {
        listenersRef.current.set('data-received', fn)
      })

      return () => {
        listenersRef.current.get('data-received')?.()
      }
    },
    []
  )

  /**
   * Listen to data sent events
   */
  const onDataSent = useCallback(
    (callback: EventCallback<SerialEventData>, filter?: (event: SerialEventData) => boolean) => {
      const unlisten = listen<SerialEventData>('data-sent', (event) => {
        if (!filter || filter(event.payload)) {
          callback(event.payload)
        }
      })

      unlisten.then((fn) => {
        listenersRef.current.set('data-sent', fn)
      })

      return () => {
        listenersRef.current.get('data-sent')?.()
      }
    },
    []
  )

  /**
   * Listen to port status changed events
   */
  const onPortStatusChanged = useCallback(
    (callback: EventCallback<PortStatusEventData>, filter?: (event: PortStatusEventData) => boolean) => {
      const unlisten = listen<PortStatusEventData>('port-status-changed', (event) => {
        if (!filter || filter(event.payload)) {
          callback(event.payload)
        }
      })

      unlisten.then((fn) => {
        listenersRef.current.set('port-status-changed', fn)
      })

      return () => {
        listenersRef.current.get('port-status-changed')?.()
      }
    },
    []
  )

  /**
   * Listen to virtual port created events
   */
  const onVirtualPortCreated = useCallback(
    (callback: EventCallback<VirtualPortEventData>, filter?: (event: VirtualPortEventData) => boolean) => {
      const unlisten = listen<VirtualPortEventData>('virtual-port-created', (event) => {
        if (!filter || filter(event.payload)) {
          callback(event.payload)
        }
      })

      unlisten.then((fn) => {
        listenersRef.current.set('virtual-port-created', fn)
      })

      return () => {
        listenersRef.current.get('virtual-port-created')?.()
      }
    },
    []
  )

  /**
   * Listen to virtual port stopped events
   */
  const onVirtualPortStopped = useCallback(
    (callback: EventCallback<VirtualPortEventData>, filter?: (event: VirtualPortEventData) => boolean) => {
      const unlisten = listen<VirtualPortEventData>('virtual-port-stopped', (event) => {
        if (!filter || filter(event.payload)) {
          callback(event.payload)
        }
      })

      unlisten.then((fn) => {
        listenersRef.current.set('virtual-port-stopped', fn)
      })

      return () => {
        listenersRef.current.get('virtual-port-stopped')?.()
      }
    },
    []
  )

  /**
   * Listen to virtual port stats updated events
   */
  const onVirtualPortStatsUpdated = useCallback(
    (callback: EventCallback<VirtualPortEventData>, filter?: (event: VirtualPortEventData) => boolean) => {
      const unlisten = listen<VirtualPortEventData>('virtual-port-stats-updated', (event) => {
        if (!filter || filter(event.payload)) {
          callback(event.payload)
        }
      })

      unlisten.then((fn) => {
        listenersRef.current.set('virtual-port-stats-updated', fn)
      })

      return () => {
        listenersRef.current.get('virtual-port-stats-updated')?.()
      }
    },
    []
  )

  /**
   * Listen to error events
   */
  const onError = useCallback(
    (callback: EventCallback<ErrorEventData>, filter?: (event: ErrorEventData) => boolean) => {
      const unlisten = listen<ErrorEventData>('error-occurred', (event) => {
        if (!filter || filter(event.payload)) {
          callback(event.payload)
        }
      })

      unlisten.then((fn) => {
        listenersRef.current.set('error-occurred', fn)
      })

      return () => {
        listenersRef.current.get('error-occurred')?.()
      }
    },
    []
  )

  /**
   * Listen to custom events
   */
  const onCustomEvent = useCallback(
    <T = unknown>(eventName: string, callback: EventCallback<T>, filter?: (event: T) => boolean) => {
      const unlisten = listen<T>(eventName, (event) => {
        if (!filter || filter(event.payload)) {
          callback(event.payload)
        }
      })

      unlisten.then((fn) => {
        listenersRef.current.set(eventName, fn)
      })

      return () => {
        listenersRef.current.get(eventName)?.()
      }
    },
    []
  )

  /**
   * Cleanup all listeners
   */
  useEffect(() => {
    return () => {
      listenersRef.current.forEach((unlisten) => unlisten())
      listenersRef.current.clear()
    }
  }, [])

  return {
    onDataReceived,
    onDataSent,
    onPortStatusChanged,
    onVirtualPortCreated,
    onVirtualPortStopped,
    onVirtualPortStatsUpdated,
    onError,
    onCustomEvent,
  }
}

/**
 * useSerialDataEvents - Specialized hook for serial data events
 */
export function useSerialDataEvents(portId?: string) {
  const { onDataReceived, onDataSent } = useEvents()

  const onDataReceivedForPort = useCallback(
    (callback: EventCallback<SerialEventData>) => {
      return onDataReceived(
        callback,
        portId ? (event) => event.port_id === portId : undefined
      )
    },
    [onDataReceived, portId]
  )

  const onDataSentForPort = useCallback(
    (callback: EventCallback<SerialEventData>) => {
      return onDataSent(
        callback,
        portId ? (event) => event.port_id === portId : undefined
      )
    },
    [onDataSent, portId]
  )

  return {
    onDataReceived: onDataReceivedForPort,
    onDataSent: onDataSentForPort,
  }
}
