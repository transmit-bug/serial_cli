import { useState, useEffect, useCallback } from 'react'
import { useNavigationStore } from '@/stores'
import { usePorts } from '@/contexts/PortContext'
import { useData } from '@/contexts/DataContext'
import { useShortcuts } from '@/contexts/ShortcutContext'
import { useScriptActions } from '@/contexts/ScriptActionContext'
import { cn } from '@/lib/utils'

interface Command {
  id: string
  label: string
  description?: string
  icon: string
  shortcut?: string
  action: () => void
  category: string
}

export function CommandPalette() {
  const { currentView, navigateTo } = useNavigationStore()
  const { listPorts } = usePorts()
  const { clearPackets } = useData()
  const { isCommandPaletteOpen, closeCommandPalette, openShortcutsHelp } = useShortcuts()
  const { createNewScript } = useScriptActions()
  const [query, setQuery] = useState('')
  const [selectedIndex, setSelectedIndex] = useState(0)

  // Define all available commands
  const commands: Command[] = [
    // Navigation
    {
      id: 'nav-ports',
      label: 'Go to Ports',
      icon: '🔌',
      shortcut: '⌘1',
      category: 'Navigation',
      action: () => navigateTo('ports'),
    },
    {
      id: 'nav-data',
      label: 'Go to Data Monitor',
      icon: '📊',
      shortcut: '⌘2',
      category: 'Navigation',
      action: () => navigateTo('data'),
    },
    {
      id: 'nav-scripts',
      label: 'Go to Scripts',
      icon: '📜',
      shortcut: '⌘3',
      category: 'Navigation',
      action: () => navigateTo('scripts'),
    },
    {
      id: 'nav-protocols',
      label: 'Go to Protocols',
      icon: '🔧',
      shortcut: '⌘4',
      category: 'Navigation',
      action: () => navigateTo('protocols'),
    },
    {
      id: 'nav-settings',
      label: 'Go to Settings',
      icon: '⚙️',
      shortcut: '⌘5',
      category: 'Navigation',
      action: () => navigateTo('settings'),
    },
    // Ports
    {
      id: 'ports-refresh',
      label: 'Refresh Ports',
      icon: '🔄',
      shortcut: '⌘R',
      category: 'Ports',
      action: () => {
        navigateTo('ports')
        listPorts()
      },
    },
    // Data
    {
      id: 'data-clear',
      label: 'Clear Data',
      icon: '🗑️',
      shortcut: '⌘⇧C',
      category: 'Data',
      action: () => clearPackets(),
    },
    // Scripts
    {
      id: 'script-new',
      label: 'New Script',
      icon: '📄',
      shortcut: '⌘N',
      category: 'Scripts',
      action: () => {
        navigateTo('scripts')
        createNewScript()
      },
    },
    // General
    {
      id: 'show-shortcuts',
      label: 'Show Keyboard Shortcuts',
      icon: '⌨️',
      shortcut: '⌘/',
      category: 'General',
      action: () => {
        closeCommandPalette()
        openShortcutsHelp()
      },
    },
  ]

  // Filter commands based on query
  const filteredCommands = commands.filter(cmd => {
    const searchStr = `${cmd.label} ${cmd.category}`.toLowerCase()
    return query === '' || searchStr.includes(query.toLowerCase())
  })

  // Reset selected index when query changes
  useEffect(() => {
    setSelectedIndex(0)
  }, [query])

  // Handle keyboard navigation
  const handleKeyDown = useCallback((e: React.KeyboardEvent) => {
    if (e.key === 'ArrowDown') {
      e.preventDefault()
      setSelectedIndex(prev => (prev + 1) % filteredCommands.length)
    } else if (e.key === 'ArrowUp') {
      e.preventDefault()
      setSelectedIndex(prev => (prev - 1 + filteredCommands.length) % filteredCommands.length)
    } else if (e.key === 'Enter' && filteredCommands.length > 0) {
      e.preventDefault()
      filteredCommands[selectedIndex].action()
      closeCommandPalette()
      setQuery('')
    } else if (e.key === 'Escape') {
      closeCommandPalette()
      setQuery('')
    }
  }, [filteredCommands, selectedIndex, closeCommandPalette])

  // Close on outside click
  useEffect(() => {
    if (!isCommandPaletteOpen) {
      setQuery('')
      setSelectedIndex(0)
    }
  }, [isCommandPaletteOpen])

  if (!isCommandPaletteOpen) return null

  // Group commands by category
  const categories = Array.from(new Set(filteredCommands.map(cmd => cmd.category)))

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center">
      {/* Backdrop */}
      <div
        className="absolute inset-0 bg-black/50 backdrop-blur-sm animate-fade-in"
        onClick={closeCommandPalette}
      />

      {/* Command Palette */}
      <div className="relative w-full max-w-2xl mx-4 bg-bg-floating border border-border rounded-lg shadow-xl overflow-hidden animate-slide-up">
        {/* Search Input */}
        <div className="flex items-center px-4 border-b border-border">
          <span className="text-lg mr-3">🔍</span>
          <input
            type="text"
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            onKeyDown={handleKeyDown}
            placeholder="Type a command or search..."
            className="flex-1 bg-transparent border-0 outline-none py-4 text-text-primary placeholder:text-text-tertiary"
            autoFocus
          />
          <span className="text-xs text-text-tertiary">ESC</span>
        </div>

        {/* Command List */}
        <div className="overflow-y-auto py-2" style={{ maxHeight: 'calc(100vh - 300px)' }}>
          {filteredCommands.length === 0 ? (
            <div className="px-4 py-8 text-center text-text-tertiary">
              No commands found
            </div>
          ) : (
            categories.map(category => {
              const categoryCommands = filteredCommands.filter(cmd => cmd.category === category)
              return (
                <div key={category}>
                  {/* Category Header */}
                  <div className="px-4 py-2 text-xs font-medium text-text-tertiary uppercase tracking-wide">
                    {category}
                  </div>

                  {/* Commands */}
                  {categoryCommands.map((cmd, idx) => {
                    const globalIndex = filteredCommands.indexOf(cmd)
                    const isSelected = globalIndex === selectedIndex
                    return (
                      <button
                        key={cmd.id}
                        onClick={() => {
                          cmd.action()
                          closeCommandPalette()
                          setQuery('')
                        }}
                        className={cn(
                          'w-full flex items-center gap-3 px-4 py-3 text-left transition-colors',
                          isSelected
                            ? 'bg-signal/10 text-signal'
                            : 'hover:bg-bg-elevated text-text-primary'
                        )}
                      >
                        <span className="text-lg">{cmd.icon}</span>
                        <div className="flex-1">
                          <div className={cn(
                            'text-sm font-medium',
                            isSelected ? 'text-signal' : ''
                          )}>
                            {cmd.label}
                          </div>
                          {cmd.description && (
                            <div className="text-xs text-text-tertiary mt-0.5">
                              {cmd.description}
                            </div>
                          )}
                        </div>
                        {cmd.shortcut && (
                          <span className="text-xs text-text-tertiary font-mono">
                            {cmd.shortcut}
                          </span>
                        )}
                      </button>
                    )
                  })}
                </div>
              )
            })
          )}
        </div>

        {/* Footer */}
        <div className="px-4 py-2 border-t border-border flex items-center justify-between text-xs text-text-tertiary">
          <div className="flex items-center gap-4">
            <span>↑↓ Navigate</span>
            <span>↩ Select</span>
            <span>ESC Close</span>
          </div>
          <div>{filteredCommands.length} commands</div>
        </div>
      </div>
    </div>
  )
}
