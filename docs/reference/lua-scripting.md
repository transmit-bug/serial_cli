# Lua Scripting Reference

Serial CLI embeds a LuaJIT runtime, enabling scriptable serial communication, custom protocol definitions, and data processing pipelines. This reference covers the full Lua scripting capabilities.

## Architecture

The Lua integration is organized under `src/lua/` with the following modules:

| Module | Purpose |
|---|---|
| `bindings.rs` | LuaBindings — Autonomous mode for CLI scripts (serial_open/close/send/recv) |
| `runtime.rs` | ScriptRuntime — unified tool function registration (log/json/hex/string-bytes/time) |
| `engine.rs` | LuaEngine — lightweight wrapper (internal utility) |
| `executor.rs` | ScriptEngine — script execution engine with argument passing |

**Note**: `pool.rs`, `cache.rs`, and `stdlib.rs` were removed during ScriptCore unification. All consumers now use the unified `ScriptRuntime` for tool function registration. |

## Script Execution

### Running a Script

```bash
serial-cli run <script.lua> [args...]
```

Scripts are loaded from disk and executed within an isolated Lua state. All registered APIs are available by default.

### Argument Passing

Command-line arguments are made available to Lua scripts in two ways:

1. **`arg` table** — A Lua table indexed from 1:
   ```lua
   -- serial-cli run script.lua hello world
   print(arg[1])  -- "hello"
   print(arg[2])  -- "world"
   print(arg.n)   -- 2 (argument count)
   ```

2. **Individual globals** — `arg1`, `arg2`, etc.:
   ```lua
   print(arg1)  -- "hello"
   print(arg2)  -- "world"
   ```

### Execution Order

When `serial-cli run` is invoked:

1. A `ScriptEngine` is created with a fresh Lua state.
2. All API bindings are registered (`register_all_apis`).
3. Standard library utilities are registered (`ScriptRuntime::register_all`).
4. The script file is read from disk.
5. Arguments are injected into the global scope.
6. The script is executed.

Each script execution uses an isolated Lua state; globals do not persist between invocations.

## API Reference

### Serial Port Operations

#### `serial_open(port_name, config_table)`

Opens a serial port and returns a port ID string.

```lua
local port_id = serial_open("/dev/ttyUSB0", {
    baudrate = 115200,
    data_bits = 8,
    stop_bits = 1,
    parity = "none",       -- "none", "odd", "even"
    timeout = 1000,        -- milliseconds
    flow_control = "none", -- "none", "software", "hardware"
    dtr_enable = true,
    rts_enable = true,
})
```

Configuration table fields (all optional, defaults shown):

| Field | Type | Default | Description |
|---|---|---|---|
| `baudrate` | integer | 115200 | Baud rate |
| `data_bits` | integer | 8 | Data bits (5-8) |
| `stop_bits` | integer | 1 | Stop bits (1-2) |
| `parity` | string | "none" | Parity: "none", "odd", "even" |
| `timeout` | integer | 1000 | Read timeout in milliseconds |
| `flow_control` | string | "none" | "none", "software", "hardware" |
| `dtr_enable` | boolean | true | DTR signal enable |
| `rts_enable` | boolean | true | RTS signal enable |

#### `serial_close(port_id)`

Closes an open serial port.

```lua
serial_close(port_id)
```

#### `serial_send(port_id, data)`

Sends data to the serial port. Returns the number of bytes written.

```lua
local bytes_written = serial_send(port_id, "ATZ\r\n")
```

Data is passed as a string (raw bytes).

#### `serial_recv(port_id, timeout_ms)`

Receives data from the serial port with a timeout. Returns the received data as a string.

```lua
local data = serial_recv(port_id, 1000)  -- 1 second timeout
```

#### `serial_list()`

Returns a table of all available serial ports.

```lua
local ports = serial_list()
for i, port in ipairs(ports) do
    print(port.port_name, port.port_type)
end
```

Each entry contains:

