import { useConnectionStore } from '@/stores'

/**
 * ErrorState - 错误状态
 *
 * 显示错误信息和恢复选项
 */
export function ErrorState() {
  const { error, disconnect } = useConnectionStore()

  return (
    <div className="flex-1 flex items-center justify-center p-8">
      <div className="max-w-md w-full space-y-6">
        {/* 错误图标 */}
        <div className="flex justify-center">
          <div className="w-16 h-16 rounded-full bg-alert/10 border-2 border-alert/30 flex items-center justify-center">
            <svg
              className="w-8 h-8 text-alert"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
              />
            </svg>
          </div>
        </div>

        {/* 错误消息 */}
        <div className="text-center space-y-2">
          <h2 className="text-2xl font-semibold text-text-primary">
            连接失败
          </h2>
          <p className="text-text-secondary">
            {error || '无法连接到串口设备'}
          </p>
        </div>

        {/* 恢复选项 */}
        <div className="space-y-3">
          <div className="text-sm text-text-tertiary p-4 bg-bg-deep rounded-lg border border-border">
            <p className="font-medium mb-2">可能的原因：</p>
            <ul className="space-y-1 text-xs">
              <li>• 串口已被其他程序占用</li>
              <li>• 没有访问权限</li>
              <li>• 设备未连接或已被移除</li>
              <li>• 配置参数不正确</li>
            </ul>
          </div>

          <div className="flex gap-3">
            <button
              onClick={() => window.location.reload()}
              className="flex-1 px-4 py-2 bg-signal text-black rounded-md hover:bg-signal/90 font-medium transition-colors"
            >
              重新连接
            </button>
            <button
              onClick={disconnect}
              className="flex-1 px-4 py-2 bg-bg-base text-text-primary rounded-md hover:bg-bg-elevated border border-border font-medium transition-colors"
            >
              返回首页
            </button>
          </div>
        </div>
      </div>
    </div>
  )
}
