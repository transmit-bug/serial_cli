import { Panel } from '@/components/ui/panel'
import { cn } from '@/lib/utils'
import { FileCode, Upload, Trash2, AlertCircle } from 'lucide-react'
import { useState, useRef, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { protocolsStorage } from '@/lib/storage'
import { useProtocolStore } from '@/stores/protocolStore'

interface CustomProtocol {
  id: string
  name: string
  version: string
  description: string
  type: 'custom'
  status: 'active' | 'inactive'
  filePath?: string  // Absolute path to the .lua file on disk
}

export function ProtocolPanel() {
  const { protocols, activeProtocol, setActiveProtocol, loadProtocols, enableProtocol, disableProtocol, loading } = useProtocolStore()
  const [customProtocols, setCustomProtocols] = useState<CustomProtocol[]>([])
  const [isLoading, setIsLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [validationStatus, setValidationStatus] = useState<Map<string, 'valid' | 'invalid'>>(new Map())
  const fileInputRef = useRef<HTMLInputElement>(null)

  // Load built-in protocols from backend on mount
  useEffect(() => {
    loadProtocols()
  }, [loadProtocols])

  // Load custom protocols from localStorage on mount
  useEffect(() => {
    const saved = protocolsStorage.get()
    if (saved.length > 0) {
      setCustomProtocols(saved.filter(p => p.type === 'custom') as CustomProtocol[])
    }
  }, [])

  const handleSelectProtocol = (id: string) => {
    setActiveProtocol(id)
  }

  const toggleProtocol = async (id: string, type: 'built-in' | 'custom') => {
    try {
      if (type === 'built-in') {
        const proto = protocols.find(p => p.name === id)
        if (!proto) return
        // Built-in protocols are always loaded — toggle UI state only
        // In a future version, backend could track active state per protocol
      } else {
        const custom = customProtocols.find(p => p.id === id)
        if (!custom) return
        if (custom.status === 'active') {
          await invoke('unload_protocol', { name: custom.name })
          setCustomProtocols(prev => prev.map(p =>
            p.id === id ? { ...p, status: 'inactive' } : p
          ))
        } else {
          if (!custom.filePath) {
            setError(`Cannot load protocol "${custom.name}": file path missing`)
            return
          }
          await invoke('load_protocol', { path: custom.filePath })
          setCustomProtocols(prev => prev.map(p =>
            p.id === id ? { ...p, status: 'active' } : p
          ))
        }
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to toggle protocol')
    }
  }

  const deleteCustomProtocol = (id: string) => {
    const protocol = customProtocols.find(p => p.id === id)
    const updated = customProtocols.filter(p => p.id !== id)
    setCustomProtocols(updated)
    protocolsStorage.set(updated.map(p => ({
      ...p,
      lastModified: Date.now(),
    })))

    if (activeProtocol === id) {
      setActiveProtocol(null)
    }

    if (protocol) {
      setValidationStatus(prev => {
        const next = new Map(prev)
        next.delete(protocol.name)
        return next
      })
    }
  }

  const loadCustomProtocol = async (file: File) => {
    setIsLoading(true)
    setError(null)

    try {
      // Read file content
      const content = await file.text()

      // Save the file to the backend's protocols directory via Tauri command
      const filePath = await invoke<string>('save_protocol_file', {
        name: file.name,
        content,
      })

      // Validate protocol syntax
      try {
        await invoke('validate_protocol', { path: filePath })
        setValidationStatus(prev => new Map(prev).set(file.name, 'valid'))
      } catch (err) {
        setValidationStatus(prev => new Map(prev).set(file.name, 'invalid'))
        throw err
      }

      // Load protocol via Tauri using absolute file path
      const protocolInfo = await invoke<any>('load_protocol', { path: filePath })

      const newProtocol: CustomProtocol = {
        id: `custom-${Date.now()}`,
        name: file.name.replace('.lua', ''),
        version: '1.0',
        description: protocolInfo.description || 'Custom Lua protocol',
        type: 'custom',
        status: 'inactive',
        filePath,
      }

      const updated = [...customProtocols, newProtocol]
      setCustomProtocols(updated)
      protocolsStorage.set(updated.map(p => ({
        ...p,
        lastModified: Date.now(),
      })))
      setActiveProtocol(newProtocol.id)

    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : String(err)
      setError(errorMsg)
      setValidationStatus(prev => new Map(prev).set(file.name, 'invalid'))
    } finally {
      setIsLoading(false)
    }
  }

  const handleFileSelect = (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0]
    if (file) {
      loadCustomProtocol(file)
    }
  }

  // All protocols merged: built-in from store + custom from localStorage
  const allProtocols = [
    ...protocols.map(p => ({ ...p, type: 'built-in' as const })),
    ...customProtocols,
  ]

  return (
    <div className="space-y-6">
      {/* Error Display */}
      {error && (
        <div className="p-3 rounded-md bg-alert/10 border border-alert/30">
          <div className="flex items-center gap-2 text-alert text-sm">
            <AlertCircle size={16} strokeWidth={1.5} />
            <span className="font-medium">Protocol Error</span>
          </div>
          <p className="text-xs text-alert mt-1 font-mono">{error}</p>
        </div>
      )}

      {/* Protocol Grid */}
      <div className="grid grid-cols-1 md:grid-cols-2 gap-6 w-full">
        {/* Built-in Protocols */}
        <Panel title="Built-in Protocols" variant="signal" actions={
          <span className="text-xs text-text-tertiary font-mono">
            {loading ? 'Loading...' : `${protocols.length} protocols`}
          </span>
        }>
          {loading ? (
            <div className="py-8 text-center text-xs text-text-tertiary">Loading protocols...</div>
          ) : protocols.length === 0 ? (
            <div className="py-8 text-center text-xs text-text-tertiary">
              <p>No built-in protocols available</p>
              <p className="mt-1">Check backend protocol configuration</p>
            </div>
          ) : (
            <div className="space-y-2">
              {protocols.map(protocol => {
                const isActive = activeProtocol === protocol.name
                return (
                  <div
                    key={protocol.name}
                    className={cn(
                      'group p-3 rounded-md border transition-all duration-200 cursor-pointer',
                      isActive
                        ? 'bg-signal/10 border-signal/30'
                        : 'bg-bg-deep border-border hover:border-signal/30 hover:bg-bg-elevated'
                    )}
                    onClick={() => handleSelectProtocol(protocol.name)}
                  >
                    <div className="flex items-start justify-between">
                      <div className="flex items-center gap-3">
                        <div className="p-2 rounded-md bg-bg-elevated">
                          <FileCode size={18} strokeWidth={1.5} className="text-text-tertiary" />
                        </div>
                        <div>
                          <h4 className="font-medium text-sm text-text-primary">{protocol.name}</h4>
                          <p className="text-xs text-text-tertiary mt-0.5">{protocol.description}</p>
                        </div>
                      </div>
                      <span className="px-2.5 py-1 text-xs rounded-md bg-bg-elevated text-text-tertiary border border-border">
                        Built-in
                      </span>
                    </div>
                  </div>
                )
              })}
            </div>
          )}
        </Panel>

        {/* Custom Protocols */}
        <Panel
          title="Custom Protocols"
          variant="amber"
          actions={
            <>
              <button
                onClick={() => fileInputRef.current?.click()}
                className="p-1.5 rounded hover:bg-bg-elevated text-text-tertiary hover:text-text-primary transition-colors"
                title="Load protocol"
                disabled={isLoading}
              >
                <Upload size={14} strokeWidth={1.5} />
              </button>
              <input
                ref={fileInputRef}
                type="file"
                accept=".lua"
                className="hidden"
                onChange={handleFileSelect}
              />
            </>
          }
        >
          {customProtocols.length === 0 ? (
            <div className="py-12 text-center">
              <div className="inline-flex items-center justify-center w-12 h-12 rounded-full bg-bg-elevated mb-3">
                <FileCode size={20} className="text-text-tertiary" strokeWidth={1.5} />
              </div>
              <p className="text-sm text-text-tertiary">No custom protocols</p>
              <p className="text-xs text-text-tertiary mt-1">Load a .lua protocol file to get started</p>
            </div>
          ) : (
            <div className="space-y-2">
              {customProtocols.map(protocol => {
                const isActive = activeProtocol === protocol.id
                return (
                  <div
                    key={protocol.id}
                    className={cn(
                      'group p-3 rounded-md border transition-all duration-200 cursor-pointer',
                      isActive
                        ? 'bg-amber/10 border-amber/30'
                        : 'bg-bg-deep border-border hover:border-amber/30 hover:bg-bg-elevated'
                    )}
                    onClick={() => handleSelectProtocol(protocol.id)}
                  >
                    <div className="flex items-start justify-between">
                      <div className="flex items-center gap-3">
                        <div className={cn(
                          'p-2 rounded-md',
                          protocol.status === 'active' ? 'bg-amber/20' : 'bg-bg-elevated'
                        )}>
                          <FileCode size={18} strokeWidth={1.5} className={cn(
                            protocol.status === 'active' ? 'text-amber' : 'text-text-tertiary'
                          )} />
                        </div>
                        <div>
                          <h4 className="font-medium text-sm text-text-primary">{protocol.name}</h4>
                          <p className="text-xs text-text-tertiary mt-0.5">{protocol.description}</p>
                        </div>
                      </div>
                      <div className="flex items-center gap-2">
                        {validationStatus.get(protocol.name) && (
                          <span className={cn(
                            'px-1.5 py-0.5 text-[10px] rounded font-medium',
                            validationStatus.get(protocol.name) === 'valid'
                              ? 'bg-signal/10 text-signal'
                              : 'bg-alert/10 text-alert'
                          )}>
                            {validationStatus.get(protocol.name) === 'valid' ? '✓ Valid' : '✗ Invalid'}
                          </span>
                        )}
                        <button
                          onClick={(e) => {
                            e.stopPropagation()
                            toggleProtocol(protocol.id, 'custom')
                          }}
                          className={cn(
                            'px-2.5 py-1 text-xs rounded-md border transition-colors',
                            protocol.status === 'active'
                              ? 'bg-amber/20 text-amber border-amber/30'
                              : 'bg-bg-elevated text-text-tertiary border-border hover:text-text-primary'
                          )}
                        >
                          {protocol.status === 'active' ? 'Active' : 'Enable'}
                        </button>
                        <button
                          onClick={(e) => {
                            e.stopPropagation()
                            deleteCustomProtocol(protocol.id)
                          }}
                          className="opacity-0 group-hover:opacity-100 p-1.5 rounded hover:bg-alert/20 text-text-tertiary hover:text-alert transition-all"
                        >
                          <Trash2 size={14} strokeWidth={1.5} />
                        </button>
                      </div>
                    </div>
                  </div>
                )
              })}
            </div>
          )}
        </Panel>
      </div>

      {/* Protocol Details */}
      {activeProtocol && (
        <Panel title="Protocol Details" variant="default" className="w-full">
          <div className="grid grid-cols-2 gap-6">
            <div className="space-y-4">
              <div>
                <label className="text-xs text-text-tertiary uppercase tracking-wider">Name</label>
                <p className="font-medium text-text-primary mt-1">{activeProtocol}</p>
              </div>
              <div>
                <label className="text-xs text-text-tertiary uppercase tracking-wider">Type</label>
                <p className="mt-1">
                  <span className={cn(
                    'px-2 py-1 text-xs rounded-md',
                    customProtocols.find(p => p.id === activeProtocol)
                      ? 'bg-amber/10 text-amber'
                      : 'bg-signal/10 text-signal'
                  )}>
                    {customProtocols.find(p => p.id === activeProtocol) ? 'Custom' : 'Built-in'}
                  </span>
                </p>
              </div>
            </div>
            <div className="space-y-4">
              <div>
                <label className="text-xs text-text-tertiary uppercase tracking-wider">Description</label>
                <p className="text-sm text-text-secondary mt-1">
                  {customProtocols.find(p => p.id === activeProtocol)?.description ||
                   protocols.find(p => p.name === activeProtocol)?.description ||
                   'No description available'}
                </p>
              </div>
              <div>
                <label className="text-xs text-text-tertiary uppercase tracking-wider">Status</label>
                <p className="mt-1">
                  <span className={cn(
                    'px-2 py-1 text-xs rounded-md',
                    customProtocols.find(p => p.id === activeProtocol)?.status === 'active'
                      ? 'bg-amber/10 text-amber'
                      : 'bg-signal/10 text-signal'
                  )}>
                    {customProtocols.find(p => p.id === activeProtocol)?.status === 'active'
                      ? 'Active'
                      : 'Selected'}
                  </span>
                </p>
              </div>
            </div>
          </div>
        </Panel>
      )}
    </div>
  )
}
