import { NotificationSettings } from './NotificationSettings'
import { Panel } from '@/components/ui/panel'
import { cn } from '@/lib/utils'
import { useState, useRef, useMemo, useEffect } from 'react'
import { RotateCcw, Check, Download, Upload, Settings, Radio, BarChart3, Bell } from 'lucide-react'
import { exportSettings, importSettings } from '@/lib/storage'
import { useSettings } from '@/contexts/SettingsContext'
import { useSettingsStore } from '@/stores/settingsStore'
import { useDataStore } from '@/stores'
import { toast } from 'sonner'
import { useTranslation } from 'react-i18next'

type Tab = 'general' | 'serial' | 'data' | 'notifications'

interface SerialConfig {
  baudRate: number
  dataBits: number
  stopBits: number
  parity: 'none' | 'even' | 'odd'
  flowControl: 'none' | 'rts' | 'cts' | 'rtscts'
}

interface DataConfig {
  displayFormat: 'hex' | 'ascii' | 'both'
  showTimestamp: boolean
  maxPackets: number
  autoScroll: boolean
}

export function SettingsPanel() {
  const { t } = useTranslation()
  const { settings, updateSettings, resetSettings } = useSettings()
  const { config, loadConfig, saveConfig, resetConfig, error: backendError } = useSettingsStore()
  const { setDisplayFormat, setShowTimestamp, setMaxPackets } = useDataStore()
  const [activeTab, setActiveTab] = useState<Tab>('general')
  const [isImporting, setIsImporting] = useState(false)
  const fileInputRef = useRef<HTMLInputElement>(null)

  // Load backend config on mount
  useEffect(() => {
    loadConfig()
  }, [loadConfig])

  // Sync backend config to frontend settings
  useEffect(() => {
    if (!config || !config.display || !config.serial) return

    // Sync serial config
    updateSettings({
      serial: {
        baudRate: config.serial.defaultBaudrate,
        // Timeout from backend -> not directly mapped to frontend
      },
      display: {
        showTimestamp: config.display.showTimestamp,
        maxPackets: config.display.maxPackets,
        format: config.display.format,
      },
    })

    // Sync data store
    setDisplayFormat(config.display.format === 'both' ? 'mixed' : config.display.format)
    setShowTimestamp(config.display.showTimestamp)
    setMaxPackets(config.display.maxPackets)
  }, [config]) // eslint-disable-line react-hooks/exhaustive-deps

  // Show backend errors
  useEffect(() => {
    if (backendError) {
      toast.warning(t('settings.backendWarning', { error: backendError }))
    }
  }, [backendError])

  // Derived serial config from global settings
  const serialConfig = useMemo(() => ({
    baudRate: settings.serial.baudRate,
    dataBits: settings.serial.dataBits,
    stopBits: settings.serial.stopBits,
    parity: settings.serial.parity,
    flowControl: settings.serial.flowControl,
  }), [settings.serial])

  // Derived data config from global settings
  const dataConfig = useMemo(() => ({
    displayFormat: settings.display.format,
    showTimestamp: settings.display.showTimestamp,
    maxPackets: settings.display.maxPackets,
    autoScroll: settings.display.autoScroll,
  }), [settings.display])

  const tabs: { id: Tab; label: string; icon: React.ElementType }[] = [
    { id: 'general', label: t('settings.generalTitle'), icon: Settings },
    { id: 'serial', label: t('settings.serialTitle'), icon: Radio },
    { id: 'data', label: t('settings.dataTitle'), icon: BarChart3 },
    { id: 'notifications', label: t('settings.notificationsTitle'), icon: Bell },
  ]

  const resetToDefaults = async () => {
    resetSettings()
    try {
      await resetConfig()
      toast.success(t('settings.resetSuccess'))
    } catch {
      toast(t('settings.resetPartial'))
    }
  }

  const handleExport = () => {
    const success = exportSettings()
    if (success) {
      toast.success(t('settings.exportSuccess'))
    } else {
      toast.error(t('settings.exportFailed'))
    }
  }

  const handleImport = async (event: React.ChangeEvent<HTMLInputElement>) => {
    const file = event.target.files?.[0]
    if (!file) return

    setIsImporting(true)
    try {
      await importSettings(file)
      toast.success(t('settings.importSuccess'))
      window.location.reload()
    } catch (error) {
      toast.error(`${t('settings.importFailed')}: ${error instanceof Error ? error.message : t('common.unknownError')}`)
    } finally {
      setIsImporting(false)
      if (fileInputRef.current) {
        fileInputRef.current.value = ''
      }
    }
  }

  return (
    <div className="space-y-6 w-full">
      {/* Settings Header */}
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-lg font-medium text-text-primary">{t('settings.title')}</h2>
          <p className="text-sm text-text-tertiary mt-0.5">{t('settings.subtitle')}</p>
        </div>
        <div className="flex items-center gap-2">
          <button
            onClick={async () => {
              try {
                await saveConfig()
                toast.success(t('settings.saveSuccess'))
              } catch {
                toast.error(t('settings.saveFailed'))
              }
            }}
            className="flex items-center gap-1.5 px-3 py-1.5 text-sm rounded-md bg-signal text-black border border-signal hover:opacity-90 transition-colors"
            title={t('settings.saveToBackendHint')}
          >
            <Check size={14} strokeWidth={1.5} />
            {t('common.save')}
          </button>
          <button
            onClick={handleExport}
            className="flex items-center gap-1.5 px-3 py-1.5 text-sm rounded-md bg-bg-elevated text-text-secondary border border-border hover:text-text-primary transition-colors"
            title={t('settings.exportHint')}
          >
            <Download size={14} strokeWidth={1.5} />
            {t('common.export')}
          </button>
          <button
            onClick={() => fileInputRef.current?.click()}
            disabled={isImporting}
            className="flex items-center gap-1.5 px-3 py-1.5 text-sm rounded-md bg-bg-elevated text-text-secondary border border-border hover:text-text-primary transition-colors disabled:opacity-50"
            title={t('settings.importHint')}
          >
            <Upload size={14} strokeWidth={1.5} />
            {isImporting ? t('common.importing') : t('common.import')}
          </button>
          <input
            ref={fileInputRef}
            type="file"
            accept=".json"
            className="hidden"
            onChange={handleImport}
          />
          <button
            onClick={resetToDefaults}
            className="flex items-center gap-1.5 px-3 py-1.5 text-sm rounded-md bg-bg-elevated text-text-secondary border border-border hover:text-text-primary transition-colors"
          >
            <RotateCcw size={14} strokeWidth={1.5} />
            {t('common.reset')}
          </button>
        </div>
      </div>

      {/* Tab Navigation */}
      <div className="flex items-center gap-1 border-b border-border">
        {tabs.map((tab) => {
          const TabIcon = tab.icon
          return (
            <button
              key={tab.id}
              onClick={() => setActiveTab(tab.id)}
              className={cn(
                'flex items-center gap-1.5 px-4 py-2 text-sm font-medium border-b-2 -mb-px transition-colors',
                activeTab === tab.id
                  ? 'border-signal text-signal'
                  : 'border-transparent text-text-tertiary hover:text-text-secondary'
              )}
            >
              <TabIcon size={14} strokeWidth={1.5} />
              {tab.label}
            </button>
          )
        })}
      </div>

      {/* Tab Content */}
      {activeTab === 'general' && (
        <Panel title={t('settings.generalTitle')} variant="default">
          <div className="space-y-6">
            <div className="flex items-center justify-between">
              <div>
                <div className="text-sm text-text-primary">{t('settings.autoCheckUpdates')}</div>
                <div className="text-xs text-text-tertiary">{t('settings.autoCheckUpdatesHint')}</div>
              </div>
              <ToggleSwitch
                checked={settings.general.autoCheckUpdates}
                onChange={(v) => updateSettings({ general: { autoCheckUpdates: v } })}
              />
            </div>

            <div className="flex items-center justify-between">
              <div>
                <div className="text-sm text-text-primary">{t('settings.sendAnalytics')}</div>
                <div className="text-xs text-text-tertiary">{t('settings.sendAnalyticsHint')}</div>
              </div>
              <ToggleSwitch
                checked={settings.general.sendAnalytics}
                onChange={(v) => updateSettings({ general: { sendAnalytics: v } })}
              />
            </div>

            <div className="flex items-center justify-between">
              <div>
                <div className="text-sm text-text-primary">{t('settings.minimizeToTray')}</div>
                <div className="text-xs text-text-tertiary">{t('settings.minimizeToTrayHint')}</div>
              </div>
              <ToggleSwitch
                checked={settings.general.minimizeToTray}
                onChange={(v) => updateSettings({ general: { minimizeToTray: v } })}
              />
            </div>

            <div className="pt-4 border-t border-border">
              <div className="text-sm text-text-primary mb-2">{t('settings.language')}</div>
              <select
                value={settings.general.language}
                onChange={(e) => updateSettings({ general: { language: e.target.value } })}
                className="w-full max-w-xs px-3 py-2 bg-bg-deep border border-border rounded-md text-sm text-text-primary"
              >
                <option value="en">{t('settings.language.en')}</option>
                <option value="zh">{t('settings.language.zh')}</option>
              </select>
            </div>
          </div>
        </Panel>
      )}

      {activeTab === 'serial' && (
        <Panel title={t('settings.serialTitle')} variant="signal">
          <div className="space-y-4">
            <div className="grid grid-cols-2 gap-4">
              <div>
                <label className="text-xs text-text-tertiary uppercase tracking-wider block mb-2">
                  {t('terminal.baudrate')}
                </label>
                <select
                  value={serialConfig.baudRate}
                  onChange={(e) => updateSettings({ serial: { baudRate: parseInt(e.target.value) } })}
                  className="w-full px-3 py-2 bg-bg-deep border border-border rounded-md text-sm text-text-primary font-mono"
                >
                  <option value={1200}>1200</option>
                  <option value={2400}>2400</option>
                  <option value={4800}>4800</option>
                  <option value={9600}>9600</option>
                  <option value={19200}>19200</option>
                  <option value={38400}>38400</option>
                  <option value={57600}>57600</option>
                  <option value={115200}>115200</option>
                  <option value={230400}>230400</option>
                  <option value={460800}>460800</option>
                  <option value={921600}>921600</option>
                </select>
              </div>

              <div>
                <label className="text-xs text-text-tertiary uppercase tracking-wider block mb-2">
                  {t('terminal.dataBits')}
                </label>
                <select
                  value={serialConfig.dataBits}
                  onChange={(e) => updateSettings({ serial: { dataBits: parseInt(e.target.value) } })}
                  className="w-full px-3 py-2 bg-bg-deep border border-border rounded-md text-sm text-text-primary font-mono"
                >
                  <option value={5}>5</option>
                  <option value={6}>6</option>
                  <option value={7}>7</option>
                  <option value={8}>8</option>
                </select>
              </div>

              <div>
                <label className="text-xs text-text-tertiary uppercase tracking-wider block mb-2">
                  {t('terminal.stopBits')}
                </label>
                <select
                  value={serialConfig.stopBits}
                  onChange={(e) => updateSettings({ serial: { stopBits: parseInt(e.target.value) } })}
                  className="w-full px-3 py-2 bg-bg-deep border border-border rounded-md text-sm text-text-primary font-mono"
                >
                  <option value={1}>1</option>
                  <option value={2}>2</option>
                </select>
              </div>

              <div>
                <label className="text-xs text-text-tertiary uppercase tracking-wider block mb-2">
                  {t('terminal.parity')}
                </label>
                <select
                  value={serialConfig.parity}
                  onChange={(e) => updateSettings({ serial: { parity: e.target.value as SerialConfig['parity'] } })}
                  className="w-full px-3 py-2 bg-bg-deep border border-border rounded-md text-sm text-text-primary font-mono"
                >
                  <option value="none">{t('terminal.parityNone')}</option>
                  <option value="even">{t('terminal.parityEven')}</option>
                  <option value="odd">{t('terminal.parityOdd')}</option>
                </select>
              </div>

              <div>
                <label className="text-xs text-text-tertiary uppercase tracking-wider block mb-2">
                  {t('terminal.flowControl')}
                </label>
                <select
                  value={serialConfig.flowControl}
                  onChange={(e) => updateSettings({ serial: { flowControl: e.target.value as SerialConfig['flowControl'] } })}
                  className="w-full px-3 py-2 bg-bg-deep border border-border rounded-md text-sm text-text-primary font-mono"
                >
                  <option value="none">{t('terminal.flowNone')}</option>
                  <option value="rts">RTS</option>
                  <option value="cts">CTS</option>
                  <option value="rtscts">RTS/CTS</option>
                </select>
              </div>
            </div>

            <div className="pt-4 border-t border-border">
              <div className="flex items-center justify-between">
                <div>
                  <div className="text-sm text-text-primary">{t('settings.autoReconnect')}</div>
                  <div className="text-xs text-text-tertiary">{t('settings.autoReconnectHint')}</div>
                </div>
                <ToggleSwitch
                  checked={settings.serial.autoReconnect ?? true}
                  onChange={(v) => updateSettings({ serial: { autoReconnect: v } })}
                />
              </div>
            </div>
          </div>
        </Panel>
      )}

      {activeTab === 'data' && (
        <Panel title={t('settings.dataTitle')} variant="info">
          <div className="space-y-4">
            <div>
              <label className="text-xs text-text-tertiary uppercase tracking-wider block mb-2">
                {t('settings.displayFormat')}
              </label>
              <div className="flex items-center gap-2">
                {(['hex', 'ascii', 'both'] as const).map((format) => (
                  <button
                    key={format}
                    onClick={() => updateSettings({ display: { format } })}
                    className={cn(
                      'px-4 py-2 text-sm rounded-md border transition-colors',
                      dataConfig.displayFormat === format
                        ? 'bg-info/10 text-info border-info/30'
                        : 'bg-bg-elevated text-text-tertiary border-border hover:text-text-primary'
                    )}
                  >
                    {format.toUpperCase()}
                  </button>
                ))}
              </div>
            </div>

            <div className="flex items-center justify-between">
              <div>
                <div className="text-sm text-text-primary">{t('settings.showTimestamps')}</div>
                <div className="text-xs text-text-tertiary">{t('settings.showTimestampsHint')}</div>
              </div>
              <ToggleSwitch
                checked={dataConfig.showTimestamp}
                onChange={(checked) => updateSettings({ display: { showTimestamp: checked } })}
              />
            </div>

            <div className="flex items-center justify-between">
              <div>
                <div className="text-sm text-text-primary">{t('settings.autoScroll')}</div>
                <div className="text-xs text-text-tertiary">{t('settings.autoScrollHint')}</div>
              </div>
              <ToggleSwitch
                checked={dataConfig.autoScroll}
                onChange={(checked) => updateSettings({ display: { autoScroll: checked } })}
              />
            </div>

            <div className="pt-4 border-t border-border">
              <label className="text-sm font-medium text-text-primary block mb-2">
                {t('settings.maxPackets')}
              </label>
              <input
                type="range"
                min="100"
                max="10000"
                step="100"
                value={dataConfig.maxPackets}
                onChange={(e) => updateSettings({ display: { maxPackets: parseInt(e.target.value) } })}
                className="w-full max-w-xs"
              />
              <div className="text-xs text-text-tertiary font-mono mt-1">
                {t('settings.currentPackets', { count: dataConfig.maxPackets })}
              </div>
            </div>
          </div>
        </Panel>
      )}

      {activeTab === 'notifications' && <NotificationSettings />}
    </div>
  )
}

function ToggleSwitch({
  checked,
  onChange,
}: {
  checked: boolean
  onChange?: (checked: boolean) => void
}) {
  const handleChange = () => {
    onChange?.(!checked)
  }

  return (
    <button
      onClick={handleChange}
      className={cn(
        'w-12 h-6 rounded-full p-1 transition-colors relative',
        checked ? 'bg-signal' : 'bg-bg-elevated'
      )}
    >
      <div className={cn(
        'w-4 h-4 rounded-full bg-white transition-transform flex items-center justify-center',
        checked ? 'translate-x-6' : 'translate-x-0'
      )}>
        {checked && <Check size={10} strokeWidth={3} className="text-signal" />}
      </div>
    </button>
  )
}
