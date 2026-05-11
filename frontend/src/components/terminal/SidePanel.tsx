import { Card } from '@/components/ui/card'
import { useConnectionStore } from '@/stores'
import { Activity, Cpu, Zap, Settings } from 'lucide-react'

/**
 * SidePanel - 右侧辅助面板
 *
 * 包含：
 * - 端口详情卡片
 * - 数据统计卡片
 * - 协议控制卡片
 * - 快捷操作卡片
 */
export function SidePanel() {
  const { portName, config } = useConnectionStore()

  return (
    <div className="h-full overflow-y-auto bg-bg-deep">
      <div className="p-4 space-y-4">
        {/* 端口详情卡片 */}
        <Card className="p-4 border-border/50">
          <div className="flex items-center gap-2 mb-3">
            <Activity className="w-4 h-4 text-signal" />
            <h3 className="text-sm font-medium text-text-primary">端口详情</h3>
          </div>
          <div className="space-y-2 text-xs">
            <div className="flex justify-between">
              <span className="text-text-tertiary">端口</span>
              <span className="text-text-primary font-mono">{portName || '-'}</span>
            </div>
            <div className="flex justify-between">
              <span className="text-text-tertiary">波特率</span>
              <span className="text-text-primary font-mono">{config?.baudrate || '-'} bps</span>
            </div>
            <div className="flex justify-between">
              <span className="text-text-tertiary">数据位</span>
              <span className="text-text-primary font-mono">{config?.databits || '-'} bit</span>
            </div>
            <div className="flex justify-between">
              <span className="text-text-tertiary">停止位</span>
              <span className="text-text-primary font-mono">{config?.stopbits || '-'}</span>
            </div>
            <div className="flex justify-between">
              <span className="text-text-tertiary">校验位</span>
              <span className="text-text-primary font-mono">{config?.parity || '-'}</span>
            </div>
          </div>
        </Card>

        {/* 数据统计卡片 */}
        <Card className="p-4 border-border/50">
          <div className="flex items-center gap-2 mb-3">
            <Activity className="w-4 h-4 text-info" />
            <h3 className="text-sm font-medium text-text-primary">数据统计</h3>
          </div>
          <div className="space-y-2 text-xs">
            <div className="flex justify-between">
              <span className="text-text-tertiary">RX 包</span>
              <span className="text-text-primary font-mono">-</span>
            </div>
            <div className="flex justify-between">
              <span className="text-text-tertiary">TX 包</span>
              <span className="text-text-primary font-mono">-</span>
            </div>
            <div className="flex justify-between">
              <span className="text-text-tertiary">RX 字节</span>
              <span className="text-text-primary font-mono">-</span>
            </div>
            <div className="flex justify-between">
              <span className="text-text-tertiary">TX 字节</span>
              <span className="text-text-primary font-mono">-</span>
            </div>
          </div>
        </Card>

        {/* 协议控制卡片 */}
        <Card className="p-4 border-border/50">
          <div className="flex items-center gap-2 mb-3">
            <Cpu className="w-4 h-4 text-amber" />
            <h3 className="text-sm font-medium text-text-primary">协议控制</h3>
          </div>
          <div className="text-xs text-text-tertiary text-center py-2">
            无活动协议
          </div>
        </Card>

        {/* 快捷操作卡片 */}
        <Card className="p-4 border-border/50">
          <div className="flex items-center gap-2 mb-3">
            <Zap className="w-4 h-4 text-signal" />
            <h3 className="text-sm font-medium text-text-primary">快捷操作</h3>
          </div>
          <div className="space-y-2">
            <button className="w-full px-3 py-2 text-xs text-left text-text-secondary hover:text-text-primary hover:bg-bg-elevated rounded transition-colors">
              清空接收数据
            </button>
            <button className="w-full px-3 py-2 text-xs text-left text-text-secondary hover:text-text-primary hover:bg-bg-elevated rounded transition-colors">
              清空发送数据
            </button>
            <button className="w-full px-3 py-2 text-xs text-left text-text-secondary hover:text-text-primary hover:bg-bg-elevated rounded transition-colors">
              导出数据
            </button>
          </div>
        </Card>

        {/* 设置链接 */}
        <Card className="p-4 border-border/50">
          <div className="flex items-center gap-2">
            <Settings className="w-4 h-4 text-text-tertiary" />
            <button className="flex-1 text-left text-xs text-text-secondary hover:text-text-primary">
              打开设置
            </button>
          </div>
        </Card>
      </div>
    </div>
  )
}
