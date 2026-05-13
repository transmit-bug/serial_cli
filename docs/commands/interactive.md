# Interactive Command

> `serial-cli interactive` — start an interactive REPL shell for serial communication.

## Overview

The interactive command provides a readline-based REPL (Read-Eval-Print Loop) for manual serial port exploration and debugging. When no subcommand is specified, `serial-cli` defaults to interactive mode.

```bash
serial-cli interactive   # explicit
serial-cli               # implicit default
```

## REPL Commands

Once in the interactive shell, the following commands are available:

| Command | Description |
|---------|-------------|
| `list` | List available serial ports |
| `open <port>` | Open a serial port (e.g., `open /dev/ttyUSB0`) |
| `close` | Close the currently open port |
| `send <data>` | Send data to the open port |
| `recv <bytes>` | Receive up to N bytes from the open port |
| `status` | Show current port status and configuration |
| `help` | Display available commands |
| `quit` | Exit the interactive shell |

## Example Session

```bash
$ serial-cli
Serial CLI Interactive Shell
Type 'help' for available commands, 'quit' to exit

serial> list
Available serial ports:
  - /dev/ttyUSB0 (UsbPort)
  - /dev/ttyACM0 (AcmPort)

serial> open /dev/ttyUSB0
Port opened successfully
Port ID: /dev/ttyUSB0-abc123

serial> send AT
Sent 2 bytes

serial> recv 64
Received (4 bytes): OK

serial> quit
```

## Configuration

The interactive shell uses the same configuration loading system as all other commands (project config → global config → defaults). See [Configuration Reference](../reference/configuration.md) for details.

## Global Flags

```bash
serial-cli --verbose interactive   # Enable debug logging
serial-cli --json interactive      # Enable JSON output (less useful in interactive mode)
```

## See Also

- [Getting Started Guide](../guides/getting-started.md)
- [Configuration Reference](../reference/configuration.md)
- [Lua Scripting](../reference/lua-scripting.md) — for automating tasks instead of manual REPL
