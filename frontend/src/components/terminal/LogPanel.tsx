import { useState } from 'react'
import { Card } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Minus, Maximize2, X } from 'lucide-react'

/**
 * LogPanel - 底部日志面板
 *
 * 特性：
 * - 可折叠
 * - 可调整高度
 * - 显示系统和调试日志
 */
export function LogPanel() {
  const [logs, setLogs] = useState<string[]>([])
  const [isExpanded, setIsExpanded] = useState(true)

  return (
    <div className={`${isExpanded ? 'h-48' : 'h-12'} transition-all duration-200`}>
      <Card className="h-full m-0 rounded-none border-t border-x-0 border-b-0">
        {/* 标题栏 */}
        <div className="flex items-center justify-between px-4 py-2 border-b border-border">
          <div className="flex items-center gap-2">
            <h3 className="text-sm font-medium text-text-primary">系统日志</h3>
            <span className="text-xs text-text-tertiary bg-bg-base px-2 py-0.5 rounded">
              {logs.length} 条
            </span>
          </div>

          <div className="flex items-center gap-1">
            <Button
              variant="ghost"
              size="sm"
              onClick={() => setIsExpanded(!isExpanded)}
            >
              {isExpanded ? <Minus className="w-4 h-4" /> : <Maximize2 className="w-4 h-4" />}
            </Button>
            <Button
              variant="ghost"
              size="sm"
              onClick={() => setLogs([])}
            >
              <X className="w-4 h-4" />
            </Button>
          </div>
        </div>

        {/* 日志内容 */}
        {isExpanded && (
          <div className="h-36 overflow-y-auto px-4 py-2">
            {logs.length === 0 ? (
              <div className="text-center text-text-tertiary text-xs py-4">
                暂无日志
              </div>
            ) : (
              <div className="space-y-1">
                {logs.map((log, index) => (
                  <div
                    key={index}
                    className="text-xs font-mono text-text-secondary py-0.5"
                  >
                    {log}
                  </div>
                ))}
              </div>
            )}
          </div>
        )}
      </Card>
    </div>
  )
}
