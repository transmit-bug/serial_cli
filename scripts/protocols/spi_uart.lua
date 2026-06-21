-- SPI over UART protocol
-- Common serial-SPI bridge format (e.g., based on FT232H, CH341, or custom bridges).
-- Command frame: [CMD][LEN][DATA...]
-- Response frame: [STATUS][LEN][DATA...]
--
-- Typical commands:
--   0x01 + [CS] + [len] + [data...] = SPI transfer (full-duplex)
--   0x02 + [CS]                      = Assert CS (chip select low)
--   0x03 + [CS]                      = Deassert CS (chip select high)
--   0x04 + [speed_lo] + [speed_hi]   = Set SPI clock speed

SCRIPT_META = {
    name = "spi_uart",
    version = "1.0.0",
    description = "SPI over UART bridge protocol",
    author = "serial_cli",
    data_format = "binary",
    min_frame_size = 3,
    tags = {"spi", "uart", "flash", "eeprom", "embedded", "bridge"},
}

local frame_buffer = {}

-- Frame: [STX:0xBB][LEN:2B (LE)][DATA...][CS:1B]
-- CS = XOR of all LEN and DATA bytes
local STX = 0xBB

local function compute_cs(bytes)
    local cs = 0
    for _, b in ipairs(bytes) do
        cs = bit.bxor(cs, b)
    end
    return cs
end

function on_send(data)
    if #data < 1 then
        log_warn("SPI on_send: empty command")
        return nil
    end

    local len = #data
    local len_lo = bit.band(len, 0xFF)
    local len_hi = bit.band(bit.rshift(len, 8), 0xFF)

    local cs_bytes = { len_lo, len_hi }
    for _, b in ipairs(data) do
        table.insert(cs_bytes, b)
    end
    local cs = compute_cs(cs_bytes)

    local frame = { STX, len_lo, len_hi }
    for _, b in ipairs(data) do
        table.insert(frame, b)
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

        if #frame_buffer < 4 then
            return nil
        end

        local len_lo = frame_buffer[2]
        local len_hi = frame_buffer[3]
        local len = len_lo + bit.lshift(len_hi, 8)
        local frame_len = 3 + len + 1  -- STX + LEN(2) + DATA + CS

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
        local cs_bytes = { frame[2], frame[3] }
        for i = 4, frame_len - 1 do
            table.insert(cs_bytes, frame[i])
        end
        local expected_cs = compute_cs(cs_bytes)
        if expected_cs == frame[frame_len] then
            -- Return data (skip STX, LEN, CS)
            local result = {}
            for i = 4, frame_len - 1 do
                table.insert(result, frame[i])
            end
            return result
        else
            log_warn("SPI recv: CS mismatch")
        end
    end

    return nil
end

-- ── Actions ─────────────────────────────────────────────────────────────────

--- Build a full-duplex SPI transfer command.
-- @param cs number    Chip select line (0, 1, 2, ...)
-- @param data table   TX data bytes (RX will be same length)
-- @return table       Command bytes
function action_transfer(cs, data)
    local cmd = { 0x01, cs, #data }
    for _, b in ipairs(data) do
        table.insert(cmd, b)
    end
    return cmd
end

--- Build a write-only SPI command (assert CS, write, deassert CS).
-- @param cs number    Chip select line
-- @param data table   TX data bytes
-- @return table       Command bytes
function action_write(cs, data)
    local cmd = { 0x02, cs }  -- Assert CS
    for _, b in ipairs(data) do
        table.insert(cmd, b)
    end
    table.insert(cmd, 0x03)  -- Deassert CS
    table.insert(cmd, cs)
    return cmd
end

--- Build a read register command (common SPI device pattern).
-- @param cs number     Chip select line
-- @param reg number    Register address
-- @param len number    Number of bytes to read
-- @return table        Command bytes
function action_read_reg(cs, reg, len)
    -- Send register address with read bit (bit 7 set for many SPI devices)
    local cmd = { 0x02, cs, bit.bor(reg, 0x80) }
    -- Clock out len dummy bytes to receive data
    for i = 1, len do
        table.insert(cmd, 0x00)
    end
    table.insert(cmd, 0x03, cs)  -- Deassert CS
    return cmd
end

--- Build a write register command.
-- @param cs number     Chip select line
-- @param reg number    Register address
-- @param value number  Value to write
-- @return table        Command bytes
function action_write_reg(cs, reg, value)
    local cmd = { 0x02, cs, bit.band(reg, 0x7F), value }
    table.insert(cmd, 0x03, cs)  -- Deassert CS
    return cmd
end

--- Set SPI clock speed.
-- @param hz number  Clock speed in Hz
-- @return table     Command bytes
function action_set_speed(hz)
    local lo = bit.band(hz, 0xFF)
    local hi = bit.band(bit.rshift(hz, 8), 0xFF)
    return { 0x04, lo, hi }
end

--- Parse an SPI response.
-- @param response table  Response from on_recv
-- @return table { data, length }
function action_parse_response(response)
    if not response or #response == 0 then
        return { _error = "empty response" }
    end
    return {
        data = response,
        length = #response,
    }
end
