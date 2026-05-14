import { Check } from 'lucide-react'
import { cn } from '@/lib/utils'

interface ToggleSwitchProps {
  checked: boolean
  onChange?: (checked: boolean) => void
  disabled?: boolean
  className?: string
}

/**
 * ToggleSwitch - Reusable toggle switch component
 *
 * Shared between SettingsPanel, NotificationSettings, and other areas.
 * Uses signal color for checked state.
 */
export function ToggleSwitch({ checked, onChange, disabled, className }: ToggleSwitchProps) {
  return (
    <button
      onClick={() => onChange?.(!checked)}
      disabled={disabled}
      role="switch"
      aria-checked={checked}
      className={cn(
        'w-12 h-6 rounded-full p-1 transition-colors relative',
        'disabled:opacity-50 disabled:cursor-not-allowed',
        checked ? 'bg-signal' : 'bg-bg-elevated',
        className,
      )}
    >
      <div className={cn(
        'w-4 h-4 rounded-full bg-white transition-transform flex items-center justify-center',
        checked ? 'translate-x-6' : 'translate-x-0',
      )}>
        {checked && <Check size={10} strokeWidth={3} className="text-signal" />}
      </div>
    </button>
  )
}
