import { Panel } from '@/components/ui/panel'
import { cn } from '@/lib/utils'
import { Play, FilePlus, Save, FolderOpen, Trash2, Download, Upload, Loader2, StopCircle, AlertCircle, CheckCircle } from 'lucide-react'
import { useState, useRef, useEffect, useMemo, useCallback } from 'react'
import Editor from '@monaco-editor/react'
import { invoke } from '@tauri-apps/api/core'
import { scriptsStorage } from '@/lib/storage'
import { parseError } from '@/lib/errors'
import { useScriptActions } from '@/contexts/ScriptActionContext'
import { useTranslation } from 'react-i18next'

const DEFAULT_SCRIPT = `-- Lua Script for Serial CLI
-- Use the serial API to communicate with devices

function init()
  print("Initializing script...")
  -- Open serial port
  -- serial.open("/dev/ttyUSB0", 9600, 8, 'N', 1)
end

function main()
  print("Running main loop...")

  -- Send data
  -- serial.write("Hello, World!")

  -- Read data
  -- local data = serial.read()
  -- print("Received: " .. data)
end

function cleanup()
  print("Cleaning up...")
  -- serial.close()
end

-- Entry point
init()
main()
cleanup()
`

interface ScriptFile {
  id: string
  name: string
  content: string
  lastModified: number
}

