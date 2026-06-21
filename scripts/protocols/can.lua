-- CAN bus over serial protocol
-- Common serial-CAN adapter frame format (SLCAN / Lawicel compatible):
--   Standard frame (11-bit ID):  tIIIDLDDDDDDDD\r
--   Extended frame (29-bit ID):  TIIIIIIIIDLDDDDDDDD\r
--   Remote frame (standard):     rIIIDL\r
--   Remote frame (extended):     RIIIIIIIIL\r
-- Where:
--   I = CAN ID (hex), L = data length (hex 0-8), D = data bytes (hex pairs)
--
-- NOTE: Uses Lua 5.1 compatible bitwise operations via bit library.

SCRIPT_META = {
    name = "can",
    version = "1.0.0",
    description = "CAN bus over serial (SLCAN/Lawicel compatible)",
    author = "serial_cli",
    data_format = "binary",
    min_frame_size = 5,
    tags = {"can", "automotive", "industrial", "slcan", "lawicel"}
,
}

_actions = {
    standard = {
        label = "📦 标准帧",
        group = "CAN",
        icon = "box",
        params = {
            { name = "id",   type = "hex",    label = "CAN ID (hex)" },
            { name = "data", type = "hex",    label = "数据 (hex, 空格分隔)" },
        },
    },
    extended = {
        label = "📦 扩展帧",
        group = "CAN",
        icon = "box-select",
        params = {
            { name = "id",   type = "hex",    label = "扩展 ID (hex)" },
            { name = "data", type = "hex",    label = "数据 (hex, 空格分隔)" },
        },
    },
    remote = {
        label = "🔄 远程帧",
        group = "CAN",
        icon = "refresh-cw",
        params = {
            { name = "id",  type = "hex",    label = "CAN ID (hex)" },
            { name = "dlc", type = "number", default = 8, label = "DLC (0-8)" },
        },
    },
}



local frame_buffer = ""

-- ── Helpers ──────────────────────────────────────────────────────────────────

local function hex_encode_byte(b)
    return string.format("%02X", b)
end

local function hex_decode_byte(s)
    return tonumber(s, 16)
end

local function hex_encode_id(id, extended)
    if extended then
        return string.format("%08X", id)
    else
        return string.format("%03X", id)
    end
end

local function hex_decode_id(s)
    return tonumber(s, 16)
end

-- ── Callbacks ────────────────────────────────────────────────────────────────

function on_send(data)
    -- Expect data as a Lua table describing a CAN frame:
    --   { id_lo, id_hi, [id_ext_lo, id_ext_hi], dlc, data0, data1, ... }
    -- Simple binary format: [CAN_ID: 2 or 4 bytes][DLC: 1 byte][DATA: 0-8 bytes]
    -- We encode to SLCAN text format for serial transmission.

    if #data < 3 then
        log_warn("CAN on_send: data too short (min 3 bytes: id_lo, id_hi, dlc)")
        return nil
    end

    local id_lo = data[1]
    local id_hi = data[2]
    local idx = 3
    local can_id, extended

    local remaining = #data - 2  -- after 2-byte ID
    if remaining >= 2 then
        local potential_dlc = data[3]
        if remaining > 9 then
            extended = true
        elseif remaining <= 9 then
            local dlc3 = data[3]
            if dlc3 <= 8 and remaining == 1 + dlc3 then
                extended = false
            elseif remaining >= 3 then
                local dlc5 = data[5]
                if dlc5 <= 8 and remaining == 3 + dlc5 then
                    extended = true
                else
                    extended = false
                end
            else
                extended = false
            end
        end
    else
        extended = false
    end

    if extended then
        can_id = bit.bor(id_lo, bit.lshift(id_hi, 8),
                         bit.lshift(data[3], 16), bit.lshift(data[4], 24))
        idx = 5
    else
        can_id = bit.bor(id_lo, bit.lshift(id_hi, 8))
        idx = 3
    end

    local dlc = data[idx]
    if dlc > 8 then
        log_warn("CAN on_send: DLC > 8 (" .. dlc .. ")")
        return nil
    end

    local payload = {}
    for i = 1, dlc do
        table.insert(payload, data[idx + i])
    end

    -- Build SLCAN frame
    local frame
    if extended then
        frame = "T" .. hex_encode_id(can_id, true) .. string.format("%X", dlc)
    else
        frame = "t" .. hex_encode_id(can_id, false) .. string.format("%X", dlc)
    end

    for _, b in ipairs(payload) do
        frame = frame .. hex_encode_byte(b)
    end
    frame = frame .. "\r"

    return string_to_bytes(frame)
