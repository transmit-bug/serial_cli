# Interactive Shell

The interactive shell provides a REPL (Read-Eval-Print Loop) for exploring and debugging serial communication without writing scripts or chaining CLI subcommands.

## Starting the Shell

Run `serial-cli` with no subcommand (or explicitly with `interactive`):

```bash
serial-cli              # default mode when no subcommand given
serial-cli interactive  # explicit invocation
```

## REPL Commands

The shell reads from standard input line by line. Blank lines are ignored. Type `help` at any time for a summary.

### Core Commands

| Command | Description |
|---|---|
| `help` | Show available commands and their usage |
| `list` | List available serial ports on the system |
| `quit`, `exit` | Exit the shell |

### Port Management

```
open <port>           # Open a serial port (e.g., /dev/ttyUSB0, COM3)
close [port_id]       # Close current port, or a specific port by ID
status                # Show current port configuration and protocol
```

Opening a port automatically closes any previously opened port. The shell tracks the current port ID internally.

### Data Transfer

```
send <data>           # Send data to the current port
recv [n]              # Read up to N bytes (default: 64)
```

- `send` transmits the argument string as raw bytes. Multi-word arguments are joined with spaces.
- `recv` displays received data as UTF-8 text when possible, falling back to hex. If no data is available, it prints a message and returns immediately (non-blocking).

### Protocol Management

```
protocol              # Show current protocol and available protocols
protocol list         # List all built-in protocols
protocol set <name>   # Attach a protocol to the current port
protocol clear        # Remove protocol from the current port (raw mode)
protocol show         # Show detailed protocol status
```

Built-in protocols:

| Name | Description |
|---|---|
| `modbus_rtu` | Modbus RTU protocol |
| `modbus_ascii` | Modbus ASCII protocol |
| `at_command` | AT Command protocol |
| `line` | Line-based protocol |

### Hardware Signal Control

```
dtr [on|off]          # Get or set DTR (Data Terminal Ready) signal
rts [on|off]          # Get or set RTS (Request to Send) signal
```

With no argument, the current signal state is displayed. Accepts `on`/`off`, `true`/`false`, `1`/`0`, or `enable`/`disable`.

## Typical Workflow

```
open /dev/ttyUSB0       # 1. Open the target port
status                  # 2. Verify configuration (baud rate, data bits, etc.)
protocol set modbus_rtu # 3. (Optional) Attach a protocol
send 01 03 00 00 00 01  # 4. Send data
recv 32                 # 5. Read response
close                   # 6. Close the port
quit                    # 7. Exit
```

## Example Session

```
$ serial-cli
Serial CLI Interactive Shell
Type 'help' for available commands, 'quit' to exit

serial> list
Available serial ports:
  - /dev/ttyUSB0 (USB)
  - /dev/ttyS0 (Unknown)

serial> open /dev/ttyUSB0
Closing current port...
Port opened successfully
Port ID: ttyUSB0_1

serial> status
Current port ID: ttyUSB0_1
Port name: /dev/ttyUSB0
Configuration:
  Baud rate: 9600
  Data bits: 8
  Stop bits: 1
  Parity: None
  Flow control: None
  Protocol: (none - raw mode)

serial> send AT
Sent 2 bytes

serial> recv 64
Reading up to 64 bytes...
Received (4 bytes as text): AT
OK

serial> protocol set at_command
Protocol 'at_command' set for port
Data will now be processed using the at_command protocol

serial> close
Port closed successfully

serial> quit
Goodbye!
```

## Global Flags

The `--json` and `--verbose` flags apply when launching the shell and affect logging and output throughout the session:

```bash
serial-cli --json       # Logging output uses JSON format
serial-cli --verbose    # Enable DEBUG-level tracing
serial-cli --json --verbose  # Both
```

These flags are consumed at startup and cannot be changed from within the REPL.

## Non-Interactive Alternative

For one-off operations, the equivalent non-interactive commands are:

```bash
# List ports
serial-cli list-ports

# Send data
serial-cli send /dev/ttyUSB0 "AT"

# Interactive mode
serial-cli interactive
serial-cli              # (same thing, no subcommand)
```

The interactive shell is intended for exploration, debugging, and ad-hoc testing. For automated or repeated workloads, prefer Lua scripts (`serial-cli run script.lua`).
