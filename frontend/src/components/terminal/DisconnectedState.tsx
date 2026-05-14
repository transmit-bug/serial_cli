import React, { useState } from 'react'
import { useConnectionStore, useNavigationStore } from '@/stores'
import { toast } from 'sonner'
import { usePorts } from '@/contexts/PortContext'
import { Card } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Plug, Settings, ArrowRight, Loader2, Network } from 'lucide-react'
import { useTranslation } from 'react-i18next'

/**
 * DisconnectedState - 未连接状态
 */
export function DisconnectedState() {
  const { availablePorts: ports, listPorts } = usePorts()
  const { connect } = useConnectionStore()
  const [connectingPort, setConnectingPort] = useState<string | null>(null)
  const { t } = useTranslation()

  // 初始加载端口列表
  React.useEffect(() => {
    listPorts()
  }, [])

  const handleConnect = async (portName: string, isVirtual: boolean) => {
    setConnectingPort(portName)
    try {
      const config = {
        baudrate: 115200,
        databits: 8,
        stopbits: 1,
        parity: 'none',
        timeout_ms: 1000,
        flow_control: 'none',
        dtr_enable: true,
        rts_enable: true,
      }
      await connect(portName, config, isVirtual)
    } catch (error) {
      console.error('Failed to connect:', error)
      toast.error(t('terminal.connectionFailed'))
    } finally {
      setConnectingPort(null)
    }
  }

  return (
    <div className="flex-1 flex items-center justify-center p-8">
      <div className="max-w-4xl w-full space-y-8">
        {/* 欢迎消息 */}
        <div className="text-center space-y-4">
          <div className="inline-flex items-center justify-center w-16 h-16 rounded-full bg-signal/10 border-2 border-signal/30 mb-4">
            <Plug className="w-8 h-8 text-signal" />
          </div>
          <h1 className="text-3xl font-display font-semibold text-text-primary">
            欢迎使用 Serial CLI
          </h1>
          <p className="text-text-secondary text-lg">
            选择一个串口开始通信
          </p>
        </div>

        {/* 端口列表 */}
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
          {ports.length === 0 ? (
            <Card className="col-span-full p-12 text-center border-dashed">
              <div className="flex flex-col items-center gap-4">
                <Plug className="w-12 h-12 text-text-tertiary" />
                <div className="space-y-2">
                  <p className="text-text-secondary">未检测到串口设备</p>
                  <p className="text-sm text-text-tertiary">
                    请确保设备已连接，然后点击刷新
                  </p>
                </div>
                <Button
                  variant="outline"
                  onClick={listPorts}
                  className="mt-4"
                >
                  <ArrowRight className="w-4 h-4 mr-2" />
                  刷新端口列表
                </Button>
              </div>
            </Card>
          ) : (
            ports.map((port) => (
              <Card
                key={port.port_name}
                className={`p-6 transition-colors cursor-pointer group ${
                  port.is_virtual
                    ? 'hover:border-info/50 border-info/20'
                    : 'hover:border-signal/50'
                }`}
                onClick={() => handleConnect(port.port_name, port.is_virtual || false)}
              >
                <div className="space-y-4">
                  <div className="flex items-start justify-between">
                    <div className="flex items-center gap-3">
                      <div className={`w-10 h-10 rounded-lg flex items-center justify-center transition-colors ${
                        port.is_virtual
                          ? 'bg-info/10 group-hover:bg-info/20'
                          : 'bg-bg-deep group-hover:bg-signal/10'
                      }`}>
                        {port.is_virtual ? (
                          <Network className="w-5 h-5 text-info group-hover:text-info/80 transition-colors" />
                        ) : (
                          <Plug className="w-5 h-5 text-text-tertiary group-hover:text-signal transition-colors" />
                        )}
                      </div>
                      <div>
                        <div className="flex items-center gap-2">
                          <h3 className={`font-medium group-hover:text-signal transition-colors ${
                            port.is_virtual ? 'text-info' : 'text-text-primary'
                          }`}>
                            {port.port_name}
                          </h3>
                          {port.is_virtual && (
                            <span className="text-[10px] px-1.5 py-0.5 rounded bg-info/10 text-info font-medium">
                              {t('terminal.virtualPort')}
                            </span>
                          )}
                        </div>
                        <p className="text-xs text-text-tertiary mt-1">
                          {port.port_type}
                        </p>
                      </div>
                    </div>
                  </div>
                  <Button
                    variant="signal"
                    size="sm"
                    className={`w-full ${
                      port.is_virtual ? 'bg-info/10 text-info border-info/30 hover:bg-info/20' : ''
                    }`}
                    disabled={connectingPort === port.port_name}
                    onClick={(e) => {
                      e.stopPropagation()
                      handleConnect(port.port_name, port.is_virtual || false)
                    }}
                  >
                    {connectingPort === port.port_name ? (
                      <>
                        <Loader2 className="w-4 h-4 mr-2 animate-spin" />
                        {t('connection.connecting')}
                      </>
                    ) : (
                      t('common.connect')
                    )}
                  </Button>
                </div>
              </Card>
            ))
          )}
        </div>

        {/* 底部操作 */}
        <div className="flex items-center justify-between pt-6 border-t border-border">
          <Button
            variant="ghost"
            onClick={() => {
              const { navigateTo } = useNavigationStore.getState()
              navigateTo('settings')
            }}
            className="text-text-secondary hover:text-text-primary"
          >
            <Settings className="w-4 h-4 mr-2" />
            配置默认设置
          </Button>
          <Button
            variant="outline"
            onClick={listPorts}
          >
            刷新端口列表
          </Button>
        </div>
      </div>
    </div>
  )
}
