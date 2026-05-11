import { useState } from 'react'
import { Button } from '@/components/ui/button'
import { useConnectionStore, useDataStore } from '@/stores'
import { Send, FileText, Clock } from 'lucide-react'
import { toast } from 'sonner'

/**
 * TxSender - TX 发送区
 */
export function TxSender() {
  const { portId } = useConnectionStore()
  const { addTxPacket } = useDataStore()
  const [inputMode, setInputMode] = useState<'hex' | 'ascii'>('hex')
  const [inputData, setInputData] = useState('')
  const [isSending, setIsSending] = useState(false)

  const handleSend = async () => {
    if (!portId) {
      toast.error('未连接到串口')
      return
    }

    if (!inputData.trim()) {
      toast.error('请输入要发送的数据')
      return
    }

    setIsSending(true)
    try {
      let data: number[] = []

      if (inputMode === 'hex') {
        const hex = inputData.replace(/\s/g, '')
        if (!/^[0-9A-Fa-f]*$/.test(hex)) {
          throw new Error('无效的十六进制数据')
        }
        // 将十六进制字符串转换为字节数组
        const len = hex.length
        for (let i = 0; i < len; i += 2) {
          data.push(parseInt(hex.substr(i, 2), 16))
        }
      } else {
        data = Array.from(new TextEncoder().encode(inputData))
      }

      // TODO: 调用 Tauri 命令发送数据
      // await invoke('send_data', { portId, data })

      addTxPacket({
        portId,
        direction: 'tx',
        data,
        timestamp: Date.now(),
      })

      toast.success('已发送 ' + data.length + ' 字节')
      setInputData('')
    } catch (error) {
      toast.error(error instanceof Error ? error.message : '发送失败')
    } finally {
      setIsSending(false)
    }
  }

  return (
    <div className="h-full flex flex-col bg-bg-deep">
      {/* 工具栏 */}
      <div className="flex items-center justify-between px-4 py-2 border-b border-border">
        <div className="flex items-center gap-2">
          <span className="text-sm font-medium text-text-primary">发送数据 (TX)</span>

          {/* 模式切换 */}
          <div className="flex bg-bg-base rounded-md border border-border">
            {(['hex', 'ascii'] as const).map((mode) => (
              <button
                key={mode}
                onClick={() => setInputMode(mode)}
                className={
                  'px-3 py-1 text-xs font-medium rounded transition-colors ' +
                  (inputMode === mode
                    ? 'bg-signal text-black'
                    : 'text-text-secondary hover:text-text-primary hover:bg-bg-elevated')
                }
              >
                {mode.toUpperCase()}
              </button>
            ))}
          </div>
        </div>

        <div className="flex items-center gap-2">
          <Button variant="ghost" size="sm">
            <Clock className="w-4 h-4" />
          </Button>
          <Button variant="ghost" size="sm">
            <FileText className="w-4 h-4" />
          </Button>
        </div>
      </div>

      {/* 输入区域 */}
      <div className="flex-1 p-4 space-y-3">
        {/* 快捷按钮 */}
        <div className="flex gap-2">
          <Button
            variant="outline"
            size="sm"
            onClick={() => setInputData((prev) => prev + 'AT')}
          >
            AT
          </Button>
          <Button
            variant="outline"
            size="sm"
            onClick={() => setInputData((prev) => prev + '\\r\\n')}
          >
            CRLF
          </Button>
        </div>

        {/* 输入框 */}
        <div className="flex-1 relative">
          <textarea
            value={inputData}
            onChange={(e) => setInputData(e.target.value)}
            placeholder={inputMode === 'hex' ? '输入十六进制数据' : '输入 ASCII 字符'}
            className="w-full h-full min-h-0 bg-bg-base border border-border rounded-lg p-3 text-sm font-mono resize-none focus:outline-none focus:border-signal/50 transition-colors"
            disabled={isSending}
          />
          <div className="absolute bottom-2 right-2 text-xs text-text-tertiary">
            {inputData.length} 字符
          </div>
        </div>

        {/* 发送按钮 */}
        <Button
          variant="signal"
          onClick={handleSend}
          disabled={isSending || !inputData.trim()}
          className="w-full"
        >
          <Send className="w-4 h-4 mr-2" />
          {isSending ? '发送中...' : '发送数据'}
        </Button>
      </div>
    </div>
  )
}
