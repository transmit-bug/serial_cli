import { useState, useRef, useEffect } from 'react'
import { Card } from '@/components/ui/card'
import { useQuickCommandStore, type QuickCommand, type QuickCommandMode } from '@/stores/quickCommandStore'
import { useProtocolStore } from '@/stores'
import { Send, Plus, Edit2, Trash2, X, Check, GripVertical, ChevronUp, ChevronDown } from 'lucide-react'
import { useTranslation } from 'react-i18next'

/**
 * QuickCommandsCard - 快捷指令列表卡片
 *
 * 功能：
 * - 显示快捷指令列表，点击一键发送
 * - 支持添加、编辑、删除指令
 * - 支持拖拽排序（基础版：上下移动按钮）
 * - 可绑定协议编码
 */
export function QuickCommandsCard() {
  const { t } = useTranslation()
  const commands = useQuickCommandStore((s) => s.commands)
  const addCommand = useQuickCommandStore((s) => s.addCommand)
  const updateCommand = useQuickCommandStore((s) => s.updateCommand)
  const deleteCommand = useQuickCommandStore((s) => s.deleteCommand)
  const executeCommand = useQuickCommandStore((s) => s.executeCommand)
  const reorderCommands = useQuickCommandStore((s) => s.reorderCommands)
  const { protocols: protocolInfos } = useProtocolStore()
  const protocolNames = protocolInfos.map((p) => p.name)

  const [editingId, setEditingId] = useState<string | null>(null)
  const [isAdding, setIsAdding] = useState(false)
  const [dragFrom, setDragFrom] = useState<number | null>(null)

  // Edit form state
  const [editName, setEditName] = useState('')
  const [editData, setEditData] = useState('')
  const [editMode, setEditMode] = useState<QuickCommandMode>('ascii')
  const [editProtocol, setEditProtocol] = useState<string | undefined>(undefined)
  const [editNewline, setEditNewline] = useState(true)

  const nameInputRef = useRef<HTMLInputElement>(null)

  // Focus name input when adding/editing
  useEffect(() => {
    if (isAdding || editingId) {
      nameInputRef.current?.focus()
    }
  }, [isAdding, editingId])

  const startEdit = (cmd: QuickCommand) => {
    setEditingId(cmd.id)
    setEditName(cmd.name)
    setEditData(cmd.data)
    setEditMode(cmd.mode)
    setEditProtocol(cmd.protocol)
    setEditNewline(cmd.appendNewline ?? true)
    setIsAdding(false)
  }

  const startAdd = () => {
    setEditingId(null)
    setEditName('')
    setEditData('')
    setEditMode('ascii')
    setEditProtocol(undefined)
    setEditNewline(true)
    setIsAdding(true)
  }

  const cancelEdit = () => {
    setEditingId(null)
    setIsAdding(false)
  }

  const saveEdit = () => {
    if (!editName.trim() || !editData.trim()) return

    if (isAdding) {
      addCommand({
        name: editName.trim(),
        data: editData.trim(),
        mode: editMode,
        protocol: editProtocol,
        appendNewline: editNewline,
      })
    } else if (editingId) {
      updateCommand(editingId, {
        name: editName.trim(),
        data: editData.trim(),
        mode: editMode,
        protocol: editProtocol,
        appendNewline: editNewline,
      })
    }

    cancelEdit()
  }

  const moveCommand = (index: number, direction: 'up' | 'down') => {
    const toIndex = direction === 'up' ? index - 1 : index + 1
    if (toIndex < 0 || toIndex >= commands.length) return
    reorderCommands(index, toIndex)
  }

  const handleDragStart = (index: number) => {
    setDragFrom(index)
  }

  const handleDragOver = (e: React.DragEvent, index: number) => {
    e.preventDefault()
    e.dataTransfer.dropEffect = 'move'
  }

  const handleDrop = (e: React.DragEvent, toIndex: number) => {
    e.preventDefault()
    if (dragFrom !== null && dragFrom !== toIndex) {
      reorderCommands(dragFrom, toIndex)
    }
    setDragFrom(null)
  }

  const handleDragEnd = () => {
    setDragFrom(null)
  }

  return (
    <Card className="p-4 border-border/50">
      <div className="flex items-center justify-between mb-3">
        <div className="flex items-center gap-2">
          <Send className="w-4 h-4 text-signal" />
          <h3 className="text-sm font-medium text-text-primary">{t('terminal.quickCommands')}</h3>
          <span className="text-xs text-text-tertiary bg-bg-base px-2 py-0.5 rounded">
            {commands.length}
          </span>
        </div>

        {!isAdding && !editingId && (
          <button
            onClick={startAdd}
            className="p-1 rounded hover:bg-bg-elevated transition-colors text-text-secondary hover:text-text-primary"
            title={t('terminal.addCommand')}
          >
            <Plus className="w-3.5 h-3.5" />
          </button>
        )}
      </div>

      {/* Command list */}
      <div className="space-y-1">
        {commands.map((cmd, index) => (
          editingId === cmd.id ? (
            // Edit form
            <EditForm
              key={cmd.id}
              name={editName}
              data={editData}
              mode={editMode}
              protocol={editProtocol}
              appendNewline={editNewline}
              protocols={protocolNames}
              nameRef={index === 0 ? nameInputRef : undefined}
              onNameChange={setEditName}
              onDataChange={setEditData}
              onModeChange={setEditMode}
              onProtocolChange={setEditProtocol}
              onNewlineChange={setEditNewline}
              onSave={saveEdit}
              onCancel={cancelEdit}
            />
          ) : (
            // Command row
            <div
              key={cmd.id}
              draggable
              onDragStart={() => handleDragStart(index)}
              onDragOver={(e) => handleDragOver(e, index)}
              onDrop={(e) => handleDrop(e, index)}
              onDragEnd={handleDragEnd}
              className={`group flex items-center gap-1 px-2 py-1.5 rounded text-xs transition-colors ${
                dragFrom === index ? 'opacity-50' : ''
              } hover:bg-bg-elevated`}
            >
              {/* Drag handle */}
              <button
                className="cursor-grab opacity-0 group-hover:opacity-60 hover:!opacity-100 p-0.5"
                draggable={false}
              >
                <GripVertical className="w-3 h-3 text-text-tertiary" />
              </button>

              {/* Move buttons */}
              <div className="flex flex-col gap-0.5 opacity-0 group-hover:opacity-60">
                <button
                  onClick={() => moveCommand(index, 'up')}
                  disabled={index === 0}
                  className="p-0 disabled:opacity-30 hover:!opacity-100"
                  aria-label="Move up"
                >
                  <ChevronUp className="w-2.5 h-2.5" />
                </button>
                <button
                  onClick={() => moveCommand(index, 'down')}
                  disabled={index === commands.length - 1}
                  className="p-0 disabled:opacity-30 hover:!opacity-100"
                  aria-label="Move down"
                >
                  <ChevronDown className="w-2.5 h-2.5" />
                </button>
              </div>

              {/* Execute button */}
              <button
                onClick={() => executeCommand(cmd)}
                className="flex-1 text-left px-2 py-0.5 rounded hover:bg-bg-base transition-colors"
                title={`${cmd.name}: ${cmd.data}`}
              >
                <span className="text-text-primary font-medium">{cmd.name}</span>
                {cmd.protocol && (
                  <span className="ml-1 text-amber text-[10px]">[{cmd.protocol}]</span>
                )}
              </button>

              {/* Edit / Delete */}
              <div className="flex items-center gap-0.5 opacity-0 group-hover:opacity-100 transition-opacity">
                <button
                  onClick={() => startEdit(cmd)}
                  className="p-0.5 rounded hover:bg-bg-base text-text-tertiary hover:text-text-primary"
                >
                  <Edit2 className="w-3 h-3" />
                </button>
                <button
                  onClick={() => deleteCommand(cmd.id)}
                  className="p-0.5 rounded hover:bg-bg-base text-text-tertiary hover:text-alert"
                >
                  <Trash2 className="w-3 h-3" />
                </button>
              </div>
            </div>
          )
        ))}

        {/* Add form */}
        {isAdding && (
          <EditForm
            name={editName}
            data={editData}
            mode={editMode}
            protocol={editProtocol}
            appendNewline={editNewline}
            protocols={protocolNames}
            nameRef={nameInputRef}
            onNameChange={setEditName}
            onDataChange={setEditData}
            onModeChange={setEditMode}
            onProtocolChange={setEditProtocol}
            onNewlineChange={setEditNewline}
            onSave={saveEdit}
            onCancel={cancelEdit}
          />
        )}

        {/* Empty state */}
        {commands.length === 0 && !isAdding && (
          <div className="text-center text-xs text-text-tertiary py-3">
            <p>{t('terminal.noQuickCommands')}</p>
            <button
              onClick={startAdd}
              className="text-signal hover:underline mt-1 inline-block"
            >
              {t('terminal.addFirstCommand')}
            </button>
          </div>
        )}
      </div>
    </Card>
  )
}

