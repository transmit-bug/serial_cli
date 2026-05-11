import { useDataStore } from '@/stores'
import { VirtualList } from '@/components/ui/virtual-list'
import { Button } from '@/components/ui/button'
import { Trash2, Download, Search } from 'lucide-react'
import { useState } from 'react'
import { toast } from 'sonner'

/**
 * RxDataViewer - RX 数据显示区
 */
export function RxDataViewer() {
  const { rxPackets, clearPackets, setDisplayFormat, displayFormat } = useDataStore()
  const [searchQuery, setSearchQuery] = useState('')
  const [showSearch, setShowSearch] = useState(false)

  const filteredPackets = searchQuery
    ? rxPackets.filter(p => {
        const hex = new Uint8Array(p.data).reduce((str, byte) => str + byte.toString(16).padStart(2, '0'), '')
        const ascii = String.fromCharCode(...p.data)
        return hex.includes(searchQuery.toUpperCase()) || ascii.includes(searchQuery)
      })
    : rxPackets

  const formatData = (data: number[]) => {
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
    const text = filteredPackets
      .map(p => '[' + new Date(p.timestamp).toISOString() + '] ' + formatData(p.data))
      .join('\n')

    const blob = new Blob([text], { type: 'text/plain' })
    const url = URL.createObjectURL(blob)
    const a = document.createElement('a')
    a.href = url
    a.download = 'rx-data-' + Date.now() + '.txt'
    a.click()
    URL.revokeObjectURL(url)

    toast.success('数据已导出')
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
          <Button
            variant="ghost"
            size="sm"
            onClick={handleExport}
          >
            <Download className="w-4 h-4" />
          </Button>
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
                  {formatData(packet.data)}
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
