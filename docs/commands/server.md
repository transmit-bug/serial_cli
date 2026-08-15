# Server Command

> `serial-cli server` — start, stop, and interact with the Server Mode daemon (JSON-RPC 2.0 over Unix socket and TCP).

## Overview

Server Mode runs a persistent daemon that provides a JSON-RPC 2.0 interface for serial port and protocol management. It is designed for **AI agents, automation workflows, and LAN remote access**, reducing latency from 50-200ms (one-shot CLI) to 1-5ms (persistent connection).

**Two transports, one protocol:**

- **Unix socket** (Linux/macOS): local access for the same machine, `0600` permissions.
- **TCP** (cross-platform): LAN remote access. Clients on the same network connect to `device_ip:port` and speak the identical JSON-RPC protocol.

**Platform support**: Linux, macOS, and Windows.

## Subcommands

### `server start`

Start the server daemon in the background (detached process; on Unix it becomes its own session leader via `setsid`, on Windows it spawns a detached process).

```bash
# Start with defaults (Unix socket + TCP on 0.0.0.0:23333)
serial-cli server start

# Custom socket path
serial-cli server start --socket-path /tmp/my-serial.sock

# Custom TCP port and bind address
serial-cli server start --port 25000 --bind 192.168.1.50

# Local-only: disable the TCP listener entirely
serial-cli server start --no-tcp

# Custom log file / connection limit
serial-cli server start --log /var/log/serial-server.log --max-connections 20

# Output:
# ✓ Server started successfully
#   PID: 12345
#   Socket: /tmp/serial-cli.sock
#   TCP: 0.0.0.0:23333
#   Log: ~/.cache/serial_cli/server.log
#   Max connections: 10
```

**Options:**

| Flag | Default | Description |
|------|---------|-------------|
| `--socket-path <path>` | `/tmp/serial-cli.sock` | Unix socket path |
| `--port <port>` | `23333` | TCP port for LAN remote access |
| `--bind <ip>` | `0.0.0.0` | Address the TCP listener binds to |
| `--no-tcp` | `false` | Disable the TCP listener (local access only) |
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
#   TCP Port: 23333
#   Log: ~/.cache/serial_cli/server.log
#   Max Connections: 10
#   Uptime: 5m 30s
```

### `server call <method> <json_args>`

Send a JSON-RPC 2.0 request to the running server and print the response. Targets the local daemon by default (Unix socket first, TCP `localhost` fallback); use `--remote` to reach a daemon on another machine.

```bash
# List available ports (local)
serial-cli server call port_list '{}'

# Open a serial port on a remote device over the LAN
serial-cli server call --remote 192.168.1.50:23333 port_open '{"port": "/dev/ttyUSB0", "baudrate": 115200}'

# Send data
serial-cli server call port_send '{"connection_id": "conn_123", "data": "AT"}'

# Receive data
serial-cli server call port_recv '{"connection_id": "conn_123", "timeout": 1000}'

# Get server statistics
serial-cli server call server_stats '{}'

# Read args from stdin
echo '{"port": "/dev/ttyUSB0"}' | serial-cli server call port_open --stdin
```

**Options:**

| Flag | Description |
|------|-------------|
| `--remote <ip:port>` | Target a remote daemon (e.g., `192.168.1.50:23333`) instead of the local session |
| `--stdin` | Read the JSON args from stdin instead of the positional argument |

**Available RPC Methods (identical over both transports):**

| Method | Description |
|--------|-------------|
| `port_list` | List available serial ports |
| `port_open` | Open a serial port (returns `connection_id`) |
| `port_close` | Close a serial port connection |
| `port_send` | Send data to an open port |
| `port_recv` | Receive data from an open port |
| `port_subscribe` / `port_unsubscribe` | Subscribe to / unsubscribe from real-time data push notifications |
| `script_list` | List available protocols |
| `script_load` | Load a custom protocol from Lua script |
| `script_unload` | Unload a custom protocol |
| `connection_list` | List active connections |
| `server_stats` | Get server statistics |

### `server daemon` (internal)

Internal foreground entry point used by `server start` (and by the e2e test suite). Not intended for direct user invocation.

### `server service install` / `server service uninstall`

Registers (or removes) daemon **auto-start on boot**, per platform:

- **Linux** — systemd unit. As root: `/etc/systemd/system/serial-cli.service` (system mode); otherwise `~/.config/systemd/user/` (user mode, prints an `enable-linger` tip for boot-without-login)
- **macOS** — launchd LaunchAgent `~/Library/LaunchAgents/com.serial-cli.daemon.plist` (loaded via `launchctl bootstrap`)
- **Windows** — Task Scheduler task `SerialCLIDaemon` running at startup

```bash
# Register auto-start with defaults (TCP 23333, bind 0.0.0.0)
serial-cli server service install

