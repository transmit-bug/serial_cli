import { useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { Button } from '@/components/ui/button'
import { useConnectionStore, useDataStore, useProtocolStore } from '@/stores'
import { Send, FileText, Clock } from 'lucide-react'
import { toast } from 'sonner'
import { useTranslation } from 'react-i18next'

/**
 * TxSender - TX 发送区
 */
export function TxSender() {
  const { portId } = useConnectionStore()
  const { addTxPacket } = useDataStore()
  const { protocols, activeProtocol } = useProtocolStore()
  const { t } = useTranslation()
  const [inputMode, setInputMode] = useState<'hex' | 'ascii'>('hex')
  const [inputData, setInputData] = useState('')
  const [isSending, setIsSending] = useState(false)
  const [useProtocolEncode, setUseProtocolEncode] = useState(false)

  const handleSend = async () => {
    if (!portId) {
      toast.error(t('toast.noPortConnected'))
      return
    }

    if (!inputData.trim()) {
      toast.error(t('toast.noInputData'))
      return
    }

    if (useProtocolEncode && !activeProtocol) {
      toast.error(t('toast.selectProtocol'))
      return
    }

    setIsSending(true)
    try {
      let data: number[] = []

      if (inputMode === 'hex') {
        const hex = inputData.replace(/\s/g, '')
        if (!/^[0-9A-Fa-f]*$/.test(hex)) {
          throw new Error(t('toast.invalidHex'))
        }
        // 将十六进制字符串转换为字节数组
        const len = hex.length
        for (let i = 0; i < len; i += 2) {
          data.push(parseInt(hex.substr(i, 2), 16))
        }
      } else {
        data = Array.from(new TextEncoder().encode(inputData))
      }

      // Apply protocol encoding if enabled
      if (useProtocolEncode && activeProtocol) {
        try {
          const encodedData = await invoke<number[]>('protocol_encode', {
            protocolName: activeProtocol,
            data,
          })
          data = encodedData
        } catch (encodeError) {
          toast.error(`${t('toast.encodingFailed')}: ${encodeError instanceof Error ? encodeError.message : t('common.unknownError')}`)
          return
        }
      }

      // Invoke Tauri command to send data to serial port
      const bytesWritten = await invoke<number>('send_data', {
        portId,
        data,
      })

      addTxPacket({
        portId,
        direction: 'tx',
        data,
        timestamp: Date.now(),
      })

      toast.success(t('toast.sendSuccess', { bytes: bytesWritten }))
      setInputData('')
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('toast.sendFailed'))
    } finally {
      setIsSending(false)
    }
  }

  return (
    <div className="h-full flex flex-col bg-bg-deep">
      {/* 工具栏 */}
      <div className="flex items-center justify-between px-4 py-2 border-b border-border">
        <div className="flex items-center gap-2">
          <span className="text-sm font-medium text-text-primary">{t('terminal.txData')}</span>

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

          {/* 协议编码开关 */}
          {activeProtocol && (
            <button
              onClick={() => setUseProtocolEncode(!useProtocolEncode)}
              className={
                'px-3 py-1 text-xs font-medium rounded-md border transition-colors ' +
                (useProtocolEncode
                  ? 'bg-amber/20 text-amber border-amber/30'
                  : 'bg-bg-base text-text-secondary border-border hover:text-text-primary hover:bg-bg-elevated')
              }
            >
              {useProtocolEncode ? `✓ ${activeProtocol}` : activeProtocol}
            </button>
          )}
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
        {/* 快捷指令提示 */}
        <div className="flex items-center gap-2 text-xs text-text-tertiary">
          <span>{t('terminal.quickCommandHint', '快捷指令在右侧面板')}</span>
          <button
            onClick={() => setInputData((prev) => prev + '\\r\\n')}
            className="px-2 py-0.5 text-[10px] bg-bg-base border border-border rounded hover:bg-bg-elevated transition-colors"
          >
            + CRLF
          </button>
        </div>

        {/* 输入框 */}
        <div className="flex-1 relative">
          <textarea
            value={inputData}
            onChange={(e) => setInputData(e.target.value)}
            placeholder={inputMode === 'hex' ? t('terminal.hexInput') : t('terminal.asciiInput')}
            className="w-full h-full min-h-0 bg-bg-base border border-border rounded-lg p-3 text-sm font-mono resize-none focus:outline-none focus:border-signal/50 transition-colors"
            disabled={isSending}
          />
          <div className="absolute bottom-2 right-2 text-xs text-text-tertiary">
            {inputData.length} {t('terminal.characterCount')}
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
          {isSending ? t('terminal.sending') : t('terminal.sendData')}
        </Button>
      </div>
    </div>
  )
}
