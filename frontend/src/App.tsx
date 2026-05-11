import React from 'react'
import { PortProvider } from './contexts/PortContext'
import { VirtualPortProvider } from './contexts/VirtualPortContext'
import { DataProvider } from './contexts/DataContext'
import { ShortcutProvider } from './contexts/ShortcutContext'
import { NotificationProvider } from './contexts/NotificationContext'
import { ScriptActionProvider } from './contexts/ScriptActionContext'
import { SettingsProvider } from './contexts/SettingsContext'
import { useGlobalShortcuts } from './hooks/useGlobalShortcuts'
import { Sidebar } from './components/layout/Sidebar'
import { TopBar } from './components/layout/TopBar'
import { TerminalWorkbench } from './components/terminal'
import { VirtualPortsPanel } from './components/virtual/VirtualPortsPanel'
import { ScriptPanel } from './components/scripting/ScriptPanel'
import { ProtocolPanel } from './components/protocols/ProtocolPanel'
import { SettingsPanel } from './components/settings/SettingsPanel'
import { Toaster } from './components/ui/toast'
import { CommandPalette } from './components/shortcuts/CommandPalette'
import { KeyboardShortcutsHelp } from './components/shortcuts/KeyboardShortcutsHelp'
import { cn } from './lib/utils'
import { useNavigationStore } from './stores'

function AppContent() {
  const { currentView } = useNavigationStore()

  // Register global shortcuts
  useGlobalShortcuts()

  const viewComponents: Record<string, React.ComponentType> = {
    terminal: TerminalWorkbench,
    virtual: VirtualPortsPanel,
    scripts: ScriptPanel,
    protocols: ProtocolPanel,
    settings: SettingsPanel,
  }

  const CurrentView = viewComponents[currentView] || TerminalWorkbench

  return (
    <div className="app-background h-screen flex flex-col">
      <TopBar />
      <div className="flex flex-1 overflow-hidden">
        <Sidebar />
        <main className="flex-1 overflow-auto">
          <div className={cn(
            "p-6 min-h-full",
            "animate-fade-in"
          )}>
            <CurrentView />
          </div>
        </main>
      </div>
      <Toaster />

      {/* Global overlays */}
      <CommandPalette />
      <KeyboardShortcutsHelp />
    </div>
  )
}

function App() {
  return (
    <React.StrictMode>
      <NotificationProvider>
        <ShortcutProvider>
          <ScriptActionProvider>
            <SettingsProvider>
              <PortProvider>
                <VirtualPortProvider>
                  <DataProvider>
                    <AppContent />
                  </DataProvider>
                </VirtualPortProvider>
              </PortProvider>
            </SettingsProvider>
          </ScriptActionProvider>
        </ShortcutProvider>
      </NotificationProvider>
    </React.StrictMode>
  )
}

export default App
