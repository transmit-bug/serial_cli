-- Modbus ASCII Protocol Implementation
-- 基于 Modbus ASCII 标准的编解码协议
-- 帧格式: : [地址] [功能码] [数据...] [LRC] CR LF
-- 编码: 每个字节转换为 2 个 HEX 字符

SCRIPT_META = {
    name = "modbus_ascii",
    version = "1.0.0",
    description = "Modbus ASCII protocol (industrial communication)",
    author = "serial_cli",
    data_format = "binary",
    min_frame_size = 2,
    tags = {"modbus", "ascii", "industrial"},
}

-- 计算 LRC (Longitudinal Redundancy Check)
local function calculate_lrc(data)
    local lrc = 0
    for i = 1, #data do
        lrc = lrc + data[i]
    end
    -- LRC 是补码的低 8 位
    lrc = (256 - (lrc % 256)) % 256
    return lrc
end

-- 字节数组转 HEX 字符串
local function bytes_to_hex(data)
    local hex = ""
    for i = 1, #data do
        hex = hex .. string.format("%02X", data[i])
    end
    return hex
end

-- HEX 字符串转字节数组
local function hex_to_bytes(hex)
    local bytes = {}
    -- 移除可能的空格和换行符
    hex = hex:gsub("[%s\r\n:]", "")

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

-- 发送回调: 将二进制数据编码为 Modbus ASCII 格式
function on_send(data)
    if type(data) ~= "table" then
        return nil
    end

    -- 计算 LRC
    local lrc = calculate_lrc(data)

    -- 添加 LRC 到数据末尾
    local data_with_lrc = {}
    for i = 1, #data do
        data_with_lrc[i] = data[i]
    end
    data_with_lrc[#data + 1] = lrc

    -- 转换为 HEX 字符串并添加帧边界
    local hex_str = ":" .. bytes_to_hex(data_with_lrc) .. "\r\n"

    -- 转换为字节数组（ASCII 编码）
    local result = {}
    for i = 1, #hex_str do
        result[i] = string.byte(hex_str, i)
    end

    return result
end

-- 接收回调: 将 Modbus ASCII 数据解码为二进制
function on_recv(data)
    if type(data) ~= "table" then
        return nil
    end

    -- 将字节数组转换为字符串
    local str = ""
    for i = 1, #data do
        str = str .. string.char(data[i])
    end

    -- 移除帧边界（: 和 CR LF）
    str = str:gsub("^:", ""):gsub("\r?\n?$", "")

    -- 转换为字节数组
    local bytes = hex_to_bytes(str)

    if #bytes < 2 then
        return nil  -- 数据太短，无法包含 LRC
    end

    -- 分离数据和 LRC
    local data_bytes = {}
    for i = 1, #bytes - 1 do
        data_bytes[i] = bytes[i]
    end
    local received_lrc = bytes[#bytes]

    -- 验证 LRC
    local calculated_lrc = calculate_lrc(data_bytes)
    if received_lrc ~= calculated_lrc then
        return nil  -- LRC 校验失败
    end

    return data_bytes
end
