-- Modbus RTU protocol
-- Frame: [slave_id][function_code][data...][CRC16_lo][CRC16_hi]

SCRIPT_META = {
    name = "modbus_rtu",
    version = "1.0.0",
    description = "Modbus RTU protocol (industrial communication)",
    author = "serial_cli",
    data_format = "binary",
    min_frame_size = 4,
    tags = {"modbus", "rtu", "industrial"}
,
}

_actions = {
    read_coils = {
        label = "📡 读线圈",
        group = "Modbus",
        icon = "radio",
        params = {
            { name = "slave", type = "number", default = 1, label = "从站地址" },
            { name = "addr",  type = "number", default = 0, label = "起始地址" },
            { name = "count", type = "number", default = 1, label = "数量" },
        },
    },
    read_input_registers = {
        label = "📊 读输入寄存器",
        group = "Modbus",
        icon = "bar-chart",
        params = {
            { name = "slave", type = "number", default = 1, label = "从站地址" },
            { name = "addr",  type = "number", default = 0, label = "起始地址" },
            { name = "count", type = "number", default = 1, label = "数量" },
        },
    },
    read_holding_registers = {
        label = "📋 读保持寄存器",
        group = "Modbus",
        icon = "book-open",
        params = {
            { name = "slave", type = "number", default = 1, label = "从站地址" },
            { name = "addr",  type = "number", default = 0, label = "起始地址" },
            { name = "count", type = "number", default = 1, label = "数量" },
        },
    },
    write_single_register = {
        label = "✏️ 写单个寄存器",
        group = "Modbus",
        icon = "edit",
        confirm = true,
        params = {
            { name = "slave",  type = "number", default = 1, label = "从站地址" },
            { name = "addr",   type = "number", default = 0, label = "寄存器地址" },
            { name = "value",  type = "number",                label = "值" },
        },
    },
    write_multiple_registers = {
        label = "📝 写多个寄存器",
        group = "Modbus",
        icon = "edit-3",
        confirm = true,
        params = {
            { name = "slave",  type = "number", default = 1, label = "从站地址" },
            { name = "addr",   type = "number", default = 0, label = "起始地址" },
            { name = "values", type = "string",               label = "值 (逗号分隔)" },
        },
    },
}



local frame_buffer = {}

-- CRC16 for Modbus RTU (polynomial 0xA001)
local function crc16(data)
    local crc = 0xFFFF
    for _, byte in ipairs(data) do
        crc = bit.bxor(crc, byte)
        for _ = 1, 8 do
            if bit.band(crc, 1) == 1 then
                crc = bit.bxor(bit.rshift(crc, 1), 0xA001)
            else
                crc = bit.rshift(crc, 1)
            end
        end
    end
    return crc
end

function on_send(data)
    -- data = [slave_id][function_code][payload...]
    local crc = crc16(data)
    local lo = bit.band(crc, 0xFF)
    local hi = bit.band(bit.rshift(crc, 8), 0xFF)
    local frame = {}
    for _, b in ipairs(data) do
        table.insert(frame, b)
    end
    table.insert(frame, lo)
    table.insert(frame, hi)
    return frame
end

function on_recv(data)
    -- Accumulate bytes into buffer
    for _, b in ipairs(data) do
        table.insert(frame_buffer, b)
    end

    -- Minimum Modbus RTU frame: slave_id(1) + func(1) + CRC(2) = 4 bytes
    if #frame_buffer < 4 then
        return nil
    end

    -- Try to determine frame length from function code
    local func_code = frame_buffer[2]
    local frame_len

    -- Exception response: slave(1) + func(1|0x80) + exception(1) + CRC(2) = 5
    if bit.band(func_code, 0x80) == 0x80 then
        frame_len = 5
    elseif func_code == 0x03 or func_code == 0x04 then
        -- Read response: slave(1) + func(1) + byte_count(1) + data(N) + CRC(2)
        if #frame_buffer < 3 then return nil end
        local byte_count = frame_buffer[3]
        frame_len = 3 + byte_count + 2
    elseif func_code == 0x06 or func_code == 0x10 then
        -- Write response: fixed 8 bytes
        frame_len = 8
    elseif func_code == 0x01 or func_code == 0x02 then
        -- Read coils/discrete inputs: same as 0x03/0x04
        if #frame_buffer < 3 then return nil end
        local byte_count = frame_buffer[3]
        frame_len = 3 + byte_count + 2
    else
        -- Unknown function — try minimum frame
        frame_len = 4
    end

    -- Check if we have enough bytes
    if #frame_buffer < frame_len then
        return nil
    end

    -- Extract one frame
    local frame = {}
    for i = 1, frame_len do
        table.insert(frame, frame_buffer[i])
    end

    -- Remove consumed bytes from buffer
    local remaining = {}
    for i = frame_len + 1, #frame_buffer do
        table.insert(remaining, frame_buffer[i])
    end
    frame_buffer = remaining

    -- Verify CRC
    local payload = {}
    for i = 1, frame_len - 2 do
        table.insert(payload, frame[i])
    end
    local expected_crc = crc16(payload)
    local got_lo = frame[frame_len - 1]
    local got_hi = frame[frame_len]
    local got_crc = bit.bor(got_lo, bit.lshift(got_hi, 8))

    if expected_crc ~= got_crc then
        log_warn("Modbus CRC mismatch: expected " .. string.format("0x%04X", expected_crc) ..
                 " got " .. string.format("0x%04X", got_crc))
        return nil
    end

    -- Return payload without CRC
    return payload
end
