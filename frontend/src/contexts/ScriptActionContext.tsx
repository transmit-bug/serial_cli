import React, { createContext, useContext, useState, useCallback } from 'react'

interface ScriptActionContextType {
  createNewScript: () => void
  runCurrentScript: () => void
  validateCurrentScript: () => void
  registerCallbacks: (callbacks: {
    createNewScript: () => void
    runCurrentScript: () => void
    validateCurrentScript: () => void
  }) => () => void
}

const ScriptActionContext = createContext<ScriptActionContextType | undefined>(undefined)

export function ScriptActionProvider({ children }: { children: React.ReactNode }) {
  const [callbacks, setCallbacks] = useState<{
    createNewScript: () => void
    runCurrentScript: () => void
    validateCurrentScript: () => void
  } | null>(null)

  const registerCallbacks = useCallback((cb: {
    createNewScript: () => void
    runCurrentScript: () => void
    validateCurrentScript: () => void
  }) => {
    setCallbacks(cb)
    return () => setCallbacks(null)
  }, [])

  const createNewScript = useCallback(() => {
    callbacks?.createNewScript()
  }, [callbacks])

  const runCurrentScript = useCallback(() => {
    callbacks?.runCurrentScript()
  }, [callbacks])

  const validateCurrentScript = useCallback(() => {
    callbacks?.validateCurrentScript()
  }, [callbacks])

  return (
    <ScriptActionContext.Provider value={{
      createNewScript,
      runCurrentScript,
      validateCurrentScript,
      registerCallbacks,
    }}>
      {children}
    </ScriptActionContext.Provider>
  )
}

export function useScriptActions() {
  const context = useContext(ScriptActionContext)
  if (!context) {
    throw new Error('useScriptActions must be used within ScriptActionProvider')
  }
  return context
}
