import { useNavigationStore } from '@/stores'
import { cn } from '@/lib/utils'
import {
  Plug,
  BarChart3,
  FileCode2,
  Cpu,
  Settings,
  Network,
} from 'lucide-react'

const navItems = [
  { id: 'ports', label: 'Ports', icon: Plug, shortcut: '⌘1' },
  { id: 'virtual', label: 'Virtual Ports', icon: Network, shortcut: '⌘2' },
  { id: 'data', label: 'Data Monitor', icon: BarChart3, shortcut: '⌘3' },
  { id: 'scripts', label: 'Scripts', icon: FileCode2, shortcut: '⌘4' },
  { id: 'protocols', label: 'Protocols', icon: Cpu, shortcut: '⌘5' },
  { id: 'settings', label: 'Settings', icon: Settings, shortcut: '⌘6' },
] as const

export function Sidebar() {
  const { currentView, navigateTo } = useNavigationStore()

  return (
    <aside className="w-64 border-r border-border bg-bg-deep flex flex-col">
      <div className="p-6 border-b border-border/50">
        <h1 className="text-xl font-display font-semibold text-signal">
          Serial CLI
        </h1>
        <p className="text-xs text-text-tertiary mt-1 uppercase tracking-wider">
          Command Center
        </p>
      </div>

      <nav className="flex-1 px-3 py-4 space-y-1">
        {navItems.map((item) => {
          const Icon = item.icon
          const isActive = currentView === item.id

          return (
            <button
              key={item.id}
              onClick={() => navigateTo(item.id as any)}
              className={cn(
                'group w-full flex items-center justify-between px-3 py-2 rounded-md text-sm font-medium transition-all duration-200',
                isActive
                  ? 'bg-signal/10 text-signal border border-signal/30 shadow-sm'
                  : 'text-text-secondary hover:text-text-primary hover:bg-bg-elevated border border-transparent'
              )}
            >
              <div className="flex items-center gap-3">
                <Icon
                  size={18}
                  strokeWidth={1.5}
                  className={cn(
                    'transition-colors',
                    isActive ? 'text-signal' : 'text-text-secondary group-hover:text-text-primary'
                  )}
                />
                <span>{item.label}</span>
              </div>
              <span className={cn(
                'text-xs font-mono transition-opacity duration-200',
                isActive ? 'text-signal/60' : 'text-text-tertiary opacity-0 group-hover:opacity-100'
              )}>
                {item.shortcut}
              </span>
            </button>
          )
        })}
      </nav>

      <div className="p-4 border-t border-border/50 space-y-2">
        <div className="flex items-center justify-between text-xs text-text-tertiary">
          <span className="font-mono">v0.1.0</span>
          <span className="flex items-center gap-1.5">
            <span className="w-1.5 h-1.5 rounded-full bg-signal/50"></span>
            Tauri 2.0
          </span>
        </div>

        {/* Keyboard shortcut hint */}
        <div className="pt-2 border-t border-border/30">
          <div className="flex items-center gap-2 text-xs text-text-tertiary">
            <kbd className="px-1.5 py-0.5 font-mono text-[10px] bg-bg-elevated border border-border rounded">
              ⌘K
            </kbd>
            <span>Command Palette</span>
          </div>
        </div>
      </div>
    </aside>
  )
}
