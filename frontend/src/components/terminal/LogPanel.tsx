import { useState, useEffect, useRef, useCallback } from 'react'
import { listen } from '@tauri-apps/api/event'
import { Card } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Minus, Maximize2, X } from 'lucide-react'
import { useNotificationStore, NotificationType } from '@/stores/notificationStore'
import { useConnectionStore } from '@/stores'
import { useTranslation } from 'react-i18next'

export type LogLevel = 'info' | 'warn' | 'error' | 'debug'

export interface LogEntry {
  id: string
  timestamp: string
  level: LogLevel
  source: 'system' | 'port' | 'protocol' | 'script'
  message: string
}

const MAX_LOG_ENTRIES = 500

function levelFromNotification(type: NotificationType): LogLevel {
  switch (type) {
    case 'error': return 'error'
    case 'warning': return 'warn'
    case 'success': return 'info'
    case 'info': return 'info'
  }
}

function levelColor(level: LogLevel): string {
  switch (level) {
    case 'error': return 'text-alert'
    case 'warn': return 'text-amber'
    case 'info': return 'text-signal'
    case 'debug': return 'text-text-tertiary'
  }
}

/**
 * LogPanel - 底部日志面板
 *
 * 特性：
 * - 订阅 Tauri 事件（端口状态、错误等）
 * - 监听 notificationStore 中的通知
 * - 可折叠、可清空
 * - 自动裁剪旧日志（最多 500 条）
 */
export function LogPanel() {
  const [logs, setLogs] = useState<LogEntry[]>([])
  const [isExpanded, setIsExpanded] = useState(true)
  const notifications = useNotificationStore((s) => s.notifications)
  const { status, portName } = useConnectionStore()
  const processedNotifications = useRef<Set<string>>(new Set())
  const scrollRef = useRef<HTMLDivElement>(null)
  const { t } = useTranslation()

  // Add a log entry
  const addLog = useCallback((level: LogLevel, source: LogEntry['source'], message: string) => {
    setLogs((prev) => {
      const entry: LogEntry = {
        id: `${Date.now()}-${Math.random()}`,
        timestamp: new Date().toLocaleTimeString('en-US', { hour12: false }),
        level,
        source,
        message,
      }
      const next = [...prev, entry]
      // Auto-trim to prevent memory issues
      if (next.length > MAX_LOG_ENTRIES) {
        return next.slice(next.length - MAX_LOG_ENTRIES)
      }
      return next
    })
  }, [])

  // Listen to Tauri events on mount
  useEffect(() => {
    const unlistenPromises: Promise<(() => void)>[] = []

    // Listen for port status changes
    unlistenPromises.push(
      listen('port-status-changed', (event) => {
        const payload = event.payload as Record<string, unknown>
        const portId = payload.port_id as string || 'unknown'
        addLog('info', 'port', `Port "${portId}" status changed`)
      }),
    )

    // Listen for errors
    unlistenPromises.push(
      listen('error-occurred', (event) => {
        const payload = event.payload as Record<string, unknown>
        const errorMsg = payload.error as string || 'Unknown error'
        addLog('error', 'system', errorMsg)
      }),
    )

    // Listen for virtual port events
    unlistenPromises.push(
      listen('virtual-port-created', (event) => {
        const payload = event.payload as Record<string, unknown>
        const portId = payload.port_id as string || 'unknown'
        addLog('info', 'system', `Virtual port "${portId}" created`)
      }),
    )

    unlistenPromises.push(
      listen('virtual-port-stopped', (event) => {
        const payload = event.payload as Record<string, unknown>
        const portId = payload.port_id as string || 'unknown'
        addLog('warn', 'system', `Virtual port "${portId}" stopped`)
      }),
    )

    // Cleanup listeners on unmount
    return () => {
      unlistenPromises.forEach((promise) => {
        promise.then((unlisten) => unlisten())
      })
    }
  }, [addLog])

  // Sync connection state changes to log
  useEffect(() => {
    if (status === 'connected' && portName) {
      addLog('info', 'port', `Connected to ${portName}`)
    } else if (status === 'disconnected') {
      addLog('warn', 'port', 'Disconnected')
    }
  }, [status, portName])

  // Sync notifications to log (only new ones)
  useEffect(() => {
    for (const notification of notifications) {
      if (!processedNotifications.current.has(notification.id)) {
        processedNotifications.current.add(notification.id)
        const level = levelFromNotification(notification.type)
        addLog(level, 'system', `${notification.title}: ${notification.message}`)
      }
    }
  }, [notifications, addLog])

  // Auto-scroll to bottom when new logs arrive
  useEffect(() => {
    if (scrollRef.current) {
      scrollRef.current.scrollTop = scrollRef.current.scrollHeight
    }
  }, [logs])

  // Periodically clean up old processed notification IDs (prevent memory leak)
  useEffect(() => {
    const interval = setInterval(() => {
      if (processedNotifications.current.size > MAX_LOG_ENTRIES * 2) {
        const ids = Array.from(processedNotifications.current)
        const recentIds = ids.slice(-MAX_LOG_ENTRIES)
        processedNotifications.current.clear()
        for (const id of recentIds) {
          processedNotifications.current.add(id)
        }
      }
    }, 60000)
    return () => clearInterval(interval)
  }, [])

  const clearLogs = () => {
    setLogs([])
    processedNotifications.current.clear()
  }

  return (
    <div className={`${isExpanded ? 'h-48' : 'h-12'} transition-all duration-200`}>
      <Card className="h-full m-0 rounded-none border-t border-x-0 border-b-0">
        {/* 标题栏 */}
        <div className="flex items-center justify-between px-4 py-2 border-b border-border">
          <div className="flex items-center gap-2">
            <h3 className="text-sm font-medium text-text-primary">{t('terminal.systemLogs')}</h3>
            <span className="text-xs text-text-tertiary bg-bg-base px-2 py-0.5 rounded">
              {logs.length} {t('terminal.entries')}
            </span>
          </div>

          <div className="flex items-center gap-1">
            <Button
              variant="ghost"
              size="sm"
              onClick={() => setIsExpanded(!isExpanded)}
            >
              {isExpanded ? <Minus className="w-4 h-4" /> : <Maximize2 className="w-4 h-4" />}
            </Button>
            <Button
              variant="ghost"
              size="sm"
              onClick={clearLogs}
            >
              <X className="w-4 h-4" />
            </Button>
          </div>
        </div>

        {/* 日志内容 */}
        {isExpanded && (
          <div ref={scrollRef} className="h-36 overflow-y-auto px-4 py-2">
            {logs.length === 0 ? (
              <div className="text-center text-text-tertiary text-xs py-4">
                <p>{t('terminal.noLogs')}</p>
                <p className="text-[10px] mt-1 opacity-60">{t('terminal.noLogsHint')}</p>
              </div>
            ) : (
              <div className="space-y-0.5">
                {logs.map((log) => (
                  <div
                    key={log.id}
                    className="text-xs font-mono py-0.5 flex items-start gap-2"
                  >
                    <span className="text-text-tertiary flex-shrink-0 opacity-60">
                      {log.timestamp}
                    </span>
                    <span className={`flex-shrink-0 ${levelColor(log.level)}`}>
                      [{log.source}]
                    </span>
                    <span className={levelColor(log.level)}>{log.message}</span>
                  </div>
                ))}
              </div>
            )}
          </div>
        )}
      </Card>
    </div>
  )
}