export function ScriptPanel() {
  const { t } = useTranslation()
  const [scripts, setScripts] = useState<ScriptFile[]>([])
  const [activeScriptId, setActiveScriptId] = useState<string | null>(null)
  const [scriptContent, setScriptContent] = useState(DEFAULT_SCRIPT)
  const [isRunning, setIsRunning] = useState(false)
  const [isValidating, setIsValidating] = useState(false)
  const [output, setOutput] = useState<string[]>([])
  const [selectedFile, setSelectedFile] = useState<File | null>(null)
  const [error, setError] = useState<string | null>(null)
  const fileInputRef = useRef<HTMLInputElement>(null)
  const outputRef = useRef<HTMLDivElement>(null)
  const { registerCallbacks } = useScriptActions()

  const activeScript = scripts.find(s => s.id === activeScriptId)

  // Limit output lines to prevent memory issues
  const MAX_OUTPUT_LINES = 1000

  // Auto-scroll output to bottom
  useEffect(() => {
    if (outputRef.current) {
      outputRef.current.scrollTop = outputRef.current.scrollHeight
    }
  }, [output])

  // Auto-save script content (debounced)
  useEffect(() => {
    if (!activeScriptId) return
    const timer = setTimeout(() => {
      const updatedScripts = scripts.map(s =>
        s.id === activeScriptId
          ? { ...s, content: scriptContent, lastModified: Date.now() }
          : s
      )
      setScripts(updatedScripts)
      scriptsStorage.set(updatedScripts)
    }, 1000)
    return () => clearTimeout(timer)
  }, [scriptContent, activeScriptId, scripts])

  // Load scripts from storage on mount
  useEffect(() => {
    const savedScripts = scriptsStorage.get()
    if (savedScripts.length > 0) {
      setScripts(savedScripts)
    }
  }, [])

  const createNewScript = useCallback(() => {
    const newScript: ScriptFile = {
      id: Date.now().toString(),
      name: `untitled-${scripts.length + 1}.lua`,
      content: DEFAULT_SCRIPT,
      lastModified: Date.now(),
    }
    const updatedScripts = [...scripts, newScript]
    setScripts(updatedScripts)
    scriptsStorage.set(updatedScripts)
    setActiveScriptId(newScript.id)
    setScriptContent(newScript.content)
    setError(null) // Clear any previous errors
  }, [scripts])

  const runCurrentScript = async () => {
    if (!scriptContent.trim()) {
      setError(t('toast.emptyScript'))
      return
    }
    await runScript()
  }

  // Register callbacks for global shortcuts
  const scriptContentRef = useRef(scriptContent)
  useEffect(() => {
    scriptContentRef.current = scriptContent
  }, [scriptContent])

  useEffect(() => {
    const unregister = registerCallbacks({
      createNewScript,
      runCurrentScript: () => {
        if (scriptContentRef.current.trim()) runScript()
      },
      validateCurrentScript: () => {
        if (scriptContentRef.current.trim()) validateScript()
      },
    })
    return unregister
  }, [registerCallbacks, createNewScript])

  const runScript = async () => {
    if (isRunning) return
    setIsRunning(true)
    setError(null)
    setOutput(prev => {
      const newOutput = [...prev, `[${new Date().toLocaleTimeString()}] Starting script execution...`]
      // Limit output lines
      return newOutput.length > MAX_OUTPUT_LINES ? newOutput.slice(-MAX_OUTPUT_LINES) : newOutput
    })

    try {
      // Capture console output
      const originalLog = console.log
      const originalError = console.error
      const logs: string[] = []

      console.log = (...args) => {
        logs.push(args.join(' '))
        originalLog.apply(console, args)
      }

      console.error = (...args) => {
        logs.push(`ERROR: ${args.join(' ')}`)
        originalError.apply(console, args)
      }

      try {
        const result = await invoke<string>('execute_script', { script: scriptContent })
        setOutput(prev => {
          const newOutput = [...prev, `[${new Date().toLocaleTimeString()}] ✓ ${result}`]
          return newOutput.length > MAX_OUTPUT_LINES ? newOutput.slice(-MAX_OUTPUT_LINES) : newOutput
        })

        // Add any captured logs
        if (logs.length > 0) {
          setOutput(prev => {
            const newOutput = [...prev, ...logs.map(log => `[${new Date().toLocaleTimeString()}] ${log}`)]
            return newOutput.length > MAX_OUTPUT_LINES ? newOutput.slice(-MAX_OUTPUT_LINES) : newOutput
          })
        }
      } finally {
        // Restore console functions
        console.log = originalLog
        console.error = originalError
      }
    } catch (err) {
      const appError = parseError(err)
      setError(appError.message)
      setOutput(prev => {
        const newOutput = [...prev, `[${new Date().toLocaleTimeString()}] ✗ Script execution failed`, `[${new Date().toLocaleTimeString()}] Error: ${appError.message}`]
        return newOutput.length > MAX_OUTPUT_LINES ? newOutput.slice(-MAX_OUTPUT_LINES) : newOutput
      })
    } finally {
      setIsRunning(false)
    }
  }

  const validateCurrentScript = async () => {
    if (!scriptContent.trim()) {
      setError(t('toast.emptyScript'))
      return
    }
    await validateScript()
  }

  const validateScript = async () => {
    if (isValidating) return
    setIsValidating(true)
    setError(null)
    setOutput(prev => {
      const newOutput = [...prev, `[${new Date().toLocaleTimeString()}] Starting script validation...`]
      return newOutput.length > MAX_OUTPUT_LINES ? newOutput.slice(-MAX_OUTPUT_LINES) : newOutput
    })

    try {
      const result = await invoke<string>('validate_script', { script: scriptContent })
      setOutput(prev => {
        const newOutput = [...prev, `[${new Date().toLocaleTimeString()}] ✓ ${result}`]
        return newOutput.length > MAX_OUTPUT_LINES ? newOutput.slice(-MAX_OUTPUT_LINES) : newOutput
      })
    } catch (err) {
      const appError = parseError(err)
      setError(appError.message)
      setOutput(prev => {
        const newOutput = [...prev, `[${new Date().toLocaleTimeString()}] ✗ Script validation failed`, `[${new Date().toLocaleTimeString()}] Error: ${appError.message}`]
        return newOutput.length > MAX_OUTPUT_LINES ? newOutput.slice(-MAX_OUTPUT_LINES) : newOutput
      })
    } finally {
      setIsValidating(false)
    }
  }

  const deleteScript = (id: string) => {
    const script = scripts.find(s => s.id === id)
    if (script && !window.confirm(`Delete script "${script.name}"?`)) return
    const updatedScripts = scripts.filter(s => s.id !== id)
    setScripts(updatedScripts)
    scriptsStorage.set(updatedScripts)

    if (activeScriptId === id) {
      setActiveScriptId(null)
      setScriptContent(DEFAULT_SCRIPT)
    }
  }

  const saveScript = () => {
    if (activeScriptId) {
      const updatedScripts = scripts.map(s =>
        s.id === activeScriptId
          ? { ...s, content: scriptContent, lastModified: Date.now() }
          : s
      )
      setScripts(updatedScripts)
      scriptsStorage.set(updatedScripts)
      setOutput(prev => [...prev, `[${new Date().toLocaleTimeString()}] Script saved: ${activeScript?.name}`])
    }
  }

  const loadScriptFile = (file: File) => {
    const reader = new FileReader()
    reader.onload = (e) => {
      const content = e.target?.result as string
      const newScript: ScriptFile = {
        id: Date.now().toString(),
        name: file.name,
        content,
        lastModified: Date.now(),
      }
      const updatedScripts = [...scripts, newScript]
      setScripts(updatedScripts)
      scriptsStorage.set(updatedScripts)
      setActiveScriptId(newScript.id)
      setScriptContent(content)
      setOutput(prev => [...prev, `[${new Date().toLocaleTimeString()}] Loaded: ${file.name}`])
    }
    reader.readAsText(file)
  }

  const handleFileSelect = (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0]
    if (file) {
      loadScriptFile(file)
    }
  }

  const exportScript = () => {
    if (activeScript) {
      const blob = new Blob([activeScript.content], { type: 'text/plain' })
      const url = URL.createObjectURL(blob)
      const a = document.createElement('a')
      a.href = url
      a.download = activeScript.name
      a.click()
      URL.revokeObjectURL(url)
      setOutput(prev => [...prev, `[${new Date().toLocaleTimeString()}] Exported: ${activeScript.name}`])
    }
  }

  useEffect(() => {
    if (activeScript) {
      setScriptContent(activeScript.content)
    }
  }, [activeScriptId])

  return (
    <div className="space-y-6">
      {/* Scripts List & Editor */}
      <div className="grid grid-cols-1 md:grid-cols-4 gap-6 w-full">
        {/* Sidebar - Script Files */}
        <Panel title={t('scripts.title')} variant="amber" className="col-span-1">
          <div className="space-y-2">
            {/* Action buttons */}
            <div className="flex items-center gap-2 mb-4">
              <button
                onClick={createNewScript}
                className="flex-1 flex items-center justify-center gap-1.5 px-3 py-1.5 text-xs rounded-md bg-amber/10 text-amber border border-amber/30 hover:bg-amber/20 transition-colors"
              >
                <FilePlus size={14} strokeWidth={1.5} />
                {t('scripts.newScript')}
              </button>
              <button
                onClick={() => fileInputRef.current?.click()}
                className="p-1.5 rounded-md hover:bg-bg-elevated text-text-tertiary hover:text-text-primary transition-colors"
                title={t('scripts.loadFile')}
              >
                <FolderOpen size={14} strokeWidth={1.5} />
              </button>
              <input
                ref={fileInputRef}
                type="file"
                accept=".lua"
                className="hidden"
                onChange={handleFileSelect}
              />
            </div>

            {/* Script list */}
            <div className="space-y-1">
              {scripts.length === 0 ? (
                <div className="py-8 text-center text-xs text-text-tertiary">
                  <p>{t('scripts.noScripts')}</p>
                  <p className="mt-1">{t('scripts.noScriptsHint')}</p>
                </div>
              ) : (
                scripts.map(script => (
                  <div
                    key={script.id}
                    className={cn(
                      'group flex items-center justify-between px-3 py-2 rounded-md text-xs cursor-pointer transition-colors',
                      activeScriptId === script.id
                        ? 'bg-amber/10 text-amber border border-amber/30'
                        : 'hover:bg-bg-elevated text-text-secondary'
                    )}
                    onClick={() => {
                      setActiveScriptId(script.id)
                      setScriptContent(script.content)
                    }}
                  >
                    <div className="flex items-center gap-2 min-w-0">
                      <FilePlus size={14} strokeWidth={1.5} className="flex-shrink-0" />
                      <span className="truncate">{script.name}</span>
                    </div>
                    <button
                      onClick={(e) => {
                        e.stopPropagation()
                        deleteScript(script.id)
                      }}
                      className="opacity-0 group-hover:opacity-100 p-1 rounded hover:bg-alert/20 text-text-tertiary hover:text-alert transition-all"
                    >
                      <Trash2 size={12} strokeWidth={1.5} />
                    </button>
                  </div>
                ))
              )}
            </div>
          </div>
        </Panel>

        {/* Editor */}
        <Panel
          title={activeScript?.name || t('scripts.editor')}
          variant="default"
          className="col-span-3"
          actions={
            <>
              <button
                onClick={saveScript}
                disabled={!activeScriptId}
                className="p-1.5 rounded hover:bg-bg-elevated text-text-tertiary hover:text-text-primary transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
                title={t('common.save')}
              >
                <Save size={14} strokeWidth={1.5} />
              </button>
              <button
                onClick={exportScript}
                disabled={!activeScript}
                className="p-1.5 rounded hover:bg-bg-elevated text-text-tertiary hover:text-text-primary transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
                title={t('common.export')}
              >
                <Download size={14} strokeWidth={1.5} />
              </button>
              <button
                onClick={validateScript}
                disabled={isValidating}
                className="flex items-center gap-1.5 px-3 py-1.5 text-xs rounded-md bg-amber/10 text-amber border border-amber/30 hover:bg-amber/20 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
                title="Validate script syntax"
              >
                {isValidating ? (
                  <Loader2 size={12} strokeWidth={1.5} className="animate-spin" />
                ) : (
                  <CheckCircle size={12} strokeWidth={1.5} />
                )}
                {isValidating ? 'Validating...' : 'Validate'}
              </button>
              <button
                onClick={runScript}
                disabled={isRunning}
                className="flex items-center gap-1.5 px-3 py-1.5 text-xs rounded-md bg-signal/10 text-signal border border-signal/30 hover:bg-signal/20 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
              >
                {isRunning ? (
                  <Loader2 size={12} strokeWidth={1.5} className="animate-spin" />
                ) : (
                  <Play size={12} strokeWidth={1.5} />
                )}
                {isRunning ? t('scripts.running') : t('scripts.run')}
              </button>
            </>
          }
        >
          <div className="rounded-md overflow-hidden border border-border/50" style={{ height: '500px', minHeight: '300px' }}>
            <Editor
              height="100%"
              defaultLanguage="lua"
              theme="vs-dark"
              value={scriptContent}
              onChange={(value) => setScriptContent(value || '')}
              options={{
                minimap: { enabled: false },
                fontSize: 13,
                lineNumbers: 'on',
                scrollBeyondLastLine: false,
                automaticLayout: true,
                padding: { top: 12, bottom: 12 },
              }}
            />
          </div>
        </Panel>
      </div>

      {/* Output Console */}
      <Panel title={t('scripts.output')} variant="default" className="w-full">
        {error && (
          <div className="mb-3 p-3 rounded-md bg-alert/10 border border-alert/30">
            <div className="flex items-start gap-2">
              <AlertCircle size={16} strokeWidth={1.5} className="mt-0.5 flex-shrink-0 text-alert" />
              <div className="flex-1">
                <p className="text-sm text-alert font-medium">{t('scripts.executionError')}</p>
                <p className="text-xs text-alert mt-1 font-mono">{error}</p>
              </div>
            </div>
          </div>
        )}
        <div ref={outputRef} className="h-32 overflow-y-auto font-mono text-xs bg-bg-deepest rounded-md p-3 border border-border/50">
          {output.length === 0 ? (
            <p className="text-text-tertiary">{t('scripts.outputPlaceholder')}</p>
          ) : (
            output.map((line, i) => (
              <div
                key={i}
                className={cn(
                  'py-0.5',
                  line.includes('✓') && 'text-signal',
                  line.includes('✗') && 'text-alert',
                  line.includes('Error:') && 'text-alert'
                )}
              >
                {line}
              </div>
            ))
          )}
        </div>
      </Panel>
    </div>
  )
}
