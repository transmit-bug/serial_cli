import React, { createContext, useContext, useState, useCallback, useEffect, useRef } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import type { CapturedPacket, VirtualPortConfig, VirtualPortInfo, VirtualPortStats } from '../types/tauri'
import type { VirtualPortEventData } from '../hooks/useEvents'

interface VirtualPortContextType {
  virtualPorts: Map<string, VirtualPortInfo>
  portStats: Map<string, VirtualPortStats>
  isLoading: boolean
  error: string | null
  createVirtualPort: (config: VirtualPortConfig) => Promise<string>
  listVirtualPorts: () => Promise<void>
  stopVirtualPort: (id: string) => Promise<void>
  getPortStats: (id: string) => Promise<VirtualPortStats>
  getCapturedPackets: (id: string) => Promise<CapturedPacket[]>
  refreshPorts: () => Promise<void>
}

const VirtualPortContext = createContext<VirtualPortContextType | undefined>(undefined)

export function VirtualPortProvider({ children }: { children: React.ReactNode }) {
  const [virtualPorts, setVirtualPorts] = useState<Map<string, VirtualPortInfo>>(new Map())
  const [portStats, setPortStats] = useState<Map<string, VirtualPortStats>>(new Map())
  const [isLoading, setIsLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)

  // Use ref to track refresh interval
  const refreshIntervalRef = useRef<ReturnType<typeof setInterval> | null>(null)

  const createVirtualPort = useCallback(async (config: VirtualPortConfig) => {
    setIsLoading(true)
    try {
      const id = await invoke<string>('create_virtual_port', { config })

      // Refresh the list after creation
      await listVirtualPorts()

      setError(null)
      return id
    } catch (e) {
      const errorMsg = e instanceof Error ? e.message : 'Failed to create virtual port'
      setError(errorMsg)
      console.error('createVirtualPort error:', e)
      throw e
    } finally {
      setIsLoading(false)
    }
  }, [])

  const listVirtualPorts = useCallback(async () => {
    setIsLoading(true)
    try {
      const ports = await invoke<VirtualPortInfo[]>('list_virtual_ports')
      const portsMap = new Map(ports.map(p => [p.id, p]))
      setVirtualPorts(portsMap)
      setError(null)
    } catch (e) {
      const errorMsg = e instanceof Error ? e.message : 'Failed to list virtual ports'
      setError(errorMsg)
      console.error('listVirtualPorts error:', e)
    } finally {
      setIsLoading(false)
    }
  }, [])

  const stopVirtualPort = useCallback(async (id: string) => {
    setIsLoading(true)
    try {
      await invoke('stop_virtual_port', { id })

      // Remove from local state
      setVirtualPorts(prev => {
        const next = new Map(prev)
        next.delete(id)
        return next
      })

      setPortStats(prev => {
        const next = new Map(prev)
        next.delete(id)
        return next
      })

      setError(null)
    } catch (e) {
      const errorMsg = e instanceof Error ? e.message : 'Failed to stop virtual port'
      setError(errorMsg)
      console.error('stopVirtualPort error:', e)
      throw e
    } finally {
      setIsLoading(false)
    }
  }, [])

  const getPortStats = useCallback(async (id: string) => {
    try {
      const stats = await invoke<VirtualPortStats>('get_virtual_port_stats', { id })

      // Update stats in local state
      setPortStats(prev => {
        const next = new Map(prev)
        next.set(id, stats)
        return next
      })

      return stats
    } catch (e) {
      console.error(`Failed to get stats for port ${id}:`, e)
      throw e
    }
  }, [])

  const getCapturedPackets = useCallback(async (id: string) => {
    return await invoke<CapturedPacket[]>('get_captured_packets', { id })
  }, [])

  // Use ref to track current ports for refresh (avoids recreating callback on state changes)
  const virtualPortsRef = useRef(virtualPorts)
  useEffect(() => {
    virtualPortsRef.current = virtualPorts
  }, [virtualPorts])

  const refreshPorts = useCallback(async () => {
    try {
      // Refresh list
      await listVirtualPorts()

      // Check health of each port and remove stopped ones
      const portsToRemove: string[] = []

      for (const [id, _port] of virtualPortsRef.current.entries()) {
        try {
          const isHealthy = await invoke<boolean>('check_virtual_port_health', { id })
          if (!isHealthy) {
            console.log(`Virtual port ${id} is no longer healthy, removing...`)
            portsToRemove.push(id)
          }
        } catch (e) {
          console.error(`Failed to check health of virtual port ${id}:`, e)
          portsToRemove.push(id)
        }
      }

      // Remove unhealthy ports
      if (portsToRemove.length > 0) {
        setVirtualPorts(prev => {
          const next = new Map(prev)
          portsToRemove.forEach(id => next.delete(id))
          return next
        })

        setPortStats(prev => {
          const next = new Map(prev)
          portsToRemove.forEach(id => next.delete(id))
          return next
        })
      }
    } catch (e) {
      console.error('Failed to refresh virtual ports:', e)
    }
  }, [listVirtualPorts])

  // Setup auto-refresh for stats
  useEffect(() => {
    // Clear any existing interval
    if (refreshIntervalRef.current) {
      clearInterval(refreshIntervalRef.current)
    }

    // Start refresh interval if there are active ports
    const currentPorts = virtualPortsRef.current
    if (currentPorts.size > 0) {
      refreshIntervalRef.current = setInterval(() => {
        // Refresh stats for each port
        currentPorts.forEach((_port, id) => {
          getPortStats(id).catch(err => {
            console.error(`Failed to refresh stats for ${id}:`, err)
          })
        })
      }, 5000) // Update stats every 5 seconds

      console.log('Started virtual port stats refresh')
    }

    // Cleanup function
    return () => {
      if (refreshIntervalRef.current) {
        clearInterval(refreshIntervalRef.current)
        refreshIntervalRef.current = null
      }
    }
  }, [virtualPorts.size, getPortStats])

  // Setup event listeners for real-time updates
  useEffect(() => {
    const unlisteners: Promise<() => void>[] = []

    // Listen for virtual port created events
    const unlistenCreated = listen<VirtualPortEventData>('virtual-port-created', (event) => {
      console.log('Virtual port created event:', event.payload)
      // Refresh the list to get the new port
      listVirtualPorts().catch(err => {
        console.error('Failed to refresh ports after creation:', err)
      })
    })
    unlisteners.push(unlistenCreated)

    // Listen for virtual port stopped events
    const unlistenStopped = listen<VirtualPortEventData>('virtual-port-stopped', (event) => {
      console.log('Virtual port stopped event:', event.payload)
      const portId = event.payload.port_id
      // Remove from local state
      setVirtualPorts(prev => {
        const next = new Map(prev)
        next.delete(portId)
        return next
      })
      setPortStats(prev => {
        const next = new Map(prev)
        next.delete(portId)
        return next
      })
    })
    unlisteners.push(unlistenStopped)

    // Listen for stats updated events
    const unlistenStats = listen<VirtualPortEventData>('virtual-port-stats-updated', (event) => {
      console.log('Virtual port stats updated event:', event.payload)
      const stats = event.payload.stats
      if (!stats) return

      // Update stats in local state
      setPortStats(prev => {
        const next = new Map(prev)
        next.set(stats.id, stats)
        return next
      })
    })
    unlisteners.push(unlistenStats)

    // Cleanup function
    return () => {
      Promise.all(unlisteners).then(unlisteners => {
        unlisteners.forEach(unlisten => {
          unlisten()
        })
      })
    }
  }, [listVirtualPorts])

  // Auto-list ports on mount
  useEffect(() => {
    listVirtualPorts()
  }, [listVirtualPorts])

  return (
    <VirtualPortContext.Provider value={{
      virtualPorts,
      portStats,
      isLoading,
      error,
      createVirtualPort,
      listVirtualPorts,
      stopVirtualPort,
      getPortStats,
      getCapturedPackets,
      refreshPorts,
    }}>
      {children}
    </VirtualPortContext.Provider>
  )
}

export function useVirtualPorts() {
  const context = useContext(VirtualPortContext)
  if (!context) {
    throw new Error('useVirtualPorts must be used within VirtualPortProvider')
  }
  return context
}
