# Protocol Reference

**Updated**: 2026-06-18
**Version**: 0.6.0

---

## What Is a Script?

In Serial CLI v0.6.0+, a **Script** is a Lua program that defines lifecycle callbacks to control port behavior. The unified script system replaces the old Protocol + Hook Script separation.

### Lifecycle Callbacks

```lua
function on_open(port)     -- port opened
function on_send(data)     -- before sending (encode/frame data)
function on_recv(data)     -- on receiving (decode/parse data)
function on_timer()        -- periodic timer
function on_close()        -- port closing
```

Frame encoding/decoding (the former Protocol role) is implemented through `on_send`/`on_recv`:

```lua
-- Example: Modbus RTU as a Script
function on_send(data)
    local frame = add_slave_id(data)
    frame = add_crc(frame)
    return frame
end

function on_recv(data)
    local frame = buffer_accumulate(data)
    if not check_crc(frame) then return nil end
    return strip_headers(frame)
end
```

---

## Built-In Scripts

Serial CLI ships with four built-in scripts, bundled at compile time via `include_str!()`:

| Name           | Description |
|----------------|-------------|
| `modbus_rtu`   | Modbus RTU protocol (Binary industrial communication) |
| `modbus_ascii` | Modbus ASCII protocol (Text-based industrial communication) |
| `at_command`   | AT Command protocol (Modem control commands) |
| `line`         | Line-based protocol (Simple text line communication) |

Built-in script names are reserved; custom scripts cannot use these names.

---

## Protocol Formats

### modbus_rtu

**Modbus RTU** is the binary variant of the Modbus protocol, widely used in industrial automation.

#### Frame Format

```
[Slave ID][Function Code][Data...][CRC Low][CRC High]
   1 byte      1 byte       N bytes     1 byte      1 byte
```

#### Encoding (`on_send`)

- Appends a **CRC-16** checksum (Modbus polynomial `0xA001`, initial value `0xFFFF`) to the payload.
- CRC is stored in **little-endian** order (low byte first).

#### Parsing (`on_recv`)

- Validates minimum frame length (4 bytes: slave + function + 2-byte CRC).
- Computes CRC over all bytes except the trailing two, compares against received CRC.
- On success, returns the payload with the CRC stripped.
- On mismatch, returns an error.

#### Use Cases

- Reading/writing registers on Modbus RTU devices (PLCs, sensors, motor drives).
- Environments with reliable serial links where binary efficiency matters.

---

### modbus_ascii

**Modbus ASCII** is the text-encoded variant of Modbus, using ASCII hex representation for human-readable frames.

#### Frame Format

```
: [Slave ID][Function Code][Data...][LRC] \r \n
  2 chars      2 chars       N*2 chars   2 chars    2 chars
```

- Starts with a colon (`:`) delimiter.
- Every byte is represented as two uppercase hex characters.
- Ends with a **Longitudinal Redundancy Check (LRC)** followed by `\r\n`.

#### Encoding (`on_send`)

