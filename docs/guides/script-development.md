# Serial CLI 脚本引擎 — 协议脚本开发指南

本文档面向有 Lua 基础的第三方开发者，介绍如何编写 Serial CLI 的协议脚本。

## 概述

Serial CLI 的脚本引擎允许你用 Lua 编写**自包含的串口协议驱动**。一个脚本可以：

- **帧编解码** — 在发送/接收数据时添加/校验协议帧头、CRC、分隔符等
- **设备交互** — 通过 `serial_query` 实现完整的请求-响应业务逻辑
- **定时任务** — 通过 `on_timer` 实现心跳包、周期性状态查询
- **UI 集成** — 通过 `action_*` 函数暴露可调用的设备操作

## 快速开始

### 最小可运行脚本

```lua
-- echo.lua: 回显协议，原样返回收到的数据

SCRIPT_META = {
    name = "echo",
    version = "1.0.0",
    description = "Echo protocol - returns received data as-is",
}

function on_send(data)
    return data
end

function on_recv(data)
    return data
end
```

### 加载和运行

```bash
# 加载脚本
serial-cli script load echo.lua

# 列出已加载脚本
serial-cli script list

# 使用脚本打开端口
serial-cli open /dev/ttyUSB0 --script echo
```

## SCRIPT_META（可选元数据）

脚本通过定义全局 Lua table `SCRIPT_META` 来声明元数据。该 table **完全可选**，缺失不影响脚本运行。

```lua
SCRIPT_META = {
    -- 基本信息
    name = "modbus_rtu",           -- 唯一标识符
    version = "1.2.0",             -- semver 格式
    description = "Modbus RTU protocol",
    author = "serial_cli",         -- 作者
    license = "MIT",               -- 许可证
    homepage = "https://github.com/...",  -- 详情页

    -- 分类
    tags = {"modbus", "industrial"},  -- 用于搜索/分类

    -- 协议特征（仅供参考，不参与引擎逻辑）
    data_format = "binary",        -- "binary" 或 "text"
    min_frame_size = 4,            -- 最小完整帧字节数
}
```

### 字段说明

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `name` | string | 否 | 唯一标识符，同名覆盖 |
| `version` | string | 否 | semver 格式版本号 |
| `description` | string | 否 | 人类可读描述 |
| `author` | string | 否 | 作者名称 |
| `license` | string | 否 | 许可证标识 |
| `homepage` | string | 否 | 详情页 URL |
| `tags` | string[] | 否 | 用于搜索和分类的标签列表 |
| `data_format` | string | 否 | `"binary"` 或 `"text"` |
| `min_frame_size` | number | 否 | 最小完整帧字节数（binary 模式） |

### 元数据的使用

- `description` 会替代脚本文件名作为默认描述显示
- `name` 和 `version` 为未来网络获取脚本预留扩展点
- `tags` 用于搜索和筛选脚本

## 生命周期回调

脚本可以定义以下回调函数，引擎会在相应事件触发时调用：

### on_open(port, config)

端口打开时触发。

```lua
function on_open(port, config)
    log_info("Port opened: " .. port)
    log_info("Baudrate: " .. config.baudrate)
    log_info("Data bits: " .. config.databits)
end
```

**参数：**
- `port` (string): 端口名称（如 `/dev/ttyUSB0`）
- `config` (table): 端口配置
  - `baudrate` (number): 波特率
  - `databits` (number): 数据位
  - `stopbits` (number): 停止位
  - `timeout_ms` (number): 超时时间（毫秒）

**返回值：** 忽略

### on_send(data)

发送数据时触发。用于添加协议帧头、CRC 等。

```lua
function on_send(data)
    -- data 是 byte table: {0x01, 0x03, 0x00, 0x00, 0x00, 0x01}
    local crc = calculate_crc(data)
    table.insert(data, crc & 0xFF)
    table.insert(data, (crc >> 8) & 0xFF)
    return data
end
```

**参数：**
- `data` (table): 字节数组，如 `{0x01, 0x02, 0x03}`

**返回值：**
- 字节数组 (table): 处理后的数据
- `nil`: 拦截发送（不发送任何数据）

### on_recv(data)

接收数据时触发。用于帧解析、校验、缓冲。

