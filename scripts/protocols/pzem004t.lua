-- PZEM-004T protocol (power/energy monitoring module)
-- Modbus RTU variant with specific function codes.
-- Frame: [slave_addr:1B][func_code:1B][data...][CRC16:2B]
--
-- Function codes:
--   0x03 = Read holding registers (voltage, current, power, energy, frequency, PF)
--   0x04 = Read input registers
--   0x06 = Write single register (set address, set power alarm)
--   0x42 = Read energy reset counter (custom)

SCRIPT_META = {
    name = "pzem004t",
    version = "1.0.0",
    description = "PZEM-004T power/energy monitoring module (Modbus RTU variant)",
    author = "serial_cli",
    data_format = "binary",
    min_frame_size = 4,
    tags = {"pzem", "power", "energy", "modbus", "monitoring", "ac"}
,
}

_actions = {
    read_all = {
        label = "📊 读全部参数",
        group = "PZEM-004T",
        icon = "activity",
        params = {
            { name = "addr", type = "number", default = 248, label = "从站地址" },
        },
    },
    read_voltage = {
        label = "⚡ 读电压",
        group = "PZEM-004T",
        icon = "zap",
        params = {
            { name = "addr", type = "number", default = 248, label = "从站地址" },
        },
    },
    read_current = {
        label = "🔌 读电流",
        group = "PZEM-004T",
        icon = "activity",
        params = {
            { name = "addr", type = "number", default = 248, label = "从站地址" },
        },
    },
    read_power = {
        label = "💡 读功率",
        group = "PZEM-004T",
        icon = "power",
        params = {
            { name = "addr", type = "number", default = 248, label = "从站地址" },
        },
    },
    read_energy = {
        label = "🔋 读电能",
        group = "PZEM-004T",
        icon = "battery",
        params = {
            { name = "addr", type = "number", default = 248, label = "从站地址" },
        },
    },
    set_address = {
        label = "⚙️ 设置地址",
        group = "PZEM-004T",
        icon = "settings",
        confirm = true,
        params = {
            { name = "new_addr", type = "number", label = "新地址 (1-247)" },
        },
    },
}



local frame_buffer = {}

-- CRC16 for Modbus RTU (same as modbus_rtu)
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
    -- data = [slave_id][func_code][params...]
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
    for _, b in ipairs(data) do
        table.insert(frame_buffer, b)
    end

    -- Minimum frame: slave(1) + func(1) + CRC(2) = 4
    if #frame_buffer < 4 then
        return nil
    end

    local func_code = frame_buffer[2]
    local frame_len

    if bit.band(func_code, 0x80) == 0x80 then
        frame_len = 5  -- Exception: slave + func + exception + CRC(2)
    elseif func_code == 0x03 or func_code == 0x04 then
        if #frame_buffer < 3 then return nil end
        local byte_count = frame_buffer[3]
        frame_len = 3 + byte_count + 2
    elseif func_code == 0x06 or func_code == 0x42 then
        frame_len = 8  -- Fixed response
    else
        frame_len = 4
    end

    if #frame_buffer < frame_len then
        return nil
    end

    local frame = {}
    for i = 1, frame_len do
        table.insert(frame, frame_buffer[i])
    end

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
        log_warn("PZEM CRC mismatch: expected " .. string.format("0x%04X", expected_crc) ..
                 " got " .. string.format("0x%04X", got_crc))
        return nil
    end

    return payload
end

-- ── Actions ─────────────────────────────────────────────────────────────────

--- Build a read-all-registers command (voltage, current, power, energy, freq, PF).
-- @param addr number  Slave address (default 0xF8 for broadcast)
-- @return table       Command bytes
function action_read_all(addr)
    addr = addr or 0xF8
    -- Read holding registers 0x0000, count 0x000A (10 registers = 20 bytes)
    return { addr, 0x03, 0x00, 0x00, 0x00, 0x0A }
end

--- Build a read-voltage command.
-- @param addr number  Slave address
-- @return table       Command bytes
function action_read_voltage(addr)
    addr = addr or 0xF8
    return { addr, 0x03, 0x00, 0x00, 0x00, 0x01 }
end

--- Build a read-current command.
-- @param addr number  Slave address
-- @return table       Command bytes
function action_read_current(addr)
    addr = addr or 0xF8
    return { addr, 0x03, 0x00, 0x01, 0x00, 0x01 }
end

--- Build a read-power command.
-- @param addr number  Slave address
-- @return table       Command bytes
function action_read_power(addr)
    addr = addr or 0xF8
    return { addr, 0x03, 0x00, 0x03, 0x00, 0x02 }
end

--- Build a read-energy command.
-- @param addr number  Slave address
-- @return table       Command bytes
function action_read_energy(addr)
    addr = addr or 0xF8
    return { addr, 0x03, 0x00, 0x05, 0x00, 0x02 }
end

--- Build a set-address command.
-- @param new_addr number  New slave address (1-247)
-- @return table           Command bytes
function action_set_address(new_addr)
    return { 0xF8, 0x06, 0x00, 0x02, 0x00, new_addr }
end

--- Build a set-power-alarm command.
-- @param addr number    Slave address
-- @param watts number   Alarm threshold in watts
-- @return table         Command bytes
function action_set_power_alarm(addr, watts)
    addr = addr or 0xF8
    return { addr, 0x06, 0x00, 0x01, bit.band(bit.rshift(watts, 8), 0xFF), bit.band(watts, 0xFF) }
end

--- Parse a read-all response into human-readable values.
-- @param response table  Response from on_recv (25 bytes: slave + func + bytecount + 20 data + CRC)
-- @return table { voltage, current, power, energy, frequency, power_factor, alarm }
function action_parse_all(response)
    if not response or #response < 25 then
        return { _error = "response too short (need 25 bytes)" }
    end

    -- Data starts at byte 4 (skip slave + func + bytecount)
    local voltage = (response[4] * 256 + response[5]) * 0.1       -- V (0.1V resolution)
    local current = (response[6] * 256 + response[7]) * 0.001     -- A (1mA resolution)
    local power = (response[8] * 65536 + response[9] * 256 + response[10]) * 0.1  -- W
    local energy = (response[11] * 65536 + response[12] * 256 + response[13])      -- Wh
    local frequency = (response[14] * 256 + response[15]) * 0.1    -- Hz
    local power_factor = (response[16] * 256 + response[17]) * 0.01  -- PF
    local alarm = response[18] * 256 + response[19]                -- Alarm status

    return {
        voltage = voltage,
        current = current,
        power = power,
        energy_wh = energy,
        energy_kwh = energy / 1000.0,
        frequency = frequency,
        power_factor = power_factor,
        alarm = alarm,
        alarm_active = (alarm ~= 0),
    }
end

--- Parse an exception response.
-- @param response table  Exception response (5 bytes)
-- @return table { slave, code, message }
function action_parse_exception(response)
    if not response or #response < 3 then
        return { _error = "not an exception response" }
    end
    local codes = {
        [0x01] = "Illegal function",
        [0x02] = "Illegal data address",
        [0x03] = "Illegal data value",
        [0x04] = "Slave device failure",
    }
    local code = response[3]
    return {
        slave = response[1],
        code = code,
        message = codes[code] or "Unknown error",
    }
end
