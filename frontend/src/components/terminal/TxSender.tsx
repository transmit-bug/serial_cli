import { useState, useRef, useEffect, useCallback } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'
import { readFile } from '@tauri-apps/plugin-fs'
import { Button } from '@/components/ui/button'
import { useConnectionStore, useDataStore, useProtocolStore } from '@/stores'
import { Send, FileText, Clock, ChevronUp, ChevronDown, X } from 'lucide-react'
import { toast } from 'sonner'
import { cn } from '@/lib/utils'
import { useTranslation } from 'react-i18next'

const MAX_FILE_SIZE = 1024 * 1024 // 1MB
const MAX_HISTORY = 20

interface HistoryEntry {
  content: string
  mode: 'hex' | 'ascii'
  timestamp: number
}

function historyStorage(): {
  get: () => HistoryEntry[]
  add: (entry: Omit<HistoryEntry, 'timestamp'>) => void
  clear: () => void
} {
  const KEY = 'serial-cli-send-history'
  return {
    get: () => {
      try {
        const raw = localStorage.getItem(KEY)
        return raw ? JSON.parse(raw) : []
      } catch { return [] }
    },
    add: (entry) => {
      const history = historyStorage().get()
      // Remove exact duplicate from history
      const filtered = history.filter(h => h.content !== entry.content || h.mode !== entry.mode)
      const updated = [{ ...entry, timestamp: Date.now() }, ...filtered].slice(0, MAX_HISTORY)
      localStorage.setItem(KEY, JSON.stringify(updated))
    },
    clear: () => localStorage.removeItem(KEY),
  }
}

/**
 * TxSender - TX 发送区
 *
 * Features:
 * - HEX/ASCII input mode toggle
 * - Protocol encoding toggle
 * - Send history (clock button) with keyboard navigation
 * - Load from file (file button)
 * - CRLF append helper
 */
