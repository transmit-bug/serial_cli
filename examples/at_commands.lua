#!/usr/bin/env serial-cli run
-- at_commands.lua: AT Command Protocol Example
--
-- This script demonstrates AT command communication with modems, GSM modules,
-- WiFi modules, and other AT-command compatible devices. It includes proper
-- error handling, timeout management, and common AT command patterns.
--
-- Common AT Command Devices:
--   - GSM/3G/4G/LTE modems (SIMcom, Quectel, u-blox)
--   - WiFi modules (ESP8266, ESP32)
--   - Bluetooth modules (HC-05, HC-06)
--   - GPS modules
--
-- Usage:
--   serial-cli run examples/protocols/at_commands.lua --port=/dev/ttyUSB0 --baudrate=115200
--   serial-cli run examples/protocols/at_commands.lua /dev/ttyUSB0 9600

-- Parse command line arguments
local args = {...}
local port_name = args[1] or "/dev/ttyUSB0"
local baudrate = tonumber(args[2]) or 115200
local timeout = tonumber(args[3]) or 2000  -- 2 second default timeout

-- Helper function to send AT command and get response
local function send_at_command(port, command, expected_response, timeout_ms)
    timeout_ms = timeout_ms or timeout

    -- Format command with CRLF
    local full_command = command .. "\r\n"

    log_info("Sending: " .. command)

    -- Send command
    local ok, write_err = pcall(function()
        port:write(full_command)
    end)

    if not ok then
        log_error("Failed to send command: " .. tostring(write_err))
        return nil, write_err
    end

    -- Wait a bit for response
    sleep_ms(100)

    -- Read response
    local response = port:read(256)
    if response and #response > 0 then
        -- Clean up response (remove extra whitespace)
        response = response:match("^%s*(.-)%s*$")
        log_info("Response: " .. response)

        -- Check for expected response
        if expected_response and response:find(expected_response) then
            return response
        elseif response:find("OK") then
            return response
        elseif response:find("ERROR") then
            return nil, "Command returned ERROR"
        else
            return response
        end
    else
        log_warn("No response received (timeout)")
        return nil, "Timeout"
    end
end

-- Main execution
local function main()
    log_info("=== AT Command Protocol Example ===")
    log_info("Port: " .. port_name .. " @ " .. baudrate)

    -- Open serial port
    local ok, port = pcall(function()
        return serial.open(port_name, {
            baudrate = baudrate,
            timeout = timeout,
            databits = 8,
            stopbits = 1,
            parity = "none"
        })
    end)

    if not ok then
        log_error("Failed to open port: " .. tostring(port))
        return 1
    end

    log_info("Port opened successfully")

    -- Set protocol to AT Command (if available)
    local ok, proto_err = pcall(function()
        port:set_protocol("at_command")
    end)

    if not ok then
        log_info("Note: AT command protocol not set, continuing with raw mode: " .. tostring(proto_err))
    end

    -- Test 1: Basic AT command (checks if device is responding)
    log_info("\n--- Test 1: Basic AT Command ---")
    local response, err = send_at_command(port, "AT", "OK")
    if response then
        log_info("✓ Device is responding")
    else
        log_error("✗ Device not responding: " .. tostring(err))
    end

    -- Test 2: Get module information
    log_info("\n--- Test 2: Module Information ---")
    response, err = send_at_command(port, "ATI", nil)
    if response then
        log_info("Module Info: " .. response)
    else
        log_warn("ATI not supported or failed: " .. tostring(err))
    end

    -- Test 3: Check SIM card (GSM modules)
    log_info("\n--- Test 3: SIM Card Status ---")
    response, err = send_at_command(port, "AT+CPIN?", "READY")
    if response and response:find("READY") then
        log_info("✓ SIM card ready")
    elseif response then
        log_info("SIM Status: " .. response)
    else
        log_info("SIM check not supported (may not be a GSM module)")
    end

    -- Test 4: Signal strength (GSM/LTE modules)
    log_info("\n--- Test 4: Signal Strength ---")
    response, err = send_at_command(port, "AT+CSQ", nil)
    if response then
        -- Parse CSQ response: +CSQ: <rssi>,<ber>
        local rssi, ber = response:match("+CSQ:%s*(%d+),%s*(%d+)")
        if rssi then
            log_info(string.format("Signal: RSSI=%d, BER=%s", rssi, ber))

            -- Convert RSSI to approximate signal strength
            local rssi_num = tonumber(rssi)
            if rssi_num >= 20 then
                log_info("Signal quality: Excellent")
            elseif rssi_num >= 15 then
                log_info("Signal quality: Good")
            elseif rssi_num >= 10 then
                log_info("Signal quality: Fair")
            else
                log_info("Signal quality: Poor")
            end
        end
    else
        log_info("Signal check not supported")
    end

    -- Test 5: Network registration (GSM modules)
    log_info("\n--- Test 5: Network Registration ---")
    response, err = send_at_command(port, "AT+CREG?", "1")
    if response then
        local stat = response:match("+CREG:%s*%d+,(%d+)")
        if stat == "1" then
            log_info("✓ Registered to home network")
        elseif stat == "5" then
            log_info("✓ Registered to roaming network")
        else
            log_info("Registration status: " .. (stat or "unknown"))
        end
    else
        log_info("Network registration check not supported")
    end

    -- Test 6: Firmware version (device specific)
    log_info("\n--- Test 6: Firmware Version ---")
    response, err = send_at_command(port, "AT+GMR", nil)
    if response then
        log_info("Firmware: " .. response)
    else
        log_info("Firmware version command not supported")
    end

    -- Test 7: Reset to factory defaults (optional, commented out)
    -- log_info("\n--- Test 7: Factory Reset ---")
    -- response, err = send_at_command(port, "AT&F", "OK")
    -- if response then
    --     log_info("✓ Reset to factory defaults")
    -- else
    --     log_warn("Factory reset failed")
    -- end

    -- Clean up
    log_info("\n--- Cleaning Up ---")
    port:close()
    log_info("Port closed successfully")
    log_info("\n=== AT Command Example Complete ===")

    return 0
end

-- Execute with error handling
local success, error_msg = pcall(main)
if not success then
    log_error("Fatal error: " .. tostring(error_msg))
    os.exit(1)
end

os.exit(0)
