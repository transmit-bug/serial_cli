-- DL/T 645 protocol (Chinese smart electricity meter communication standard)
-- Frame format:
--   [0x68][Addr:6B][0x68][Ctrl:1B][Len:1B][Data:NB][CS:1B][0x16]
--
-- Addresses are 6 bytes (BCD-encoded meter number), padded with 0xAA if unused.
-- Data bytes are XOR-summed for CS.
-- The control byte encodes direction (bit 7) and function code (bits 0-5).
-- Data bytes are transmitted as (byte + 0x33) per the standard — we handle
-- encoding/decoding transparently in on_send/on_recv.

SCRIPT_META = {
    name = "dlt645",
    version = "1.0.0",
    description = "DL/T 645 Chinese smart electricity meter protocol",
    author = "serial_cli",
    data_format = "binary",
    min_frame_size = 12,
    tags = {"dlt645", "electricity", "meter", "smart-grid", "china", "gb"}
,
}

_actions = {
    read_data = {
        label = "📊 读数据",
        group = "DL/T 645",
        icon = "bar-chart",
        params = {
            { name = "addr", type = "hex", label = "表地址 (hex, 12位)" },
            { name = "di",   type = "hex", label = "数据标识 (hex, 8位)" },
        },
    },
    read_address = {
        label = "🔍 读表地址",
        group = "DL/T 645",
        icon = "search",
    },
    set_baudrate = {
        label = "⚙️ 设置波特率",
        group = "DL/T 645",
        icon = "settings",
        confirm = true,
        params = {
            { name = "addr",   type = "hex",    label = "表地址 (hex)" },
            { name = "baud",   type = "number", label = "波特率 (0-3)" },
        },
    },
}



local frame_buffer = {}

-- ── Helpers ──────────────────────────────────────────────────────────────────

-- BCD encode a number to N bytes (LSB first)
local function bcd_encode(num, n)
    local bytes = {}
    for i = 1, n do
        local lo = num % 10
        num = math.floor(num / 10)
        local hi = num % 10
        num = math.floor(num / 10)
        table.insert(bytes, bit.bor(bit.lshift(hi, 4), lo))
    end
    return bytes
end

-- BCD decode N bytes to a number (LSB first)
local function bcd_decode(bytes, start, n)
    local result = 0
    local multiplier = 1
    for i = 0, n - 1 do
        local b = bytes[start + i]
        local lo = bit.band(b, 0x0F)
        local hi = bit.band(bit.rshift(b, 4), 0x0F)
        result = result + (lo * multiplier) + (hi * multiplier * 10)
        multiplier = multiplier * 100
    end
    return result
end

-- Compute CS (XOR of all bytes from 0x68 to last data byte)
local function compute_cs(frame, len)
    -- frame: [0x68, addr*6, 0x68, ctrl, len, data*len]
    -- CS = XOR of bytes 0..(9+len-1)
    local cs = 0
    for i = 1, 10 + len do
        cs = bit.bxor(cs, frame[i])
    end
    return cs
end

-- XOR a byte with 0x33 (DL/T 645 data offset)
local function offset_encode(b)
    return bit.band(b + 0x33, 0xFF)
end

local function offset_decode(b)
    return bit.band(b - 0x33, 0xFF)
end

-- ── Callbacks ────────────────────────────────────────────────────────────────