# Custom port / local-only
serial-cli server service install --port 25000 --bind 127.0.0.1
serial-cli server service install --no-tcp

# Remove auto-start
serial-cli server service uninstall
```

> The service only *registers* auto-start; it does not start the daemon
> immediately. Start once with `serial-cli server start` (or reboot).

> **Alternative (Linux)**: the `.deb` package ships the same unit at
> `/usr/lib/systemd/system/serial-cli.service` — enable it with
> `sudo systemctl enable --now serial-cli`. A unit written by
> `server service install` (in `/etc/systemd/system/`) takes precedence
> over the packaged one.

## Session Management

The server stores session metadata to track the daemon process:

- **Linux/macOS**: `~/.cache/serial_cli/server_session.json`
- **Windows**: `%LOCALAPPDATA%\serial_cli\server_session.json`
- The session file records the PID, socket path, TCP port, start time, and configuration.
- On `server start`, stale sessions (process no longer running) are automatically cleaned up.
- On `server stop`, the session file is removed after successful shutdown.

## Security

> **No authentication in v1.** TCP remote access is intended for trusted corporate LANs. Any host that can reach the daemon's TCP port can operate its serial ports.

Mitigations available today:

- `--bind` narrows the listening interface (e.g., `--bind 127.0.0.1` for local-only).
- `--no-tcp` disables the TCP listener entirely.
- The Unix socket remains `0600` (owner-only).

Authentication (token handshake) is tracked as a follow-up issue.

## Error Handling

| Error | Cause | Resolution |
|-------|-------|------------|
| "Server already running" | A daemon process is already active | Run `server stop` first, or use `server status` to check |
| "Server is not running" | No session file or process dead | Run `server start` to launch the daemon |
| "Failed to connect to socket ..." | Socket file missing or permissions issue | Check socket exists (`ls -la /tmp/serial-cli.sock`) and start the server |
| "Failed to connect to <addr>" | Remote daemon unreachable | Verify the device is on the network, the daemon is running, and the port is correct (`server status`) |
| "Server failed to start" | Daemon process exited immediately | Check the log file for details: `tail ~/.cache/serial_cli/server.log` |
| "Port X is already in use by connection Y" | Another Connection holds that Port (OS-level exclusivity) | Close the holding connection first, or use a different Port |

## Example Workflow

### Local

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

### Remote device on the LAN

```bash
# On the target device (e.g., a Linux board):
serial-cli server start --port 23333

# From your workstation:
serial-cli server call --remote 192.168.1.50:23333 port_list '{}'
CONN=$(serial-cli server call --remote 192.168.1.50:23333 port_open '{"port": "/dev/ttyUSB0"}' | jq -r '.result.connection_id')
serial-cli server call --remote 192.168.1.50:23333 port_send "{\"connection_id\": \"$CONN\", \"data\": \"AT\"}"
serial-cli server call --remote 192.168.1.50:23333 port_recv "{\"connection_id\": \"$CONN\", \"timeout\": 1000}"
```

## See Also

- [Server Mode User Guide](../../docs/ai/SERVER_MODE.md) — detailed API reference and AI integration examples
