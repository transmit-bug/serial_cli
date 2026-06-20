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
    name: "modbus_ascii",
    type: "protocol",
    labelKey: "protocolTemplates.modbusAscii",
    descriptionKey: "protocolTemplates.modbusAsciiDesc",
    content: `-- Modbus ASCII Protocol
-- Frame format: : [ADDR] [FUNC] [DATA...] [LRC] CR LF
-- Each byte is encoded as 2 HEX ASCII characters

local function calculate_lrc(data)
  local lrc = 0
  for i = 1, #data do
    lrc = lrc + data:byte(i)
  end
  -- LRC is the two's complement of the sum
  lrc = (256 - (lrc % 256)) % 256
  return lrc
end

local function bytes_to_hex(data)
  local hex = ""
  for i = 1, #data do
    hex = hex .. string.format("%02X", data:byte(i))
  end
  return hex
end

local function hex_to_bytes(hex)
  local bytes = {}
  -- Remove colons, spaces, and line endings
  hex = hex:gsub("[:%s%z]", ""):gsub("\\r", ""):gsub("\\n", "")
  
  for i = 1, #hex, 2 do
    local byte_str = hex:sub(i, i + 1)
    if #byte_str == 2 then
      local byte = tonumber(byte_str, 16)
      if byte then
        table.insert(bytes, byte)
      end
    end
  end
  return bytes
end

function on_encode(data)
  -- data: table with addr, func, and data fields
  local addr = data.addr or 0x01
  local func = data.func or 0x03
  local payload = data.data or {}
  
  -- Build the frame: ADDR + FUNC + DATA
  local frame = string.char(addr, func)
  for i = 1, #payload do
    frame = frame .. string.char(payload[i])
  end
  
  -- Calculate and append LRC
  local lrc = calculate_lrc(frame)
  frame = frame .. string.char(lrc)
  
  -- Convert to ASCII hex with frame delimiters
  return ":" .. bytes_to_hex(frame) .. "\\r\\n"
end

function on_frame(data)
  -- Remove frame delimiters
  local str = tostring(data)
  str = str:gsub("^:", ""):gsub("\\r?\\n?$", "")
  
  -- Convert to bytes
  local bytes = hex_to_bytes(str)
  
  if #bytes < 2 then
    return { error = "Frame too short" }
  end
  
  -- Verify LRC
  local data_bytes = string.sub(str, 1, #str - 2)
  local received_lrc = bytes[#bytes]
  local calculated_lrc = calculate_lrc(string.sub(data, 1, #data - 2))
  
  if received_lrc ~= calculated_lrc then
    return { error = "LRC checksum mismatch" }
  end
  
  -- Parse Modbus frame
  local addr = bytes[1]
  local func = bytes[2]
  
  -- Check for error response (function code + 0x80)
  if func >= 0x80 then
    return {
      error = true,
      addr = addr,
      func = func - 0x80,
      exception = bytes[3],
    }
  end
  
  -- Normal response
  local result = {
    addr = addr,
    func = func,
    data = {},
  }
  
  -- Extract data bytes (excluding addr, func, and LRC)
  for i = 3, #bytes - 1 do
    table.insert(result.data, bytes[i])
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
