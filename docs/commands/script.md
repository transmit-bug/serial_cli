# Script Management Commands

Manage built-in and custom Lua-based protocol scripts.

## Overview

Serial CLI includes several built-in scripts for common serial communication scenarios. Custom scripts can be added via Lua files that implement callback functions (`on_send()`, `on_recv()`).

```bash
serial-cli script <subcommand> [options]
```

## Built-in Scripts

| Name | Description |
|---|---|
| `modbus_rtu` | Modbus RTU protocol (Industrial communication) |
| `modbus_ascii` | Modbus ASCII protocol (Industrial communication) |
| `at_command` | AT Command protocol (Modem control) |
| `line` | Line-based protocol (Text-based communication) |

Built-in scripts are compiled into the binary and cannot be unloaded.

## Subcommands

### `script list [--detailed]`

List all registered scripts.

```bash
# Compact listing (names only)
serial-cli script list

# Detailed listing with descriptions and file paths
serial-cli script list --detailed
```

The `--detailed` flag shows descriptions for built-in scripts and script paths plus load timestamps for custom scripts.

### `script info <name>`

Show detailed information about a specific script.

```bash
serial-cli script info modbus_rtu
serial-cli script info my_custom_script
```

Output includes the script type (built-in or custom), description or script path, version, and load time.

### `script validate <path>`

Validate a Lua script without loading or registering it. Useful for checking scripts before deployment.

```bash
serial-cli script validate /path/to/my_script.lua
```

The validator checks that the script implements the required callback functions and contains no Lua syntax errors.

### `script load <path> [--name <name>]`

Validate and register a custom script from a Lua file.

```bash
# Name inferred from filename (without extension)
serial-cli script load /path/to/my_script.lua

# Explicit name override
serial-cli script load /path/to/my_script.lua --name my_script
```

The script is validated first, then saved to the configuration file so it persists across sessions. Built-in script names are reserved and cannot be reused.

### `script unload <name>`

Remove a custom script from the configuration.

```bash
serial-cli script unload my_custom_script
```

Built-in scripts cannot be unloaded. Attempting to unload a built-in returns an error.

### `script reload <name>`

Re-validate and reload a custom script from its original file path.

```bash
serial-cli script reload my_custom_script
```

Useful after editing a script to pick up changes without restarting the application. The file path is the one recorded at load time.

### `script hot-reload <action>`

Manage automatic hot-reloading of custom scripts.

```bash
serial-cli script hot-reload enable
serial-cli script hot-reload disable
serial-cli script hot-reload status
```

When enabled, the application monitors loaded custom scripts for file changes and automatically reloads them. This is persisted in the configuration file.

## Custom Lua Scripts

Custom scripts are Lua files that implement the following callbacks:

```lua
-- Required callbacks
function on_send(data)
    -- Transform outgoing data (add headers, checksums, etc.)
    -- Return the transformed bytes
end

function on_recv(data)
    -- Transform incoming data (parse frames, extract payload)
    -- Return the transformed bytes, or nil on incomplete frame
end
```

The engine calls `on_recv()` on incoming serial data and `on_send()` on outgoing data. Scripts are validated before loading to ensure all required callbacks are present.

See the Lua scripting reference for the full runtime interface available to custom scripts.

## Full Lifecycle Example

The following sequence demonstrates the complete custom script workflow:

```bash
# 1. Validate the script before loading
serial-cli script validate /home/user/scripts/custom_proto.lua

# 2. Load the script (name inferred from filename)
serial-cli script load /home/user/scripts/custom_proto.lua

# 3. Verify it appears in the script list
serial-cli script list --detailed

# 4. View script details
serial-cli script info custom_proto

# 5. After editing the script, reload to pick up changes
serial-cli script reload custom_proto

# 6. When no longer needed, unload it
serial-cli script unload custom_proto
```

Alternatively, enable hot-reload so edits are picked up automatically:

```bash
serial-cli script hot-reload enable
# Edit custom_proto.lua -- changes are automatically detected and reloaded
serial-cli script hot-reload status   # Verify hot-reload is active
```
