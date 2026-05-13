import { useMemo, useEffect } from 'react'
import { Card } from '@/components/ui/card'
import { useConnectionStore, useDataStore, useProtocolStore, useNavigationStore } from '@/stores'
import { Activity, Cpu, Zap, Settings } from 'lucide-react'
import { toast } from 'sonner'
import { useTranslation } from 'react-i18next'
import { QuickCommandsCard } from './QuickCommandsCard'

/**
 * SidePanel - 右侧辅助面板
 *
 * 包含：
 * - 端口详情卡片
 * - 数据统计卡片（后端真实统计）
 * - 协议控制卡片
 * - 快捷操作卡片
 */
export function SidePanel() {
  const { portName, config, portStatus, refreshPortStatus, statsPollingInterval } = useConnectionStore()
  const { rxPackets, txPackets, clearRxPackets, clearTxPackets } = useDataStore()
  const { activeProtocol } = useProtocolStore()
  const { navigateTo } = useNavigationStore()
  const { t } = useTranslation()

  // Poll backend for real stats every 2s
  useEffect(() => {
    refreshPortStatus()
    const interval = setInterval(refreshPortStatus, statsPollingInterval)
    return () => clearInterval(interval)
  }, [refreshPortStatus, statsPollingInterval])

  // Backend stats (real byte counts from port)
  const backendStats = useMemo(() => {
    const s = portStatus?.stats
    return {
      rxBytes: s?.bytes_received ?? 0,
      txBytes: s?.bytes_sent ?? 0,
      rxPackets: s?.packets_received ?? 0,
      txPackets: s?.packets_sent ?? 0,
      lastActivity: s?.last_activity ?? null,
    }
  }, [portStatus])

  // Frontend packet counts (for display in data store, complementary)
  const frontendPacketCount = useMemo(() => {
    return {
      rx: rxPackets.length,
      tx: txPackets.length,
    }
  }, [rxPackets, txPackets])

  const handleClearRx = () => {
    clearRxPackets()
    toast.success(t('terminal.clearRxSuccess'))
  }

  const handleClearTx = () => {
    clearTxPackets()
    toast.success(t('terminal.clearTxSuccess'))
  }

  const handleExport = () => {
    // Export data as hex dump
    const lines: string[] = []
    for (const p of [...rxPackets, ...txPackets]) {
      const ts = new Date(p.timestamp).toISOString()
      const dir = p.direction === 'rx' ? 'RX' : 'TX'
      const hex = p.data.map((b) => b.toString(16).padStart(2, '0')).join(' ')
      lines.push(`[${ts}] ${dir}: ${hex}`)
    }
    const content = lines.join('\n')
    const blob = new Blob([content], { type: 'text/plain' })
    const url = URL.createObjectURL(blob)
    const a = document.createElement('a')
    a.href = url
    a.download = `serial-capture-${Date.now()}.txt`
    a.click()
    URL.revokeObjectURL(url)
    toast.success(t('terminal.exportSuccess'))
  }

  return (
    <div className="h-full overflow-y-auto bg-bg-deep">
      <div className="p-4 space-y-4">
        {/* 端口详情卡片 */}
        <Card className="p-4 border-border/50">
          <div className="flex items-center gap-2 mb-3">
            <Activity className="w-4 h-4 text-signal" />
            <h3 className="text-sm font-medium text-text-primary">{t('terminal.portDetails')}</h3>
          </div>
          <div className="space-y-2 text-xs">
            <div className="flex justify-between">
              <span className="text-text-tertiary">{t('terminal.port')}</span>
              <span className="text-text-primary font-mono">{portName || '-'}</span>
            </div>
            <div className="flex justify-between">
              <span className="text-text-tertiary">{t('terminal.baudrate')}</span>
              <span className="text-text-primary font-mono">{config?.baudrate || '-'} {t('connection.bps')}</span>
            </div>
            <div className="flex justify-between">
              <span className="text-text-tertiary">{t('terminal.dataBits')}</span>
              <span className="text-text-primary font-mono">{config?.databits || '-'} {t('terminal.bit')}</span>
            </div>
            <div className="flex justify-between">
              <span className="text-text-tertiary">{t('terminal.stopBits')}</span>
              <span className="text-text-primary font-mono">{config?.stopbits || '-'}</span>
            </div>
            <div className="flex justify-between">
              <span className="text-text-tertiary">{t('terminal.parity')}</span>
              <span className="text-text-primary font-mono">{config?.parity || '-'}</span>
            </div>
          </div>
        </Card>

        {/* 数据统计卡片 */}
        <Card className="p-4 border-border/50">
          <div className="flex items-center gap-2 mb-3">
            <Activity className="w-4 h-4 text-info" />
            <h3 className="text-sm font-medium text-text-primary">{t('terminal.dataStats')}</h3>
          </div>
          <div className="space-y-2 text-xs">
            <div className="flex justify-between">
              <span className="text-text-tertiary">{t('terminal.rxPackets')}</span>
              <span className="text-text-primary font-mono">{backendStats.rxPackets}</span>
            </div>
            <div className="flex justify-between">
              <span className="text-text-tertiary">{t('terminal.txPackets')}</span>
              <span className="text-text-primary font-mono">{backendStats.txPackets}</span>
            </div>
            <div className="flex justify-between">
              <span className="text-text-tertiary">{t('terminal.rxBytes')}</span>
              <span className="text-text-primary font-mono">{backendStats.rxBytes}</span>
            </div>
            <div className="flex justify-between">
              <span className="text-text-tertiary">{t('terminal.txBytes')}</span>
              <span className="text-text-primary font-mono">{backendStats.txBytes}</span>
            </div>
            {backendStats.lastActivity && (
              <div className="flex justify-between">
                <span className="text-text-tertiary">{t('terminal.lastActivity')}</span>
                <span className="text-text-primary font-mono">
                  {new Date(backendStats.lastActivity).toLocaleTimeString()}
                </span>
              </div>
            )}
          </div>
        </Card>

        {/* 协议控制卡片 */}
        <Card className="p-4 border-border/50">
          <div className="flex items-center gap-2 mb-3">
            <Cpu className="w-4 h-4 text-amber" />
            <h3 className="text-sm font-medium text-text-primary">{t('terminal.protocolControl')}</h3>
          </div>
          {activeProtocol ? (
            <div className="text-xs space-y-1">
              <div className="flex justify-between">
                <span className="text-text-tertiary">{t('terminal.activeProtocol')}</span>
                <span className="text-text-primary font-mono">{activeProtocol}</span>
              </div>
            </div>
          ) : (
            <div className="text-xs text-text-tertiary text-center py-2">
              {t('terminal.noActiveProtocol')}
            </div>
          )}
        </Card>

        {/* 快捷指令列表 */}
        <QuickCommandsCard />

        {/* 快捷操作卡片 */}
        <Card className="p-4 border-border/50">
          <div className="flex items-center gap-2 mb-3">
            <Zap className="w-4 h-4 text-signal" />
            <h3 className="text-sm font-medium text-text-primary">{t('terminal.quickActions')}</h3>
          </div>
          <div className="space-y-2">
            <button
              className="w-full px-3 py-2 text-xs text-left text-text-secondary hover:text-text-primary hover:bg-bg-elevated rounded transition-colors"
              onClick={handleClearRx}
            >
              {t('terminal.clearRx')}
            </button>
            <button
              className="w-full px-3 py-2 text-xs text-left text-text-secondary hover:text-text-primary hover:bg-bg-elevated rounded transition-colors"
              onClick={handleClearTx}
            >
              {t('terminal.clearTx')}
            </button>
            <button
              className="w-full px-3 py-2 text-xs text-left text-text-secondary hover:text-text-primary hover:bg-bg-elevated rounded transition-colors"
              onClick={handleExport}
            >
              {t('terminal.exportData')}
            </button>
          </div>
        </Card>

        {/* 设置链接 */}
        <Card className="p-4 border-border/50">
          <div className="flex items-center gap-2">
            <Settings className="w-4 h-4 text-text-tertiary" />
            <button
              className="flex-1 text-left text-xs text-text-secondary hover:text-text-primary"
              onClick={() => navigateTo('settings')}
            >
              {t('terminal.openSettings')}
            </button>
          </div>
        </Card>
      </div>
    </div>
  )
}
