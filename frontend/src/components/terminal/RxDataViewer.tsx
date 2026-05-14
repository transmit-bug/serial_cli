import { useDataStore, useProtocolStore } from '@/stores'
import { VirtualList } from '@/components/ui/virtual-list'
import { Button } from '@/components/ui/button'
import { Trash2, Download, Search, ChevronDown } from 'lucide-react'
import { useState, useEffect, useRef } from 'react'
import { toast } from 'sonner'
import { invoke } from '@tauri-apps/api/core'

/**
 * RxDataViewer - RX 数据显示区
 */
export function RxDataViewer() {
  const { rxPackets, clearPackets, setDisplayFormat, displayFormat } = useDataStore()
  const { activeProtocol } = useProtocolStore()
  const [searchQuery, setSearchQuery] = useState('')
  const [showSearch, setShowSearch] = useState(false)
  const [useProtocolDecode, setUseProtocolDecode] = useState(false)
  const [decodedData, setDecodedData] = useState<Map<string, number[] | null>>(new Map())
  const [isDecoding, setIsDecoding] = useState(false)
  const [exportFormat, setExportFormat] = useState<'txt' | 'csv' | 'json'>('txt')
  const [showExportMenu, setShowExportMenu] = useState(false)
  const exportMenuRef = useRef<HTMLDivElement>(null)

  // Close export menu on click outside
  useEffect(() => {
    if (!showExportMenu) return
    const handler = (e: MouseEvent) => {
      if (exportMenuRef.current && !exportMenuRef.current.contains(e.target as Node)) {
        setShowExportMenu(false)
      }
    }
    document.addEventListener('mousedown', handler)
    return () => document.removeEventListener('mousedown', handler)
  }, [showExportMenu])

  // Auto-decode new packets when protocol decode is enabled
  useEffect(() => {
    const decodePackets = async () => {
      if (!useProtocolDecode || !activeProtocol) return

      setIsDecoding(true)
      try {
        const newDecodedData = new Map(decodedData)

        for (const packet of rxPackets) {
          const packetKey = `${packet.timestamp}-${packet.data.length}`
          if (newDecodedData.has(packetKey)) continue

          try {
            const decoded = await invoke<number[]>('protocol_decode', {
              protocol: activeProtocol,
              data: packet.data,
            })
            newDecodedData.set(packetKey, decoded)
          } catch (decodeError) {
            console.error('Decode error:', decodeError)
            newDecodedData.set(packetKey, null)
          }
        }

        setDecodedData(newDecodedData)
      } finally {
        setIsDecoding(false)
      }
    }

    if (useProtocolDecode && activeProtocol && rxPackets.length > 0) {
      decodePackets()
    }
  }, [useProtocolDecode, activeProtocol, rxPackets])

  const filteredPackets = searchQuery
    ? rxPackets.filter(p => {
        const hex = new Uint8Array(p.data).reduce((str, byte) => str + byte.toString(16).padStart(2, '0'), '')
        const ascii = String.fromCharCode(...p.data)
        return hex.includes(searchQuery.toUpperCase()) || ascii.includes(searchQuery)
      })
    : rxPackets

  const formatData = (data: number[], timestamp: number) => {
    // Check if decoded data is available
    if (useProtocolDecode && activeProtocol) {
      const packetKey = `${timestamp}-${data.length}`
      const decoded = decodedData.get(packetKey)
      if (decoded !== undefined) {
        if (decoded === null) {
          return '(incomplete frame)'
        }
        const renderData = decoded
        if (displayFormat === 'hex') {
          return new Uint8Array(renderData).reduce((str, byte) => str + byte.toString(16).padStart(2, '0'), '')
        } else if (displayFormat === 'ascii') {
          return String.fromCharCode(...renderData)
        } else {
          const hex = new Uint8Array(renderData).reduce((str, byte) => str + byte.toString(16).padStart(2, '0'), '')
          const ascii = String.fromCharCode(...renderData)
          return hex + ' | ' + ascii
        }
      }
    }

    if (displayFormat === 'hex') {
      return new Uint8Array(data).reduce((str, byte) => str + byte.toString(16).padStart(2, '0'), '')
    } else if (displayFormat === 'ascii') {
      return String.fromCharCode(...data)
    } else {
      const hex = new Uint8Array(data).reduce((str, byte) => str + byte.toString(16).padStart(2, '0'), '')
      const ascii = String.fromCharCode(...data)
      return hex + ' | ' + ascii
    }
  }

  const handleExport = () => {
    let content: string
    let filename: string
    let mimeType: string

    if (exportFormat === 'csv') {
      // CSV format with headers
      const headers = ['Timestamp', 'Direction', 'Length', 'Data']
      const rows = filteredPackets.map(p => [
        new Date(p.timestamp).toISOString(),
        'RX',
        p.data.length.toString(),
        formatData(p.data, p.timestamp)
      ])
      content = [headers.join(','), ...rows.map(row => row.map(cell => `"${cell}"`).join(','))].join('\n')
      filename = `rx-data-${Date.now()}.csv`
      mimeType = 'text/csv'
    } else if (exportFormat === 'json') {
      // JSON format
      const json = filteredPackets.map(p => ({
        timestamp: new Date(p.timestamp).toISOString(),
        direction: 'RX',
        length: p.data.length,
        data: formatData(p.data, p.timestamp),
        rawBytes: p.data
      }))
      content = JSON.stringify(json, null, 2)
      filename = `rx-data-${Date.now()}.json`
      mimeType = 'application/json'
    } else {
      // Plain text format (default)
      content = filteredPackets
        .map(p => '[' + new Date(p.timestamp).toISOString() + '] ' + formatData(p.data, p.timestamp))
        .join('\n')
      filename = `rx-data-${Date.now()}.txt`
      mimeType = 'text/plain'
    }

    const blob = new Blob([content], { type: mimeType })
    const url = URL.createObjectURL(blob)
    const a = document.createElement('a')
    a.href = url
    a.download = filename
    a.click()
    URL.revokeObjectURL(url)

    toast.success(`数据已导出为 ${exportFormat.toUpperCase()} 格式`)
    setShowExportMenu(false)
  }

  return (
    <div className="h-full flex flex-col bg-bg-deep">
      {/* 工具栏 */}
      <div className="flex items-center justify-between px-4 py-2 border-b border-border">
        <div className="flex items-center gap-2">
          <span className="text-sm font-medium text-text-primary">接收数据 (RX)</span>
          <span className="text-xs text-text-tertiary bg-bg-base px-2 py-0.5 rounded">
            {filteredPackets.length} 包
          </span>
        </div>

        <div className="flex items-center gap-2">
          {/* 格式切换 */}
          <div className="flex bg-bg-base rounded-md border border-border">
            {(['hex', 'ascii', 'mixed'] as const).map((fmt) => (
              <button
                key={fmt}
                onClick={() => setDisplayFormat(fmt)}
                className={
                  'px-3 py-1 text-xs font-medium rounded transition-colors ' +
                  (displayFormat === fmt
                    ? 'bg-signal text-black'
                    : 'text-text-secondary hover:text-text-primary hover:bg-bg-elevated')
                }
              >
                {fmt.toUpperCase()}
              </button>
            ))}
          </div>

          {/* 协议解码开关 */}
          {activeProtocol && (
            <button
              onClick={() => {
                setUseProtocolDecode(!useProtocolDecode)
                if (!useProtocolDecode) {
                  setDecodedData(new Map())
                }
              }}
              disabled={isDecoding}
              className={
                'px-3 py-1 text-xs font-medium rounded-md border transition-colors disabled:opacity-50 disabled:cursor-not-allowed ' +
                (useProtocolDecode
                  ? 'bg-amber/20 text-amber border-amber/30'
                  : 'bg-bg-base text-text-secondary border-border hover:text-text-primary hover:bg-bg-elevated')
              }
            >
              {isDecoding ? 'Decoding...' : useProtocolDecode ? `✓ ${activeProtocol}` : activeProtocol}
            </button>
          )}

          {/* 搜索按钮 */}
          <Button
            variant="ghost"
            size="sm"
            onClick={() => setShowSearch(!showSearch)}
          >
            <Search className="w-4 h-4" />
          </Button>

          {/* 清空按钮 */}
          <Button
            variant="ghost"
            size="sm"
            onClick={() => {
              clearPackets()
              toast.success('数据已清空')
            }}
          >
            <Trash2 className="w-4 h-4" />
          </Button>

          {/* 导出按钮 */}
          <div ref={exportMenuRef} className="relative">
            <Button
              variant="ghost"
              size="sm"
              onClick={() => setShowExportMenu(!showExportMenu)}
            >
              <Download className="w-4 h-4" />
              <ChevronDown className="w-3 h-3 ml-1" />
            </Button>

            {/* 导出格式下拉菜单 */}
            {showExportMenu && (
              <div className="absolute right-0 mt-2 w-32 bg-bg-base border border-border rounded-md shadow-lg z-10">
                <div className="py-1">
                  {(['txt', 'csv', 'json'] as const).map((format) => (
                    <button
                      key={format}
                      onClick={() => {
                        setExportFormat(format)
                        handleExport()
                      }}
                      className="w-full px-3 py-2 text-left text-xs hover:bg-bg-elevated transition-colors flex items-center justify-between"
                    >
                      <span className="text-text-secondary">{format.toUpperCase()}</span>
                      {exportFormat === format && (
                        <span className="text-signal">✓</span>
                      )}
                    </button>
                  ))}
                </div>
              </div>
            )}
          </div>
        </div>
      </div>

      {/* 搜索栏 */}
      {showSearch && (
        <div className="px-4 py-2 border-b border-border bg-bg-base">
          <input
            type="text"
            placeholder="搜索 HEX 或 ASCII 数据..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="w-full bg-bg-deep border border-border rounded px-3 py-2 text-sm text-text-primary placeholder:text-text-tertiary focus:outline-none focus:border-signal/50"
            autoFocus
          />
        </div>
      )}

      {/* 数据列表（虚拟滚动） */}
      <div className="flex-1 overflow-hidden">
        <VirtualList
          data={filteredPackets}
          renderItem={(packet) => (
            <div className="px-4 py-2 border-b border-border/50 hover:bg-bg-elevated transition-colors">
              <div className="flex items-start gap-3">
                <span className="text-xs text-text-tertiary font-mono mt-0.5">
                  {new Date(packet.timestamp).toLocaleTimeString()}
                </span>
                <span className="flex-1 font-mono text-sm text-text-primary break-all">
                  {formatData(packet.data, packet.timestamp)}
                </span>
                <span className="text-xs text-text-tertiary">
                  {packet.data.length} B
                </span>
              </div>
            </div>
          )}
          height="100%"
          itemHeight={(packet) => Math.max(40, Math.ceil(packet.data.length / 32) * 20)}
          emptyContent={<div className="text-center text-text-tertiary p-8">暂无接收数据</div>}
        />
      </div>
    </div>
  )
}