| Field | Type | Description |
|---|---|---|
| `port_name` | string | Device path (e.g., "/dev/ttyUSB0") |
| `port_type` | string | Port type identifier |

### Protocol Operations

#### `protocol_encode(protocol_name, data)`

Encodes data according to the specified protocol. Returns the encoded string.

```lua
-- Line protocol: appends newline if not present
local encoded = protocol_encode("lines", "Hello")  -- "Hello\n"

-- AT Command protocol: appends CRLF if not present
local at_cmd = protocol_encode("at_command", "ATZ")  -- "ATZ\r\n"

-- Modbus RTU: appends CRC
local modbus = protocol_encode("modbus_rtu", "\x01\x03\x00\x00\x00\x01")
```

Built-in protocols: `lines`, `at_command`, `modbus_rtu`, `modbus_ascii`.

#### `protocol_decode(protocol_name, data)`

Decodes data according to the specified protocol. Returns the decoded string.

```lua
local decoded = protocol_decode("lines", "Hello\n")
```

#### `script_list()`

Returns a table of all available protocols.

```lua
local protocols = script_list()
for i, proto in ipairs(protocols) do
    print(proto.name, proto.description, proto.type)
end
```

#### `protocol_info(protocol_name)`

Returns detailed information about a specific protocol.

```lua
local info = protocol_info("modbus_rtu")
print(info.name)        -- "modbus_rtu"
print(info.description) -- "Modbus RTU protocol"
print(info.type)        -- "built-in"
```

#### `script_load(path)`

Loads a custom protocol script from a file path. Returns `(success, message)`.

```lua
local ok, msg = script_load("/path/to/my_protocol.lua")
if ok then
    print(msg)  -- "Protocol loaded: my_protocol (from /path/to/my_protocol.lua)"
end
```

The protocol name is derived from the filename stem.

#### `script_unload(name)`

Unloads a custom protocol by name. Returns `(success, message)`.

```lua
local ok, msg = script_unload("my_protocol")
```

#### `protocol_reload(name)`

Reloads a custom protocol from its registered path. Returns `(success, message)`.

```lua
local ok, msg = protocol_reload("my_protocol")
```

#### `protocol_validate(path)`

Validates a protocol script file. Returns `(success, message)`.

```lua
local ok, msg = protocol_validate("/path/to/script.lua")
```

### Virtual Serial Port Operations

#### `virtual_create(backend, monitor)`

Creates a virtual serial port pair. Returns a table with the pair information.

```lua
local pair = virtual_create("pty", false)
print(pair.id)       -- unique identifier
print(pair.port_a)   -- e.g., "/dev/pts/3"
print(pair.port_b)   -- e.g., "/dev/pts/4"
print(pair.backend)  -- "Pty"
print(pair.running)  -- true/false
```

Available backends: `"pty"` (Linux/macOS), `"namedpipe"` (Windows), `"socat"`. Omit or pass `nil` for auto-detection.

#### `virtual_stop(id)`

Stops a virtual serial port pair.

```lua
virtual_stop(pair_id)
```

### Logging Functions

#### `log_info(message)`, `log_debug(message)`, `log_warn(message)`, `log_error(message)`

Log messages through the application's tracing system.

```lua
log_info("Starting communication")
log_debug("Received 42 bytes")
log_warn("Timeout on port /dev/ttyUSB0")
log_error("Connection failed")
```

### Utility Functions

#### JSON Utilities

```lua
local json_str = json_encode(value)  -- Serializes a value to a debug string
local decoded = json_decode(json_str)  -- Not fully implemented; returns nil
```

#### `sleep_ms(milliseconds)`

Sleeps for the specified number of milliseconds.

```lua
sleep_ms(500)  -- Sleep for 500ms
```

### Hex Utilities (from ScriptRuntime)

#### `hex_encode(bytes)`

Encodes a byte array (Lua table) to a lowercase hex string.

```lua
local hex = hex_encode({0x41, 0x42, 0x43})  -- "414243"
```

