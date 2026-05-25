# Config Command

> `serial-cli config` -- manage TOML-based configuration for serial ports, logging, Lua, tasks, output, and virtual ports.

## Overview

Serial CLI uses a TOML-based configuration system with three-tier loading:

1. **Project-level**: `.serial-cli.toml` in the current directory
2. **Global-level**: platform-specific config directory
   - **macOS / Linux**: `$XDG_CONFIG_HOME/serial-cli/config.toml` (typically `~/.config/serial-cli/config.toml`)
   - **Windows**: `%APPDATA%\serial-cli\config.toml`
3. **Built-in defaults**: compiled-in fallback values

Changes made with `config set` only affect the in-memory configuration. Use `config save` to persist them to disk.

## Subcommands

### `config show [--json]`

Display the current (in-memory) configuration.

```bash
serial-cli config show
serial-cli config show --json
```

### `config set <key> <value>`

Modify a configuration value in memory. Changes are ephemeral — use `config save` to persist.

```bash
serial-cli config set serial.baudrate 9600
serial-cli config set logging.level debug
serial-cli config set output.json_pretty false
```

For the full list of valid keys, types, and defaults, see the [Configuration Reference](../reference/configuration.md).

### `config save [--path <path>]`

Persist the current in-memory configuration to a TOML file.

```bash
serial-cli config save
serial-cli config save --path /etc/serial-cli/config.toml
```

### `config reset`

Restore all configuration values to built-in defaults.

```bash
serial-cli config reset
```

## Validation

The configuration manager validates settings when `config set` is called and warns if the result is invalid:

- **baudrate** must not be zero
- **databits** must be 5–8
- **stopbits** must be 1 or 2
- **parity** must be `none`, `odd`, or `even`
- **logging.level** must be one of `error`, `warn`, `info`, `debug`, `trace`
- **max_concurrent** must not be zero

Invalid configurations cannot be saved to a file.
