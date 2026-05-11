import { useDataStore, useConnectionStore } from '@/stores'
import { RxDataViewer } from './RxDataViewer'
import { TxSender } from './TxSender'
import { SidePanel } from './SidePanel'
import { LogPanel } from './LogPanel'

/**
 * ConnectedState - 已连接状态
 *
 * 显示完整的工作台：
 * - RX 数据显示区（60%）
 * - TX 发送区（30%）
 * - 右侧辅助面板（30%）
 * - 底部日志面板（可折叠）
 */
export function ConnectedState() {
  const { rxPackets } = useDataStore()
  const { portName, disconnect } = useConnectionStore()
  const showLogs = false

  return (
    <div className="flex-1 flex flex-col overflow-hidden">
      {/* 顶部连接状态栏 */}
      <div className="flex items-center justify-between px-4 py-2 bg-bg-deep border-b border-border">
        <div className="flex items-center gap-3">
          <div className="w-2 h-2 rounded-full bg-signal animate-pulse" />
          <div className="text-sm">
            <span className="text-text-tertiary">已连接: </span>
            <span className="text-text-primary font-medium ml-1">{portName}</span>
          </div>
        </div>
        <button
          onClick={disconnect}
          className="px-3 py-1.5 text-sm bg-alert/10 text-alert border border-alert/30 rounded hover:bg-alert/20 transition-colors"
        >
          断开连接
        </button>
      </div>

      {/* 主工作区 */}
      <div className="flex-1 flex overflow-hidden">
        {/* 左侧：RX + TX（70%） */}
        <div className="flex-1 flex flex-col overflow-hidden">
          {/* RX 数据显示区（60%） */}
          <div className="flex-1 border-r border-border">
            <RxDataViewer />
          </div>

          {/* TX 发送区（30%） */}
          <div className="h-1/3 border-t border-border">
            <TxSender />
          </div>
        </div>

        {/* 右侧：辅助面板（30%） */}
        <div className="w-80 border-l border-border">
          <SidePanel />
        </div>
      </div>

      {/* 底部日志面板（可折叠） */}
      {showLogs && (
        <div className="h-48 border-t border-border">
          <LogPanel />
        </div>
      )}
    </div>
  )
}