#### `hex_decode(hex_string)`

Decodes a hex string to a byte array (Lua table).

```lua
local bytes = hex_decode("414243")  -- {0x41, 0x42, 0x43}
```

#### `hex_to_bytes(hex_string)`

Converts a hex string to a Lua byte array table (alias for `hex_decode`).

```lua
local bytes = hex_to_bytes("010203")
assert(bytes[1] == 1)
assert(bytes[2] == 2)
assert(bytes[3] == 3)
```

### String Utilities (from ScriptRuntime)

#### `string_to_hex(str)`

Converts a string to its hex representation.

```lua
local hex = string_to_hex("AB")  -- "4142"
```

#### `string_from_hex(hex_string)`

Converts a hex string to a UTF-8 string.

```lua
local str = string_from_hex("414243")  -- "ABC"
```

### Data Conversion (from ScriptRuntime)

#### `bytes_to_hex(value)`

Converts a byte array (table) or string to a hex string.

```lua
-- From string
local hex = bytes_to_hex("AB")  -- "4142"

-- From table
local hex = bytes_to_hex({0x41, 0x42, 0x43})  -- "414243"
```

#### `bytes_to_string(bytes_table)`

Converts a byte array table to a UTF-8 string. Errors on invalid UTF-8.

```lua
local bytes = {0x48, 0x65, 0x6c, 0x6c, 0x6f}  -- "Hello"
local str = bytes_to_string(bytes)  -- "Hello"
```

#### `string_to_bytes(str)`

Converts a string to a byte array table.

```lua
local bytes = string_to_bytes("ABC")
-- bytes = {0x41, 0x42, 0x43}
```

### Time Utilities (from ScriptRuntime)

#### `time_now()`

Returns the current Unix timestamp in seconds.

```lua
local timestamp = time_now()
```

## Configuration

Lua behavior is configurable via the application's TOML configuration file (typically `~/.config/serial-cli/config.toml`).

### Sandbox Mode

```toml
[lua]
enable_sandbox = true   # Default: true
```

When enabled, the Lua sandbox restricts access to potentially dangerous operations. The sandbox mode is enforced at the configuration level and controls which APIs and system resources are accessible to scripts.

### Memory Limit

```toml
[lua]
memory_limit_mb = 128   # Default: 128 MB
```

Sets the maximum memory allocation for the Lua runtime. If a script exceedses this limit, execution is terminated.

### Timeout

```toml
[lua]
timeout_seconds = 300   # Default: 300 seconds (5 minutes)
```

Sets the maximum execution time for a Lua script. Scripts that exceed this timeout are terminated.

### Example Configuration

```toml
[lua]
memory_limit_mb = 256
timeout_seconds = 60
enable_sandbox = false
```

## Custom Protocol Scripts

Custom protocols are defined by implementing three callback functions in a Lua script. The protocol engine invokes these callbacks when processing serial data.

### Required Callbacks

#### `on_frame(data)`

Parses incoming raw bytes. Receives `data` as a byte array table (1-indexed). Should return the parsed payload as a byte array table, a string, or the original data unchanged.

```lua
function on_frame(data)
    -- Process incoming bytes
    -- Return parsed result as a table of bytes
    local result = {}
    for i = 1, #data do
        table.insert(result, data[i])
    end
    return result
end
```

#### `on_encode(data)`

Encodes outgoing data. Receives `data` as a byte array table. Should return the encoded frame as a byte array table or string.

```lua
function on_encode(data)
    -- Add protocol framing (e.g., header, checksum)
    local result = {}
    table.insert(result, 0x01)  -- Start byte
    for i = 1, #data do
        table.insert(result, data[i])
    end
    table.insert(result, 0x04)  -- End byte
    return result
end
```

#### `on_reset()` (optional)

Called when the protocol parser should be reset to its initial state.

```lua
function on_reset()
    -- Clear any internal state
end
```

### Error Handling

If a callback errors or returns an unexpected type, the protocol engine falls back to passing through the original data unchanged. Use `pcall` within callbacks for controlled error handling.