```lua
local buffer = {}

function on_recv(data)
    -- 累积数据到缓冲区
    for _, b in ipairs(data) do
        table.insert(buffer, b)
    end

    -- 检查是否收到完整帧
    if #buffer < 4 then
        return nil  -- 数据不足，等待更多数据
    end

    -- 验证 CRC
    if not verify_crc(buffer) then
        log_warn("CRC mismatch")
        buffer = {}  -- 清空缓冲区
        return nil   -- 返回 nil 抑制输出
    end

    -- 提取有效数据
    local payload = extract_payload(buffer)
    buffer = {}  -- 清空缓冲区
    return payload
end
```

**参数：**
- `data` (table): 字节数组

**返回值：**
- 字节数组 (table): 处理后的数据（传递给上层）
- `nil`: 抑制输出（数据被缓冲或丢弃）

**重要：** `on_recv` 的返回值决定了 `serial_query` 是否认为收到了完整响应。当 `on_recv` 返回非 nil 时，`serial_query` 会将该数据作为响应返回。

### on_close()

端口关闭时触发。

```lua
function on_close()
    log_info("Port closed")
    -- 清理资源
end
```

**参数：** 无

**返回值：** 忽略

### on_timer()

定时器回调。返回下一次触发的间隔（毫秒）。

```lua
local heartbeat_counter = 0

function on_timer()
    heartbeat_counter = heartbeat_counter + 1
    log_debug("Heartbeat #" .. heartbeat_counter)

    -- 发送心跳包
    serial_send("port_id", {0xFF, 0x01})

    -- 返回下一次间隔（毫秒），返回 0 停止定时器
    return 1000  -- 每秒触发一次
end
```

**参数：** 无

**返回值：**
- number: 下一次触发间隔（毫秒）
- 0: 停止定时器

## I/O 接口

### serial_open(port, config) -> port_id

打开串口。

```lua
local port_id = serial_open("/dev/ttyUSB0", {
    baudrate = 115200,
    data_bits = 8,
    stop_bits = 1,
    parity = "none",
    timeout = 1000,
    flow_control = "none",
    dtr_enable = true,
    rts_enable = true,
})
```

**参数：**
- `port` (string): 端口路径（如 `/dev/ttyUSB0`、`COM3`）
- `config` (table): 端口配置（所有字段可选，有默认值）

**返回值：** port_id (string)

**错误：** 端口不存在或已被占用时抛出错误

### serial_close(port_id)

关闭串口。

```lua
serial_close(port_id)
```

### serial_send(port_id, data)

发送数据（单向，不等待响应）。

```lua
serial_send(port_id, {0x01, 0x03, 0x00, 0x00, 0x00, 0x01})
```

**参数：**
- `port_id` (string): 端口 ID
- `data` (table): 字节数组

### serial_recv(port_id, timeout_ms) -> data | nil

接收数据（单向）。

```lua
local data = serial_recv(port_id, 1000)
if data then
    log_info("Received: " .. bytes_to_hex(data))
else
    log_warn("Receive timeout")
end
```

**参数：**
- `port_id` (string): 端口 ID
- `timeout_ms` (number): 超时时间（毫秒）

**返回值：** 字节数组 (table) 或 `nil`（超时）

### serial_query(port_id, data, timeout_ms) -> response | nil

发送请求并等待完整响应。这是最常用的请求-响应模式。

```lua
-- 读取 Modbus 寄存器
local request = {0x01, 0x03, 0x00, 0x00, 0x00, 0x01}
local response = serial_query(port_id, request, 1000)

if response then
    local value = (response[4] << 8) | response[5]
    log_info("Register value: " .. value)
else
    log_error("Device not responding")
end
```

**参数：**
- `port_id` (string): 端口 ID
- `data` (table): 请求数据（字节数组）
- `timeout_ms` (number): 超时时间（毫秒）

**返回值：**
- 字节数组 (table): 完整的响应帧
- `nil`: 超时未收到完整响应

**工作原理：**
1. 发送 `data`（通过 `on_send` 处理）
2. 循环读取接收数据
3. 每个数据块传入 `on_recv` 回调
4. 当 `on_recv` 返回非 nil 时，返回该数据
5. 超时返回 nil

### serial_flush(port_id)

清空接收缓冲区。用于错误恢复。

