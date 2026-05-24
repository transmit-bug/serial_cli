import { useState } from "react";
import { useTranslation } from "react-i18next";
import type { FileType } from "./EditorPage";

interface Template {
  name: string;
  labelKey: string;
  descriptionKey: string;
  type: FileType;
  content: string;
}

const TEMPLATES: Template[] = [
  // Script templates
  {
    name: "basic_tx_rx",
    type: "script",
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
    type: "script",
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
    type: "script",
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
    type: "script",
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
    type: "script",
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
  // Protocol templates
  {
    name: "modbus_rtu",
    type: "protocol",
    labelKey: "protocolTemplates.modbusRtu",
    descriptionKey: "protocolTemplates.modbusRtuDesc",
    content: `-- Modbus RTU Protocol
-- CRC16 implementation

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

function on_encode(data)
  -- data: { addr=1, func=3, start=0, count=1 }
  local frame = string.char(
    data.addr or 1,
    data.func or 3,
    (data.start or 0) >> 8,
    (data.start or 0) & 0xFF,
    (data.count or 1) >> 8,
    (data.count or 1) & 0xFF
  )
  local crc = crc16(frame)
  return frame .. string.char(crc & 0xFF, crc >> 8)
end

function on_frame(data)
  if #data < 4 then
    return { error = "Frame too short" }
  end

  local addr = data:byte(1)
  local func = data:byte(2)
  local result = { addr = addr, func = func }

  if func >= 0x80 then
    result.error = true
    result.exception = data:byte(3)
  elseif #data > 4 then
    local byte_count = data:byte(3)
    result.byte_count = byte_count
    result.payload = {}
    for i = 1, byte_count do
      result.payload[i] = data:byte(3 + i)
    end
  end

  return result
end
`,
  },
  {
    name: "custom_frame",
    type: "protocol",
    labelKey: "protocolTemplates.customFrame",
    descriptionKey: "protocolTemplates.customFrameDesc",
    content: `-- Custom Frame Protocol
-- Format: STX(0xAA) | LEN | CMD | DATA[...] | CHECKSUM | ETX(0x55)

local STX = 0xAA
local ETX = 0x55

local function checksum(data)
  local sum = 0
  for i = 1, #data do
    sum = (sum + data:byte(i)) & 0xFF
  end
  return sum ~ 0xFF
end

function on_encode(data)
  local cmd = data.cmd or 0x01
  local payload = data.payload or {}
  local payload_bytes = string.char(table.unpack(payload))

  local body = string.char(cmd) .. payload_bytes
  local len = #body
  local cksum = checksum(string.char(len) .. body)

  return string.char(STX, len) .. body .. string.char(cksum, ETX)
end

function on_frame(data)
  if #data < 4 then
    return { error = "Frame too short" }
  end
  if data:byte(1) ~= STX then
    return { error = "Invalid STX" }
  end
  if data:byte(#data) ~= ETX then
    return { error = "Invalid ETX" }
  end

  local len = data:byte(2)
  local cmd = data:byte(3)
  local payload = {}
  for i = 1, len - 1 do
    payload[i] = data:byte(3 + i)
  end

  local expected_cksum = checksum(data:sub(2, 2 + len))
  local actual_cksum = data:byte(3 + len)
  if expected_cksum ~= actual_cksum then
    return { error = "Checksum mismatch" }
  end

  return {
    cmd = cmd,
    payload = payload,
    length = len,
  }
end
`,
  },
  {
    name: "simple_at",
    type: "protocol",
    labelKey: "protocolTemplates.simpleAt",
    descriptionKey: "protocolTemplates.simpleAtDesc",
    content: `-- Simple AT Protocol
-- Line-based command/response protocol

function on_encode(data)
  local cmd = data.cmd or ""
  local suffix = data.suffix or "\\r\\n"
  return cmd .. suffix
end

function on_frame(data)
  local str = tostring(data)
  -- Remove trailing CRLF/LF
  str = str:gsub("\\r\\n$", ""):gsub("\\n$", ""):gsub("\\r$", "")

  local result = { raw = str }

  -- Parse AT response patterns
  if str:match("^%+") then
    result.type = "urc"
    result.key = str:match("^%+(%w+)")
    result.value = str:match("^%+%w+%s*:%s*(.+)$")
  elseif str == "OK" then
    result.type = "ok"
  elseif str == "ERROR" then
    result.type = "error"
  elseif str:match("^AT") then
    result.type = "command"
    result.cmd = str
  else
    result.type = "data"
  end

  return result
end
`,
  },
];

const SCRIPT_TEMPLATES = TEMPLATES.filter((t) => t.type === "script");
const PROTOCOL_TEMPLATES = TEMPLATES.filter((t) => t.type === "protocol");

interface TemplateListProps {
  onLoadTemplate: (content: string, type: FileType) => void;
}

export function TemplateList({ onLoadTemplate }: TemplateListProps) {
  const { t } = useTranslation();
  const [expandedScripts, setExpandedScripts] = useState(false);
  const [expandedProtocols, setExpandedProtocols] = useState(false);

  return (
    <div className="border-t border-border">
      {/* Script templates */}
      <button
        className="w-full flex items-center justify-between px-3 py-2 text-xs text-text-muted hover:text-text transition"
        onClick={() => setExpandedScripts(!expandedScripts)}
      >
        <span>{t("scriptTemplates.title")}</span>
        <span className="text-[10px]">{expandedScripts ? "▾" : "▸"}</span>
      </button>
      {expandedScripts && (
        <div className="px-3 pb-2 space-y-1">
          {SCRIPT_TEMPLATES.map((tpl) => (
            <button
              key={tpl.name}
              className="w-full text-left px-2 py-1.5 rounded text-xs bg-surface hover:bg-surface-hover transition"
              onClick={() => onLoadTemplate(tpl.content, "script")}
            >
              <div className="font-medium">{t(tpl.labelKey)}</div>
              <div className="text-[10px] text-text-muted">
                {t(tpl.descriptionKey)}
              </div>
            </button>
          ))}
        </div>
      )}

      {/* Protocol templates */}
      <button
        className="w-full flex items-center justify-between px-3 py-2 text-xs text-text-muted hover:text-text transition"
        onClick={() => setExpandedProtocols(!expandedProtocols)}
      >
        <span>{t("protocolTemplates.title")}</span>
        <span className="text-[10px]">{expandedProtocols ? "▾" : "▸"}</span>
      </button>
      {expandedProtocols && (
        <div className="px-3 pb-2 space-y-1">
          {PROTOCOL_TEMPLATES.map((tpl) => (
            <button
              key={tpl.name}
              className="w-full text-left px-2 py-1.5 rounded text-xs bg-surface hover:bg-surface-hover transition"
              onClick={() => onLoadTemplate(tpl.content, "protocol")}
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
