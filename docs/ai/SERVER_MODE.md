# Server Mode - AI/Automation Workflow

**Version**: 0.6.0
**Status**: Production-Ready
**Updated**: 2026-06-06

---

## What is Server Mode?

Server Mode is a persistent daemon that provides a JSON-RPC 2.0 interface for serial port and protocol management. It's designed specifically for **AI agents and automation workflows**.

### Key Benefits

- **Persistent connections** - No repeated open/close overhead (50-200ms → 1-5ms)
- **Protocol persistence** - Custom scripts loaded once, available globally
- **AI-friendly API** - Standard JSON-RPC 2.0 interface
- **Multi-client support** - Multiple AI agents can share connections
- **LAN remote access** - TCP transport lets clients on the same network operate a device's serial ports remotely (`server call --remote ip:port`)

---

## Remote Access (TCP)

`server start` listens on TCP **0.0.0.0:23333** by default (in addition to the local Unix socket), so a target device on the same LAN can be reached from any workstation:

```bash
# On the target device (Linux board, industrial PC, ...):
serial-cli server start --port 23333

# From your workstation (Windows/macOS/Linux):
serial-cli server call --remote 192.168.1.50:23333 port_list '{}'
```

Options: `--bind <ip>` narrows the listening interface, `--no-tcp` disables TCP entirely. The JSON-RPC contract is byte-identical over Unix socket and TCP, so any existing script works remotely by adding `--remote`.

> ⚠️ **No authentication in v1.** TCP remote access is designed for trusted corporate LANs. Authentication (token handshake) is tracked as a follow-up issue.

### GUI remote devices

The Tauri GUI has a **Remote Devices** page (sidebar → 远程设备) for operating remote Daemons without touching the CLI:

1. Add a device (name + IP + port, persisted locally) and hit **Test connection** to verify the Daemon is reachable.
2. Select the device to open its workbench: list Ports, open one (with baud rate), and send/receive data on the resulting Connection.

All GUI operations go through the library's [`RemoteRpcClient`] (newline-framed JSON-RPC over TCP, one connection per call).

---

## Quick Start

### 1. Start the Server

```bash
# Start with default settings
serial-cli server start

# Output:
# ✓ Server started successfully
#   PID: 12345
#   Socket: /tmp/serial-cli.sock
#   Log: /tmp/serial-cli-server.log
#   Max connections: 10
```

### 2. Check Status

```bash
serial-cli server status

# Output:
# Server Status:
#
#   PID: 12345
#   Status: Running ✓
#   Socket: /tmp/serial-cli.sock
#   ...
```

### 3. Make RPC Calls

```bash
# List available ports
serial-cli server call port_list '{}'

# List protocols
serial-cli server call script_list '{}'

# Get server statistics
serial-cli server call server_stats '{}'
```

### 4. Stop the Server

```bash
serial-cli server stop

# Output:
# ✓ Server stopped successfully
```

---

## API Reference

### Port Management

#### port_list

List all available serial ports.

```bash
serial-cli server call port_list '{}'
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "result": {
    "ports": [
      {
        "port_name": "/dev/ttyUSB0",
        "port_type": "UsbPort"
      }
    ]
  },
  "id": 1
}
```

#### port_open

Open a serial port with optional protocol.

```bash
serial-cli server call port_open '{
  "port": "/dev/ttyUSB0",
  "baudrate": 115200,
  "protocol": "modbus_rtu"
}'
```

**Parameters:**
- `port` (required): Port name
- `baudrate` (optional): Baud rate (default: 115200)
- `protocol` (optional): Protocol name

**Response:**
```json
{
  "jsonrpc": "2.0",
  "result": {
    "connection_id": "conn_123456",
    "port": "/dev/ttyUSB0",
    "protocol": "modbus_rtu"
  },
  "id": 1
}
```

#### port_send

Send data to an open port.

```bash
serial-cli server call port_send '{
  "connection_id": "conn_123456",
  "data": "ATEST"
}'
```