1. Prefix with `:`.
2. Convert each payload byte to two hex characters (`0`-`F`).
3. Compute LRC: `LRC = (~sum(data) + 1) & 0xFF` (two's complement of the byte sum).
4. Append LRC as two hex characters.
5. Suffix with `\r\n`.

#### Parsing (`on_recv`)

1. Validates the leading `:` delimiter.
2. Locates `\r` end delimiter.
3. Decodes hex pairs into bytes (case-insensitive).
4. Verifies the trailing LRC byte.
5. Returns the parsed payload on success.

#### Use Cases

- Debugging Modbus traffic (frames are human-readable in a terminal/sniffer).
- Devices that only support ASCII mode.
- Scenarios where line-based tools (like `cat` or `minicom`) are used for diagnostics.

---

### at_command

**AT Command** protocol implements the Hayes command set used for modem and cellular module control.

#### Encoding (`on_send`)

- Appends a **termination string** (default `\r\n`) to outgoing commands.
- Termination is always appended, even if the input already contains it.

#### Parsing (`on_recv`)

- Returns data as-is for normal responses.
- Detects `ERROR` substrings in responses and returns an error.
- Responses containing `OK` pass through normally.

#### Configuration

| Parameter      | Default | Description |
|----------------|---------|-------------|
| `timeout_ms`   | 1000    | Command timeout in milliseconds |
| `termination`  | `\r\n`  | Command termination sequence |

#### Use Cases

- Configuring cellular modems, GSM/LTE modules (SIM7600, ESP8266, etc.).
- Sending Hayes AT commands to dial-up modems.
- Querying signal strength (`AT+CSQ`), network registration (`AT+CREG?`), etc.

---

### line

**Line** protocol is a simple text-based protocol that treats each line as a message frame.

#### Encoding (`on_send`)

- Appends a **separator** (default `\n`) to outgoing data.
- If the data already ends with the separator, it is **not** duplicated.

#### Parsing (`on_recv`)

- Passes data through unchanged. Frame boundaries are determined by the serial line discipline or external tools (sniffer, timeout).

#### Configuration

| Parameter   | Default | Description |
|-------------|---------|-------------|
| `separator` | `\n`    | Line separator byte(s) |

#### Use Cases

- Interacting with CLI-based serial devices (routers, embedded shells).
- Simple text protocols where each message is a newline-terminated string.
- Prototyping and testing serial communication.

---

## Script System Architecture

The script subsystem is organized as follows:

```
src/script/
  mod.rs          -- ScriptInfo, module root
  manager.rs      -- ScriptManager (load/unload/reload/list)
  built_in/
    mod.rs        -- Built-in registration
    modbus_rtu.lua
    at_command.lua
    line.lua
```

### ScriptManager

`ScriptManager` handles the full lifecycle of scripts:

| Method                        | Description |
|-------------------------------|-------------|
| `load(path)`                  | Loads a `.lua` script, validates it, and registers it. |
| `unload(name)`                | Removes a script from the registry. |
| `reload(name)`                | Unloads then re-loads from the original file path. |
| `list()`                      | Lists all scripts (built-in + custom). |
| `get(name)`                   | Returns a `SerialScriptEngine` for the named script. |

### Validation

Before a Lua script is loaded, the validator performs these checks:

1. **File exists** and is readable.
2. **Lua syntax** is valid (script executes without syntax errors).
3. **Required callbacks** are present (at least one of `on_send`, `on_recv`, `on_open`, `on_close`, `on_timer`).

### Hot-Reload

Scripts can be hot-reloaded at runtime. When a script file is modified, use:

```bash
serial-cli script reload --name my_script
```

Or enable automatic file watching (if configured).

---

## CLI Commands

```bash
# List all scripts (built-in + custom)
serial-cli script list

# Show script information
serial-cli script info <name>

# Validate a script without loading
serial-cli script validate --path my_script.lua

# Load a custom script
serial-cli script load --path my_script.lua

# Reload a script (manual)
serial-cli script reload --name my_script

# Unload a custom script
serial-cli script unload --name my_script
```

---

## Writing Custom Scripts

Custom scripts are Lua files that define lifecycle callbacks. They are loaded at runtime via the `ScriptManager`.

### Required Callbacks

Every Lua script must define at least one callback:

```lua
-- Encode outgoing data
function on_send(data)
    -- data: Lua table of bytes (1-indexed)
    -- return: Lua table of bytes (the framed output)
    return result
end

-- Parse incoming raw bytes
function on_recv(data)
    -- data: Lua table of bytes (1-indexed)
    -- return: Lua table of bytes (the parsed payload)
    return result
end
```

### Optional Callbacks

```lua
function on_open(port)
    -- Called when port is opened
end

function on_close()
    -- Called when port is closing
end

function on_timer()
    -- Called periodically (if timer is configured)
end
```

### Return Value Types

Lua callbacks may return:

| Lua Type  | Behavior |
|-----------|----------|
| `table`   | Interpreted as a 1-indexed array of byte values. |
| `string`  | Converted to raw bytes. |
| `number`  | Wrapped as a single-byte result. |
| `nil`     | Indicates incomplete data (for `on_recv` buffering). |

### Error Handling

If a Lua callback throws an error, the script falls back to **passthrough** mode (returns the original input data unchanged). This ensures the communication pipeline remains resilient to script errors.

### Example: Custom Binary Protocol

```lua
-- Protocol: custom_binary
-- Frame: [Length: 1 byte][Payload: N bytes][Checksum: 1 byte (XOR)]

function on_send(data)
    local len = #data
    local checksum = 0
    for _, byte in ipairs(data) do
        checksum = checksum ~ byte
    end

    local result = {}
    table.insert(result, len)        -- length prefix
    for _, byte in ipairs(data) do
        table.insert(result, byte)   -- payload
    end
    table.insert(result, checksum)   -- checksum
    return result
end

function on_recv(data)
    if #data < 3 then
        return data  -- passthrough if too short
    end

    local len = data[1]
    if #data < 2 + len then
        return data  -- incomplete frame
    end

    -- Verify checksum (XOR of payload bytes)
    local checksum = 0
    for i = 2, 1 + len do
        checksum = checksum ~ data[i]
    end

    local expected_checksum = data[2 + len]
    if checksum ~= expected_checksum then
        error("Checksum mismatch")
    end

    -- Return payload only
    local result = {}
    for i = 2, 1 + len do
        table.insert(result, data[i])
    end
    return result
end
```

Save this as `custom_binary.lua` and load it:

```bash
serial-cli script load --path custom_binary.lua
```

---

## Lua API

For the complete Lua API reference (serial I/O, JSON, hex utilities, etc.), see [Lua Scripting Reference](lua-scripting.md).

---

## See Also

- [Architecture](../dev/ARCH.md) — Full system architecture
- [Lua Scripting](lua-scripting.md) — Lua API reference
- [Unified Script System Design](../dev/UNIFIED-SCRIPT-SYSTEM.md) — Design decision document
