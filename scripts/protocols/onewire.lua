-- 1-Wire over UART protocol (DS18B20, DS1822, etc.)
-- Uses UART-based 1-Wire bridge (e.g., DS2480B, or bit-bang via serial).
-- Command format: [CMD][DATA...]
-- Response format: [STATUS][DATA...]
--
-- Commands:
--   0x01                    = Reset (returns presence pulse)
--   0x02 + [ROM:8B]         = Select device by ROM code
--   0x03                    = Skip ROM (broadcast)
--   0x04 + [cmd] + [len]    = Write command, then read len bytes
--   0x05                    = Read ROM (single device on bus)
--   0x06                    = Search ROM (enumerate devices)

SCRIPT_META = {
    name = "onewire",
    version = "1.0.0",
    description = "1-Wire over UART bridge protocol (DS18B20, etc.)",
    author = "serial_cli",
    data_format = "binary",
    min_frame_size = 2,
    tags = {"1wire", "onewire", "ds18b20", "temperature", "uart"},
}

local frame_buffer = {}

-- Simple framing: [LEN:1B][CMD:1B][DATA...][CS:1B]
-- CS = XOR of CMD and DATA bytes

local function compute_cs(cmd, data)
    local cs = cmd
    for _, b in ipairs(data) do
        cs = bit.bxor(cs, b)
    end
    return cs
end

function on_send(data)
    if #data < 1 then
        log_warn("1-Wire on_send: empty command")
        return nil
    end

    local cmd = data[1]
    local payload = {}
    for i = 2, #data do
        table.insert(payload, data[i])
    end

    local len = #data
    local cs = compute_cs(cmd, payload)

    local frame = { len, cmd }
    for _, b in ipairs(payload) do
        table.insert(frame, b)
    end
    table.insert(frame, cs)
    return frame
end

function on_recv(data)
    for _, b in ipairs(data) do
        table.insert(frame_buffer, b)
    end

    while #frame_buffer >= 3 do
        local len = frame_buffer[1]
        local frame_len = len + 2  -- LEN + DATA + CS

        if #frame_buffer < frame_len then
            return nil
        end

        -- Extract frame
        local frame = {}
        for i = 1, frame_len do
            table.insert(frame, frame_buffer[i])
        end

        -- Remove consumed bytes
        local remaining = {}
        for i = frame_len + 1, #frame_buffer do
            table.insert(remaining, frame_buffer[i])
        end
        frame_buffer = remaining

        -- Verify CS
        local cs_data = {}
        for i = 2, frame_len - 1 do
            table.insert(cs_data, frame[i])
        end
        local expected_cs = 0
        for _, b in ipairs(cs_data) do
            expected_cs = bit.bxor(expected_cs, b)
        end

        if expected_cs == frame[frame_len] then
            -- Return status + data (skip LEN and CS)
            local result = {}
            for i = 2, frame_len - 1 do
                table.insert(result, frame[i])
            end
            return result
        else
            log_warn("1-Wire recv: CS mismatch")
        end
    end

    return nil
end

-- ── Actions ─────────────────────────────────────────────────────────────────

--- Build a temperature read command for DS18B20.
-- @param rom table|nil  8-byte ROM code, or nil for skip-ROM (single device)
-- @return table         Command bytes
function action_read_temp(rom)
    if rom then
        -- Select specific device, then issue Convert T (0x44) and Read Scratchpad (0xBE)
        return { 0x04, 0x55, rom[1], rom[2], rom[3], rom[4], rom[5], rom[6], rom[7], rom[8],
                 0x44, 0x00,  -- Convert T + 1 dummy read
                 0x04, 0x55, rom[1], rom[2], rom[3], rom[4], rom[5], rom[6], rom[7], rom[8],
                 0xBE, 0x09 } -- Read Scratchpad + 9 bytes
    else
        return { 0x03, 0x44, 0x00,  -- Skip ROM + Convert T + 1 dummy
                 0x03, 0xBE, 0x09 } -- Skip ROM + Read Scratchpad + 9 bytes
    end
end

--- Parse DS18B20 temperature from scratchpad data.
-- @param data table  Scratchpad bytes (9 bytes from Read Scratchpad)
-- @return table { temp_c, temp_f, raw, crc_ok }
function action_parse_temp(data)
    if not data or #data < 9 then
        return { _error = "need 9 bytes (scratchpad)" }
    end

    -- CRC8 check
    local crc = 0
    for i = 1, 8 do
        local b = data[i]
        for _ = 1, 8 do
            local mix = bit.band(bit.bxor(crc, b), 0x01)
            crc = bit.rshift(crc, 1)
            if mix ~= 0 then
                crc = bit.bxor(crc, 0x8C)  -- CRC8 polynomial
            end
            b = bit.rshift(b, 1)
        end
    end
    local crc_ok = (crc == data[9])

    -- Temperature: bytes 1 (LSB) and 2 (MSB), 12-bit resolution
    local raw = data[1] + bit.lshift(data[2], 8)
    -- Sign extend for negative temperatures
    if raw > 32767 then
        raw = raw - 65536
    end
    local temp_c = raw / 16.0
    local temp_f = temp_c * 9.0 / 5.0 + 32.0

    return {
        temp_c = temp_c,
        temp_f = temp_f,
        raw = raw,
        crc_ok = crc_ok,
    }
end

--- Build a Read ROM command (only works with single device on bus).
-- @return table  Command bytes
function action_read_rom()
    return { 0x05 }
end

--- Build a Search ROM command.
-- @return table  Command bytes
function action_search_rom()
    return { 0x06 }
end

--- Parse a ROM code from Read ROM response.
-- @param data table  8-byte ROM code
-- @return table { family, serial, crc, hex }
function action_parse_rom(data)
    if not data or #data < 8 then
        return { _error = "need 8 bytes (ROM code)" }
    end

    -- CRC8 of first 7 bytes
    local crc = 0
    for i = 1, 7 do
        local b = data[i]
        for _ = 1, 8 do
            local mix = bit.band(bit.bxor(crc, b), 0x01)
            crc = bit.rshift(crc, 1)
            if mix ~= 0 then
                crc = bit.bxor(crc, 0x8C)
            end
            b = bit.rshift(b, 1)
        end
    end

    local hex = ""
    for _, b in ipairs(data) do
        hex = hex .. string.format("%02X", b)
    end

    return {
        family = data[1],
        serial = { data[2], data[3], data[4], data[5], data[6], data[7] },
        crc = data[8],
        crc_ok = (crc == data[8]),
        hex = hex,
    }
end
