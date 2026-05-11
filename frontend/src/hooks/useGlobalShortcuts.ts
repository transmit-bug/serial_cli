import { useEffect } from 'react'
import { useHotkeys } from 'react-hotkeys-hook'
import { useNavigationStore } from '@/stores'
import { usePorts } from '@/contexts/PortContext'
import { useData } from '@/contexts/DataContext'
import { useShortcuts } from '@/contexts/ShortcutContext'
import { useScriptActions } from '@/contexts/ScriptActionContext'
import { useWindow } from '@/hooks/useWindow'

export function useGlobalShortcuts() {
  const { currentView, navigateTo } = useNavigationStore()
  const { listPorts } = usePorts()
  const { clearPackets } = useData()
  const { openCommandPalette, openShortcutsHelp } = useShortcuts()
  const { createNewScript, runCurrentScript } = useScriptActions()
  const { hideWindow } = useWindow()

  // Command Palette: Cmd/Ctrl + K
  useHotkeys('mod+k', (e) => {
    e.preventDefault()
    openCommandPalette()
  }, { enableOnFormTags: true })

  // Navigation: Cmd/Ctrl + 1-5
  useHotkeys('mod+1', (e) => {
    e.preventDefault()
    navigateTo('ports')
  })
  useHotkeys('mod+2', (e) => {
    e.preventDefault()
    navigateTo('virtual')
  })
  useHotkeys('mod+3', (e) => {
    e.preventDefault()
    navigateTo('data')
  })
  useHotkeys('mod+4', (e) => {
    e.preventDefault()
    navigateTo('scripts')
  })
  useHotkeys('mod+5', (e) => {
    e.preventDefault()
    navigateTo('protocols')
  })
  useHotkeys('mod+6', (e) => {
    e.preventDefault()
    navigateTo('settings')
  })

  // Ports: Cmd/Ctrl + R (only when in ports view or global)
  useHotkeys('mod+r', (e) => {
    e.preventDefault()
    if (currentView === 'ports') {
      listPorts()
    }
  })

  // Scripts: Cmd/Ctrl + N
  useHotkeys('mod+n', (e) => {
    e.preventDefault()
    navigateTo('scripts')
    createNewScript()
  })

  // Scripts: Cmd/Ctrl + Enter (only in scripts view)
  useHotkeys('mod+enter', (e) => {
    e.preventDefault()
    if (currentView === 'scripts') {
      runCurrentScript()
    }
  })

  // Data: Cmd/Ctrl + Shift + C
  useHotkeys('mod+shift+c', (e) => {
    e.preventDefault()
    clearPackets()
  })

  // Settings: Cmd/Ctrl + ,
  useHotkeys('mod+,', (e) => {
    e.preventDefault()
    navigateTo('settings')
  })

  // Shortcuts Help: Cmd/Ctrl + /
  useHotkeys('mod+/', (e) => {
    e.preventDefault()
    openShortcutsHelp()
  })

  // Escape: Close modals/dialogs
  useHotkeys('escape', (e) => {
    // This will be handled by the modal components themselves
    // Just prevent default behavior
    e.preventDefault()
  })

  // Hide window: Cmd/Ctrl + W (optional)
  useHotkeys('mod+w', (e) => {
    e.preventDefault()
    hideWindow()
  })
}
