# Port Commands

## `port list`

Discovers all available serial ports on the system.

```bash
serial-cli port list
```

### Description

Enumerates serial ports using the system's native port discovery. On Linux and macOS this includes devices such as `/dev/ttyUSB0`, `/dev/ttyACM0`, and `/dev/cu.*`. On Windows it lists `COM` ports.

### Output

By default, output is human-readable. Use `--json` for machine-readable output.

```bash
$ serial-cli port list
Available serial ports:
  - /dev/ttyUSB0 (UsbPort)
  - /dev/ttyS0 (Unknown)
```

```bash
$ serial-cli --json port list
[
  {
    "port_name": "/dev/ttyUSB0",
    "port_type": "UsbPort"
  },
  {
    "port_name": "/dev/ttyS0",
    "port_type": "Unknown"
  }
]
```

| Field | Type | Description |
| --- | --- | --- |
| `port_name` | string | System port identifier (e.g., `/dev/ttyUSB0`, `COM1`) |
| `port_type` | string | Internal port type classification |

### Options

This command has no subcommand-specific flags. Use global `--json` for JSON output.

### Handler

Source: `src/cli/commands/ports.rs` -- `list_ports()`

---

## `port send`

Sends raw data to a serial port and reads any response.

```bash
serial-cli port send --port /dev/ttyUSB0 "AT"
serial-cli port send -p /dev/ttyUSB0 "AT"
```

### Description

The simplest way to communicate with a device over a serial port. Opens the specified port with default configuration, writes the data, waits briefly for a response, reads up to 1024 bytes, then closes the port.

### Port Configuration

Uses default settings (`SerialConfig::default()`):

| Setting | Value |
| --- | --- |
| Baud rate | 115200 |
| Data bits | 8 |
| Parity | None |
| Stop bits | 1 |

### Execution Flow

1. Open the port with default configuration (115200, 8N1)
2. Write the data string as raw bytes
3. Wait 100 ms for the device to respond
4. Read up to 1024 bytes from the receive buffer
5. Close the port

### Arguments

| Argument | Short | Required | Description |
| --- | --- | --- | --- |
| `--port` | `-p` | Yes | Port name (e.g., `/dev/ttyUSB0`, `COM1`) |
| `<data>` | -- | Yes | Plain text data to send |

### Example

```bash
$ serial-cli send "AT" -p /dev/ttyUSB0
Received response (4 bytes): OK
```

If no response is received within the read window:

```bash
$ serial-cli send "ATZ" -p /dev/ttyUSB0
```

(The command completes silently when the device does not reply, or logs a note at the tracing level.)

### Error Conditions

The command fails with an error when:

- The port does not exist or cannot be opened
- Permission is denied (e.g., user not in `dialout` group on Linux)
- The port is already in use by another process
- The underlying write or read operation fails at the OS level

### Handler

Source: `src/cli/commands/ports.rs` -- `send_data()`
