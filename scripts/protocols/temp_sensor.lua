-- temp_sensor.lua: Modbus RTU Temperature Sensor Driver
--
-- A complete device driver demonstrating script imports via require().
-- Uses modbus_rtu_lib for CRC calculation, frame building, and response parsing.
--
-- Supported device: Generic Modbus RTU temperature/humidity sensor
-- Register map:
--   0x0000: Temperature (signed 16-bit, 0.1°C resolution)
--   0x0001: Humidity (unsigned 16-bit, 0.1% resolution)
--   0x0002: Alarm status (bit flags)
--
-- Usage:
--   serial-cli open /dev/ttyUSB0 --script temp_sensor --baudrate 9600

local modbus = require("modbus_rtu_lib")

SCRIPT_META = {
    name = "temp_sensor",
    version = "1.0.0",
    description = "Modbus RTU temperature/humidity sensor driver",
    author = "serial_cli",
    license = "MIT",
    tags = {"modbus", "temperature", "humidity", "sensor"},
    data_format = "binary",
    min_frame_size = 4,
}

-- Configuration
local SLAVE_ID = 1          -- Modbus slave address
local TEMP_REG = 0x0000     -- Temperature register
local HUMID_REG = 0x0001    -- Humidity register
local ALARM_REG = 0x0002    -- Alarm status register

-- State
local port_id = nil
local recv_buffer = {}

-- ========== Callbacks ==========

function on_open(port, config)
    port_id = port
    log_info("Temperature sensor driver opened on " .. port)
    log_info("Baudrate: " .. config.baudrate)
end

function on_send(data)
    -- Append CRC16 using the library
    return modbus.append_crc(data)
end

function on_recv(data)
    -- Accumulate data into frame buffer
    for _, b in ipairs(data) do
        table.insert(recv_buffer, b)
    end

    -- Need at least 5 bytes for a valid response (addr + func + byte_count + data + crc)
    if #recv_buffer < 5 then
        return nil
    end

    -- Check if we have a complete frame
    local func_code = recv_buffer[2]

    -- Exception response: 3 bytes + 2 CRC = 5 bytes
    if bit.band(func_code, 0x80) ~= 0 then
        if #recv_buffer >= 5 then
            local frame = {}
            for i = 1, 5 do
                frame[i] = recv_buffer[i]
            end
            recv_buffer = {}

            if modbus.verify_crc(frame) then
                return frame
            else
                log_warn("CRC mismatch on exception response")
                return nil
            end
        end
        return nil
    end

    -- Normal read response: addr(1) + func(1) + byte_count(1) + data(N) + crc(2)
    local byte_count = recv_buffer[3]
    local frame_len = 3 + byte_count + 2

    if #recv_buffer < frame_len then
        return nil  -- Need more data
    end

    -- Extract complete frame
    local frame = {}
    for i = 1, frame_len do
        frame[i] = recv_buffer[i]
    end

    -- Remove consumed bytes from buffer
    local remaining = {}
    for i = frame_len + 1, #recv_buffer do
        table.insert(remaining, recv_buffer[i])
    end
    recv_buffer = remaining

    -- Verify CRC
    if not modbus.verify_crc(frame) then
        log_warn("CRC mismatch, discarding frame")
        return nil
    end

    return frame
end

function on_close()
    log_info("Temperature sensor driver closed")
    port_id = nil
    recv_buffer = {}
end

-- ========== Helper Functions ==========

--- Read a single register value.
-- @param reg_addr number: register address
-- @return number|nil: register value, or nil on error
local function read_register(reg_addr)
    if not port_id then
        return nil
    end

    local request = modbus.build_read_request(SLAVE_ID, 0x03, reg_addr, 1)
    local response = serial_query(port_id, request, 1000)

    if not response then
        log_warn("Timeout reading register 0x" .. string.format("%04X", reg_addr))
        return nil
    end

    if modbus.is_exception(response) then
        local code = modbus.exception_code(response)
        log_error("Modbus exception: " .. modbus.exception_to_string(code))
        return nil
    end

    local regs = modbus.parse_registers(response)
    if regs and #regs >= 1 then
        return regs[1]
    end
    return nil
end

-- ========== UI Actions ==========

--- Read current temperature.
-- @return table: { temperature = number (°C) } or { _error = string }
function action_read_temperature()
    local raw = read_register(TEMP_REG)
    if raw == nil then
        return { _error = "read_failed" }
    end

    -- Convert unsigned 16-bit to signed, then scale by 0.1
    if raw >= 0x8000 then
        raw = raw - 0x10000
    end
    local temperature = raw / 10.0

    return {
        temperature = temperature,
        unit = "°C",
        raw = raw,
    }
end

--- Read current humidity.
-- @return table: { humidity = number (%RH) } or { _error = string }
function action_read_humidity()
    local raw = read_register(HUMID_REG)
    if raw == nil then
        return { _error = "read_failed" }
    end

    local humidity = raw / 10.0

    return {
        humidity = humidity,
        unit = "%RH",
        raw = raw,
    }
end

--- Read alarm status.
-- @return table: { alarms = table } or { _error = string }
function action_read_alarms()
    local raw = read_register(ALARM_REG)
    if raw == nil then
        return { _error = "read_failed" }
    end

    return {
        alarms = {
            temp_high = bit.band(raw, 0x01) ~= 0,
            temp_low = bit.band(raw, 0x02) ~= 0,
            humidity_high = bit.band(raw, 0x04) ~= 0,
            humidity_low = bit.band(raw, 0x08) ~= 0,
            sensor_fault = bit.band(raw, 0x10) ~= 0,
        },
        raw = raw,
    }
end

--- Read all sensor values at once.
-- @return table: { temperature, humidity, alarms } or { _error = string }
function action_read_all()
    if not port_id then
        return { _error = "port_not_open" }
    end

    -- Read 3 consecutive registers starting from 0x0000
    local request = modbus.build_read_request(SLAVE_ID, 0x03, 0x0000, 3)
    local response = serial_query(port_id, request, 1000)

    if not response then
        return { _error = "timeout" }
    end

    if modbus.is_exception(response) then
        local code = modbus.exception_code(response)
        return {
            _error = "modbus_exception",
            code = code,
            message = modbus.exception_to_string(code),
        }
    end

    local regs = modbus.parse_registers(response)
    if not regs or #regs < 3 then
        return { _error = "parse_error" }
    end

    -- Temperature: signed 16-bit, 0.1°C
    local temp_raw = regs[1]
    if temp_raw >= 0x8000 then
        temp_raw = temp_raw - 0x10000
    end

    -- Alarm flags
    local alarm_raw = regs[3]

    return {
        temperature = temp_raw / 10.0,
        humidity = regs[2] / 10.0,
        alarms = {
            temp_high = bit.band(alarm_raw, 0x01) ~= 0,
            temp_low = bit.band(alarm_raw, 0x02) ~= 0,
            humidity_high = bit.band(alarm_raw, 0x04) ~= 0,
            humidity_low = bit.band(alarm_raw, 0x08) ~= 0,
            sensor_fault = bit.band(alarm_raw, 0x10) ~= 0,
        },
    }
end

--- Write temperature alarm thresholds.
-- @param high number: high threshold (°C)
-- @param low number: low threshold (°C)
-- @return table: { success = boolean } or { _error = string }
function action_set_temp_alarm(high, low)
    -- This is a simplified example - real devices would have specific register addresses
    return { _error = "not_implemented" }
end