end

function on_recv(data)
    local str = bytes_to_string(data)
    frame_buffer = frame_buffer .. str

    local results = {}

    while true do
        -- Find a complete SLCAN frame terminated by \r
        local cr_pos = frame_buffer:find("\r", 1, true)
        if not cr_pos then
            break
        end

        local frame = frame_buffer:sub(1, cr_pos - 1)
        frame_buffer = frame_buffer:sub(cr_pos + 1)

        if #frame > 0 then
            local cmd = frame:sub(1, 1)
            local parsed = nil

            if cmd == "t" then
                -- Standard frame: tIIILDDDD...
                if #frame >= 5 then
                    local id_str = frame:sub(2, 4)
                    local dlc = tonumber(frame:sub(5, 5), 16)
                    if dlc and dlc <= 8 and #frame >= 5 + dlc * 2 then
                        local can_id = hex_decode_id(id_str)
                        parsed = {
                            bit.band(can_id, 0xFF),
                            bit.band(bit.rshift(can_id, 8), 0xFF),
                            dlc,
                        }
                        for i = 1, dlc do
                            local byte_str = frame:sub(5 + (i - 1) * 2 + 1, 5 + i * 2)
                            table.insert(parsed, hex_decode_byte(byte_str))
                        end
                    end
                end

            elseif cmd == "T" then
                -- Extended frame: TIIIIIIIILDDDD...
                if #frame >= 10 then
                    local id_str = frame:sub(2, 9)
                    local dlc = tonumber(frame:sub(10, 10), 16)
                    if dlc and dlc <= 8 and #frame >= 10 + dlc * 2 then
                        local can_id = hex_decode_id(id_str)
                        parsed = {
                            bit.band(can_id, 0xFF),
                            bit.band(bit.rshift(can_id, 8), 0xFF),
                            bit.band(bit.rshift(can_id, 16), 0xFF),
                            bit.band(bit.rshift(can_id, 24), 0xFF),
                            dlc,
                        }
                        for i = 1, dlc do
                            local byte_str = frame:sub(10 + (i - 1) * 2 + 1, 10 + i * 2)
                            table.insert(parsed, hex_decode_byte(byte_str))
                        end
                    end
                end

            elseif cmd == "r" then
                -- Remote frame (standard): rIIIL
                if #frame >= 5 then
                    local id_str = frame:sub(2, 4)
                    local dlc = tonumber(frame:sub(5, 5), 16)
                    if dlc then
                        local can_id = hex_decode_id(id_str)
                        parsed = {
                            bit.band(can_id, 0xFF),
                            bit.band(bit.rshift(can_id, 8), 0xFF),
                            bit.bor(dlc, 0x40),  -- flag as remote
                        }
                    end
                end

            elseif cmd == "R" then
                -- Remote frame (extended): RIIIIIIIIL
                if #frame >= 10 then
                    local id_str = frame:sub(2, 9)
                    local dlc = tonumber(frame:sub(10, 10), 16)
                    if dlc then
                        local can_id = hex_decode_id(id_str)
                        parsed = {
                            bit.band(can_id, 0xFF),
                            bit.band(bit.rshift(can_id, 8), 0xFF),
                            bit.band(bit.rshift(can_id, 16), 0xFF),
                            bit.band(bit.rshift(can_id, 24), 0xFF),
                            bit.bor(dlc, 0x40),
                        }
                    end
                end

            elseif cmd == "z" or cmd == "Z" then
                -- Status/error responses — pass through as raw
                parsed = string_to_bytes(frame)
            end

            if parsed then
                table.insert(results, parsed)
            end
        end
    end

    if #results == 0 then
        return nil
    end

    -- Return the first frame
    if #results == 1 then
        return results[1]
    end

    -- Multiple frames: concatenate with a 0x00 separator
    local combined = {}
    for i, frame_data in ipairs(results) do
        for _, b in ipairs(frame_data) do
            table.insert(combined, b)
        end
        if i < #results then
            table.insert(combined, 0x00)
            table.insert(combined, 0x00)
        end
    end
    return combined
