# AI Usage Guide

This guide explains how AI agents (Claude, GPT, etc.) can effectively use Serial CLI for automation tasks.

## Quick Start for AI

### Basic Pattern

```lua
-- AI Script Template
local ok, result = pcall(serial_open, "/dev/ttyUSB0", {baudrate=115200})
if not ok then
  print(json_encode({status="error", error=result}))
  os.exit(1)
end

serial_send(result, "AT\r\n")
local response = serial_recv(result, 1000)
print(json_encode({status="ok", data=response}))
serial_close(result)
```

### CLI Usage for AI

```bash
# Execute AI-generated scripts
serial-cli run script.lua

# Get machine-readable output
serial-cli port list --json
serial-cli protocol list --json

# Batch processing
serial-cli batch run commands.txt --concurrent
```

## JSON Output Format

All commands support `--json` flag for AI parsing:

```bash
# Port enumeration
serial-cli port list --json
# Output: [{"port_name": "/dev/ttyUSB0", "port_type": "Usb"}]

# Protocol information
serial-cli protocol list --json
# Output: {"protocols": [...], "count": 4}
```

## Lua API for AI

Serial CLI provides a full Lua API for serial I/O, protocol handling, JSON, hex utilities, and more.

> For the complete API reference, see [Lua Scripting Reference](../reference/lua-scripting.md).

### Quick Reference

```lua
-- Serial I/O
local port_id = serial_open("/dev/ttyUSB0", {baudrate=115200})
serial_send(port_id, "AT\r\n")
local data = serial_recv(port_id, 1000)
serial_close(port_id)

-- JSON
local output = json_encode({status="ok", data=response})
local config = json_decode('{"baudrate": 115200}')

-- Protocols
local encoded = protocol_encode("at_command", "ATZ")
local decoded = protocol_decode("at_command", response)

-- Hex utilities
local hex_str = string_to_hex("Hello")
local original = string_from_hex(hex_str)

-- Time
local now = time_now()
sleep_ms(100)
```

## Common Automation Patterns

### 1. Device Discovery

```lua
-- List all available ports
local ports = serial_list()
for i, port in ipairs(ports) do
  print(json_encode({
    port_name = port.port_name,
    port_type = port.port_type
  }))
end
```

### 2. Retry Logic

```lua
local max_retries = 3
local retry_delay = 1000 -- ms

for attempt = 1, max_retries do
  local ok, port_id = pcall(serial_open, "/dev/ttyUSB0", {baudrate=115200})
  if ok then
    -- Success, continue operations
    serial_send(port_id, "AT\r\n")
    local response = serial_recv(port_id, 1000)
    print(json_encode({status="ok", data=response}))
    serial_close(port_id)
    os.exit(0)
  end

  if attempt < max_retries then
    sleep_ms(retry_delay)
  end
end

-- All retries failed
print(json_encode({status="error", error="Max retries exceeded"}))
os.exit(1)
```

### 3. Batch Commands

```lua
-- Execute multiple commands sequentially
local commands = {"AT\r\n", "ATI\r\n", "AT+VER\r\n"}
local results = {}

for i, cmd in ipairs(commands) do
  serial_send(port_id, cmd)
  local response = serial_recv(port_id, 1000)
  results[i] = {
    command = cmd,
    response = response
  }
end

print(json_encode({status="ok", results=results}))
```

## Protocol Handling

Built-in protocols (`modbus_rtu`, `modbus_ascii`, `at_command`, `line`) and custom Lua protocols are supported. See [Protocol Reference](../reference/protocols.md) for encoding/parsing details.

```lua
local encoded = protocol_encode("at_command", "ATZ")
local decoded = protocol_decode("at_command", response)
local protocols = protocol_list()

local ok, result = pcall(protocol_load, "/path/to/custom.lua")
if ok then
  print(json_encode({status="ok", message=result}))
else
  print(json_encode({status="error", error=result}))
end
```

## Best Practices for AI

### 1. Always Use JSON for Output