### Loading Custom Protocols

1. Place the script in a known location (e.g., `~/.config/serial-cli/protocols/`).
2. Load it via the CLI:
   ```bash
   serial-cli protocol load /path/to/my_protocol.lua
   ```
3. Or load it programmatically from another script:
   ```lua
   local ok, msg = script_load("/path/to/my_protocol.lua")
   ```

The protocol name is automatically derived from the filename stem (e.g., `my_protocol.lua` becomes `my_protocol`).

## Example Scripts

### Basic Serial Communication

Read from a serial port and print received data:

```lua
-- open_port.lua
local port_id = serial_open("/dev/ttyUSB0", {
    baudrate = 9600,
    data_bits = 8,
    stop_bits = 1,
    parity = "none",
    timeout = 2000,
})

log_info("Opened port: " .. port_id)

local data = serial_recv(port_id, 2000)
if data and #data > 0 then
    log_info("Received: " .. data)
    -- Print as hex for debugging
    log_info("Hex: " .. bytes_to_hex(data))
end

serial_close(port_id)
log_info("Closed port")
```

Run with:

```bash
serial-cli run scripts/open_port.lua
```

### AT Command Modem Communication

Send an AT command and read the response:

```lua
-- at_command.lua
local command = arg1 or "ATI"

local port_id = serial_open("/dev/ttyUSB0", {
    baudrate = 115200,
    timeout = 3000,
})

-- Encode and send the AT command
local encoded = protocol_encode("at_command", command)
serial_send(port_id, encoded)
log_info("Sent: " .. command)

-- Wait and receive response
sleep_ms(200)
local response = serial_recv(port_id, 3000)
if response then
    log_info("Response: " .. response)
end

serial_close(port_id)
```

Run with:

```bash
serial-cli run scripts/at_command.lua ATZ
```

### Custom Protocol Definition

Define a simple length-prefixed protocol:

```lua
-- length_prefix.lua
-- Protocol: [length: 1 byte][payload: N bytes]

function on_frame(data)
    -- Extract payload (skip first byte which is length)
    if #data < 1 then
        return data
    end

    local length = data[1]
    local result = {}

    -- Collect payload bytes
    for i = 2, math.min(length + 1, #data) do
        table.insert(result, data[i])
    end

    return result
end

function on_encode(data)
    -- Prepend length byte
    local result = {}
    table.insert(result, #data)  -- Length prefix
    for i = 1, #data do
        table.insert(result, data[i])
    end
    return result
end

function on_reset()
    -- No state to reset
end
```

Load with:

```bash
serial-cli protocol load protocols/length_prefix.lua
```

### Data Processing Pipeline

Convert hex input to ASCII and send over serial:

```lua
-- hex_to_serial.lua
local hex_input = arg1

if not hex_input then
    log_error("Usage: serial-cli run hex_to_serial.lua <hex>")
    return
end

-- Decode hex to bytes
local bytes = hex_to_bytes(hex_input)
local ascii = bytes_to_string(bytes)
log_info("Decoded: " .. ascii)

-- Open port and send
local port_id = serial_open("/dev/ttyUSB0", {
    baudrate = 115200,
    timeout = 1000,
})

serial_send(port_id, ascii)
log_info("Sent " .. #bytes .. " bytes")

-- Receive response
sleep_ms(100)
local response = serial_recv(port_id, 1000)
if response then
    log_info("Response: " .. response)
    log_info("Response hex: " .. string_to_hex(response))
end

serial_close(port_id)
```

Run with:

```bash
serial-cli run scripts/hex_to_serial.lua "48656c6c6f"
```

### Protocol Listing Script

List all available protocols and their details:

```lua
-- list_protocols.lua
local protocols = script_list()

log_info("Available protocols:")
for i, proto in ipairs(protocols) do
    log_info(string.format("  [%d] %s - %s (%s)", i, proto.name, proto.description, proto.type))
end
```
