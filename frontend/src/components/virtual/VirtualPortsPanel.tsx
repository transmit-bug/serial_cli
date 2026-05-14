import { useVirtualPorts } from '@/contexts/VirtualPortContext'
import { Panel } from '@/components/ui/panel'
import { cn } from '@/lib/utils'
import { useState, useMemo, useCallback } from 'react'
import React from 'react'
import { useTranslation } from 'react-i18next'
import { toast } from 'sonner'
import {
  RefreshCw,
  Network,
  Play,
  Circle,
  Unplug,
  AlertCircle,
  Info,
  Activity,
  Zap,
  Clock,
  Clipboard,
  Eye,
} from 'lucide-react'
import type { VirtualPortConfig, CapturedPacket } from '@/types/tauri'

interface CreatingPort {
  config: VirtualPortConfig
}

const DEFAULT_CONFIG: VirtualPortConfig = {
  backend: 'pty',
  buffer_size: 8192,
  monitor: false,
}

const BACKEND_OPTIONS = [
  { value: 'pty', label: 'PTY (Pseudo-terminal)', description: 'Unix/Linux/macOS only' },
]

const BUFFER_SIZES = [4096, 8192, 16384, 32768, 65536]

function formatUptime(seconds: number): string {
  if (seconds < 60) return `${seconds}s`
  if (seconds < 3600) return `${Math.floor(seconds / 60)}m ${seconds % 60}s`
  const hours = Math.floor(seconds / 3600)
  const mins = Math.floor((seconds % 3600) / 60)
  return `${hours}h ${mins}m`
}

