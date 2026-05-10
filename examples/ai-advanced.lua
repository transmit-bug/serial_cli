#!/usr/bin/env serial-cli run
-- AI Advanced Example
-- Demonstrates complex automation: device discovery, protocol handling, batch operations

-- Configuration
local MAX_RETRIES = 3
local RETRY_DELAY = 1000  -- ms
local COMMAND_TIMEOUT = 2000  -- ms

-- Test commands to execute
local TEST_COMMANDS = {
  {name = "AT", data = "AT\r\n"},
  {name = "ATI", data = "ATI\r\n"},
  {name = "AT+VER", data = "AT+VER\r\n"},
  {name = "AT+TEMP", data = "AT+TEMP?\r\n"}
}

-- State management
local state = {
  port_id = nil,
  results = {},
  errors = {},
  start_time = time_now()
}

-- Helper functions
local function log_context(message, context)
  local entry = {
    timestamp = time_now(),
    message = message,
    context = context or {}
  }
  table.insert(state.results, entry)
  log_info(message)
end

local function output_final_result()
  local duration_ms = time_now() - state.start_time

  local final_result = {
    status = "ok",
    duration_ms = duration_ms,
    commands_executed = #state.results,
    errors = #state.errors,
    results = state.results,
    error_details = state.errors
  }

  print(json_encode(final_result))
end

-- Step 1: Device Discovery
local function discover_devices()
  log_context("Starting device discovery")

  local ok, ports = pcall(serial_list)
  if not ok then
    table.insert(state.errors, {
      operation = "device_discovery",
      message = ports
    })
    return nil
  end

  log_context("Found " .. #ports .. " serial ports", {count = #ports})

  -- Find first available USB port
  for i, port in ipairs(ports) do
    if string.find(port.port_type:lower(), "usb") then
      log_context("Selected USB port", {port = port.port_name})
      return port.port_name
    end
  end

  -- Fallback to first port
  if #ports > 0 then
    log_context("Using first available port", {port = ports[1].port_name})
    return ports[1].port_name
  end

  table.insert(state.errors, {
    operation = "device_discovery",
    message = "No serial ports found"
  })
  return nil
end

-- Step 2: Connect with Retry Logic
local function connect_with_retry(port_name, baudrate)
  log_context("Connecting to port", {port = port_name, baudrate = baudrate})

  for attempt = 1, MAX_RETRIES do
    log_context("Connection attempt " .. attempt .. "/" .. MAX_RETRIES)

    local ok, port_id = pcall(serial_open, port_name, {
      baudrate = baudrate,
      databits = 8,
      stopbits = 1,
      parity = "none",
      timeout = COMMAND_TIMEOUT
    })

    if ok then
      log_context("Connection successful", {port_id = port_id})
      return port_id
    end

    log_context("Connection failed", {
      attempt = attempt,
      error = port_id
    })

    if attempt < MAX_RETRIES then
      log_context("Retrying after delay", {delay_ms = RETRY_DELAY})
      sleep_ms(RETRY_DELAY)
    end
  end

  table.insert(state.errors, {
    operation = "connect",
    message = "Max retries exceeded"
  })
  return nil
end

-- Step 3: Execute Command with Protocol Support
local function execute_command(command_info)
  local command_entry = {
    name = command_info.name,
    data = command_info.data,
    start_time = time_now()
  }

  log_context("Executing command", {name = command_info.name})

  -- Send command
  local ok, bytes_sent = pcall(serial_send, state.port_id, command_info.data)
  if not ok then
    command_entry.status = "error"
    command_entry.error = bytes_sent
    table.insert(state.errors, {
      operation = "send_command",
      command = command_info.name,
      message = bytes_sent
    })
    table.insert(state.results, command_entry)
    return false
  end

  command_entry.bytes_sent = bytes_sent

  -- Receive response
  local ok, response = pcall(serial_recv, state.port_id, COMMAND_TIMEOUT)
  if not ok then
    response = "No response (timeout)"
  end

  command_entry.response = response
  command_entry.response_hex = string_to_hex(response)
  command_entry.response_length = string.len(response)
  command_entry.duration_ms = time_now() - command_entry.start_time

  -- Parse response status
  if string.find(response:upper(), "OK") then
    command_entry.status = "ok"
  elseif string.find(response:upper(), "ERROR") then
    command_entry.status = "error"
    command_entry.error_code = "command_error"
  else
    command_entry.status = "unknown"
  end

  log_context("Command completed", {
    name = command_info.name,
    status = command_entry.status,
    duration_ms = command_entry.duration_ms
  })

  table.insert(state.results, command_entry)
  return command_entry.status == "ok"
end

-- Step 4: Batch Command Execution
local function execute_command_sequence()
  log_context("Starting command sequence", {count = #TEST_COMMANDS})

  local success_count = 0
  for i, cmd in ipairs(TEST_COMMANDS) do
    log_context("Command " .. i .. "/" .. #TEST_COMMANDS, {name = cmd.name})

    if execute_command(cmd) then
      success_count = success_count + 1
    end

    -- Small delay between commands
    if i < #TEST_COMMANDS then
      sleep_ms(100)
    end
  end

  log_context("Command sequence completed", {
    total = #TEST_COMMANDS,
    successful = success_count,
    failed = #TEST_COMMANDS - success_count
  })

  return success_count > 0
end

-- Step 5: Protocol Information
local function show_protocol_info()
  log_context("Fetching protocol information")

  local ok, protocols = pcall(protocol_list)
  if ok then
    local protocol_names = {}
    for i, proto in ipairs(protocols) do
      table.insert(protocol_names, proto.name)
    end

    log_context("Available protocols", {
      protocols = table.concat(protocol_names, ", "),
      count = #protocols
    })

    return protocols
  else
    log_context("Failed to get protocol list")
    return {}
  end
end

-- Cleanup
local function cleanup()
  if state.port_id then
    log_context("Closing port", {port_id = state.port_id})
    local ok, result = pcall(serial_close, state.port_id)
    if not ok then
      log_context("Warning: Failed to close port", {error = result})
    end
    state.port_id = nil
  end
end

-- Main execution
local function main()
  log_context("Starting AI advanced example")

  -- Set up cleanup on exit
  local exit_handler = function()
    cleanup()
    output_final_result()
  end

  -- Execute steps
  local port_name = discover_devices()
  if not port_name then
    log_context("Device discovery failed")
    exit_handler()
    os.exit(1)
  end

  local baudrate = 115200
  state.port_id = connect_with_retry(port_name, baudrate)
  if not state.port_id then
    log_context("Connection failed")
    exit_handler()
    os.exit(1)
  end

  -- Show protocol info
  show_protocol_info()

  -- Execute command sequence
  local success = execute_command_sequence()

  -- Cleanup
  cleanup()

  if success then
    log_context("Example completed successfully")
    output_final_result()
    os.exit(0)
  else
    log_context("Example completed with errors")
    output_final_result()
    os.exit(1)
  end
end

-- Execute with comprehensive error handling
local success, error_msg = pcall(main)
if not success then
  table.insert(state.errors, {
    operation = "main",
    message = error_msg
  })
  cleanup()
  output_final_result()
  os.exit(1)
end