export function TxSender() {
  const { t } = useTranslation()
  const { portId } = useConnectionStore()
  const { addTxPacket } = useDataStore()
  const { protocols, activeProtocol } = useProtocolStore()
  const [inputMode, setInputMode] = useState<'hex' | 'ascii'>('hex')
  const [inputData, setInputData] = useState('')
  const [isSending, setIsSending] = useState(false)
  const [useProtocolEncode, setUseProtocolEncode] = useState(false)
  const [hexValid, setHexValid] = useState(true)

  // History state
  const [showHistory, setShowHistory] = useState(false)
  const [historyIndex, setHistoryIndex] = useState(-1)
  const [inputBeforeHistory, setInputBeforeHistory] = useState('')
  const [historyEntries, setHistoryEntries] = useState<HistoryEntry[]>([])
  const historyRef = useRef(historyStorage())
  const inputRef = useRef<HTMLTextAreaElement>(null)
  const handleSendRef = useRef<() => Promise<void>>()

  // Load history on mount
  useEffect(() => {
    setHistoryEntries(historyRef.current.get())
  }, [])

  // Real-time hex validation
  useEffect(() => {
    if (inputMode === 'hex' && inputData.trim()) {
      const hex = inputData.replace(/\s/g, '')
      setHexValid(/^[0-9A-Fa-f]*$/.test(hex) && hex.length % 2 === 0)
    } else {
      setHexValid(true)
    }
  }, [inputMode, inputData])

  // Keyboard navigation for history (up/down arrows in textarea)
  const handleKeyDown = useCallback((e: React.KeyboardEvent<HTMLTextAreaElement>) => {
    // Ctrl+Enter to send
    if (e.key === 'Enter' && (e.ctrlKey || e.metaKey)) {
      e.preventDefault()
      handleSendRef.current?.()
      return
    }

    if (e.key === 'ArrowUp') {
      e.preventDefault()
      if (historyEntries.length === 0) return

      if (historyIndex === -1) {
        setInputBeforeHistory(inputData)
      }

      const nextIndex = Math.min(historyIndex + 1, historyEntries.length - 1)
      setHistoryIndex(nextIndex)
      setInputData(historyEntries[nextIndex].content)
      setInputMode(historyEntries[nextIndex].mode)
    } else if (e.key === 'ArrowDown') {
      e.preventDefault()
      if (historyIndex === -1) return

      const nextIndex = historyIndex - 1
      if (nextIndex === -1) {
        setInputData(inputBeforeHistory)
      } else {
        setInputData(historyEntries[nextIndex].content)
        setInputMode(historyEntries[nextIndex].mode)
      }
      setHistoryIndex(nextIndex)
    } else if (e.key === 'Escape' && showHistory) {
      setShowHistory(false)
    }
  }, [historyEntries, historyIndex, inputBeforeHistory, inputData, showHistory])

  const addToHistory = useCallback((content: string, mode: 'hex' | 'ascii') => {
    historyRef.current.add({ content, mode })
    setHistoryEntries(historyRef.current.get())
  }, [])

  // Register send function ref for keyboard handler
  useEffect(() => {
    handleSendRef.current = handleSend
  }) // eslint-disable-line react-hooks/exhaustive-deps -- always use latest

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
        if (!/^[0-9A-Fa-f]*$/.test(hex) || hex.length % 2 !== 0) {
          throw new Error(t('toast.invalidHex'))
        }
        for (let i = 0; i < hex.length; i += 2) {
          data.push(parseInt(hex.substring(i, 2), 16))
        }
      } else {
        data = Array.from(new TextEncoder().encode(inputData))
      }

      // Apply protocol encoding if enabled
      if (useProtocolEncode && activeProtocol) {
        try {
          const encodedData = await invoke<number[]>('protocol_encode', {
            protocol: activeProtocol,
            data,
          })
          data = encodedData
        } catch (encodeError) {
          toast.error(`${t('toast.encodingFailed')}: ${encodeError instanceof Error ? encodeError.message : t('common.unknownError')}`)
          return
        }
      }

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

      // Save to history
      addToHistory(inputData, inputMode)

      toast.success(t('toast.sendSuccess', { bytes: bytesWritten }))
      setInputData('')
      setHistoryIndex(-1)
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('toast.sendFailed'))
    } finally {
      setIsSending(false)
    }
  }

  const handleHistorySelect = useCallback((entry: HistoryEntry) => {
    setInputData(entry.content)
    setInputMode(entry.mode)
    setShowHistory(false)
    setHistoryIndex(-1)
    inputRef.current?.focus()
  }, [])

  const handleClearHistory = useCallback(() => {
    historyRef.current.clear()
    setHistoryEntries([])
    setHistoryIndex(-1)
    toast.success(t('terminal.historyCleared'))
  }, [t])

  const handleLoadFile = useCallback(async () => {
    try {
      const selected = await open({
        title: t('terminal.loadFileTitle'),
        multiple: false,
      })

      if (!selected) return

      const file = await readFile(selected as string)
      if (file.length > MAX_FILE_SIZE) {
        toast.error(t('toast.fileTooLarge'))
        return
      }

      const text = new TextDecoder().decode(file)
      setInputData(text)
      setInputMode(inputMode === 'hex' ? 'ascii' : inputMode)
      toast.success(t('terminal.fileLoaded', { file: selected }))
    } catch (error) {
      if (error instanceof Error && error.message.includes('not found')) return
      toast.error(t('toast.loadFileFailed'))
    }
  }, [inputMode, t])

  return (
    <div className="h-full flex flex-col bg-bg-deep">
      {/* Toolbar */}
      <div className="flex items-center justify-between px-4 py-2 border-b border-border">
        <div className="flex items-center gap-2">
          <span className="text-sm font-medium text-text-primary">{t('terminal.txData')}</span>

          {/* Mode toggle */}
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

          {/* Protocol encoding toggle */}
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
          {/* Send history */}
          <div className="relative">
            <Button
              variant="ghost"
              size="sm"
              onClick={() => setShowHistory(!showHistory)}
              className={cn(showHistory && 'text-signal')}
              title={t('terminal.sendHistory')}
            >
              <Clock className="w-4 h-4" />
            </Button>

            {/* History dropdown */}
            {showHistory && (
              <div className="absolute right-0 bottom-full mb-2 w-72 max-h-[60vh] bg-bg-elevated border border-border rounded-lg shadow-xl overflow-hidden z-50 flex flex-col">
                <div className="flex items-center justify-between px-3 py-2 border-b border-border">
                  <span className="text-xs font-medium text-text-primary">{t('terminal.sendHistory')}</span>
                  <div className="flex items-center gap-1">
                    {historyEntries.length > 0 && (
                      <button
                        onClick={handleClearHistory}
                        className="p-0.5 rounded hover:bg-bg-base text-text-tertiary hover:text-alert transition-colors"
                        title={t('terminal.clearHistory')}
                      >
                        <X className="w-3 h-3" />
                      </button>
                    )}
                    <button
                      onClick={() => setShowHistory(false)}
                      className="p-0.5 rounded hover:bg-bg-base text-text-tertiary hover:text-text-primary transition-colors"
                    >
                      <X className="w-3 h-3" />
                    </button>
                  </div>
                </div>

                <div className="overflow-y-auto flex-1 min-h-0">
                  {historyEntries.length === 0 ? (
                    <div className="px-3 py-4 text-center text-xs text-text-tertiary">
                      {t('terminal.noHistory')}
                    </div>
                  ) : (
                    historyEntries.map((entry, idx) => (
                      <button
                        key={entry.timestamp}
                        onClick={() => handleHistorySelect(entry)}
                        className="w-full text-left px-3 py-2 hover:bg-bg-base border-b border-border/50 last:border-b-0 transition-colors"
                      >
                        <div className="flex items-center gap-2">
                          <span className={cn(
                            'px-1 py-0.5 rounded text-[10px] font-bold',
                            entry.mode === 'hex' ? 'bg-amber/20 text-amber' : 'bg-info/20 text-info'
                          )}>
                            {entry.mode === 'hex' ? 'HEX' : 'ASC'}
                          </span>
                          <span className="flex-1 text-xs font-mono text-text-secondary truncate">
                            {entry.content}
                          </span>
                        </div>
                        <div className="text-[10px] text-text-tertiary mt-0.5">
                          {new Date(entry.timestamp).toLocaleTimeString()}
                        </div>
                      </button>
                    ))
                  )}
                </div>

                {historyEntries.length > 0 && (
                  <div className="px-3 py-1.5 text-[10px] text-text-tertiary border-t border-border bg-bg-deepest">
                    {t('terminal.historyHint')}
                  </div>
                )}
              </div>
            )}
          </div>

          {/* Load from file */}
          <Button
            variant="ghost"
            size="sm"
            onClick={handleLoadFile}
            title={t('terminal.loadFile')}
          >
            <FileText className="w-4 h-4" />
          </Button>
        </div>
      </div>

      {/* Input area */}
      <div className="flex-1 p-4 space-y-3">
        {/* CRLF helper */}
        <div className="flex items-center gap-2 text-xs text-text-tertiary">
          <button
            onClick={() => setInputData((prev) => prev + '\r\n')}
            className="px-2 py-0.5 text-[10px] bg-bg-base border border-border rounded hover:bg-bg-elevated transition-colors"
          >
            + CRLF
          </button>
        </div>

        {/* Input textarea */}
        <div className="flex-1 relative">
          <textarea
            ref={inputRef}
            value={inputData}
            onChange={(e) => {
              setInputData(e.target.value)
              if (historyIndex !== -1) {
                setHistoryIndex(-1)
                setInputBeforeHistory('')
              }
            }}
            onKeyDown={handleKeyDown}
            placeholder={inputMode === 'hex' ? t('terminal.hexInput') : t('terminal.asciiInput')}
            className="w-full h-full min-h-0 bg-bg-base border border-border rounded-lg p-3 text-sm font-mono resize-none focus:outline-none focus:border-signal/50 transition-colors"
            disabled={isSending}
          />
          <div className="absolute bottom-2 right-2 flex items-center gap-1 text-xs text-text-tertiary">
            {inputMode === 'hex' && inputData.trim() && (
              <span className={hexValid ? 'text-signal' : 'text-alert'}>
                {hexValid ? '✓' : '✗ Invalid HEX'}
              </span>
            )}
            <span>{inputData.length} {t('terminal.characterCount')}</span>
          </div>
        </div>

        {/* Send button */}
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
