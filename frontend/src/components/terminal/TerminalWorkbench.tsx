import { useConnectionStore } from '@/stores'
import { DisconnectedState } from './DisconnectedState'
import { ConnectedState } from './ConnectedState'
import { ErrorState } from './ErrorState'

/**
 * TerminalWorkbench - 主终端工作台
 *
 * 状态驱动的布局：
 * - 未连接：显示端口选择和快速连接向导
 * - 已连接：显示完整的工作台（RX/TX/辅助面板）
 * - 错误：显示错误信息和恢复选项
 */
export function TerminalWorkbench() {
  const { status } = useConnectionStore()

  return (
    <div className="terminal-workbench h-full flex flex-col">
      {status === 'disconnected' && <DisconnectedState />}
      {status === 'connected' && <ConnectedState />}
      {status === 'connecting' && <ConnectingState />}
      {status === 'error' && <ErrorState />}
    </div>
  )
}

function ConnectingState() {
  return (
    <div className="flex-1 flex items-center justify-center">
      <div className="text-center">
        <div className="inline-flex items-center gap-3 px-6 py-4 rounded-lg bg-bg-elevated border border-amber/30">
          <div className="w-5 h-5 border-2 border-amber border-t-transparent rounded-full animate-spin" />
          <div className="text-amber font-medium">正在连接串口...</div>
        </div>
      </div>
    </div>
  )
}
