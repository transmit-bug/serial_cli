# UI Actions - Script Function Integration

**Version**: 0.6.0  
**Feature**: Automatically expose Lua script functions as UI buttons

## Overview

UI Actions allow you to define Lua functions in your scripts that automatically appear as clickable buttons in the Serial CLI interface. This enables rapid prototyping and efficient debugging workflows.

## Quick Start

### 1. Define Action Functions

Create a Lua script with functions prefixed with `action_`:

```lua
function action_send_at()
    serial_send(port_id, "AT\r\n")
    log_info("Sent: AT")
end

function action_reset_device()
    serial_send(port_id, "ATZ\r\n")
    log_info("Device reset")
end
```

### 2. Attach Script to Port

- Open the **Script Panel** in Serial CLI
- Paste or load your script
- Click **Attach** to link it to the current port

### 3. Use the Buttons

- Navigate to **SidePanel → Script Actions** section
- Click any button to execute the corresponding function
- Buttons automatically appear when you attach the script

## Naming Convention

Functions with the `action_` prefix are automatically discovered and exposed as UI buttons:

```lua
function action_<name>()
    -- Your code here
end
```

The function name is automatically converted to a human-readable label:
- `action_send_at` → "Send At"
- `action_reset_device` → "Reset Device"
- `action_check_signal_strength` → "Check Signal Strength"

## Metadata Configuration

You can optionally provide metadata to customize button appearance and behavior using the `_actions` global table:

```lua
_actions = {
    send_at = {
        label = "📡 Send AT",           -- Custom label (supports emoji)
        group = "Basic Commands",        -- Group name for organization
        icon = "radio",                  -- Lucide React icon name
        confirm = true                   -- Show confirmation dialog
    },
    reset_device = {
        label = "🔄 Reset Device",
        group = "Device Control",
        confirm = true
    }
}

function action_send_at()
    serial_send(port_id, "AT\r\n")
end

function action_reset_device()
    serial_send(port_id, "ATZ\r\n")
end
```

### Metadata Fields

| Field | Type | Description |
|-------|------|-------------|
| `label` | string | Display text for the button (supports emoji) |
| `group` | string | Group name for organizing buttons |
| `icon` | string | Lucide React icon name (e.g., "radio", "refresh-cw") |
| `confirm` | boolean | If `true`, shows a confirmation dialog before execution |

## Examples

### Basic AT Commands

```lua
function action_send_at()
    serial_send(port_id, "AT\r\n")
end

function action_query_info()
    serial_send(port_id, "ATI\r\n")
    local response = serial_recv(port_id, 2000)
    log_info("Device info: " .. (response or "No response"))
end
```

### Advanced: Multi-Step Operations

```lua
function action_full_reset_sequence()
    log_info("Starting reset sequence...")

    -- Step 1: Send reset command
    serial_send(port_id, "AT+RST\r\n")
    sleep_ms(1000)

    -- Step 2: Wait for ready
    local ready = false
    for attempt = 1, 5 do
        local response = serial_recv(port_id, 1000)
        if response and response:find("READY") then
            ready = true
            break
        end
        sleep_ms(500)
    end

    -- Step 3: Report result
    if ready then
        log_info("Device ready after reset")
    else
        log_error("Device not ready after reset")
    end
end

_actions = {
    full_reset_sequence = {
        label = "🔄 Full Reset Sequence",
        group = "Advanced",
        confirm = true
    }
}
```

### Diagnostic Functions

```lua
function action_check_signal()
    serial_send(port_id, "AT+CSQ\r\n")
    local response = serial_recv(port_id, 2000)

    if response then
        local rssi = response:match("CSQ:%s*(%d+),")
        if rssi then
            local strength = tonumber(rssi)
            if strength > 20 then
                log_info("Signal strength: Excellent (" .. rssi .. ")")
            elseif strength > 10 then
                log_info("Signal strength: Good (" .. rssi .. ")")
            else
                log_warn("Signal strength: Weak (" .. rssi .. ")")
            end
        end
    end
end

function action_run_diagnostics()
    log_info("=== Starting Diagnostics ===")

    -- Test 1: AT command
    serial_send(port_id, "AT\r\n")
    local r1 = serial_recv(port_id, 1000)
    log_info("AT Test: " .. (r1 and "PASS" or "FAIL"))

    -- Test 2: Signal check
    action_check_signal()

    -- Test 3: Module info
    serial_send(port_id, "ATI\r\n")
    local r3 = serial_recv(port_id, 1000)
    log_info("Module Info: " .. (r3 or "Unavailable"))

    log_info("=== Diagnostics Complete ===")
end

_actions = {
    check_signal = { label = "📊 Signal Check", group = "Diagnostic" },
    run_diagnostics = { label = "🔧 Run Diagnostics", group = "Diagnostic" }
}
```

## Best Practices

### 1. Use Descriptive Function Names

```lua
-- Good: Clear and specific
function action_send_reset_command()
    serial_send(port_id, "AT+RST\r\n")
end

-- Avoid: Too generic
function action_reset()
    serial_send(port_id, "AT+RST\r\n")
end
```

### 2. Add Logging

```lua
function action_send_at()
    log_info("Sending AT command...")
    serial_send(port_id, "AT\r\n")
    local response = serial_recv(port_id, 1000)
    if response then
        log_info("Response: " .. response)
    else
        log_warn("No response received")
    end
end
```

### 3. Error Handling

```lua
function action_safe_operation()
    local ok, err = pcall(function()
        -- Your operation here
        serial_send(port_id, "AT+COMMAND\r\n")
        local response = serial_recv(port_id, 2000)
        if not response then
            error("No response received")
        end
    end)

    if not ok then
        log_error("Operation failed: " .. tostring(err))
    end
end
```

### 4. Use Confirmation for Destructive Actions

```lua
_actions = {
    factory_reset = {
        label = "🏭 Factory Reset",
        confirm = true  -- Always confirm for dangerous operations
    }
}
```

## Available Lua APIs

All standard Serial CLI Lua APIs are available in action functions:

### Serial Communication

```lua
serial_send(port_id, "data\r\n")
local response = serial_recv(port_id, 1000)
local ports = serial_list()
```

### Protocol Operations

```lua
local encoded = protocol_encode("at_command", "ATZ")
local decoded = protocol_decode("at_command", response)
```

### Utilities

```lua
log_info("Info message")
log_warn("Warning message")
log_error("Error message")

local json = json_encode({key = "value"})
local hex = string_to_hex("Hello")  -- "48656c6c6f"

sleep_ms(100)  -- Sleep for 100ms
local now = time_now()  -- Unix timestamp
```

## Troubleshooting

### Buttons Not Appearing

**Problem**: Script Actions section is empty

**Solutions**:
1. Ensure your script is **attached** to the port (not just loaded)
2. Check that functions use the `action_` prefix
3. Verify the script has no syntax errors
4. Check the browser console for error messages

### Function Not Found

**Problem**: Clicking button shows "Function not found"

**Solutions**:
1. Reload the script after attaching
2. Check for typos in function names
3. Ensure the script is using the correct Lua syntax

### No Response from Device

**Problem**: Action executes but device doesn't respond

**Solutions**:
1. Check port configuration (baud rate, parity, etc.)
2. Verify device is connected and powered
3. Add `log_info()` statements to debug the flow
4. Check if the command format is correct for your device

## See Also

- [Lua Scripting Reference](../reference/lua-scripting.md)
- [Protocol Reference](../reference/protocols.md)
- [Examples](../../examples/)
