import { shortcuts, formatKey } from '@/lib/shortcuts'
import { useShortcuts } from '@/contexts/ShortcutContext'
import { cn } from '@/lib/utils'
import { useTranslation } from 'react-i18next'

// Map shortcut descriptions to translation keys
const DESCRIPTION_KEYS: Record<string, string> = {
  'Open command palette': 'shortcuts.cmdPalette',
  'Terminal View': 'shortcuts.terminalView',
  'Virtual Ports View': 'shortcuts.virtualPortsView',
  'Scripts View': 'shortcuts.scriptsView',
  'Protocols View': 'shortcuts.protocolsView',
  'Settings View': 'shortcuts.settingsView',
  'Refresh ports': 'shortcuts.refreshPorts',
  'New script': 'shortcuts.newScript',
  'Run script': 'shortcuts.runScript',
  'Clear data': 'shortcuts.clearData',
  'Open settings': 'shortcuts.openSettings',
  'Show keyboard shortcuts': 'shortcuts.showShortcuts',
  'Close modal/dialog': 'shortcuts.closeModal',
}

const CATEGORY_KEYS: Record<string, string> = {
  navigation: 'shortcuts.catNavigation',
  terminal: 'shortcuts.catTerminal',
  scripts: 'shortcuts.catScripts',
  data: 'shortcuts.catData',
  general: 'shortcuts.catGeneral',
}

export function KeyboardShortcutsHelp() {
  const { isShortcutsHelpOpen, closeShortcutsHelp } = useShortcuts()
  const { t } = useTranslation()

  if (!isShortcutsHelpOpen) return null

  // Group shortcuts by category
  const categories = Array.from(new Set(shortcuts.map(s => s.category)))

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center">
      {/* Backdrop */}
      <div
        className="absolute inset-0 bg-black/50 backdrop-blur-sm animate-fade-in"
        onClick={closeShortcutsHelp}
      />

      {/* Modal */}
      <div className="relative w-full max-w-3xl mx-4 bg-bg-floating border border-border rounded-lg shadow-xl overflow-hidden animate-slide-up">
        {/* Header */}
        <div className="flex items-center justify-between px-6 py-4 border-b border-border">
          <div className="flex items-center gap-3">
            <span className="text-2xl">⌨️</span>
            <div>
              <h2 className="text-lg font-semibold text-text-primary">{t('shortcuts.title')}</h2>
              <p className="text-sm text-text-tertiary">{t('shortcuts.subtitle')}</p>
            </div>
          </div>
          <button
            onClick={closeShortcutsHelp}
            className="w-8 h-8 flex items-center justify-center rounded hover:bg-bg-elevated text-text-secondary hover:text-text-primary transition-colors"
          >
            ×
          </button>
        </div>

        {/* Content */}
        <div className="p-6 overflow-y-auto" style={{ maxHeight: 'calc(100vh - 200px)' }}>
          <div className="grid gap-6">
            {categories.map(category => {
              const categoryShortcuts = shortcuts.filter(s => s.category === category)
              return (
                <div key={category}>
                  <h3 className="text-sm font-semibold text-text-tertiary uppercase tracking-wide mb-3">
                    {t(CATEGORY_KEYS[category] || category)}
                  </h3>
                  <div className="space-y-2">
                    {categoryShortcuts.map(shortcut => (
                      <div
                        key={shortcut.key}
                        className={cn(
                          'flex items-center justify-between p-3 rounded-md',
                          'border border-border hover:border-signal/30 transition-colors'
                        )}
                      >
                        <span className="text-sm text-text-primary">
                          {t(DESCRIPTION_KEYS[shortcut.description] || shortcut.description)}
                        </span>
                        <kbd className="px-2 py-1 text-xs font-mono text-text-secondary bg-bg-deep border border-border rounded">
                          {formatKey(shortcut.key)}
                        </kbd>
                      </div>
                    ))}
                  </div>
                </div>
              )
            })}
          </div>
        </div>

        {/* Footer */}
        <div className="px-6 py-4 border-t border-border bg-bg-deep">
          <div className="flex items-center justify-between text-sm text-text-tertiary">
            <div>
              {t('shortcuts.pressEscToClose')}{' '}
              <kbd className="px-1.5 py-0.5 text-xs font-mono bg-bg-elevated border border-border rounded">Esc</kbd>
              {' '}{t('shortcuts.toClose')}
            </div>
            <div>{t('shortcuts.appVersion')}</div>
          </div>
        </div>
      </div>
    </div>
  )
}
