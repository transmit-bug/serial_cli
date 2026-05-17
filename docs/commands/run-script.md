# Run Script — Lua Script Execution

Execute a Lua script file with optional command-line arguments. This command is the gateway to the LuaJIT scripting capabilities of Serial CLI, enabling custom protocol parsing, automated device interaction, and extensible serial workflows.

## Usage

```
serial-cli run <script> [args...]
```

| Argument | Description |
|----------|-------------|
| `<script>` | Path to the `.lua` script file *(required)* |
| `[args...]` | Arguments forwarded to the script as a Lua table |

## Examples

```bash
# Run a script without arguments
serial-cli run my_script.lua

# Run a script with arguments
serial-cli run my_script.lua --port /dev/ttyUSB0 --baud 115200

# Run a script with multiple arguments
serial-cli run device_probe.lua COM3 9600 8 N 1
```

## How It Works

When `serial-cli run` is invoked, the following steps occur:

1. A `ScriptEngine` is created and all built-in Rust APIs are registered as Lua bindings.
2. Standard library utilities (`json`, `http`, `fs`, string/hex converters, etc.) are loaded.
3. The script file is read from disk.
4. The script is executed. If arguments are provided, they are passed to the script as a Lua table accessible within the script.

## Arguments in Lua

When arguments are supplied on the command line, they are forwarded to the script as a Lua table. For example:

```bash
serial-cli run query_device.lua --port /dev/ttyUSB0 --baud 9600
```

Inside `query_device.lua`, the arguments are available as a table:

```lua
-- args will be: { "--port", "/dev/ttyUSB0", "--baud", "9600" }
```

Scripts can parse this table to extract configuration values, device addresses, protocol parameters, or any other runtime inputs.

## Lua Scripting Capabilities

The `src/lua/` module provides a full LuaJIT runtime with the following features:

- **Serial port I/O** — read from and write to serial ports directly from Lua scripts.
- **Logging** — `log_info`, `log_debug`, `log_warn`, `log_error` functions for structured output.
- **String utilities** — `string_to_hex`, `string_from_hex` for encoding/decoding.
- **Hex utilities** — `hex_encode`, `hex_decode` for byte-level data manipulation.
- **Time utilities** — timestamp and duration helpers.
- **Data conversion** — utilities for converting between data formats.

For the full API reference, see the `src/lua/bindings.rs` and `src/lua/runtime.rs` source files.

## Error Handling

Script execution errors are reported through the standard Serial CLI error system:

- **I/O errors** — the script file could not be read (missing file, permissions).
- **Lua errors** — the script failed to compile or encountered a runtime error.
- **Script errors** — sandbox violations or resource limit exceeded.
