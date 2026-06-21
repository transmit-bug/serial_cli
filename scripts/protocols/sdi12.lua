-- SDI-12 protocol (environmental sensors: soil, weather, water quality)
-- Frame: [ADDR:1B][CMD:1B][!][\r][\n]  (commands)
-- Response: [ADDR:1B][DATA...\r\n] or [ADDR:1B][values...\r\n]
-- All ASCII text, addresses are 0-9, a-z, A-Z.

SCRIPT_META = {
    name = "sdi12",
    version = "1.0.0",
    description = "SDI-12 environmental sensor protocol",
    author = "serial_cli",
    data_format = "text",
    min_frame_size = 4,
    tags = {"sdi12", "environmental", "soil", "weather", "agriculture"},
}

local response_buffer = ""

-- SDI-12 uses BREAK condition (>12ms low) to wake sensors,
-- but over a serial bridge this is typically handled by the adapter.
-- We deal with ASCII text framing only.

function on_send(data)
    local str = bytes_to_string(data)
    -- Ensure command ends with "!"
    str = str:gsub("[\r\n]+$", "")  -- strip trailing whitespace
    if not str:match("!$") then
        str = str .. "!"
    end
    -- SDI-12 commands always end with !
    return string_to_bytes(str)
end

function on_recv(data)
    local str = bytes_to_string(data)
    response_buffer = response_buffer .. str

    -- SDI-12 responses end with \r\n
    if not response_buffer:match("\r\n$") then
        return nil  -- Incomplete
    end

    local result = response_buffer
    response_buffer = ""
    return string_to_bytes(result)
end

-- ── Actions ─────────────────────────────────────────────────────────────────

--- Build an SDI-12 command.
-- @param address string|number  Sensor address (0-9, a-z, A-Z)
-- @param cmd string             Command (e.g., "M", "D0", "?")
-- @return string               Full command string
function action_command(address, cmd)
    return tostring(address) .. cmd .. "!"
end

--- Build an "Acknowledge Active" command (?!).
-- @param address string  Sensor address
-- @return string         Command string
function action_acknowledge(address)
    return action_command(address, "?")
end

--- Build an "Identify" command (I!).
-- @param address string  Sensor address
-- @return string         Command string
function action_identify(address)
    return action_command(address, "I")
end

--- Build a "Start Measurement" command (M!).
-- @param address string  Sensor address
-- @return string         Command string
function action_start_measurement(address)
    return action_command(address, "M")
end

--- Build a "Start Measurement" with CRC command (MC!).
-- @param address string  Sensor address
-- @return string         Command string
function action_start_measurement_crc(address)
    return action_command(address, "MC")
end

--- Build a "Send Data" command (D0! through D9!).
-- @param address string  Sensor address
-- @param index number    Data index (0-9)
-- @return string         Command string
function action_send_data(address, index)
    return action_command(address, "D" .. tostring(index))
end

--- Build a "Continuous Measurement" command (R0! through R9!).
-- @param address string  Sensor address
-- @param index number    Repeat index (0-9)
-- @return string         Command string
function action_continuous(address, index)
    return action_command(address, "R" .. tostring(index))
end

--- Build a "Change Address" command (aAb!).
-- @param old_address string  Current address
-- @param new_address string  New address
-- @return string             Command string
function action_change_address(old_address, new_address)
    return old_address .. "A" .. new_address .. "!"
end

--- Parse an SDI-12 response.
-- @param response string  Response string (e.g., "0+21.3+0.45+12.8\r\n")
-- @return table { address, values, raw }
function action_parse_response(response)
    response = response:gsub("[\r\n]+$", "")
    if #response == 0 then
        return { _error = "empty response" }
    end

    local address = response:sub(1, 1)
    local values = {}

    -- Parse values: they are separated by + or - signs after the address
    local body = response:sub(2)
    -- Match: sign (optional) then digits and decimal point
    for val in body:gmatch("[+-]?%d+%.?%d*[eE]?[+-]?%d*") do
        local num = tonumber(val)
        if num then
            table.insert(values, num)
        end
    end

    return {
        address = address,
        values = values,
        raw = response,
    }
end

--- Parse sensor identification string.
-- @param id_str string  ID response (e.g., "013TEST  Manufacturer  Model   S-N  V1.0")
-- @return table { address, sdi12_version, vendor, model, serial, firmware }
function action_parse_id(id_str)
    id_str = id_str:gsub("[\r\n]+$", "")
    if #id_str < 4 then
        return { _error = "ID string too short" }
    end
    return {
        address = id_str:sub(1, 1),
        sdi12_version = id_str:sub(2, 4),
        vendor = id_str:sub(5, 17):match("^%s*(.-)%s*$"),
        model = id_str:sub(18, 30):match("^%s*(.-)%s*$"),
        serial = id_str:sub(31, 37):match("^%s*(.-)%s*$"),
        firmware = id_str:sub(38):match("^%s*(.-)%s*$"),
        raw = id_str,
    }
end