**Parameters:**
- `connection_id` (required): Connection ID from port_open
- `data` (required): Data to send (plain text or hex: prefix with "hex:")

**Response:**
```json
{
  "jsonrpc": "2.0",
  "result": {
    "bytes_sent": 7
  },
  "id": 1
}
```

#### port_recv

Receive data from an open port.

```bash
serial-cli server call port_recv '{
  "connection_id": "conn_123456",
  "timeout": 1000
}'
```

**Parameters:**
- `connection_id` (required): Connection ID
- `timeout` (optional): Timeout in milliseconds (default: 1000)

**Response:**
```json
{
  "jsonrpc": "2.0",
  "result": {
    "data": "414154455354",  // hex-encoded
    "bytes_read": 5
  },
  "id": 1
}
```

#### port_close

Close an open port.

```bash
serial-cli server call port_close '{
  "connection_id": "conn_123456"
}'
```

### Protocol Management

#### script_list

List all available scripts (protocols).

```bash
serial-cli server call script_list '{}'
```

#### script_load

Load a custom script (protocol) from Lua file.

```bash
serial-cli server call script_load '{
  "path": "/path/to/custom.lua",
  "name": "my_protocol"
}'
```

#### script_unload

Unload a custom script (protocol).

```bash
serial-cli server call script_unload '{
  "name": "my_protocol"
}'
```

### Connection Management

#### port_subscribe

Subscribe to real-time data push notifications for a connection.

```bash
serial-cli server call port_subscribe '{"connection_id": "xxx"}'
```

When data arrives on the subscribed connection, the server pushes events to the client via the Unix socket.

#### port_unsubscribe

Unsubscribe from data push notifications.

```bash
serial-cli server call port_unsubscribe '{"connection_id": "xxx"}'
```

#### connection_list

List all active connections.

```bash
serial-cli server call connection_list '{}'
```

#### server_stats

Get server statistics.

```bash
serial-cli server call server_stats '{}'
```

---

## Use Cases

### AI Agent Workflow

```python
# Python example for AI agents
import subprocess
import json

def rpc_call(method, params):
    request = json.dumps({"jsonrpc": "2.0", "method": method, "params": params, "id": 1})
    result = subprocess.run(
        ["serial-cli", "server", "call", method, request],
        capture_output=True,
        text=True
    )
    return json.loads(result.stdout)

# Start server
subprocess.run(["serial-cli", "server", "start"])

try:
    # Open port
    conn = rpc_call("port_open", {"port": "/dev/ttyUSB0", "protocol": "modbus_rtu"})
    connection_id = conn["result"]["connection_id"]

    # Send data
    rpc_call("port_send", {"connection_id": connection_id, "data": "ATEST"})

    # Receive data
    response = rpc_call("port_recv", {"connection_id": connection_id})
    print(f"Received: {response['result']['data']}")

finally:
    # Stop server
    subprocess.run(["serial-cli", "server", "stop"])
```

### High-Frequency Automation

```bash
#!/bin/bash
# High-frequency polling script

# Start server once
serial-cli server start

# Poll 1000 times (without repeated open/close overhead)
for i in {1..1000}; do
    serial-cli server call port_send '{
      "connection_id": "conn_123",
      "data": "AT_POLL"
    }'

    serial-cli server call port_recv '{
      "connection_id": "conn_123",
      "timeout": 100
    }'
done

# Clean up
serial-cli server stop
```

---

## Configuration

### Server Options

```bash
serial-cli server start \
  --socket-path /tmp/custom.sock     # Custom socket path
```

**Note:** Server Mode listens on TCP (default `0.0.0.0:23333`) plus the Unix socket (Linux/macOS). Requests are newline-framed JSON-RPC; each request line is terminated with `\n`.

The server socket path defaults to a platform-specific location (typically `~/.cache/serial_cli/serial-cli.sock` on Linux, or the equivalent on other platforms).

