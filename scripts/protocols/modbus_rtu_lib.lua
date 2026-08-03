-- modbus_rtu_lib.lua: Modbus RTU utility library
--
-- Provides reusable functions for Modbus RTU communication:
-- CRC16 calculation, frame building, frame parsing, exception handling.
--
-- Usage:
--   local modbus = require("modbus_rtu_lib")
--   local crc = modbus.calculate_crc(data)
--   local request = modbus.build_read_request(slave_id, 0x03, 0x0000, 1)

local M = {}

--- Calculate Modbus CRC16 checksum.
-- @param data table: byte array
-- @return number: 16-bit CRC value
function M.calculate_crc(data)
    local crc = 0xFFFF
    for _, byte in ipairs(data) do
        crc = bit.bxor(crc, byte)
        for _ = 0, 7 do
            if bit.band(crc, 1) == 1 then
                crc = bit.bxor(bit.rshift(crc, 1), 0xA001)
            else
                crc = bit.rshift(crc, 1)
            end
        end
    end
    return crc
end

--- Append CRC16 to a data frame (in-place).
-- @param data table: byte array (modified in place)
-- @return table: data with CRC appended (low byte first)
function M.append_crc(data)
    local crc = M.calculate_crc(data)
    table.insert(data, bit.band(crc, 0xFF))        -- CRC low byte
    table.insert(data, bit.band(bit.rshift(crc, 8), 0xFF)) -- CRC high byte
    return data
end

--- Verify CRC16 of a received frame.
-- @param frame table: byte array including CRC
-- @return boolean: true if CRC matches
function M.verify_crc(frame)
    if #frame < 4 then
        return false
    end
    local data_len = #frame - 2
    local data = {}
    for i = 1, data_len do
        data[i] = frame[i]
    end
    local expected = M.calculate_crc(data)
    local actual = bit.bor(frame[#frame - 1], bit.lshift(frame[#frame], 8))
    return expected == actual
end

--- Build a Modbus read request (FC 0x03 or 0x04).
-- @param slave_id number: slave address (1-247)
-- @param func_code number: function code (0x03 = hold register, 0x04 = input register)
-- @param start_addr number: starting register address
-- @param quantity number: number of registers to read
-- @return table: byte array with CRC appended
function M.build_read_request(slave_id, func_code, start_addr, quantity)
    local frame = {
        slave_id,
        func_code,
        bit.band(bit.rshift(start_addr, 8), 0xFF),
        bit.band(start_addr, 0xFF),
        bit.band(bit.rshift(quantity, 8), 0xFF),
        bit.band(quantity, 0xFF),
    }
    return M.append_crc(frame)
end

--- Build a Modbus write single register request (FC 0x06).
-- @param slave_id number: slave address (1-247)
-- @param addr number: register address
-- @param value number: value to write (0-65535)
-- @return table: byte array with CRC appended
function M.build_write_single_request(slave_id, addr, value)
    local frame = {
        slave_id,
        0x06,
        bit.band(bit.rshift(addr, 8), 0xFF),
        bit.band(addr, 0xFF),
        bit.band(bit.rshift(value, 8), 0xFF),
        bit.band(value, 0xFF),
    }
    return M.append_crc(frame)
end

--- Build a Modbus write multiple registers request (FC 0x10).
-- @param slave_id number: slave address (1-247)
-- @param start_addr number: starting register address
-- @param values table: array of register values
-- @return table: byte array with CRC appended
function M.build_write_multiple_request(slave_id, start_addr, values)
    local byte_count = #values * 2
    local frame = {
        slave_id,
        0x10,
        bit.band(bit.rshift(start_addr, 8), 0xFF),
        bit.band(start_addr, 0xFF),
        bit.band(bit.rshift(#values, 8), 0xFF),
        bit.band(#values, 0xFF),
        byte_count,
    }
    for _, val in ipairs(values) do
        table.insert(frame, bit.band(bit.rshift(val, 8), 0xFF))
        table.insert(frame, bit.band(val, 0xFF))
    end
    return M.append_crc(frame)
end

--- Check if a response is a Modbus exception.
-- @param response table: byte array
-- @return boolean: true if exception response
function M.is_exception(response)
    return response ~= nil and #response >= 2 and bit.band(response[2], 0x80) ~= 0
end

--- Get exception code from an exception response.
-- @param response table: byte array (exception response)
-- @return number: exception code (1-6)
function M.exception_code(response)
    if M.is_exception(response) then
        return response[3]
    end
    return 0
end

--- Convert exception code to human-readable string.
-- @param code number: exception code
-- @return string: description
function M.exception_to_string(code)
    local exceptions = {
        [1] = "Illegal Function",
        [2] = "Illegal Data Address",
        [3] = "Illegal Data Value",
        [4] = "Slave Device Failure",
        [5] = "Acknowledge",
        [6] = "Slave Device Busy",
    }
    return exceptions[code] or ("Unknown Exception (" .. code .. ")")
end

--- Parse register values from a read response (FC 0x03/0x04).
-- @param response table: byte array (complete response frame)
-- @return table: array of 16-bit register values, or nil on error
function M.parse_registers(response)
    if not response or #response < 5 then
        return nil
    end
    if M.is_exception(response) then
        return nil
    end
    local byte_count = response[3]
    local registers = {}
    for i = 0, byte_count / 2 - 1 do
        local hi = response[4 + i * 2]
        local lo = response[5 + i * 2]
        registers[i + 1] = bit.bor(bit.lshift(hi, 8), lo)
    end
    return registers
end

return M