export function VirtualPortsPanel() {
  const { t } = useTranslation()
  const {
    virtualPorts,
    portStats,
    isLoading,
    error,
    createVirtualPort,
    listVirtualPorts,
    stopVirtualPort,
    getCapturedPackets,
    refreshPorts,
  } = useVirtualPorts()

  const [creatingPort, setCreatingPort] = useState<CreatingPort | null>(null)
  const [creating, setCreating] = useState(false)
  const [stoppingId, setStoppingId] = useState<string | null>(null)
  const [stopError, setStopError] = useState<string | null>(null)
  const [packets, setPackets] = useState<CapturedPacket[]>([])
  const [packetsPortId, setPacketsPortId] = useState<string | null>(null)
  const [loadingPortId, setLoadingPortId] = useState<string | null>(null)

  const handleLoadPackets = useCallback(async (id: string) => {
    setLoadingPortId(id)
    try {
      const result = await getCapturedPackets(id)
      setPackets(result)
      setPacketsPortId(id)
    } catch (e) {
      const msg = e instanceof Error ? e.message : 'Failed to load captured packets'
      setStopError(msg)
    } finally {
      setLoadingPortId(null)
    }
  }, [getCapturedPackets])

  const handleCreatePort = useCallback(() => {
    setCreatingPort({ config: DEFAULT_CONFIG })
  }, [])

  const handleConfirmCreate = useCallback(async () => {
    if (!creatingPort) return

    setCreating(true)
    try {
      await createVirtualPort(creatingPort.config)
      setCreatingPort(null)
    } catch (err) {
      console.error('Failed to create virtual port:', err)
      const msg = err instanceof Error ? err.message : 'Failed to create virtual port'
      toast.error(msg)
    } finally {
      setCreating(false)
    }
  }, [creatingPort, createVirtualPort])

  const handleStopPort = useCallback(async (id: string) => {
    setStoppingId(id)
    setStopError(null)
    try {
      await stopVirtualPort(id)
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : 'Failed to stop virtual port'
      setStopError(errorMsg)
      console.error('Failed to stop virtual port:', err)
    } finally {
      setStoppingId(null)
    }
  }, [stopVirtualPort])

  const getStats = useCallback((id: string) => {
    return portStats.get(id)
  }, [portStats])

  return (
    <div className="space-y-6">
      {/* Create Port Panel */}
      <Panel
        title={t('virtualPorts.createTitle')}
        variant="signal"
        className="w-full"
        actions={
          <button
            onClick={listVirtualPorts}
            disabled={isLoading}
            className="flex items-center gap-2 px-3 py-1.5 text-sm rounded-md bg-signal/10 text-signal border border-signal/30 hover:bg-signal/20 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
          >
            <RefreshCw size={14} strokeWidth={1.5} className={isLoading ? 'animate-spin' : ''} />
            {t('virtualPorts.refresh')}
          </button>
        }
      >
        {error && (
          <div className="mb-4 p-4 rounded-md bg-alert/10 border border-alert/30 text-alert text-sm">
            <div className="flex items-start gap-2">
              <AlertCircle size={16} strokeWidth={1.5} className="mt-0.5 flex-shrink-0" />
              <p>{error}</p>
            </div>
          </div>
        )}

        {stopError && (
          <div className="mb-4 p-4 rounded-md bg-alert/10 border border-alert/30 text-alert text-sm">
            <div className="flex items-start gap-2">
              <AlertCircle size={16} strokeWidth={1.5} className="mt-0.5 flex-shrink-0" />
              <div>
                <p className="font-medium">{t('virtualPorts.stopPortFailedTitle')}</p>
                <p className="text-alert/80 mt-1">{stopError}</p>
              </div>
              <button
                onClick={() => setStopError(null)}
                className="ml-auto text-alert/60 hover:text-alert"
              >
                ×
              </button>
            </div>
          </div>
        )}

        {creatingPort ? (
          <VirtualPortConfigForm
            config={creatingPort.config}
            onChange={(config) => setCreatingPort({ ...creatingPort, config })}
            onConfirm={handleConfirmCreate}
            onCancel={() => setCreatingPort(null)}
            isProcessing={creating}
          />
        ) : (
          <div className="text-center py-8">
            <button
              onClick={handleCreatePort}
              disabled={creating}
              className="inline-flex items-center gap-2 px-6 py-3 text-sm rounded-md bg-signal/10 text-signal border border-signal/30 hover:bg-signal/20 transition-colors disabled:opacity-50"
            >
              <Network size={18} strokeWidth={1.5} />
              {t('virtualPorts.createPortPair')}
            </button>
            <p className="mt-3 text-xs text-text-tertiary">
              {t('virtualPorts.createHint')}
            </p>
          </div>
        )}
      </Panel>

      {/* Active Virtual Ports */}
      {virtualPorts.size > 0 && (
        <Panel title={t('virtualPorts.activeTitle')} variant="info" className="w-full">
          <div className="space-y-3">
            {Array.from(virtualPorts.values()).map((port) => {
              const stats = getStats(port.id)
              const isStopping = stoppingId === port.id

              return (
                <div
                  key={port.id}
                  className="p-4 rounded-md bg-bg-deep border border-border hover:border-signal/50 transition-colors"
                >
                  {/* Header */}
                  <div className="flex items-center justify-between mb-3">
                    <div className="flex items-center gap-3">
                      <div className="p-2 rounded-md bg-info/10">
                        <Network size={18} strokeWidth={1.5} className="text-info" />
                      </div>
                      <div>
                        <div className="font-mono text-sm text-text-primary">
                          {port.port_a} ↔ {port.port_b}
                        </div>
                        <div className="flex items-center gap-2 mt-0.5">
                          <div className="flex items-center gap-1.5 text-xs text-signal">
                            <div className="w-1.5 h-1.5 rounded-full bg-signal animate-pulse-slow" />
                            {t('virtualPorts.active')}
                          </div>
                          <span className="text-xs text-text-tertiary">
                            {port.backend}
                          </span>
                        </div>
                      </div>
                    </div>
                    <button
                      onClick={() => handleStopPort(port.id)}
                      disabled={isStopping}
                      className="flex items-center gap-1.5 px-3 py-1.5 text-sm rounded-md bg-alert/10 text-alert border border-alert/30 hover:bg-alert/20 transition-colors disabled:opacity-50"
                    >
                      <Unplug size={14} strokeWidth={1.5} />
                      {isStopping ? t('virtualPorts.stopping') : t('virtualPorts.stop')}
                    </button>
                  </div>

                  {/* Statistics */}
                  {stats && (
                    <div className="grid grid-cols-4 gap-3 p-3 rounded-md bg-bg-elevated">
                      <div className="flex items-center gap-2">
                        <Activity size={14} strokeWidth={1.5} className="text-text-tertiary" />
                        <div>
                          <div className="text-xs text-text-tertiary">{t('virtualPorts.bytes')}</div>
                          <div className="text-sm font-mono text-text-primary">
                            {stats.bytes_bridged.toLocaleString()}
                          </div>
                        </div>
                      </div>
                      <div className="flex items-center gap-2">
                        <Zap size={14} strokeWidth={1.5} className="text-text-tertiary" />
                        <div>
                          <div className="text-xs text-text-tertiary">{t('virtualPorts.packets')}</div>
                          <div className="text-sm font-mono text-text-primary">
                            {stats.packets_bridged.toLocaleString()}
                          </div>
                        </div>
                      </div>
                      <div className="flex items-center gap-2">
                        <Clock size={14} strokeWidth={1.5} className="text-text-tertiary" />
                        <div>
                          <div className="text-xs text-text-tertiary">{t('virtualPorts.uptime')}</div>
                          <div className="text-sm font-mono text-text-primary">
                            {formatUptime(stats.uptime_secs)}
                          </div>
                        </div>
                      </div>
                      <div className="flex items-center gap-2">
                        <Info size={14} strokeWidth={1.5} className="text-text-tertiary" />
                        <div>
                          <div className="text-xs text-text-tertiary">{t('virtualPorts.errors')}</div>
                          <div className="text-sm font-mono text-text-primary">
                            {stats.bridge_errors}
                          </div>
                        </div>
                      </div>
                    </div>
                  )}

                  {/* Monitoring Status */}
                  {stats && stats.monitoring && (
                    <div className="mt-3 flex items-center gap-2">
                      <Eye size={14} strokeWidth={1.5} className="text-signal" />
                      <span className="text-xs text-signal">{t('virtualPorts.monitoringStatus')}</span>
                      <span className="text-xs text-text-tertiary">
                        {t('virtualPorts.captureInfo', { packets: stats.capture_packets, bytes: stats.capture_bytes })}
                      </span>
                      <button
                        onClick={() => handleLoadPackets(port.id)}
                        disabled={loadingPortId !== null}
                        className="ml-auto text-xs px-2 py-1 rounded bg-signal/10 text-signal border border-signal/30 hover:bg-signal/20 transition-colors disabled:opacity-50"
                      >
                        {loadingPortId === port.id ? t('virtualPorts.loading') : t('virtualPorts.viewPackets')}
                      </button>
                    </div>
                  )}

                  {/* Port Paths */}
                  <div className="mt-3 space-y-1.5">
                    <div className="flex items-center gap-2 text-xs">
                      <span className="text-text-tertiary w-16">{t('virtualPorts.portALabel')}</span>
                      <code className="px-2 py-0.5 bg-bg-elevated rounded text-text-primary font-mono">
                        {port.port_a}
                      </code>
                      <button
                        onClick={() => {
                          navigator.clipboard.writeText(port.port_a)
                          toast.success(t('virtualPorts.copyToClipboard'))
                        }}
                        className="text-text-tertiary hover:text-text-primary"
                        title={t('virtualPorts.copyToClipboard')}
                      >
                        <Clipboard size={14} strokeWidth={1.5} />
                      </button>
                    </div>
                    <div className="flex items-center gap-2 text-xs">
                      <span className="text-text-tertiary w-16">{t('virtualPorts.portBLabel')}</span>
                      <code className="px-2 py-0.5 bg-bg-elevated rounded text-text-primary font-mono">
                        {port.port_b}
                      </code>
                      <button
                        onClick={() => {
                          navigator.clipboard.writeText(port.port_b)
                          toast.success(t('virtualPorts.copyToClipboard'))
                        }}
                        className="text-text-tertiary hover:text-text-primary"
                        title={t('virtualPorts.copyToClipboard')}
                      >
                        <Clipboard size={14} strokeWidth={1.5} />
                      </button>
                    </div>
                  </div>
                </div>
              )
            })}
          </div>
        </Panel>
      )}

      {/* Captured Packets Viewer */}
      {packets.length > 0 && packetsPortId && (
        <Panel
          title={`${t('virtualPorts.capturedPacketsTitle')} — ${virtualPorts.get(packetsPortId)?.port_a ?? packetsPortId.slice(0, 8)}`}
          variant="signal"
          className="w-full"
          actions={
            <button
              onClick={() => { setPackets([]); setPacketsPortId(null) }}
              className="text-xs px-2 py-1 rounded bg-bg-elevated text-text-tertiary border border-border hover:text-text-primary transition-colors"
            >
              {t('virtualPorts.close')}
            </button>
          }
        >
          <div className="max-h-64 overflow-y-auto space-y-1">
            {packets.map((pkt, idx) => (
              <div key={idx} className="flex items-center gap-3 px-3 py-1.5 rounded bg-bg-elevated text-xs font-mono">
                <span className={cn(
                  'px-1.5 py-0.5 rounded text-[10px] font-bold',
                  pkt.direction === 'A→B'
                    ? 'bg-info/20 text-info'
                    : 'bg-signal/20 text-signal'
                )}>
                  {pkt.direction}
                </span>
                <span className="text-text-tertiary w-12 text-right">{pkt.data.length}B</span>
                <span className="text-text-primary truncate flex-1">
                  {pkt.data.slice(0, 32).map(b => b.toString(16).padStart(2, '0')).join(' ')}
                  {pkt.data.length > 32 && ' ...'}
                </span>
                <span className="text-text-tertiary shrink-0">
                  {new Date(pkt.timestamp_millis).toLocaleTimeString()}.{String(pkt.timestamp_millis % 1000).padStart(3, '0')}
                </span>
              </div>
            ))}
          </div>
        </Panel>
      )}

      {/* Empty State */}
      {virtualPorts.size === 0 && (
        <Panel title={t('virtualPorts.title')} variant="info" className="w-full">
          <div className="p-8 text-center text-text-tertiary text-sm">
            <Network size={48} strokeWidth={1} className="mx-auto mb-4 text-text-tertiary/30" />
            <p className="mb-2">{t('virtualPorts.noPorts')}</p>
            <p className="text-xs">{t('virtualPorts.noPortsHint')}</p>
          </div>
        </Panel>
      )}
    </div>
  )
}

