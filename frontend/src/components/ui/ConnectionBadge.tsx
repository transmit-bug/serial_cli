import { cn } from '@/lib/utils'
import { Plug, AlertCircle, Loader2 } from 'lucide-react'
import type { ConnectionStatus } from '@/stores/connectionStore'
import { useTranslation } from 'react-i18next'

interface ConnectionBadgeProps {
  status: ConnectionStatus
  portName?: string
  baudrate?: number
  errorMessage?: string
  compact?: boolean
  onClick?: () => void
  className?: string
}

/**
 * ConnectionBadge — Unified connection state indicator.
 *
 * Pill-shaped badge with consistent sizing across all states.
 * Appears in TopBar, TerminalWorkbench, sidebar, etc.
 *
 * @example
 * // Full variant
 * <ConnectionBadge status="connected" portName="ttyUSB0" baudrate={9600} />
 *
 * @example
 * // Compact variant (icon + dot only)
 * <ConnectionBadge status="connected" compact />
 */
export function ConnectionBadge({
  status,
  portName,
  baudrate,
  errorMessage,
  compact = false,
  onClick,
  className,
}: ConnectionBadgeProps) {
  const { t } = useTranslation()

  const labelKey: Record<ConnectionStatus, string> = {
    disconnected: 'connection.disconnected',
    connecting: 'connection.connecting',
    connected: 'connection.connected',
    error: 'connection.error',
  }

  const configIcon = (() => {
    switch (status) {
      case 'disconnected': return <Plug size={14} strokeWidth={1.5} />
      case 'connecting': return <Loader2 size={14} strokeWidth={1.5} className="animate-spin text-amber" />
      case 'connected': return <span className="inline-block w-2 h-2 rounded-full bg-signal animate-pulse" />
      case 'error': return <AlertCircle size={14} strokeWidth={1.5} />
    }
  })()

  const configColors: Record<ConnectionStatus, string> = {
    disconnected: 'text-text-tertiary bg-bg-elevated border-border',
    connecting: 'text-amber bg-amber/10 border-amber/30',
    connected: 'text-signal bg-signal/10 border-signal/30',
    error: 'text-alert bg-alert/10 border-alert/30',
  }

  const displayLabel = () => {
    if (compact) return null
    if (status === 'connected') {
      return portName || t(labelKey[status])
    }
    if (status === 'error') {
      return errorMessage || t(labelKey[status])
    }
    return t(labelKey[status])
  }

  return (
    <button
      type="button"
      onClick={onClick}
      disabled={status === 'connecting'}
      className={cn(
        'inline-flex items-center gap-2 h-7 px-3 rounded-full border text-xs font-medium',
        'transition-colors duration-200',
        configColors,
        onClick && status !== 'connecting' ? 'cursor-pointer hover:opacity-80' : '',
        status === 'connecting' ? 'cursor-not-allowed' : '',
        className,
      )}
    >
      {configIcon}
      {!compact && (
        <span className="truncate max-w-32">{displayLabel()}</span>
      )}
      {!compact && status === 'connected' && baudrate && (
        <span className="text-[10px] opacity-60">{baudrate} {t('connection.bps')}</span>
      )}
    </button>
  )
}