function on_send(data)
    -- Expects a raw frame without 0x33 offset and without CS/0x16 tail.
    -- Format: [0x68][Addr:6B][0x68][Ctrl:1B][Len:1B][Data:NB]
    -- We add the 0x33 offset to data bytes, compute CS, append CS and 0x16.

    if #data < 10 then
        log_warn("DL/T 645 on_send: frame too short (min 10 bytes)")
        return nil
    end

    -- Validate headers
    if data[1] ~= 0x68 or data[8] ~= 0x68 then
        log_warn("DL/T 645 on_send: invalid header bytes")
        return nil
    end

    local ctrl = data[9]
    local data_len = data[10]

    if #data < 10 + data_len then
        log_warn("DL/T 645 on_send: data length mismatch (declared " .. data_len ..
                 " but only " .. (#data - 10) .. " available)")
        return nil
    end

    -- Build output frame with offset encoding on data bytes
    local frame = {}
    -- Header + address + header2 + ctrl + len (bytes 1-10)
    for i = 1, 10 do
        table.insert(frame, data[i])
    end
    -- Data bytes with 0x33 offset
    for i = 1, data_len do
        table.insert(frame, offset_encode(data[10 + i]))
    end
    -- CS
    local cs = compute_cs(frame, data_len)
    table.insert(frame, cs)
    -- Tail
    table.insert(frame, 0x16)

    return frame
end

function on_recv(data)
    -- Accumulate bytes
    for _, b in ipairs(data) do
        table.insert(frame_buffer, b)
    end

    -- Minimum frame: 0x68 + 6 addr + 0x68 + 1 ctrl + 1 len + 1 CS + 0x16 = 12 bytes
    while #frame_buffer >= 12 do
        -- Find 0x68 header
        while #frame_buffer > 0 and frame_buffer[1] ~= 0x68 do
            table.remove(frame_buffer, 1)
        end

        if #frame_buffer < 12 then
            return nil
        end

        -- Verify second 0x68 at position 8
        if frame_buffer[8] ~= 0x68 then
            table.remove(frame_buffer, 1)
        else
            local data_len = frame_buffer[10]
            local frame_len = 10 + data_len + 2  -- header(10) + data(N) + CS(1) + 0x16(1)

            if #frame_buffer < frame_len then
                return nil  -- Need more data
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

            -- Verify tail byte
            if frame[frame_len] ~= 0x16 then
                log_warn("DL/T 645 recv: invalid tail byte")
            else
                -- Verify CS
                local expected_cs = compute_cs(frame, data_len)
                local got_cs = frame[frame_len - 1]
                if expected_cs ~= got_cs then
                    log_warn("DL/T 645 recv: CS mismatch")
                else
                    -- Decode data bytes (remove 0x33 offset)
                    local decoded = {}
                    for i = 1, 10 do
                        table.insert(decoded, frame[i])
                    end
                    for i = 1, data_len do
                        table.insert(decoded, offset_decode(frame[10 + i]))
                    end
                    return decoded
                end
            end
        end
    end

    return nil
end

-- ── Actions ─────────────────────────────────────────────────────────────────

--- Build a DL/T 645 request frame.
-- @param address table|string  6-byte address table or numeric meter number
-- @param ctrl number           Control byte
-- @param data table            Data bytes
-- @return table                Frame bytes ready for on_send
function action_build_request(address, ctrl, data)
    local addr_bytes
    if type(address) == "table" then
        addr_bytes = address
        -- Pad to 6 bytes
        while #addr_bytes < 6 do
            table.insert(addr_bytes, 0xAA)
        end
    elseif type(address) == "number" then
        addr_bytes = bcd_encode(address, 6)
    else
        return { _error = "address must be table or number" }
    end

    local frame = { 0x68 }  -- First header
    for _, b in ipairs(addr_bytes) do
        table.insert(frame, b)
    end
    table.insert(frame, 0x68)  -- Second header
    table.insert(frame, ctrl)
    table.insert(frame, #data)
    for _, b in ipairs(data) do
        table.insert(frame, b)
    end

    return frame
end

--- Build a "read data" request (control code 0x11).
-- @param address table|string  Meter address
-- @param di table              4-byte Data Identifier (LSB first)
-- @return table                Frame bytes
function action_read_data(address, di)
    return action_build_request(address, 0x11, di)
end

--- Build a "read data continuation" request (control code 0x12).
-- @param address table|string  Meter address
-- @param di table              4-byte Data Identifier
-- @return table                Frame bytes
function action_read_data_next(address, di)
    return action_build_request(address, 0x12, di)
end

--- Build a "write data" request (control code 0x14).
-- @param address table|string  Meter address
-- @param di table              4-byte Data Identifier
-- @param value table           Data to write
-- @return table                Frame bytes
function action_write_data(address, di, value)
    local data = {}
    for _, b in ipairs(di) do
        table.insert(data, b)
    end
    for _, b in ipairs(value) do
        table.insert(data, b)
    end
    return action_build_request(address, 0x14, data)
end

--- Build a "read address" broadcast request (control code 0x13).
-- Uses broadcast address (all 0x99).
-- @return table Frame bytes
function action_read_address()
    local addr = { 0x99, 0x99, 0x99, 0x99, 0x99, 0x99 }
    return action_build_request(addr, 0x13, {})
end

--- Build a "write address" request (control code 0x15).
-- @param new_address table|number  New meter address
-- @return table Frame bytes
function action_write_address(new_address)
    local addr_bytes
    if type(new_address) == "table" then
        addr_bytes = new_address
    elseif type(new_address) == "number" then
        addr_bytes = bcd_encode(new_address, 6)
    else
        return { _error = "address must be table or number" }
    end
    -- Write address command uses broadcast address
    local broadcast = { 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA }
    return action_build_request(broadcast, 0x15, addr_bytes)
end

--- Build a "broadcast time sync" request (control code 0x08).
-- @param datetime table  {year, month, day, hour, minute, second} (BCD-encoded)
-- @return table Frame bytes
function action_time_sync(datetime)
    local addr = { 0x99, 0x99, 0x99, 0x99, 0x99, 0x99 }
    local data = {}
    -- Seconds, Minutes, Hours, Day, Month, Year (BCD, LSB first in each pair)
    local fields = { datetime.sec or 0, datetime.min or 0, datetime.hour or 0,
                     datetime.day or 0, datetime.month or 0, (datetime.year or 2000) % 100 }
    for _, v in ipairs(fields) do
        table.insert(data, bcd_encode(v, 1)[1])
    end
    return action_build_request(addr, 0x08, data)
end

--- Parse a DL/T 645 response frame (already decoded by on_recv).
-- @param frame table  Decoded frame from on_recv
-- @return table { address, ctrl, data, di, meter_number }
function action_parse_response(frame)
    if #frame < 10 then
        return { _error = "frame too short" }
    end

    -- Extract address (bytes 2-7)
    local addr = {}
    for i = 2, 7 do
        table.insert(addr, frame[i])
    end

    -- Decode meter number from BCD
    local meter_number = bcd_decode(addr, 1, 6)

    -- Extract control byte
    local ctrl = frame[9]
    local data_len = frame[10]

    -- Extract data bytes
    local data = {}
    for i = 1, data_len do
        table.insert(data, frame[10 + i])
    end

    -- Extract DI (first 4 bytes of data, if present)
    local di = nil
    if #data >= 4 then
        di = { data[1], data[2], data[3], data[4] }
    end

    -- Check for error bit (bit 7 of ctrl)
    local is_error = bit.band(ctrl, 0x80) ~= 0
    local func_code = bit.band(ctrl, 0x1F)

    return {
        address = addr,
        meter_number = meter_number,
        ctrl = ctrl,
        func_code = func_code,
        is_error = is_error,
        data = data,
        di = di,
        data_len = data_len,
    }
end

--- Format a Data Identifier (DI) for common meter registers.
-- @param di_num number  DI number (e.g., 0x00010000 for total active energy)
-- @return table         4-byte DI table
function action_make_di(di_num)
    return {
        bit.band(di_num, 0xFF),
        bit.band(bit.rshift(di_num, 8), 0xFF),
        bit.band(bit.rshift(di_num, 16), 0xFF),
        bit.band(bit.rshift(di_num, 24), 0xFF),
    }
end

-- Common Data Identifiers for quick reference:
-- 0x00010000  Total active energy (正向有功总电量)
-- 0x00020000  Total reactive energy (正向无功总电量)
-- 0x02010100  Voltage A (A相电压)
-- 0x02010200  Current A (A相电流)
-- 0x02020100  Active power (有功功率)
-- 0x04000401  Meter number (表号)
-- 0x04000101  Date (日期)
-- 0x04000102  Time (时间)