interface VirtualPortConfigFormProps {
  config: VirtualPortConfig
  onChange: (config: VirtualPortConfig) => void
  onConfirm: () => void
  onCancel: () => void
  isProcessing: boolean
}

const VirtualPortConfigForm = React.memo(function VirtualPortConfigForm({
  config,
  onChange,
  onConfirm,
  onCancel,
  isProcessing,
}: VirtualPortConfigFormProps) {
  const { t } = useTranslation()
  return (
    <div className="space-y-4">
      <div className="grid grid-cols-1 gap-4">
        {/* Backend Type */}
        <div>
          <label className="text-xs text-text-tertiary uppercase tracking-wider block mb-2">
            {t('virtualPorts.backendType')}
          </label>
          <div className="space-y-2">
            {BACKEND_OPTIONS.map((option) => (
              <button
                key={option.value}
                onClick={() => onChange({ ...config, backend: option.value as VirtualPortConfig['backend'] })}
                className={cn(
                  'w-full text-left p-3 rounded-md border transition-colors',
                  config.backend === option.value
                    ? 'bg-info/10 text-info border-info/30'
                    : 'bg-bg-elevated text-text-secondary border-border hover:text-text-primary'
                )}
              >
                <div>
                  <div className="font-medium text-sm">{t('virtualPorts.ptyBackend')}</div>
                  <div className="text-xs text-text-tertiary mt-0.5">{t('virtualPorts.ptyDesc')}</div>
                </div>
              </button>
            ))}
          </div>
        </div>

        {/* Buffer Size */}
        <div>
          <label className="text-xs text-text-tertiary uppercase tracking-wider block mb-2">
            {t('virtualPorts.bufferSize')}
          </label>
          <select
            value={config.buffer_size || 8192}
            onChange={(e) => onChange({ ...config, buffer_size: parseInt(e.target.value) })}
            className="w-full px-3 py-2 bg-bg-deep border border-border rounded-md text-sm text-text-primary font-mono"
          >
            {BUFFER_SIZES.map((size) => (
              <option key={size} value={size}>
                {t('virtualPorts.bytesOption', { size })}
              </option>
            ))}
          </select>
        </div>

        {/* Monitor Traffic */}
        <div className="flex items-center gap-2">
          <input
            type="checkbox"
            id="monitor"
            checked={config.monitor ?? false}
            onChange={(e) => onChange({ ...config, monitor: e.target.checked })}
            className="w-4 h-4 rounded border-border bg-bg-deep text-info accent-info"
          />
          <label htmlFor="monitor" className="text-sm text-text-secondary">
            {t('virtualPorts.monitorTraffic')}
          </label>
        </div>
      </div>

      {/* Actions */}
      <div className="flex items-center gap-2 pt-2">
        <button
          onClick={onConfirm}
          disabled={isProcessing}
          className="flex items-center gap-1.5 px-4 py-2 text-sm rounded-md bg-signal/10 text-signal border border-signal/30 hover:bg-signal/20 transition-colors disabled:opacity-50"
        >
          <Play size={14} strokeWidth={1.5} />
          {isProcessing ? t('virtualPorts.creating') : t('virtualPorts.createPortPair')}
        </button>
        <button
          onClick={onCancel}
          disabled={isProcessing}
          className="px-4 py-2 text-sm rounded-md bg-bg-elevated text-text-secondary border border-border hover:text-text-primary transition-colors disabled:opacity-50"
        >
          {t('common.cancel')}
        </button>
      </div>
    </div>
  )
})

VirtualPortConfigForm.displayName = 'VirtualPortConfigForm'
