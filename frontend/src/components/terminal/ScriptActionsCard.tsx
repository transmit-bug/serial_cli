import { useState, useEffect } from 'react'
import { Card } from '@/components/ui/card'
import { Code2, Loader2 } from 'lucide-react'
import { invoke } from '@tauri-apps/api/core'
import { useTranslation } from 'react-i18next'

interface ScriptAction {
  function_name: string
  label: string
  icon?: string
  group?: string
  confirm: boolean
}

interface ScriptActionsCardProps {
  portId: string
}

export function ScriptActionsCard({ portId }: ScriptActionsCardProps) {
  const { t } = useTranslation()
  const [actions, setActions] = useState<ScriptAction[]>([])
  const [loading, setLoading] = useState(true)
  const [executing, setExecuting] = useState<Set<string>>(new Set())
  const [error, setError] = useState<string | null>(null)

  // Load actions when portId changes
  useEffect(() => {
    loadActions()
  }, [portId])

  const loadActions = async () => {
    setLoading(true)
    setError(null)

    try {
      const result = await invoke<ScriptAction[]>('list_script_actions', { portId })
      setActions(result)
    } catch (err) {
      console.error('Failed to load script actions:', err)
      // Don't show error for "no script attached" - it's expected
      const errorMsg = err instanceof Error ? err.message : String(err)
      if (!errorMsg.includes('No script attached')) {
        setError(errorMsg)
      }
      setActions([])
    } finally {
      setLoading(false)
    }
  }

  const executeAction = async (action: ScriptAction) => {
    // Show confirmation dialog for dangerous actions
    if (action.confirm) {
      const confirmed = window.confirm(
        `${t('terminal.confirmAction')}: ${action.label}?\n\n${t('terminal.thisActionCannotBeUndone')}`
      )
      if (!confirmed) return
    }

    setExecuting(prev => new Set([...prev, action.function_name]))

    try {
      const result = await invoke<string>('call_script_function', {
        portId,
        functionName: action.function_name,
      })
      console.log(`Action ${action.function_name} result:`, result)
    } catch (err) {
      console.error('Failed to execute action:', err)
      const errorMsg = err instanceof Error ? err.message : String(err)
      alert(`${t('terminal.actionFailed')}: ${errorMsg}`)
    } finally {
      setExecuting(prev => {
        const next = new Set(prev)
        next.delete(action.function_name)
        return next
      })
    }
  }

  // Group actions by group property
  const groupedActions = actions.reduce((acc, action) => {
    const group = action.group || t('terminal.ungrouped')
    if (!acc[group]) {
      acc[group] = []
    }
    acc[group].push(action)
    return acc
  }, {} as Record<string, ScriptAction[]>)

  if (loading) {
    return (
      <Card className="p-4 border-border/50">
        <div className="flex items-center gap-2 mb-3">
          <Code2 className="w-4 h-4 text-amber" />
          <h3 className="text-sm font-medium text-text-primary">{t('terminal.scriptActions')}</h3>
        </div>
        <div className="flex items-center justify-center py-4">
          <Loader2 className="w-5 h-5 text-text-tertiary animate-spin" />
        </div>
      </Card>
    )
  }

  if (error) {
    return (
      <Card className="p-4 border-border/50">
        <div className="flex items-center gap-2 mb-3">
          <Code2 className="w-4 h-4 text-amber" />
          <h3 className="text-sm font-medium text-text-primary">{t('terminal.scriptActions')}</h3>
        </div>
        <div className="text-xs text-alert text-center py-2">
          {error}
        </div>
      </Card>
    )
  }

  if (actions.length === 0) {
    return (
      <Card className="p-4 border-border/50">
        <div className="flex items-center gap-2 mb-3">
          <Code2 className="w-4 h-4 text-amber" />
          <h3 className="text-sm font-medium text-text-primary">{t('terminal.scriptActions')}</h3>
        </div>
        <div className="text-xs text-text-tertiary text-center py-2">
          {t('terminal.noScriptActions')}
        </div>
      </Card>
    )
  }

  return (
    <Card className="p-4 border-border/50">
      <div className="flex items-center justify-between mb-3">
        <div className="flex items-center gap-2">
          <Code2 className="w-4 h-4 text-amber" />
          <h3 className="text-sm font-medium text-text-primary">{t('terminal.scriptActions')}</h3>
          <span className="text-xs text-text-tertiary bg-bg-base px-2 py-0.5 rounded">
            {actions.length}
          </span>
        </div>
      </div>

      {/* Render actions by group */}
      {Object.entries(groupedActions).map(([group, groupActions]) => (
        <div key={group} className="mb-4 last:mb-0">
          {group !== t('terminal.ungrouped') && (
            <div className="text-xs text-text-tertiary mb-2 px-1">{group}</div>
          )}

          {/* Flow layout for buttons */}
          <div className="flex flex-wrap gap-2">
            {groupActions.map(action => (
              <button
                key={action.function_name}
                onClick={() => executeAction(action)}
                disabled={executing.has(action.function_name)}
                className={`
                  flex items-center gap-1.5 px-3 py-1.5 rounded text-xs font-medium transition-all
                  ${action.confirm
                    ? 'bg-alert/10 text-alert border border-alert/30 hover:bg-alert/20'
                    : 'bg-amber/10 text-amber border border-amber/30 hover:bg-amber/20'
                  }
                  disabled:opacity-50 disabled:cursor-not-allowed
                `}
              >
                {executing.has(action.function_name) ? (
                  <Loader2 size={12} strokeWidth={2} className="animate-spin" />
                ) : action.icon ? (
                  <span>{action.icon}</span>
                ) : null}
                <span>{action.label}</span>
              </button>
            ))}
          </div>
        </div>
      ))}
    </Card>
  )
}
