import { useConnectionStore } from '@/stores'
import { AlertTriangle } from 'lucide-react'
import { useTranslation } from 'react-i18next'

/**
 * ErrorState - 错误状态
 *
 * 显示错误信息和恢复选项
 */
export function ErrorState() {
  const { error, disconnect, portName } = useConnectionStore()
  const { t } = useTranslation()

  return (
    <div className="flex-1 flex items-center justify-center p-8">
      <div className="max-w-md w-full space-y-6">
        {/* 错误图标 */}
        <div className="flex justify-center">
          <div className="w-16 h-16 rounded-full bg-alert/10 border-2 border-alert/30 flex items-center justify-center">
            <AlertTriangle className="w-8 h-8 text-alert" />
          </div>
        </div>

        {/* 错误消息 */}
        <div className="text-center space-y-2">
          <h2 className="text-2xl font-semibold text-text-primary">
            {t('connectionError.title')}
          </h2>
          <p className="text-text-secondary">
            {error || t('connectionError.defaultMessage')}
          </p>
        </div>

        {/* 恢复选项 */}
        <div className="space-y-3">
          <div className="text-sm text-text-tertiary p-4 bg-bg-deep rounded-lg border border-border">
            <p className="font-medium mb-2">{t('connectionError.possibleCauses')}</p>
            <ul className="space-y-1 text-xs">
              <li>• {t('connectionError.portOccupied')}</li>
              <li>• {t('connectionError.noPermission')}</li>
              <li>• {t('connectionError.deviceDisconnected')}</li>
              <li>• {t('connectionError.wrongConfig')}</li>
            </ul>
          </div>

          <div className="flex gap-3">
            <button
              onClick={() => disconnect()}
              className="flex-1 px-4 py-2 bg-signal text-black rounded-md hover:bg-signal/90 font-medium transition-colors"
            >
              {portName ? t('connection.reconnect') : t('connection.disconnect')}
            </button>
            <button
              onClick={() => disconnect()}
              className="flex-1 px-4 py-2 bg-bg-base text-text-primary rounded-md hover:bg-bg-elevated border border-border font-medium transition-colors"
            >
              {t('connection.returnHome')}
            </button>
          </div>
        </div>
      </div>
    </div>
  )
}
