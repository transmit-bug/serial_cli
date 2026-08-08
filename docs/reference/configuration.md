# Configuration Reference

Serial CLI uses TOML-based configuration with a layered loading system: CLI flags take highest precedence, followed by configuration files, with built-in defaults as the final fallback.

## Configuration File Locations

Serial CLI loads configuration from the following locations, checked in order:

1. **Project-level config**: `.serial-cli.toml` in the current working directory
2. **Global config**:
   - **macOS / Linux**: `~/.config/serial-cli/config.toml`
   - **Windows**: `%APPDATA%\serial-cli\config.toml`
3. **Built-in defaults**: Used when no config file is found

The first file that exists and parses successfully is used. Project-level config always takes priority over global config.

> **Note**: The `config save` command writes to `.serial-cli.toml` by default, or to the global config path if no project config exists. Use `config save <path>` to specify an explicit location.

## Configuration Precedence

```
CLI flags  >  Config file  >  Defaults
(highest)                      (lowest)
```

- **CLI flags** such as `--json` or `--verbose` override config file values for that invocation.
- **Config file** values persist across runs and override defaults.
- **Defaults** are used when neither a CLI flag nor a config value is present.

## Complete Configuration Example

```toml
# .serial-cli.toml

[serial]
baudrate   = 115200
databits   = 8
stopbits   = 1
parity     = "none"
timeout_ms = 1000

[logging]
level  = "info"
format = "text"
file   = ""

[lua]
memory_limit_mb = 128
timeout_seconds = 300
enable_sandbox  = true

[task]
max_concurrent          = 10
default_timeout_seconds = 60

[output]
json_pretty    = true
show_timestamp = true

[virtual]
backend                  = "pty"
monitor                  = false
monitor_format           = "hex"
auto_cleanup             = true
max_packets              = 0
bridge_buffer_size       = 8192
bridge_poll_interval_ms  = 10

[protocols]
hot_reload = false
```

## Section Reference

### `[serial]` — Serial Port Defaults

Default parameters applied when no explicit options are provided on the command line.

| Key | Type | Default | Description |
|---|---|---|---|
| `baudrate` | `u32` | `115200` | Baud rate (e.g., `9600`, `115200`). Must not be zero. |
| `databits` | `u8` | `8` | Data bits per frame. Valid range: 5–8. |
| `stopbits` | `u8` | `1` | Stop bits. Valid values: 1 or 2. |
| `parity` | `String` | `"none"` | Parity mode. Valid values: `none`, `odd`, `even`. |
| `timeout_ms` | `u64` | `1000` | Default read timeout in milliseconds. |

```toml
[serial]
baudrate   = 9600
databits   = 8
stopbits   = 1
parity     = "even"
timeout_ms = 2000
```

### `[logging]` — Logging Configuration

Controls the verbosity, format, and destination of log output.

| Key | Type | Default | Description |
|---|---|---|---|
| `level` | `String` | `"info"` | Minimum log level. Valid values: `error`, `warn`, `info`, `debug`, `trace`. |
| `format` | `String` | `"text"` | Log output format. Valid values: `text`, `json`. |
| `file` | `String` | `""` (stdout) | File path for log output. Empty string logs to stdout. |

```toml
[logging]
level  = "debug"
format = "json"
file   = "/tmp/serial-cli.log"
```

### `[lua]` — Lua Runtime Configuration

Settings for the embedded LuaJIT engine used to execute protocol scripts and user scripts.

| Key | Type | Default | Description |
|---|---|---|---|
| `memory_limit_mb` | `usize` | `128` | Maximum memory the Lua runtime may allocate (in megabytes). |
| `timeout_seconds` | `u64` | `300` | Maximum execution time for a Lua script (in seconds). |
| `enable_sandbox` | `bool` | `true` | Restrict Lua to a sandboxed environment. Disabling grants full system access. |

```toml
[lua]
memory_limit_mb = 256
timeout_seconds = 60
enable_sandbox  = false
```

### `[output]` — Output Configuration

Controls the formatting of structured output from commands.

| Key | Type | Default | Description |
|---|---|---|---|
| `json_pretty` | `bool` | `true` | Pretty-print JSON output with indentation. When `false`, outputs compact single-line JSON. |
| `show_timestamp` | `bool` | `true` | Include timestamps in command output. |

