-- I2C over UART protocol
-- Common serial-I2C bridge format (e.g., based on MCP2221, FT232H, or custom bridges).
-- Command frame: [CMD][DATA...]
-- Response frame: [STATUS][DATA...]
--
-- Typical commands:
--   0x01 + [addr + flag] + [len]         = Start condition + address
--   0x02 + [byte...]                     = Write data
--   0x03 + [len]                         = Read data (returns len bytes)
--   0x04                                 = Stop condition
--   0x05 + [addr + flag] + [reg]         = Write register (addr + register + value)
--   0x06 + [addr + flag] + [reg] + [len] = Read register

SCRIPT_META = {
    name = "i2c_uart",
    version = "1.0.0",
    description = "I2C over UART bridge protocol",
    author = "serial_cli",
    data_format = "binary",
    min_frame_size = 2,
    tags = {"i2c", "uart", "sensor", "embedded", "bridge"}
,
}

_actions = {
    write_reg = {
        label = "✏️ 写寄存器",
        group = "I2C",
        icon = "edit",
        params = {
            { name = "addr",  type = "number", label = "I2C 地址 (7-bit)" },
            { name = "reg",   type = "number", label = "寄存器地址" },
            { name = "value", type = "number", label = "值" },
        },
    },
    read_reg = {
        label = "📖 读寄存器",
        group = "I2C",
        icon = "book-open",
        params = {
            { name = "addr", type = "number", label = "I2C 地址 (7-bit)" },
            { name = "reg",  type = "number", label = "寄存器地址" },
            { name = "len",  type = "number", default = 1, label = "字节数" },
        },
    },
    scan = {
        label = "🔍 扫描总线",
        group = "I2C",
        icon = "search",
    },
}



local frame_buffer = {}

-- Frame: [STX:0xAA][LEN:1B][CMD:1B][DATA...][CS:1B]
-- CS = XOR of LEN, CMD, and all DATA bytes
local STX = 0xAA

local function compute_cs(frame)
    local cs = 0
    for _, b in ipairs(frame) do
        cs = bit.bxor(cs, b)
    end
    return cs
end

function on_send(data)
    -- data = [CMD][PARAM...]
    if #data < 1 then
        log_warn("I2C on_send: empty command")
        return nil
    end

    local len = #data
    local frame = { STX, len }
    for _, b in ipairs(data) do
        table.insert(frame, b)
    end
    local cs = compute_cs({ len })
    for _, b in ipairs(data) do
        cs = bit.bxor(cs, b)
    end
    table.insert(frame, cs)
    return frame
end

function on_recv(data)
    for _, b in ipairs(data) do
        table.insert(frame_buffer, b)
    end

    while true do
        -- Find STX
        local start = 1
        while start <= #frame_buffer and frame_buffer[start] ~= STX do
            start = start + 1
        end
        if start > 1 then
            local new_buf = {}
            for i = start, #frame_buffer do
                table.insert(new_buf, frame_buffer[i])
            end
            frame_buffer = new_buf
        end

        if #frame_buffer < 3 then
            return nil
        end

        local len = frame_buffer[2]
        local frame_len = 2 + len + 1  -- STX + LEN + DATA + CS

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
        local expected_cs = compute_cs(cs_data)
        if expected_cs == frame[frame_len] then
            -- Return status + data (skip STX, LEN, CS)
            local result = {}
            for i = 3, frame_len - 1 do
                table.insert(result, frame[i])
            end
            return result
        else
            log_warn("I2C recv: CS mismatch")
        end
    end

    return nil
end

-- ── Actions ─────────────────────────────────────────────────────────────────

--- Build a write-register command.
-- @param addr number  7-bit I2C address
-- @param reg number   Register address
-- @param value number Value to write
-- @return table       Command bytes
function action_write_reg(addr, reg, value)
    local addr_w = bit.lshift(addr, 1)  -- Write: R/W bit = 0
    return { 0x05, addr_w, reg, value }
end

--- Build a read-register command.
-- @param addr number  7-bit I2C address
-- @param reg number   Register address
-- @param len number   Number of bytes to read
-- @return table       Command bytes
function action_read_reg(addr, reg, len)
    local addr_r = bit.bor(bit.lshift(addr, 1), 0x01)  -- Read: R/W bit = 1
    return { 0x06, addr_r, reg, len }
end

--- Build a raw write command.
-- @param addr number  7-bit I2C address
-- @param data table   Data bytes to write
-- @return table       Command bytes
function action_write(addr, data)
    local cmd = { 0x01, bit.lshift(addr, 1), #data }
    for _, b in ipairs(data) do
        table.insert(cmd, b)
    end
    table.insert(cmd, 0x04)  -- Stop condition
    return cmd
end

--- Build a raw read command.
-- @param addr number  7-bit I2C address
-- @param len number   Number of bytes to read
-- @return table       Command bytes
function action_read(addr, len)
    return { 0x01, bit.bor(bit.lshift(addr, 1), 0x01), 0x03, len, 0x04 }
end

--- Parse an I2C response.
-- @param response table  Response from on_recv
-- @return table { status, data, error }
function action_parse_response(response)
    if not response or #response < 1 then
        return { _error = "empty response" }
    end
    local status = response[1]
    local data = {}
    for i = 2, #response do
        table.insert(data, response[i])
    end
    return {
        status = status,
        data = data,
        ok = (status == 0x00),
        error = (status ~= 0x00) and string.format("0x%02X", status) or nil,
    }
end
