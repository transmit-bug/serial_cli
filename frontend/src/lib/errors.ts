import { useTranslation } from 'react-i18next'

/**
 * Error categories for better error handling
 */
export enum ErrorCategory {
  NETWORK = 'network',
  SERIAL_PORT = 'serial_port',
  PROTOCOL = 'protocol',
  SCRIPT = 'script',
  VIRTUAL_PORT = 'virtual_port',
  VALIDATION = 'validation',
  PERMISSION = 'permission',
  UNKNOWN = 'unknown',
}

/**
 * Application error interface
 */
export interface AppError {
  category: ErrorCategory
  code?: string
  message: string
  details?: string
  suggestion?: string
  recoverable: boolean
}

/**
 * Parse error into AppError format
 */
export function parseError(error: unknown): AppError {
  if (error instanceof Error) {
    const message = error.message.toLowerCase()

    if (message.includes('serial') || message.includes('port')) {
      if (message.includes('permission') || message.includes('denied')) {
        return {
          category: ErrorCategory.PERMISSION,
          message: error.message,
          recoverable: false,
          suggestion: 'Try running the application with elevated permissions',
        }
      }
      return {
        category: ErrorCategory.SERIAL_PORT,
        message: error.message,
        recoverable: true,
        suggestion: 'Check if the port is available and not in use by another application',
      }
    }

    if (message.includes('protocol')) {
      return {
        category: ErrorCategory.PROTOCOL,
        message: error.message,
        recoverable: true,
        suggestion: 'Verify the protocol file syntax and try again',
      }
    }

    if (message.includes('script') || message.includes('lua')) {
      return {
        category: ErrorCategory.SCRIPT,
        message: error.message,
        recoverable: true,
        suggestion: 'Check the script syntax and ensure all required functions are defined',
      }
    }

    return {
      category: ErrorCategory.UNKNOWN,
      message: error.message,
      recoverable: true,
    }
  }

  if (typeof error === 'string') {
    return {
      category: ErrorCategory.UNKNOWN,
      message: error,
      recoverable: true,
    }
  }

  return {
    category: ErrorCategory.UNKNOWN,
    message: 'An unknown error occurred',
    recoverable: false,
  }
}

/**
 * Show error toast with proper formatting
 */
export function showErrorToast(error: unknown, t: (key: string, params?: any) => string): void {
  const appError = parseError(error)
  const categoryTitle = appError.category.split('_').map(w => w.charAt(0).toUpperCase() + w.slice(1)).join(' ')
  const title = t('errors.category', { defaultValue: categoryTitle })

  const description = `${appError.message}${appError.suggestion ? `\n\n💡 ${appError.suggestion}` : ''}`

  import('sonner').then(({ toast }) => {
    toast.error(title, {
      description,
      duration: appError.recoverable ? 5000 : 8000,
    })
  })
}

/**
 * Hook for error handling with internationalization
 */
export function useErrorHandler() {
  const { t } = useTranslation()

  const handleError = (error: unknown) => {
    showErrorToast(error, t)
  }

  const handleAsyncError = async <T>(asyncFn: () => Promise<T>): Promise<T | null> => {
    try {
      return await asyncFn()
    } catch (error) {
      handleError(error)
      return null
    }
  }

  return { handleError, handleAsyncError, parseError }
}
