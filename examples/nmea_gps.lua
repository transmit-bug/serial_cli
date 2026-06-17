-- Example: Custom NMEA GPS protocol script
-- Demonstrates parsing NMEA sentences from a GPS receiver

-- NMEA sentence parser
local function parse_nmea(sentence)
    -- Remove leading $ and trailing checksum
    local data = sentence:match("^%$(.-)%*%x%x$")
    if not data then
        return nil, "Invalid NMEA sentence"
    end

    -- Split by comma
    local fields = {}
    for field in data:gmatch("[^,]+") do
        table.insert(fields, field)
    end

    return fields
end

-- Validate NMEA checksum
local function validate_checksum(sentence)
    local data, checksum = sentence:match("^%$(.-)%*(%x%x)$")
    if not data then return false end

    local calc_checksum = 0
    for i = 1, #data do
        calc_checksum = calc_checksum ~ string.byte(data, i)
    end

    return string.format("%02X", calc_checksum) == checksum:upper()
end

-- Buffer for accumulating partial sentences
local buffer = ""

function on_recv(data)
    -- Convert bytes to string
    local str = bytes_to_string(data)

    -- Add to buffer
    buffer = buffer .. str

    -- Process complete sentences
    local results = {}
    while true do
        local start_pos = buffer:find("$")
        if not start_pos then
            buffer = ""
            break
        end

        local end_pos = buffer:find("\r\n", start_pos)
        if not end_pos then
            buffer = buffer:sub(start_pos)
            break
        end

        local sentence = buffer:sub(start_pos, end_pos - 1)
        buffer = buffer:sub(end_pos + 2)

        -- Validate and parse
        if validate_checksum(sentence) then
            local fields, err = parse_nmea(sentence)
            if fields then
                -- Create structured output
                local result = {
                    type = fields[1],
                    fields = fields,
                    raw = sentence
                }
                table.insert(results, json_encode(result))
            end
        end
    end

    if #results > 0 then
        return string_to_bytes(table.concat(results, "\n"))
    end

    return nil  -- No complete sentences yet
end

function on_send(data)
    -- Pass through GPS commands unchanged
    return data
end
