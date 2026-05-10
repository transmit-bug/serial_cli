#!/usr/bin/env serial-cli run
-- AI Basic Example
-- Demonstrates fundamental serial operations with structured JSON output

-- Configuration
local PORT = arg[1] or "/dev/ttyUSB0"
local BAUDRATE = tonumber(arg[2]) or 115200
local TIMEOUT = 1000  -- 1 second timeout

-- Helper function for JSON output
local function output_json(status, data_or_error)
  local result = {
    status = status,
    timestamp = time_now()
  }

  if status == "ok" then
    result.data = data_or_error
  else
    result.error = data_or_error
  end

  print(json_encode(result))
end

-- Main execution
local function main()
  log_info("Starting AI basic example")

  -- Step 1: Open port
  log_info("Opening port: " .. PORT)
  local ok, port_id = pcall(serial_open, PORT, {baudrate = BAUDRATE})
  if not ok then
    output_json("error", {
      operation = "open_port",
      port = PORT,
      message = port_id
    })
    os.exit(1)
  end

  log_info("Port opened successfully: " .. port_id)

  -- Step 2: Send data
  local test_data = "AT\r\n"
  log_info("Sending data: " .. test_data:gsub("\r", "\\r"):gsub("\n", "\\n"))

  local ok, bytes_sent = pcall(serial_send, port_id, test_data)
  if not ok then
    output_json("error", {
      operation = "send_data",
      port_id = port_id,
      message = bytes_sent
    })
    serial_close(port_id)
    os.exit(1)
  end

  log_info("Sent " .. tostring(bytes_sent) .. " bytes")

  -- Step 3: Receive response
  log_info("Waiting for response (timeout: " .. TIMEOUT .. "ms)")

  local ok, response = pcall(serial_recv, port_id, TIMEOUT)
  if not ok then
    -- Timeout is not necessarily an error
    response = "No response (timeout)"
  end

  log_info("Received response")

  -- Step 4: Process response
  local output_data = {
    port = PORT,
    port_id = port_id,
    bytes_sent = bytes_sent,
    response = response,
    response_hex = string_to_hex(response),
    response_length = string.len(response)
  }

  -- Step 5: Close port
  log_info("Closing port")
  local ok, close_result = pcall(serial_close, port_id)
  if not ok then
    log_warn("Failed to close port: " .. close_result)
  end

  -- Output final result
  output_json("ok", output_data)

  log_info("Example completed successfully")
  os.exit(0)
end

-- Execute with error handling
local success, error_msg = pcall(main)
if not success then
  output_json("error", {
    operation = "main",
    message = error_msg
  })
  os.exit(1)
end