```lua
-- Good: Structured output
print(json_encode({status="ok", data=response}))

-- Bad: Unstructured text
print("Response: " .. response)
```

### 2. Proper Error Handling

```lua
-- Good: Explicit error checking
local ok, result = pcall(serial_open, "/dev/ttyUSB0", {})
if not ok then
  print(json_encode({status="error", error=result}))
  os.exit(1)
end

-- Bad: No error handling
local port_id = serial_open("/dev/ttyUSB0", {})
```

### 3. Resource Cleanup

```lua
-- Good: Always close ports
local ok, port_id = pcall(serial_open, "/dev/ttyUSB0", {})
if ok then
  -- ... operations ...
  serial_close(port_id)
end

-- Good: Use cleanup pattern
local port_id = nil
local ok, result = pcall(serial_open, "/dev/ttyUSB0", {})
if ok then
  port_id = result
end

-- ... operations ...

if port_id then
  pcall(serial_close, port_id)  -- Best effort close
end
```

### 4. Timeout Management

```lua
-- Good: Explicit timeouts
local data = serial_recv(port_id, 5000)  -- 5 second timeout

-- Good: Configurable timeouts
local timeout = arg[1] and tonumber(arg[1]) or 1000
local response = serial_recv(port_id, timeout)
```

## Integration Examples

### Python Integration

```python
import subprocess
import json

# Execute Lua script
result = subprocess.run(
  ['serial-cli', 'run', 'script.lua'],
  capture_output=True,
  text=True
)

# Parse JSON output
try:
  output = json.loads(result.stdout)
  if output['status'] == 'ok':
    print(f"Data: {output['data']}")
  else:
    print(f"Error: {output['error']}")
except json.JSONDecodeError:
  print(f"Raw output: {result.stdout}")
```

### Shell Integration

```bash
# Get port list as JSON
PORTS=$(serial-cli port list --json)

# Parse with jq
echo "$PORTS" | jq '.[] | .port_name'

# Execute script and capture output
OUTPUT=$(serial-cli run script.lua)
STATUS=$(echo "$OUTPUT" | jq -r '.status')

if [ "$STATUS" = "ok" ]; then
  echo "Success: $(echo "$OUTPUT" | jq -r '.data')"
else
  echo "Error: $(echo "$OUTPUT" | jq -r '.error')"
fi
```

## Advanced Features

### Virtual Serial Ports

```lua
-- Create virtual port pair
local pair = virtual_create("pty", true)
print(json_encode({
  status = "ok",
  port_a = pair.port_a,
  port_b = pair.port_b
}))

-- Use for testing
local port_id = serial_open(pair.port_a, {baudrate=115200})
serial_send(port_id, "test data")
```

### Benchmarking

```lua
-- Not directly available in Lua, use CLI
-- serial-cli benchmark run serial --iterations=100
```

## Troubleshooting

### Common Issues

1. **Port not found**
   ```lua
   -- Solution: List ports first
   local ports = serial_list()
   -- Check if your port exists before opening
   ```

2. **Permission denied**
   ```bash
   # Solution: Run with appropriate permissions
   sudo serial-cli run script.lua
   # Or add user to dialout/uucp group
   ```

3. **Timeout issues**
   ```lua
   -- Solution: Increase timeout
   local data = serial_recv(port_id, 5000)  -- 5 seconds
   ```

## Debugging Tips

```lua
-- Enable logging
log_info("Opening port...")
log_debug("Port opened: " .. port_id)

-- Print intermediate values
local encoded = protocol_encode("at_command", "AT")
log_debug("Encoded: " .. string_to_hex(encoded))

-- Validate JSON output
local ok, result = pcall(json_decode, input_string)
if not ok then
  log_error("Invalid JSON: " .. result)
end
```

## Further Reading

- [CLI Reference](../../README.md)
- [Lua Scripting Reference](../reference/lua-scripting.md)
- [Protocol Reference](../reference/protocols.md)
- [Examples](../../examples/)

## Version Compatibility

This guide applies to Serial CLI v0.6.0. Features may change in future versions.