```lua
-- 超时后清空缓冲区再重试
local response = serial_query(port_id, request, 1000)
if not response then
    serial_flush(port_id)  -- 清掉残余字节
    response = serial_query(port_id, request, 1000)  -- 重试
end
```

### serial_list() -> ports

列出可用串口。

```lua
local ports = serial_list()
for _, port in ipairs(ports) do
    log_info(port.port_name .. " (" .. port.port_type .. ")")
end
```

## 工具函数

### 日志函数

```lua
log_info("Informational message")
log_debug("Debug message")
log_warn("Warning message")
log_error("Error message")
```

### JSON 函数

```lua
-- 编码
local json_str = json_encode({name = "device", value = 42})
local pretty = json_encode_pretty({name = "device", value = 42})

-- 解码
local obj = json_decode('{"name":"device","value":42}')
```

### Hex 函数

```lua
-- 字节数组转 hex 字符串
local hex = hex_encode({0xDE, 0xAD, 0xBE, 0xEF})
-- 结果: "deadbeef"

-- hex 字符串转字节数组
local bytes = hex_decode("deadbeef")
-- 结果: {0xDE, 0xAD, 0xBE, 0xEF}
```

### 字符串/字节转换

```lua
-- 字节数组转字符串
local str = bytes_to_string({0x48, 0x65, 0x6C, 0x6C, 0x6F})
-- 结果: "Hello"

-- 字符串转字节数组
local bytes = string_to_bytes("Hello")
-- 结果: {0x48, 0x65, 0x6C, 0x6C, 0x6F}

-- 字节数组转 hex 字符串
local hex = bytes_to_hex({0xDE, 0xAD})
-- 结果: "dead"
```

### 时间函数

```lua
-- 暂停指定毫秒
sleep_ms(100)

-- 获取当前时间戳（秒）
local now = time_now()
```

## UI Actions

脚本中以 `action_` 前缀命名的函数会被自动发现，暴露为宿主可调用的业务操作。

```lua
-- 定义一个读取寄存器的操作
function action_read_register(slave_id, register_addr)
    local request = build_read_request(slave_id, 0x03, register_addr, 1)
    local response = serial_query(port_id, request, 1000)

    if not response then
        return { _error = "timeout" }
    end

    if is_exception(response) then
        return { _error = "device_error", code = response[3] }
    end

    local value = (response[4] << 8) | response[5]
    return { value = value }
end

-- 定义一个写入寄存器的操作
function action_write_register(slave_id, register_addr, value)
    local request = build_write_request(slave_id, 0x06, register_addr, value)
    local response = serial_query(port_id, request, 1000)

    if not response then
        return { _error = "timeout" }
    end

    return { success = true }
end
```

**发现机制：** 宿主通过 `discover_actions()` 扫描所有 `action_*` 函数，获取函数名和参数信息。

## 错误处理约定

### 帧级错误（可恢复）

当收到损坏的帧（CRC 校验失败、帧太短等），记录警告并返回 nil：

```lua
function on_recv(data)
    if #data < 4 then
        log_warn("Frame too short: " .. #data .. " bytes")
        return nil
    end

    if not verify_crc(data) then
        log_warn("CRC mismatch")
        return nil
    end

    return extract_payload(data)
end
```

### 业务级错误（需暴露给调用方）

当设备返回错误码或业务逻辑失败时，返回带 `_error` 字段的 table：

```lua
function action_read_register(addr)
    local resp = serial_query(port_id, request, 1000)

    if not resp then
        return { _error = "timeout" }
    end

    if is_modbus_exception(resp) then
        return {
            _error = "modbus_exception",
            code = resp[3],
            message = exception_code_to_string(resp[3])
        }
    end

    return { value = parse_register_value(resp) }
end
```

### 致命错误（脚本 bug）

当遇到不应该发生的错误时，直接调用 `error()`：

```lua
function on_send(data)
    if type(data) ~= "table" then
        error("on_send: expected table, got " .. type(data))
    end
    -- ...
end
```

## 测试

使用 `script_encode` 和 `script_decode` 函数测试脚本逻辑，无需连接真实设备。

### 测试帧编解码