### Session File

The server stores session metadata in:
- **Linux/macOS**: `~/.cache/serial_cli/server_session.json`
- **Windows**: `%LOCALAPPDATA%\serial_cli\server_session.json`

### Log File

Server logs are written to:
- **Default**: `~/.cache/serial_cli/server.log`
- **Custom**: Via `--log` option

---

## Performance

### Latency Comparison

| Operation | CLI Mode | Server Mode | Improvement |
|-----------|----------|-------------|-------------|
| Single operation | 50-200ms | 1-5ms | **10-100x** |
| 100 operations | 5-20s | 0.1-0.5s | **10-100x** |

### Resource Usage

- **Memory**: ~10-20MB baseline + ~1MB per connection
- **CPU**: < 1% idle, < 5% under load
- **Connections**: Up to 10 concurrent (configurable)

---

## Troubleshooting

### Server won't start

```bash
# Check if already running
serial-cli server status

# Force stop if needed
serial-cli server stop

# Check logs
tail -f ~/.cache/serial_cli/server.log
```

### Connection refused

```bash
# Verify server is running
serial-cli server status

# Check socket permissions
ls -la /tmp/serial-cli.sock

# Should show: srw------- (user read/write only)
```

### Stale session

```bash
# Server died without cleanup?
serial-cli server stop
# This will clean up stale session files
```

---

## Advanced Topics

### Direct Socket Access

You can also connect directly to the Unix socket:

```bash
echo '{"jsonrpc":"2.0","method":"port_list","id":1}' | socat - /tmp/serial-cli.sock
```

### Integration with AI Frameworks

#### LangChain Tool Example

```python
from langchain.tools import BaseTool
from pydantic import BaseModel, Field
import subprocess

class SerialPortInput(BaseModel):
    port: str = Field(description="Serial port name")
    data: str = Field(description="Data to send")

class SerialPortTool(BaseTool):
    name = "serial_port_send"
    description = "Send data to serial port via server mode"
    args_schema = SerialPortInput

    def _run(self, port: str, data: str):
        # Open connection
        open_result = self._rpc_call("port_open", {
            "port": port,
            "protocol": "line"
        })
        connection_id = open_result["result"]["connection_id"]

        # Send data
        send_result = self._rpc_call("port_send", {
            "connection_id": connection_id,
            "data": data
        })

        # Receive response
        recv_result = self._rpc_call("port_recv", {
            "connection_id": connection_id,
            "timeout": 1000
        })

        # Close connection
        self._rpc_call("port_close", {
            "connection_id": connection_id
        })

        return recv_result["result"]["data"]

    def _rpc_call(self, method, params):
        request = json.dumps({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
            "id": 1
        })
        result = subprocess.run(
            ["serial-cli", "server", "call", method, request],
            capture_output=True,
            text=True
        )
        return json.loads(result.stdout)
```

---

## Security Considerations

### Transport Security

- **Unix socket**: created with `0600` permissions (user read/write only) — local access only.
- **TCP**: binds `0.0.0.0:23333` by default for LAN remote access. **No authentication in v1** — only expose it where the network is trusted, or narrow it with `--bind` / disable with `--no-tcp`.

### Connection Limits

The server enforces a maximum concurrent connection limit (default: 10) to prevent resource exhaustion.

### Idle Timeout

Idle connections are automatically closed after 5 minutes (configurable) to prevent resource leaks.

---

## Future Enhancements

Planned features for future releases:

- [ ] Authentication and encryption for TCP access
- [ ] WebSocket support for real-time data streaming
- [ ] Connection pooling
- [ ] Performance metrics and monitoring
- [ ] Official Python/TypeScript SDKs
- [ ] mDNS device discovery for the GUI

---

## See Also

- [Architecture Documentation](../dev/ARCH.md)
- [Protocol Reference](../reference/protocols.md)
- [AI Usage Guide](USAGE.md)
