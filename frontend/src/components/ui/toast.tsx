import { Toaster as SonnerToaster } from 'sonner'

/**
 * Toast notification system using Sonner
 *
 * Usage:
 * ```tsx
 * import { toast } from 'sonner'
 *
 * toast.success('Operation completed')
 * toast.error('Something went wrong')
 * toast.warning('Are you sure?')
 * ```
 */
export function Toaster() {
  return (
    <SonnerToaster
      position="bottom-right"
      expand={false}
      richColors
      closeButton
      toastOptions={{
        classNames: {
          toast: 'border-border/50 bg-bg-deep/95 text-text-primary',
          title: 'text-text-primary',
          description: 'text-text-secondary',
          actionButton: 'bg-signal text-black hover:bg-signal/90',
          cancelButton: 'bg-bg-base text-text-primary hover:bg-bg-elevated',
          closeButton: 'text-text-tertiary hover:text-text-primary',
        },
      }}
    />
  )
}