```lua
-- 测试 on_send 编码
local encoded = script_encode("modbus_rtu", {0x01, 0x03, 0x00, 0x00, 0x00, 0x01})
assert(#encoded == 8, "Expected 8 bytes (6 data + 2 CRC)")

-- 测试 on_recv 解码
local decoded = script_decode("modbus_rtu", encoded)
assert(decoded ~= nil, "Roundtrip failed")
assert(decoded[1] == 0x01, "Slave ID mismatch")
```

### 测试错误处理

```lua
-- 测试无效输入
local ok, err = pcall(script_encode, "modbus_rtu", "invalid")
assert(not ok, "Expected error for invalid input")
```

### 测试内置脚本

```lua
-- 测试 line 协议
local encoded = script_encode("line", "Hello")
assert(type(encoded) == "string")

-- 测试 AT 命令协议
local encoded = script_encode("at_command", "ATZ")
assert(type(encoded) == "string")
assert(encoded:match("\r$"), "AT command should end with \\r")

-- 测试 Modbus RTU 协议
local encoded = script_encode("modbus_rtu", "010300000001")
assert(type(encoded) == "string")
assert(#encoded == 16, "Expected 8 bytes (16 hex chars)")
```

## 内置脚本参考

Serial CLI 内置了 4 个协议脚本，可作为编写自定义脚本的参考模板：

### line

行协议，基于换行符分隔的文本通信。

```lua
SCRIPT_META = {
    name = "line",
    description = "Line-based protocol (text-based communication)",
    data_format = "text",
}
```

**特点：**
- `on_send`: 自动添加换行符
- `on_recv`: 直接透传数据

### at_command

AT 命令协议，用于调制解调器控制。

```lua
SCRIPT_META = {
    name = "at_command",
    description = "AT Command protocol (modem control)",
    data_format = "text",
}
```

**特点：**
- `on_send`: 确保命令以 `\r` 结尾
- `on_recv`: 缓冲响应直到收到 `OK`、`ERROR`、`+CME` 或 `+CMS`

### modbus_rtu

Modbus RTU 协议，工业通信标准。

```lua
SCRIPT_META = {
    name = "modbus_rtu",
    description = "Modbus RTU protocol (industrial communication)",
    data_format = "binary",
    min_frame_size = 4,
}
```

**特点：**
- `on_send`: 添加 CRC16 校验
- `on_recv`: 帧缓冲、CRC 验证、自动识别响应长度
- 使用 `frame_buffer` 累积不完整的帧

### modbus_ascii

Modbus ASCII 协议，工业通信标准的 ASCII 变体。

```lua
SCRIPT_META = {
    name = "modbus_ascii",
    description = "Modbus ASCII protocol (industrial communication)",
    data_format = "binary",
    min_frame_size = 2,
}
```

**特点：**
- `on_send`: 转换为 ASCII HEX 格式，添加 LRC 校验
- `on_recv`: 从 ASCII HEX 解码，验证 LRC

## 附录 A: API 速查表

### 生命周期回调

| 回调 | 输入 | 返回值 | 触发时机 |
|------|------|--------|----------|
| `on_open(port, config)` | string, table | 忽略 | 端口打开 |
| `on_send(data)` | byte table | byte table / nil | 发送数据 |
| `on_recv(data)` | byte table | byte table / nil | 接收数据 |
| `on_close()` | 无 | 忽略 | 端口关闭 |
| `on_timer()` | 无 | interval_ms | 定时触发 |

### I/O 接口

| 函数 | 参数 | 返回值 | 说明 |
|------|------|--------|------|
| `serial_open(port, config)` | string, table | port_id | 打开端口 |
| `serial_close(port_id)` | string | 无 | 关闭端口 |
| `serial_send(port_id, data)` | string, table | 无 | 单向发送 |
| `serial_recv(port_id, timeout)` | string, number | table / nil | 单向接收 |
| `serial_query(port_id, data, timeout)` | string, table, number | table / nil | 请求-响应 |
| `serial_flush(port_id)` | string | 无 | 清空缓冲区 |
| `serial_list()` | 无 | table | 列出端口 |

### 工具函数

