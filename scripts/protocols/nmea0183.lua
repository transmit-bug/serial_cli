-- NMEA 0183 protocol
-- Frame: $<talker><sentence>,<field1>,<field2>,...*<CS>\r\n
-- CS = XOR of all chars between $ and * (exclusive), 2 hex digits.
-- Example: $GPGGA,092750.000,5321.6802,N,00630.3372,W,1,8,1.03,61.7,M,55.2,M,,*76

SCRIPT_META = {
    name = "nmea0183",
    version = "1.0.0",
    description = "NMEA 0183 GPS/marine instrument protocol",
    author = "serial_cli",
    data_format = "text",
    min_frame_size = 8,
    tags = {"nmea", "gps", "marine", "navigation"}
,
}

_actions = {
    gpgga = {
        label = "📍 构造 GGA (定位)",
        group = "NMEA",
        icon = "map-pin",
        params = {
            { name = "lat", type = "string", default = "3956.1234,N", label = "纬度 (ddmm.mmmm,H)" },
            { name = "lon", type = "string", default = "11623.5678,E", label = "经度 (dddmm.mmmm,H)" },
            { name = "alt", type = "number", default = 50, label = "海拔 (m)" },
        },
    },
    gprmc = {
        label = "🗺️ 构造 RMC (推荐定位)",
        group = "NMEA",
        icon = "navigation",
        params = {
            { name = "lat", type = "string", default = "3956.1234,N", label = "纬度" },
            { name = "lon", type = "string", default = "11623.5678,E", label = "经度" },
            { name = "speed", type = "number", default = 0, label = "速度 (knots)" },
        },
    },
}



local sentence_buffer = {}

-- Compute NMEA checksum: XOR of all bytes between $ and * (exclusive)
local function nmea_checksum(str)
    local cs = 0
    -- Skip leading '$', stop at '*'
    for i = 2, #str do
        local c = str:byte(i)
        if c == string.byte("*") then
            break
        end
        cs = bit.bxor(cs, c)
    end
    return cs
end

-- Validate a complete NMEA sentence string
local function validate_sentence(s)
    -- Must start with $ and contain * followed by 2 hex digits
    if not s:match("^%$") then
        return false, "missing '$' prefix"
    end
    local star_pos = s:find("*", 1, true)
    if not star_pos then
        return false, "missing '*' checksum delimiter"
    end
    if #s < star_pos + 2 then
        return false, "checksum too short"
    end
    -- Verify checksum
    local expected = s:sub(star_pos + 1, star_pos + 2)
    local cs = nmea_checksum(s:sub(1, star_pos - 1))
    local cs_str = string.format("%02X", cs)
    if expected:upper() ~= cs_str then
        return false, "checksum mismatch: expected " .. cs_str .. " got " .. expected
    end
    return true
end

-- Parse fields from a sentence (without $ and *CS)
local function parse_fields(body)
    local fields = {}
    for field in body:gmatch("([^,]*)") do
        table.insert(fields, field)
    end
    return fields
end

function on_send(data)
    local str = bytes_to_string(data)
    -- If it already looks like a full sentence, leave it
    if str:match("^%$.*%*%x%x") then
        return data
    end
    -- If it starts with $ but no checksum, compute and append
    if str:match("^%$") then
        -- Remove trailing \r\n if present
        str = str:gsub("[\r\n]+$", "")
        local cs = nmea_checksum(str)
        str = str .. "*" .. string.format("%02X", cs) .. "\r\n"
        return string_to_bytes(str)
    end
    -- Otherwise treat as raw sentence body (without $)
    str = str:gsub("[\r\n]+$", "")
    local sentence = "$" .. str
    local cs = nmea_checksum(sentence)
    sentence = sentence .. "*" .. string.format("%02X", cs) .. "\r\n"
    return string_to_bytes(sentence)
end

