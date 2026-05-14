import { useState, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { useConnectionStore, useSerialScriptStore } from '@/stores'
import { Button } from '@/components/ui/button'
import { Code2, X, Loader2, FileCode } from 'lucide-react'
import { toast } from 'sonner'
import { useTranslation } from 'react-i18next'

/**
 * ScriptBadge - Shows script engine status for the connected port.
 * Allows attaching/detaching Lua scripts that intercept serial I/O.
 */
export function ScriptBadge() {
  const { t } = useTranslation()
  const { portId } = useConnectionStore()
  const { scriptStatus, loading, checkScriptStatus, attachScript, detachScript } = useSerialScriptStore()
  const [showAttach, setShowAttach] = useState(false)
  const [scriptSource, setScriptSource] = useState('')
  const [attaching, setAttaching] = useState(false)

  // Refresh status when port changes
  useEffect(() => {
    if (portId) checkScriptStatus(portId)
  }, [portId])

  if (!portId) return null

  const handleAttach = async () => {
    if (!scriptSource.trim() || !portId) return
    setAttaching(true)
    try {
      await attachScript(portId, scriptSource)
      toast.success('脚本已附加')
      setShowAttach(false)
      setScriptSource('')
    } catch (error) {
      toast.error(`附加脚本失败: ${error instanceof Error ? error.message : '未知错误'}`)
    } finally {
      setAttaching(false)
    }
  }

  const handleDetach = async () => {
    if (!portId) return
    try {
      await detachScript(portId)
      toast.success('脚本已分离')
    } catch (error) {
      toast.error(`分离脚本失败: ${error instanceof Error ? error.message : '未知错误'}`)
    }
  }

  const handleLoadFromFile = async () => {
    const { open } = await import('@tauri-apps/plugin-dialog')
    const { readFile } = await import('@tauri-apps/plugin-fs')
    try {
      const selected = await open({
        title: '加载 Lua 脚本',
        multiple: false,
        filters: [{ name: 'Lua', extensions: ['lua'] }],
      })
      if (!selected) return
      const content = await readFile(selected as string)
      setScriptSource(new TextDecoder().decode(content))
    } catch (error) {
      if (error instanceof Error && !error.message.includes('not found')) {
        toast.error('加载文件失败')
      }
    }
  }

  // No script attached
  if (!scriptStatus?.has_script) {
    return (
      <>
        <Button
          variant="ghost"
          size="sm"
          disabled={loading}
          onClick={() => setShowAttach(!showAttach)}
          className="text-text-secondary hover:text-text-primary"
          title="附加脚本到串口"
        >
          <Code2 className="w-4 h-4" />
        </Button>

        {showAttach && (
          <div className="absolute right-0 top-full mt-1 w-80 bg-bg-elevated border border-border rounded-lg shadow-xl z-50 overflow-hidden">
            <div className="px-3 py-2 border-b border-border flex items-center justify-between">
              <div className="flex items-center gap-2">
                <FileCode className="w-4 h-4 text-amber" />
                <span className="text-xs font-medium text-text-primary">附加 Lua 脚本</span>
              </div>
              <button
                onClick={() => setShowAttach(false)}
                className="p-0.5 rounded hover:bg-bg-base text-text-tertiary hover:text-text-primary"
              >
                <X className="w-3 h-3" />
              </button>
            </div>

            <div className="p-3 space-y-2">
              <textarea
                value={scriptSource}
                onChange={(e) => setScriptSource(e.target.value)}
                placeholder={'function on_recv(data)\n  return data\nend\n\nfunction on_send(data)\n  return data\nend'}
                className="w-full h-48 bg-bg-deep border border-border rounded p-2 text-xs font-mono resize-none focus:outline-none focus:border-amber/50"
                spellCheck={false}
              />

              <div className="flex gap-2">
                <Button
                  variant="outline"
                  size="sm"
                  onClick={handleLoadFromFile}
                  className="flex-1"
                >
                  从文件加载
                </Button>
                <Button
                  variant="amber"
                  size="sm"
                  onClick={handleAttach}
                  disabled={attaching || !scriptSource.trim()}
                  className="flex-1"
                >
                  {attaching ? (
                    <Loader2 className="w-3 h-3 mr-1 animate-spin" />
                  ) : (
                    <Code2 className="w-3 h-3 mr-1" />
                  )}
                  {attaching ? '附加中...' : '附加'}
                </Button>
              </div>
            </div>
          </div>
        )}
      </>
    )
  }

  // Script is attached
  return (
    <div className="flex items-center gap-1.5 px-2 py-1 rounded bg-amber/10 border border-amber/30">
      <Code2 className="w-3.5 h-3.5 text-amber" />
      <span className="text-xs text-amber font-medium">脚本</span>
      {scriptStatus.timer_interval_ms > 0 && (
        <span className="text-[10px] text-amber/70">
          {scriptStatus.timer_interval_ms}ms
        </span>
      )}
      <button
        onClick={handleDetach}
        disabled={loading}
        className="p-0.5 rounded hover:bg-amber/20 text-amber/70 hover:text-amber transition-colors"
        title="分离脚本"
      >
        <X className="w-3 h-3" />
      </button>
    </div>
  )
}