| 函数 | 说明 |
|------|------|
| `log_info(msg)` | 记录信息日志 |
| `log_debug(msg)` | 记录调试日志 |
| `log_warn(msg)` | 记录警告日志 |
| `log_error(msg)` | 记录错误日志 |
| `json_encode(obj)` | JSON 编码 |
| `json_encode_pretty(obj)` | JSON 编码（美化） |
| `json_decode(str)` | JSON 解码 |
| `hex_encode(bytes)` | 字节数组转 hex |
| `hex_decode(hex_str)` | hex 转字节数组 |
| `bytes_to_string(bytes)` | 字节数组转字符串 |
| `string_to_bytes(str)` | 字符串转字节数组 |
| `bytes_to_hex(bytes)` | 字节数组转 hex 字符串 |
| `sleep_ms(ms)` | 暂停指定毫秒 |
| `time_now()` | 获取当前时间戳（秒） |

## 附录 B: 脚本模板

### 最小模板

```lua
SCRIPT_META = {
    name = "my_protocol",
    version = "1.0.0",
    description = "My custom protocol",
}

function on_send(data)
    -- 处理发送数据
    return data
end

function on_recv(data)
    -- 处理接收数据
    return data
end
```

### 完整模板（带业务逻辑）

```lua
SCRIPT_META = {
    name = "my_device",
    version = "1.0.0",
    description = "My device driver",
    author = "Your Name",
    data_format = "binary",
    tags = {"my_device", "custom"},
}

local port_id = nil
local frame_buffer = {}

-- 端口打开时初始化
function on_open(port, config)
    log_info("Port opened: " .. port)
    port_id = port  -- 保存端口 ID 用于 serial_query
end

-- 发送时添加协议帧
function on_send(data)
    -- 添加帧头
    local frame = {0xAA, 0x55}
    for _, b in ipairs(data) do
        table.insert(frame, b)
    end
    -- 添加校验和
    local checksum = 0
    for _, b in ipairs(frame) do
        checksum = (checksum + b) & 0xFF
    end
    table.insert(frame, checksum)
    return frame
end

-- 接收时解析帧
function on_recv(data)
    -- 累积到缓冲区
    for _, b in ipairs(data) do
        table.insert(frame_buffer, b)
    end

    -- 检查帧头
    if #frame_buffer < 3 then
        return nil
    end

    -- 查找帧头 0xAA 0x55
    local start = nil
    for i = 1, #frame_buffer - 1 do
        if frame_buffer[i] == 0xAA and frame_buffer[i + 1] == 0x55 then
            start = i
            break
        end
    end

    if not start then
        frame_buffer = {}  -- 丢弃无效数据
        return nil
    end

    -- 检查是否有足够数据
    if #frame_buffer < start + 2 then
        return nil
    end

    local payload_len = frame_buffer[start + 2]
    local frame_len = 3 + payload_len + 1  -- 头(3) + 数据(N) + 校验(1)

    if #frame_buffer < start + frame_len - 1 then
        return nil  -- 数据不足
    end

    -- 提取帧
    local frame = {}
    for i = start, start + frame_len - 1 do
        table.insert(frame, frame_buffer[i])
    end

    -- 验证校验和
    local checksum = 0
    for i = 1, #frame - 1 do
        checksum = (checksum + frame[i]) & 0xFF
    end
    if checksum ~= frame[#frame] then
        log_warn("Checksum mismatch")
        frame_buffer = {}
        return nil
    end

    -- 提取有效数据
    local payload = {}
    for i = 4, #frame - 1 do  -- 跳过帧头(3字节)和校验(1字节)
        table.insert(payload, frame[i])
    end

    frame_buffer = {}
    return payload
end

-- 端口关闭时清理
function on_close()
    log_info("Port closed")
    port_id = nil
    frame_buffer = {}
end

-- UI 操作：读取设备状态
function action_read_status()
    if not port_id then
        return { _error = "port_not_open" }
    end

    local request = {0x01, 0x00}  -- 读取状态命令
    local response = serial_query(port_id, request, 1000)

    if not response then
        return { _error = "timeout" }
    end

    return {
        status = response[1],
        value = response[2],
    }
end

-- UI 操作：发送自定义命令
function action_send_command(cmd_id, param)
    if not port_id then
        return { _error = "port_not_open" }
    end

    local request = {cmd_id, param}
    local response = serial_query(port_id, request, 1000)

    if not response then
        return { _error = "timeout" }
    end

    return { response = response }
end
```