function on_recv(data)
    local str = bytes_to_string(data)

    -- Accumulate into buffer
    table.insert(sentence_buffer, str)
    local full = table.concat(sentence_buffer)

    -- NMEA sentences are terminated by \n (or \r\n)
    -- Try to extract complete sentences
    local results = {}
    local pos = 1
    while pos <= #full do
        -- Find the next '$'
        local dollar = full:find("$", pos, true)
        if not dollar then
            break
        end
        -- Find the next \n after the $
        local nl = full:find("\n", dollar, true)
        if not nl then
            -- Incomplete sentence, keep from $ onwards
            sentence_buffer = { full:sub(dollar) }
            return nil
        end
        -- Extract the sentence (trim \r\n)
        local sentence = full:sub(dollar, nl):gsub("[\r\n]+$", "")
        if #sentence > 0 then
            table.insert(results, sentence)
        end
        pos = nl + 1
    end

    -- Clear buffer
    sentence_buffer = {}

    if #results == 0 then
        return nil
    end

    -- Join multiple sentences with \n
    local output = table.concat(results, "\n")
    return string_to_bytes(output)
end

-- ── Actions ─────────────────────────────────────────────────────────────────

--- Parse a single NMEA sentence into its components.
-- @param sentence string Full NMEA sentence ($...*CS)
-- @return table { talker = "GP", sentence_id = "GGA", fields = {...}, raw = "..." }
function action_parse(sentence)
    local ok, err = validate_sentence(sentence)
    if not ok then
        return { _error = err }
    end
    -- Extract talker + sentence ID
    local body = sentence:match("^%$([^%*]+)")
    local talker_sentence = body:match("^([^,]+)")
    local talker = talker_sentence:sub(1, 2)
    local sentence_id = talker_sentence:sub(3)
    local fields = parse_fields(body)
    -- Remove the first field (talker+sentence) from fields
    table.remove(fields, 1)
    return {
        talker = talker,
        sentence_id = sentence_id,
        fields = fields,
        raw = sentence,
    }
end

--- Generate a GGA (Global Positioning System Fix Data) sentence.
-- @param lat string  Latitude in NMEA format (ddmm.mmmm)
-- @param ns string   "N" or "S"
-- @param lon string  Longitude in NMEA format (dddmm.mmmm)
-- @param ew string   "E" or "W"
-- @param fix number  Fix quality (0=invalid, 1=GPS, 2=DGPS, ...)
-- @param sats number Number of satellites
-- @return string Full NMEA sentence
function action_make_gga(lat, ns, lon, ew, fix, sats)
    local time = os.date("!%H%M%S.000")
    local body = "GPGGA," .. time .. "," .. lat .. "," .. ns .. "," ..
                 lon .. "," .. ew .. "," .. fix .. "," .. sats .. ",1.0,0.0,M,0.0,M,,"
    local sentence = "$" .. body
    local cs = nmea_checksum(sentence)
    return sentence .. "*" .. string.format("%02X", cs) .. "\r\n"
end

--- Generate a RMC (Recommended Minimum) sentence.
-- @param lat string  Latitude in NMEA format
-- @param ns string   "N" or "S"
-- @param lon string  Longitude in NMEA format
-- @param ew string   "E" or "W"
-- @param speed number Speed in knots
-- @param course number Course over ground in degrees
-- @return string Full NMEA sentence
function action_make_rmc(lat, ns, lon, ew, speed, course)
    local time = os.date("!%H%M%S.000")
    local date = os.date("!%d%m%y")
    local body = "GPRMC," .. time .. ",A," .. lat .. "," .. ns .. "," ..
                 lon .. "," .. ew .. "," .. speed .. "," .. course .. "," ..
                 date .. ",,,A"
    local sentence = "$" .. body
    local cs = nmea_checksum(sentence)
    return sentence .. "*" .. string.format("%02X", cs) .. "\r\n"
end

--- Validate an NMEA sentence.
-- @param sentence string Full NMEA sentence
-- @return table { valid = true/false, error = "..." or nil }
function action_validate(sentence)
    local ok, err = validate_sentence(sentence)
    return { valid = ok, error = err }
end
