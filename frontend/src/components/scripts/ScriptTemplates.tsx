import { useState } from "react";
import { useTranslation } from "react-i18next";

interface ScriptTemplate {
  name: string;
  labelKey: string;
  descriptionKey: string;
  content: string;
}

const TEMPLATES: ScriptTemplate[] = [
  {
    name: "basic_tx_rx",
    labelKey: "scriptTemplates.basicTxRx",
    descriptionKey: "scriptTemplates.basicTxRxDesc",
    content: `-- 基础收发脚本
-- Basic TX/RX script

function init()
  print("脚本初始化完成")
end

function on_data(data)
  -- 收到数据时的回调
  print("收到: " .. tostring(data))
end

function send_test()
  -- 发送测试数据
  local msg = "AT\\r\\n"
  serial:write(msg)
  print("已发送: " .. msg)
end

-- 入口
init()
`,
  },
  {
    name: "auto_responder",
    labelKey: "scriptTemplates.autoResponder",
    descriptionKey: "scriptTemplates.autoResponderDesc",
    content: `-- 自动应答脚本
-- Auto-responder script

local responses = {
  ["AT"] = "OK",
  ["AT+GMR"] = "SDK version:1.0",
  ["AT+CSQ"] = "+CSQ: 24,0",
}

function on_data(data)
  local str = tostring(data):gsub("\\r\\n", ""):gsub("\\r", ""):gsub("\\n", "")
  for pattern, response in pairs(responses) do
    if str:find(pattern) then
      serial:write(response .. "\\r\\n")
      print("应答 " .. pattern .. " -> " .. response)
      return
    end
  end
  print("未匹配: " .. str)
end

print("自动应答已启动")
`,
  },
  {
    name: "protocol_parser",
    labelKey: "scriptTemplates.protocolParser",
    descriptionKey: "scriptTemplates.protocolParserDesc",
    content: `-- 协议解析脚本
-- Protocol parser script (Modbus RTU example)

local function crc16(data)
  local crc = 0xFFFF
  for i = 1, #data do
    crc = crc ~ data:byte(i)
    for _ = 1, 8 do
      if crc % 2 == 1 then
        crc = crc >> 1
        crc = crc ~ 0xA001
      else
        crc = crc >> 1
      end
    end
  end
  return crc
end

function on_data(data)
  if #data < 4 then
    print("数据太短")
    return
  end

  local addr = data:byte(1)
  local func = data:byte(2)
  local len = #data - 2
  local payload = data:sub(3, len)

  print(string.format("地址=%d 功能=%02X 长度=%d", addr, func, len))
end

print("协议解析器已就绪")
`,
  },
  {
    name: "scheduled_task",
    labelKey: "scriptTemplates.scheduledTask",
    descriptionKey: "scriptTemplates.scheduledTaskDesc",
    content: `-- 定时任务脚本
-- Scheduled task script

local interval_ms = 5000  -- 5 秒
local counter = 0

function tick()
  counter = counter + 1
  local msg = string.format("心跳 #%d", counter)
  serial:write(msg .. "\\r\\n")
  print("发送: " .. msg)
end

-- 设置定时器
timer:start(interval_ms, tick)

print(string.format("定时任务已启动, 间隔 %dms", interval_ms))
`,
  },
  {
    name: "data_logger",
    labelKey: "scriptTemplates.dataLogger",
    descriptionKey: "scriptTemplates.dataLoggerDesc",
    content: `-- 数据记录脚本
-- Data logger script

local log_count = 0
local start_time = os.time()

function on_data(data)
  log_count = log_count + 1
  local elapsed = os.time() - start_time
  local ts = os.date("%Y-%m-%d %H:%M:%S")

  -- 打印带时间戳的日志
  print(string.format("[%s] #%d (+%ds): %s", ts, log_count, elapsed, tostring(data)))
end

function status()
  print(string.format("已记录 %d 条数据, 运行 %d 秒", log_count, os.time() - start_time))
end

print("数据记录器已启动")
`,
  },
];

interface ScriptTemplatesProps {
  onLoadTemplate: (content: string) => void;
}

export function ScriptTemplates({ onLoadTemplate }: ScriptTemplatesProps) {
  const { t } = useTranslation();
  const [expanded, setExpanded] = useState(false);

  return (
    <div className="border-b border-border">
      <button
        className="w-full flex items-center justify-between px-3 py-2 text-xs text-text-muted hover:text-text transition"
        onClick={() => setExpanded(!expanded)}
      >
        <span>{t("scriptTemplates.title")}</span>
        <span className="text-[10px]">{expanded ? "▾" : "▸"}</span>
      </button>

      {expanded && (
        <div className="px-3 pb-2 space-y-1">
          {TEMPLATES.map((tpl) => (
            <button
              key={tpl.name}
              className="w-full text-left px-2 py-1.5 rounded text-xs bg-surface hover:bg-surface-hover transition group"
              onClick={() => onLoadTemplate(tpl.content)}
            >
              <div className="font-medium">{t(tpl.labelKey)}</div>
              <div className="text-[10px] text-text-muted">
                {t(tpl.descriptionKey)}
              </div>
            </button>
          ))}
        </div>
      )}
    </div>
  );
}
