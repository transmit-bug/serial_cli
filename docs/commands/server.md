# Server Command

> `serial-cli server` — start, stop, and interact with the Server Mode daemon (JSON-RPC 2.0 over Unix socket).

## Overview

Server Mode runs a persistent daemon that provides a JSON-RPC 2.0 interface for serial port and protocol management. It is designed for **AI agents and automation workflows**, reducing latency from 50-200ms (one-shot CLI) to 1-5ms (persistent connection).

**Platform support**: Unix (Linux/macOS) only. Windows is not supported.

## Subcommands

### `server start`

Start the server daemon in the background.

```bash
# Start with defaults (Unix socket at /tmp/serial-cli.sock)
serial-cli server start

# Custom socket path
serial-cli server start --socket-path /tmp/my-serial.sock

# Custom log file
serial-cli server start --log /var/log/serial-server.log

# Limit concurrent connections
serial-cli server start --max-connections 20

# Output:
# ✓ Server started successfully
#   PID: 12345
#   Socket: /tmp/serial-cli.sock
#   Log: ~/.cache/serial_cli/server.log
#   Max connections: 10
```

**Options:**

| Flag | Default | Description |
|------|---------|-------------|
| `--socket-path <path>` | `/tmp/serial-cli.sock` | Unix socket path |
| `--port <port>` | *(none)* | TCP port (alternative to Unix socket) |
| `--log <path>` | `~/.cache/serial_cli/server.log` | Log file path |
| `--max-connections <n>` | `10` | Maximum concurrent client connections |

### `server stop`

Stop the running server daemon.

```bash
serial-cli server stop

# Output:
# ✓ Server stopped successfully
```

The stop command sends a termination signal to the daemon process and cleans up the session file.

### `server status`

Show the current server daemon status.

```bash
serial-cli server status

# Output:
# Server Status:
#
#   PID: 12345
#   Status: Running ✓
#   Socket: /tmp/serial-cli.sock
#   TCP Port: None
#   Log: ~/.cache/serial_cli/server.log
#   Max Connections: 10
#   Uptime: 5m 30s
```

### `server call <method> <json_args>`

Send a JSON-RPC 2.0 request to the running server and print the response.

```bash
# List available ports
serial-cli server call port_list '{}'

# Open a serial port
serial-cli server call port_open '{"port": "/dev/ttyUSB0", "baudrate": 115200}'

# Send data
serial-cli server call port_send '{"connection_id": "conn_123", "data": "AT"}'

# Receive data
serial-cli server call port_recv '{"connection_id": "conn_123", "timeout": 1000}'

# Get server statistics
serial-cli server call server_stats '{}'

# Read args from stdin
echo '{"port": "/dev/ttyUSB0"}' | serial-cli server call port_open --stdin
```

**Available RPC Methods:**

| Method | Description |
|--------|-------------|
| `port_list` | List available serial ports |
| `port_open` | Open a serial port (returns `connection_id`) |
| `port_close` | Close a serial port connection |
| `port_send` | Send data to an open port |
| `port_recv` | Receive data from an open port |
| `protocol_list` | List available protocols |
| `protocol_load` | Load a custom protocol from Lua script |
| `protocol_unload` | Unload a custom protocol |
| `connection_list` | List active connections |
| `server_stats` | Get server statistics |

### `server daemon` (internal)

Internal command used by `server start` to spawn the background daemon process. Not intended for direct use.

## Session Management

The server stores session metadata to track the daemon process:

- **Linux/macOS**: `~/.cache/serial_cli/server_session.json`
- The session file records the PID, socket path, start time, and configuration.
- On `server start`, stale sessions (process no longer running) are automatically cleaned up.
- On `server stop`, the session file is removed after successful shutdown.

## Error Handling

| Error | Cause | Resolution |
|-------|-------|------------|
| "Server already running" | A daemon process is already active | Run `server stop` first, or use `server status` to check |
| "Server is not running" | No session file or process dead | Run `server start` to launch the daemon |
| "Failed to connect to server" | Socket file missing or permissions issue | Check socket exists (`ls -la /tmp/serial-cli.sock`) and start the server |
| "Server failed to start" | Daemon process exited immediately | Check the log file for details: `tail ~/.cache/serial_cli/server.log` |

## Example Workflow

```bash
# 1. Start the server
serial-cli server start

# 2. List ports
serial-cli server call port_list '{}'

# 3. Open a port
CONN=$(serial-cli server call port_open '{"port": "/dev/ttyUSB0", "baudrate": 115200}' | jq -r '.result.connection_id')

# 4. Send and receive
serial-cli server call port_send "{\"connection_id\": \"$CONN\", \"data\": \"AT\"}"
serial-cli server call port_recv "{\"connection_id\": \"$CONN\", \"timeout\": 1000}"

# 5. Close the connection
serial-cli server call port_close "{\"connection_id\": \"$CONN\"}"

# 6. Stop the server
serial-cli server stop
```

## See Also

- [Server Mode User Guide](../../docs/ai/SERVER_MODE.md) — detailed API reference and AI integration examples
- [Server Mode Design](../../docs/dev/SERVER_MODE.md) — technical design (Chinese)
- [Architecture Documentation](../../docs/dev/ARCH.md) — project structure and module layout