interface EditFormProps {
  name: string
  data: string
  mode: QuickCommandMode
  protocol?: string
  appendNewline: boolean
  protocols: string[]
  nameRef?: React.RefObject<HTMLInputElement>
  onNameChange: (v: string) => void
  onDataChange: (v: string) => void
  onModeChange: (v: QuickCommandMode) => void
  onProtocolChange: (v: string | undefined) => void
  onNewlineChange: (v: boolean) => void
  onSave: () => void
  onCancel: () => void
}

function EditForm({
  name, data, mode, protocol, appendNewline, protocols,
  nameRef, onNameChange, onDataChange, onModeChange, onProtocolChange, onNewlineChange,
  onSave, onCancel,
}: EditFormProps) {
  const { t } = useTranslation()

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault()
      onSave()
    } else if (e.key === 'Escape') {
      onCancel()
    }
  }

  return (
    <div className="space-y-2 px-2 py-2 bg-bg-elevated rounded text-xs" onKeyDown={handleKeyDown}>
      <input
        ref={nameRef}
        type="text"
        value={name}
        onChange={(e) => onNameChange(e.target.value)}
        placeholder={t('terminal.commandName')}
        className="w-full px-2 py-1 bg-bg-base border border-border rounded text-text-primary placeholder:text-text-tertiary focus:outline-none focus:border-signal/50"
      />
      <textarea
        value={data}
        onChange={(e) => onDataChange(e.target.value)}
        placeholder={t('terminal.commandData')}
        rows={2}
        className="w-full px-2 py-1 bg-bg-base border border-border rounded text-text-primary font-mono placeholder:text-text-tertiary focus:outline-none focus:border-signal/50 resize-none"
      />
      <div className="flex items-center gap-2">
        {/* Mode toggle */}
        <div className="flex bg-bg-base rounded border border-border">
          {(['ascii', 'hex'] as const).map((m) => (
            <button
              key={m}
              onClick={() => onModeChange(m)}
              className={`px-2 py-0.5 text-[10px] font-medium rounded transition-colors ${
                mode === m
                  ? 'bg-signal text-black'
                  : 'text-text-secondary hover:text-text-primary'
              }`}
            >
              {m.toUpperCase()}
            </button>
          ))}
        </div>

        {/* Protocol selector */}
        {protocols.length > 0 && (
          <select
            value={protocol ?? ''}
            onChange={(e) => onProtocolChange(e.target.value || undefined)}
            className="px-2 py-0.5 bg-bg-base border border-border rounded text-text-secondary text-[10px] focus:outline-none"
          >
            <option value="">{t('terminal.noProtocol')}</option>
            {protocols.map((p) => (
              <option key={p} value={p}>{p}</option>
            ))}
          </select>
        )}

        {/* Newline toggle */}
        <label className="flex items-center gap-1 text-text-secondary cursor-pointer select-none">
          <input
            type="checkbox"
            checked={appendNewline}
            onChange={(e) => onNewlineChange(e.target.checked)}
            className="w-3 h-3 accent-signal"
          />
          <span className="text-[10px]">\\r\\n</span>
        </label>
      </div>

      <div className="flex items-center gap-1">
        <button
          onClick={onSave}
          disabled={!name.trim() || !data.trim()}
          className="px-2 py-1 bg-signal/20 text-signal border border-signal/30 rounded hover:bg-signal/30 disabled:opacity-40 transition-colors"
        >
          <Check className="w-3 h-3" />
        </button>
        <button
          onClick={onCancel}
          className="px-2 py-1 bg-bg-base text-text-secondary border border-border rounded hover:text-text-primary transition-colors"
        >
          <X className="w-3 h-3" />
        </button>
      </div>
    </div>
  )
}
