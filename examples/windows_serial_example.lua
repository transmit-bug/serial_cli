#!/usr/bin/env serial-cli run
-- windows_serial_example.lua: Windows-Specific Serial Communication
--
-- This script demonstrates serial port communication on Windows with
-- platform-specific features and configurations. Windows serial ports
-- have unique characteristics compared to Unix-like systems.
--
-- Windows-Specific Features:
--   - COM port names (COM1, COM2, COM3, etc.)
--   - Hardware flow control (RTS/CTS)
--   - Signal line control (DTR, RTS)
--   - Different timeout behavior
--
-- Prerequisites:
--   - Windows 10/11
--   - Serial drivers installed for your device
--   - Administrator privileges (may be required for some ports)
--
-- Usage:
--   serial-cli run examples/windows_serial_example.lua --port=COM3
--   serial-cli run examples/windows_serial_example.lua COM3 115200

-- Parse command line arguments
local args = {...}
local port_name = args[1] or "COM3"
local baudrate = tonumber(args[2]) or 115200

log_info("=== Windows Serial Communication Example ===")
log_info("Port: " .. port_name .. " @ " .. baudrate)

-- Windows-specific port configuration
local config = {
    -- Basic communication parameters
    baudrate = baudrate,
    data_bits = 8,
    stop_bits = 1,
    parity = "none",

    -- Flow control (important for Windows modems and devices)
    flow_control = "hardware",  -- RTS/CTS hardware flow control
    -- Alternative: "none" for no flow control
    -- Alternative: "software" for XON/XOFF

    -- Timeout configuration (Windows behavior differs from Unix)
    timeout = 1000,  -- 1 second total timeout

    -- Windows-specific signal line controls
    dtr_enable = true,   -- Data Terminal Ready - often required by modems
    rts_enable = true    -- Request to Send - used for hardware flow control
}

log_info("Configuration:")
log_info("  Data Bits: " .. config.data_bits)
log_info("  Stop Bits: " .. config.stop_bits)
log_info("  Parity: " .. config.parity)
log_info("  Flow Control: " .. config.flow_control)
log_info("  DTR: " .. tostring(config.dtr_enable))
log_info("  RTS: " .. tostring(config.rts_enable))

-- Open the serial port
log_info("\nOpening port...")
local ok, port = pcall(serial_open, port_name, config)

if not ok then
    log_error("Failed to open port: " .. tostring(port))
    log_error("\nTroubleshooting:")
    log_error("  1. Check if the port exists: serial-cli list")
    log_error("  2. Ensure device drivers are installed")
    log_error("  3. Close other programs using the port")
    log_error("  4. Run as Administrator if needed")
    log_error("  5. Check Device Manager for port conflicts")
    return 1
end

log_info("✓ Port opened successfully")

-- Send AT command (common for Windows modems)
log_info("\n--- Sending AT Command ---")
log_info("Sending: AT")

local ok, send_err = pcall(function()
    port:write("AT\r\n")
end)

if not ok then
    log_error("Failed to send data: " .. tostring(send_err))
    serial_close(port)
    return 1
end

log_info("✓ Data sent successfully")

-- Wait for response (Windows timing can be different)
sleep_ms(500)

-- Read response
log_info("\n--- Reading Response ---")
local ok, response = pcall(function()
    return port:read(256)
end)

if ok and response and #response > 0 then
    log_info("✓ Received response (" .. #response .. " bytes)")
    log_info("Response: " .. response:gsub("\r", "\\r"):gsub("\n", "\\n"))

    -- Check for common responses
    if response:find("OK") then
        log_info("Status: Device responded with OK")
    elseif response:find("ERROR") then
        log_warn("Status: Device responded with ERROR")
    else
        log_info("Status: Custom response received")
    end
else
    log_warn("No response received (timeout)")
    log_info("Note: This is normal if the device doesn't respond to AT commands")
end

-- Demonstrate Windows-specific signal control (if supported)
log_info("\n--- Signal Line Control ---")

-- Toggle DTR (Data Terminal Ready)
log_info("Toggling DTR line...")
local ok, dtr_err = pcall(function()
    -- This is device-specific - not all devices support it
    -- port:set_dtr(false)
    -- sleep_ms(100)
    -- port:set_dtr(true)
end)

if not ok then
    log_info("Note: DTR control not available or not supported")
end

-- Close the port properly (Windows requires proper cleanup)
log_info("\n--- Cleaning Up ---")
local ok, close_err = pcall(function()
    serial_close(port)
end)

if ok then
    log_info("✓ Port closed successfully")
else
    log_warn("Warning: Error closing port: " .. tostring(close_err))
end

log_info("\n=== Windows Serial Example Complete ===")
log_info("\nWindows-Specific Tips:")
log_info("  1. Use Device Manager to verify COM port numbers")
log_info("  2. Some ports require Administrator privileges")
log_info("  3. USB serial adapters may get different COM numbers on reconnect")
log_info("  4. Use 'mode' command in Command Prompt to check port status")
log_info("  5. Disable Windows' serial mouse detection on COM1 if needed")

return 0
