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
    if (!config) return

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
      toast.warning(`后端配置: ${backendError}`)
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
    { id: 'general', label: 'General', icon: Settings },
    { id: 'serial', label: 'Serial', icon: Radio },
    { id: 'data', label: 'Data', icon: BarChart3 },
    { id: 'notifications', label: 'Notifications', icon: Bell },
  ]

  const resetToDefaults = async () => {
    resetSettings()
    try {
      await resetConfig()
      toast('已恢复默认设置并同步到后端')
    } catch {
      toast('已恢复默认设置（后端同步失败）')
    }
  }

  const handleExport = () => {
    const success = exportSettings()
    if (success) {
      toast.success('设置已导出')
    } else {
      toast.error('导出设置失败')
    }
  }

  const handleImport = async (event: React.ChangeEvent<HTMLInputElement>) => {
    const file = event.target.files?.[0]
    if (!file) return

    setIsImporting(true)
    try {
      await importSettings(file)
      toast.success('设置已导入，页面将刷新')
      window.location.reload()
    } catch (error) {
      toast.error(`导入设置失败: ${error instanceof Error ? error.message : '未知错误'}`)
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
          <h2 className="text-lg font-medium text-text-primary">Settings</h2>
          <p className="text-sm text-text-tertiary mt-0.5">Configure application preferences</p>
        </div>
        <div className="flex items-center gap-2">
          <button
            onClick={async () => {
              try {
                await saveConfig()
                toast.success('设置已保存到后端')
              } catch {
                toast.error('保存到后端失败')
              }
            }}
            className="flex items-center gap-1.5 px-3 py-1.5 text-sm rounded-md bg-signal text-black border border-signal hover:opacity-90 transition-colors"
            title="Save settings to config file"
          >
            <Check size={14} strokeWidth={1.5} />
            Save
          </button>
          <button
            onClick={handleExport}
            className="flex items-center gap-1.5 px-3 py-1.5 text-sm rounded-md bg-bg-elevated text-text-secondary border border-border hover:text-text-primary transition-colors"
            title="Export all settings"
          >
            <Download size={14} strokeWidth={1.5} />
            Export
          </button>
          <button
            onClick={() => fileInputRef.current?.click()}
            disabled={isImporting}
            className="flex items-center gap-1.5 px-3 py-1.5 text-sm rounded-md bg-bg-elevated text-text-secondary border border-border hover:text-text-primary transition-colors disabled:opacity-50"
            title="Import settings from file"
          >
            <Upload size={14} strokeWidth={1.5} />
            {isImporting ? 'Importing...' : 'Import'}
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
            Reset
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
        <Panel title="General Settings" variant="default">
          <div className="space-y-6">
            <div className="flex items-center justify-between">
              <div>
                <div className="text-sm text-text-primary">Auto-check for updates</div>
                <div className="text-xs text-text-tertiary">Check for new versions on startup</div>
              </div>
              <ToggleSwitch
                checked={settings.general.autoCheckUpdates}
                onChange={(v) => updateSettings({ general: { autoCheckUpdates: v } })}
              />
            </div>

            <div className="flex items-center justify-between">
              <div>
                <div className="text-sm text-text-primary">Send usage analytics</div>
                <div className="text-xs text-text-tertiary">Help improve the app with anonymous data</div>
              </div>
              <ToggleSwitch
                checked={settings.general.sendAnalytics}
                onChange={(v) => updateSettings({ general: { sendAnalytics: v } })}
              />
            </div>

            <div className="flex items-center justify-between">
              <div>
                <div className="text-sm text-text-primary">Minimize to tray</div>
                <div className="text-xs text-text-tertiary">Close button minimizes instead of quitting</div>
              </div>
              <ToggleSwitch
                checked={settings.general.minimizeToTray}
                onChange={(v) => updateSettings({ general: { minimizeToTray: v } })}
              />
            </div>

            <div className="pt-4 border-t border-border">
              <div className="text-sm text-text-primary mb-2">Language</div>
              <select
                value={settings.general.language}
                onChange={(e) => updateSettings({ general: { language: e.target.value } })}
                className="w-full max-w-xs px-3 py-2 bg-bg-deep border border-border rounded-md text-sm text-text-primary"
              >
                <option value="en">English</option>
                <option value="zh">简体中文</option>
              </select>
            </div>
          </div>
        </Panel>
      )}

      {activeTab === 'serial' && (
        <Panel title="Serial Port Defaults" variant="signal">
          <div className="space-y-4">
            <div className="grid grid-cols-2 gap-4">
              <div>
                <label className="text-xs text-text-tertiary uppercase tracking-wider block mb-2">
                  Baud Rate
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
                  Data Bits
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
                  Stop Bits
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
                  Parity
                </label>
                <select
                  value={serialConfig.parity}
                  onChange={(e) => updateSettings({ serial: { parity: e.target.value as SerialConfig['parity'] } })}
                  className="w-full px-3 py-2 bg-bg-deep border border-border rounded-md text-sm text-text-primary font-mono"
                >
                  <option value="none">None</option>
                  <option value="even">Even</option>
                  <option value="odd">Odd</option>
                </select>
              </div>

              <div>
                <label className="text-xs text-text-tertiary uppercase tracking-wider block mb-2">
                  Flow Control
                </label>
                <select
                  value={serialConfig.flowControl}
                  onChange={(e) => updateSettings({ serial: { flowControl: e.target.value as SerialConfig['flowControl'] } })}
                  className="w-full px-3 py-2 bg-bg-deep border border-border rounded-md text-sm text-text-primary font-mono"
                >
                  <option value="none">None</option>
                  <option value="rts">RTS</option>
                  <option value="cts">CTS</option>
                  <option value="rtscts">RTS/CTS</option>
                </select>
              </div>
            </div>

            <div className="pt-4 border-t border-border">
              <div className="flex items-center justify-between">
                <div>
                  <div className="text-sm text-text-primary">Auto-reconnect on disconnect</div>
                  <div className="text-xs text-text-tertiary">Automatically reconnect if connection is lost</div>
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
        <Panel title="Data Display Settings" variant="info">
          <div className="space-y-4">
            <div>
              <label className="text-xs text-text-tertiary uppercase tracking-wider block mb-2">
                Default Display Format
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
                <div className="text-sm text-text-primary">Show Timestamps</div>
                <div className="text-xs text-text-tertiary">Display timestamp for each packet</div>
              </div>
              <ToggleSwitch
                checked={dataConfig.showTimestamp}
                onChange={(checked) => updateSettings({ display: { showTimestamp: checked } })}
              />
            </div>

            <div className="flex items-center justify-between">
              <div>
                <div className="text-sm text-text-primary">Auto-scroll</div>
                <div className="text-xs text-text-tertiary">Automatically scroll to latest data</div>
              </div>
              <ToggleSwitch
                checked={dataConfig.autoScroll}
                onChange={(checked) => updateSettings({ display: { autoScroll: checked } })}
              />
            </div>

            <div className="pt-4 border-t border-border">
              <label className="text-sm font-medium text-text-primary block mb-2">
                Max Packets in Buffer
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
                Current: {dataConfig.maxPackets} packets
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
