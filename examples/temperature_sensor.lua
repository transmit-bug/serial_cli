-- Example: Temperature sensor protocol script
-- Demonstrates a simple binary protocol for temperature readings

-- Protocol format:
-- Request: [0x01] [0x03] [sensor_id] [checksum]
-- Response: [0x01] [0x03] [temp_high] [temp_low] [humidity] [checksum]

-- Calculate XOR checksum
local function calc_checksum(data)
    local checksum = 0
    for i = 1, #data do
        checksum = checksum ~ data[i]
    end
    return checksum
end

-- Convert temperature bytes to Celsius
local function bytes_to_temp(high, low)
    local raw = (high << 8) | low
    -- Signed 16-bit value, 0.1°C resolution
    if raw > 32767 then
        raw = raw - 65536
    end
    return raw / 10.0
end

function on_send(data)
    -- data is a table of bytes
    -- Add checksum to request
    local checksum = calc_checksum(data)
    local frame = {}
    for i, b in ipairs(data) do
        table.insert(frame, b)
    end
    table.insert(frame, checksum)
    return frame
end

function on_recv(data)
    -- data is a table of bytes
    -- Minimum response length: 5 bytes (header + temp + humidity + checksum)
    if #data < 5 then
        return nil  -- Incomplete frame
    end

    -- Verify header
    if data[1] ~= 0x01 or data[2] ~= 0x03 then
        log_warn("Invalid response header")
        return nil
    end

    -- Verify checksum
    local received_checksum = data[#data]
    local calc_data = {}
    for i = 1, #data - 1 do
        table.insert(calc_data, data[i])
    end
    local expected_checksum = calc_checksum(calc_data)

    if received_checksum ~= expected_checksum then
        log_warn("Checksum mismatch: expected " .. expected_checksum .. ", got " .. received_checksum)
        return nil
    end

    -- Parse temperature and humidity
    local temp = bytes_to_temp(data[3], data[4])
    local humidity = data[5]

    -- Create JSON output
    local result = {
        temperature = temp,
        humidity = humidity,
        unit = "Celsius"
    }

    -- Return JSON as bytes
    return string_to_bytes(json_encode(result))
end