end

-- ── Actions ─────────────────────────────────────────────────────────────────

--- Build a standard (11-bit) CAN frame.
-- @param can_id number  CAN ID (0x000 - 0x7FF)
-- @param data table     Data bytes (0-8 bytes)
-- @return table         Encoded frame bytes for on_send
function action_make_frame(can_id, data)
    local dlc = #data
    if dlc > 8 then
        return { _error = "data too long, max 8 bytes" }
    end
    if can_id > 0x7FF then
        return { _error = "standard CAN ID must be <= 0x7FF, use action_make_ext_frame for 29-bit IDs" }
    end
    local frame = {
        bit.band(can_id, 0xFF),
        bit.band(bit.rshift(can_id, 8), 0xFF),
        dlc,
    }
    for _, b in ipairs(data) do
        table.insert(frame, b)
    end
    return frame
end

--- Build an extended (29-bit) CAN frame.
-- @param can_id number  CAN ID (0x00000000 - 0x1FFFFFFF)
-- @param data table     Data bytes (0-8 bytes)
-- @return table         Encoded frame bytes for on_send
function action_make_ext_frame(can_id, data)
    local dlc = #data
    if dlc > 8 then
        return { _error = "data too long, max 8 bytes" }
    end
    if can_id > 0x1FFFFFFF then
        return { _error = "extended CAN ID must be <= 0x1FFFFFFF" }
    end
    local frame = {
        bit.band(can_id, 0xFF),
        bit.band(bit.rshift(can_id, 8), 0xFF),
        bit.band(bit.rshift(can_id, 16), 0xFF),
        bit.band(bit.rshift(can_id, 24), 0xFF),
        dlc,
    }
    for _, b in ipairs(data) do
        table.insert(frame, b)
    end
    return frame
end

--- Parse a received CAN frame into human-readable fields.
-- @param frame table  Raw frame from on_recv
-- @return table { can_id, extended, dlc, data, remote }
function action_parse_frame(frame)
    if #frame < 3 then
        return { _error = "frame too short" }
    end

    local extended = false
    local can_id, dlc_idx

    -- Check if it's extended (4-byte ID + DLC at index 5)
    if #frame >= 5 then
        local potential_dlc = frame[5]
        if potential_dlc <= 8 and #frame == 4 + potential_dlc then
            extended = true
            can_id = bit.bor(frame[1], bit.lshift(frame[2], 8),
                             bit.lshift(frame[3], 16), bit.lshift(frame[4], 24))
            dlc_idx = 5
        end
    end

    if not extended then
        can_id = bit.bor(frame[1], bit.lshift(frame[2], 8))
        dlc_idx = 3
    end

    local dlc_raw = frame[dlc_idx]
    local remote = (bit.band(dlc_raw, 0x40)) ~= 0
    local dlc = bit.band(dlc_raw, 0x0F)

    local data = {}
    if not remote then
        for i = 1, dlc do
            table.insert(data, frame[dlc_idx + i])
        end
    end

    return {
        can_id = can_id,
        extended = extended,
        dlc = dlc,
        data = data,
        remote = remote,
        hex_id = string.format(extended and "0x%08X" or "0x%03X", can_id),
    }
end

--- Encode a CAN ID into a send-ready frame (common shortcut).
-- Sends a standard frame with the given ID and data.
-- @param port_id string  Port ID (set via on_open or passed in)
-- @param can_id number   CAN ID
-- @param data table      Data bytes
-- @return table          Response or error
function action_send_frame(port_id, can_id, data)
    local frame = action_make_frame(can_id, data)
    if frame._error then return frame end
    return { encoded = frame }
end