```toml
[output]
json_pretty    = false
show_timestamp = false
```

### `[virtual]` — Virtual Serial Port Configuration

Settings for virtual serial port pair creation and monitoring.

| Key | Type | Default | Description |
|---|---|---|---|
| `backend` | `String` | `"pty"` | Virtual port backend implementation. Valid values: `pty`, `socat`, `namedpipe`. |
| `monitor` | `bool` | `false` | Enable traffic monitoring by default when creating virtual port pairs. |
| `monitor_format` | `String` | `"hex"` | Monitoring output format. Valid values: `hex`, `raw`. |
| `auto_cleanup` | `bool` | `true` | Automatically clean up virtual ports on exit. |
| `max_packets` | `usize` | `0` | Maximum packets to capture during monitoring. `0` means unlimited. |
| `bridge_buffer_size` | `usize` | `8192` | Buffer size (in bytes) for the data bridge between virtual port endpoints. |
| `bridge_poll_interval_ms` | `u64` | `10` | Polling interval (in milliseconds) for the bridge data forwarder. |

```toml
[virtual]
backend                  = "socat"
monitor                  = true
monitor_format           = "raw"
auto_cleanup             = true
max_packets              = 10000
bridge_buffer_size       = 16384
bridge_poll_interval_ms  = 5
```

## Managing Configuration

### Using `config show`

Display the current active configuration (from config file or defaults):

```bash
# Human-readable output
serial-cli config show

# JSON output (suitable for piping)
serial-cli config show --json
```

### Using `config set`

Modify a configuration value in memory. Changes are **not persisted** until you run `config save`.

```bash
# Set individual values
serial-cli config set serial.baudrate 9600
serial-cli config set logging.level debug
serial-cli config set lua.enable_sandbox false
serial-cli config set virtual.backend socat
```

All valid configuration keys use the `<section>.<key>` dot notation:

```bash
# Serial settings
serial-cli config set serial.baudrate 9600
serial-cli config set serial.databits 7
serial-cli config set serial.stopbits 2
serial-cli config set serial.parity even
serial-cli config set serial.timeout_ms 2000

# Logging settings
serial-cli config set logging.level debug
serial-cli config set logging.format json
serial-cli config set logging.file /var/log/serial-cli.log

# Lua settings
serial-cli config set lua.memory_limit_mb 256
serial-cli config set lua.timeout_seconds 60
serial-cli config set lua.enable_sandbox false

# Task settings
serial-cli config set task.max_concurrent 5
serial-cli config set task.default_timeout_seconds 30

# Output settings
serial-cli config set output.json_pretty false
serial-cli config set output.show_timestamp false

# Virtual port settings
serial-cli config set virtual.backend socat
serial-cli config set virtual.monitor true
serial-cli config set virtual.monitor_format raw
serial-cli config set virtual.auto_cleanup true
serial-cli config set virtual.max_packets 10000
serial-cli config set virtual.bridge_buffer_size 16384
serial-cli config set virtual.bridge_poll_interval_ms 5
```

### Using `config save`

Persist the current in-memory configuration to a TOML file:

```bash
# Save to default location (.serial-cli.toml or global config path)
serial-cli config save

# Save to a specific path
serial-cli config save /path/to/custom-config.toml
```

### Using `config reset`

Reset the in-memory configuration to built-in defaults:

```bash
serial-cli config reset
serial-cli config save   # Persist the defaults if desired
```

### Editing TOML Directly

You can edit `.serial-cli.toml` or the global `config.toml` with any text editor. Changes take effect on the next command invocation.

```bash
# Edit project-level config
nano .serial-cli.toml

# Edit global config (macOS/Linux)
nano ~/.config/serial-cli/config.toml
```

## `config set` vs. Direct TOML Editing

| Aspect | `config set` | Direct TOML editing |
|---|---|---|
| **Persistence** | In-memory only until `config save` | Immediately persistent |
| **Validation** | Validated at set time, with warnings | Validated on next load |
| **Convenience** | Quick single-value changes | Bulk changes, copy/paste configs |
| **Version control** | Requires `config save` after | Directly tracked by git |
| **Best for** | Scripted automation, one-off tweaks | Initial setup, large refactors |

For scripted workflows, `config set` followed by `config save` provides a clean pipeline. For initial project setup or migrating configurations between machines, editing the TOML file directly is more efficient.
