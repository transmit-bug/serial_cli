// Keyboard shortcuts configuration
export interface Shortcut {
  key: string
  description: string
  category: 'navigation' | 'terminal' | 'scripts' | 'data' | 'general'
  global?: boolean // Can be used from anywhere in the app
}

export const shortcuts: Shortcut[] = [
  // Navigation
  {
    key: 'mod+k',
    description: 'Open command palette',
    category: 'general',
    global: true,
  },
  {
    key: 'mod+1',
    description: 'Terminal View',
    category: 'navigation',
    global: true,
  },
  {
    key: 'mod+2',
    description: 'Virtual Ports View',
    category: 'navigation',
    global: true,
  },
  {
    key: 'mod+3',
    description: 'Scripts View',
    category: 'navigation',
    global: true,
  },
  {
    key: 'mod+4',
    description: 'Protocols View',
    category: 'navigation',
    global: true,
  },
  {
    key: 'mod+5',
    description: 'Settings View',
    category: 'navigation',
    global: true,
  },
  // Terminal
  {
    key: 'mod+r',
    description: 'Refresh ports',
    category: 'terminal',
    global: false,
  },
  // Scripts
  {
    key: 'mod+n',
    description: 'New script',
    category: 'scripts',
    global: false,
  },
  {
    key: 'mod+enter',
    description: 'Run script',
    category: 'scripts',
    global: false,
  },
  // Data
  {
    key: 'mod+shift+c',
    description: 'Clear data',
    category: 'data',
    global: false,
  },
  // General
  {
    key: 'mod+,',
    description: 'Open settings',
    category: 'general',
    global: true,
  },
  {
    key: 'mod+/',
    description: 'Show keyboard shortcuts',
    category: 'general',
    global: true,
  },
  {
    key: 'escape',
    description: 'Close modal/dialog',
    category: 'general',
    global: true,
  },
]

// Platform-specific key display
export function formatKey(key: string): string {
  const isMac = navigator.platform.toUpperCase().indexOf('MAC') >= 0
  return key
    .replace('mod', isMac ? '⌘' : 'Ctrl')
    .replace('shift', '⇧')
    .replace('enter', '↩')
    .replace('escape', 'Esc')
}
