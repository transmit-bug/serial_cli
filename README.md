<div align="center">

  ![Serial CLI](https://img.shields.io/badge/Serial%20CLI-0.6.0-blue?style=for-the-badge&logo=rust)
  [![License](https://img.shields.io/badge/License-MIT%20%2F%20Apache--2.0-green?style=for-the-badge)](LICENSE-MIT)
  [![Rust](https://img.shields.io/badge/Rust-1.75%2B-orange?style=for-the-badge&logo=rust)](https://www.rust-lang.org)
  [![Tests](https://img.shields.io/badge/Tests-237%2B%20passing-success?style=for-the-badge)](https://github.com/transmit-bug/serial_cli)
  [![GUI](https://img.shields.io/badge/GUI-Production%20Ready-brightgreen?style=for-the-badge&logo=react)](https://reactjs.org/)

  # 🚀 Serial CLI

  **A Universal Serial Port Tool with CLI & GUI - Optimized for AI Interaction**

  [Quick Start](#-quick-start) • [Documentation](#-documentation) • [Features](#-features) • [Examples](#-examples) • [Lua Scripting](#-lua-scripting) • [Development](#-development)

</div>

---

## 💡 What is Serial CLI?

Serial CLI is a powerful, cross-platform serial communication tool built with Rust. It provides **CLI interface**, **structured JSON output**, **embedded LuaJIT scripting**, **support for multiple protocols**, and a **modern GUI application** - making it perfect for both human interaction and AI/automation workflows.

**✨ CLI Production Ready** • **🖥️ GUI Production Ready** • **🔧 237+ Tests Passing** • **🌍 Linux • macOS • Windows**

---

## 🚀 Quick Start

### Installation

```bash
# Install from source
cargo install --path .

# Or download pre-built binaries
# Visit: https://github.com/transmit-bug/serial_cli/releases

# Clone repository
git clone <repository-url>
cd serial_cli

# Development build
just dev

# Release build
just build

# Run tests
just test
```

### Basic Usage

```bash
# List available serial ports
serial-cli port list

# Send data to a port
serial-cli port send --port /dev/ttyUSB0 "AT"

# Interactive mode
serial-cli interactive

# Run Lua script
serial-cli run script.lua

# List available scripts
serial-cli script list

# Create virtual serial port pair
serial-cli virtual create --backend auto

# Start sniffing on a port
serial-cli sniff start --port /dev/ttyUSB0

# Start server daemon (for AI/automation workflows)
serial-cli server start

# Configuration management
serial-cli config show
serial-cli config set serial.baudrate 9600
```

---

## 📖 Usage Examples

### Interactive Shell

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

---

## ✨ Features

<div align="center">

| 🎯 **Universal** | 🤖 **AI-Optimized** | ⚡ **Scriptable** | 🌍 **Cross-Platform** |
|:---:|:---:|:---:|:---:|
| Works with any serial device | Structured JSON output | Embedded LuaJIT runtime | Linux • macOS • Windows |

| 📡 **Protocols** | 🔄 **Batch Mode** | 🔍 **Sniff Sessions** | 🖥️ **GUI Available** | 🚀 **Server Mode** |
|:---:|:---:|:---:|:---:|:---:|
| Modbus • AT Commands • Custom | Variables, loops, error reporting | Start/stop/stats/save | Tauri-based GUI | Daemon with JSON-RPC (12 methods) |

</div>

### Core Capabilities

- **🔌 Serial Port Management** - List, open, configure, and manage serial ports
- **📜 Lua Scripting** - Automate tasks with embedded LuaJIT (high-performance)
- **📡 Protocol Support** - Built-in Modbus RTU/ASCII, AT Commands, line-based, and **custom Lua scripts**
- **🎨 Custom Scripts** - Load custom scripts from Lua files with hot-reload support
- **🤖 AI-Friendly** - JSON output mode for easy integration with AI systems
- **🔄 Batch Processing** - Execute multiple scripts with variable substitution, loops, and per-script error reporting
- **🔍 Sniff Sessions** - Start/stop/stats/save serial traffic with background daemon and session management
- **🚀 Server Mode** - Persistent daemon with JSON-RPC 2.0 interface for AI/automation workflows:
  - 10-100x latency improvement (50-200ms → 1-5ms with persistent connections)
  - Protocol persistence (load once, use globally)
  - Multi-client support (up to 10 concurrent connections)
  - Standard JSON-RPC 2.0 API with 12 methods (including subscribe/unsubscribe)
  - Unix socket IPC (Unix) and named pipes (Windows)
  - Perfect for AI agent integration and automation
- **🖥️ GUI Application** - Modern Tauri-based GUI with:
  - Professional tool aesthetic (VS Code / Postman style)
  - Real-time data monitoring with virtual scrolling
  - Monaco script editor
  - Protocol management with hot-reload
  - Multi-format data export (TXT/CSV/JSON)
  - System notifications
  - Complete keyboard shortcuts
  - Internationalization (en/zh)
- **🔌 Virtual Serial Ports** - Pluggable backend architecture:
  - **PTY Backend** (Unix/macOS) - POSIX pseudo-terminals
  - **NamedPipe Backend** (Windows) - Windows named pipes
  - **Socat Backend** (Cross-platform) - Socat-based virtual ports
  - Platform auto-detection (defaults to best backend for your OS)
  - Runtime backend selection via CLI flag or config file

---

### Run Lua Scripts

```bash
# Run a Lua script
serial-cli run script.lua

# With arguments
serial-cli run script.lua arg1 arg2

# Run with specific port settings
serial-cli run modbus_read.lua --port /dev/ttyUSB0 --baudrate 19200
```

#### Data Sniffing — Session Management

```bash
# Start sniffing on a port (spawns background daemon)
serial-cli sniff start -p /dev/ttyUSB0 --output capture.log

# Check sniff session statistics
serial-cli sniff stats

# Save captured packets to a file
serial-cli sniff save -p backup.log

# Stop the active sniff session
serial-cli sniff stop
```

### Batch Processing — Variables & Loops

```bash
# Run a single Lua script
serial-cli batch run script.lua

# Run a batch file with variable substitution and loops
serial-cli batch run tasks.batch

# List available batch scripts
serial-cli batch list
```

**Batch file example** (`tasks.batch`):
```bash
# Set variables (also reads from environment)
set PORT /dev/ttyUSB0
set DEVICE modbus

# Run scripts with variable substitution
scripts/${DEVICE}/init.lua
scripts/${DEVICE}/read.lua

# Loop with sleep
loop 3
  scripts/${DEVICE}/poll.lua
  sleep 500
end
```

### Virtual Serial Ports — Testing & Development

```bash
# Create virtual port pair (auto-detects best backend)
serial-cli virtual create

# Create with specific backend
serial-cli virtual create --backend pty          # Unix/macOS
serial-cli virtual create --backend namedpipe   # Windows
serial-cli virtual create --backend socat       # Cross-platform (requires socat)

# Create with monitoring enabled
serial-cli virtual create --monitor --max-packets 1000

# List active virtual pairs
serial-cli virtual list

# Show statistics for a pair
serial-cli virtual stats <id>

# Stop a virtual pair
serial-cli virtual stop <id>
```

**Virtual ports are perfect for:**
- Testing serial applications without hardware
- Development and debugging
- CI/CD pipeline automation
- Protocol development and validation

**Backend Selection:**
- **Auto** (recommended): Automatically selects the best backend for your platform
- **PTY**: Best performance on Unix/macOS
- **NamedPipe**: Native Windows implementation
- **Socat**: Cross-platform alternative (requires `socat` installation)

**Set default backend in config:**
```bash
serial-cli config set virtual.backend socat
```

### Server Mode — AI/Automation Workflow

```bash
# Start the server daemon
serial-cli server start

# Check server status
serial-cli server status

# Make RPC calls
serial-cli server call port_list '{}'
serial-cli server call port_open '{"port": "/dev/ttyUSB0", "baudrate": 115200}'
serial-cli server call port_send '{"connection_id": "xxx", "data": "AT"}'
serial-cli server call port_recv '{"connection_id": "xxx", "length": 64}'
serial-cli server call server_stats '{}'

# Stop the server
serial-cli server stop
```

**Server Mode is perfect for:**
- **AI agents** - Persistent connections reduce latency by 10-100x
- **Automation workflows** - Long-running processes with connection pooling
- **Multi-client scenarios** - Multiple agents share serial port connections
- **Protocol caching** - Custom protocols loaded once, available globally
- **CI/CD pipelines** - Fast, repeatable serial operations

**Performance Benefits:**
- **Latency**: 50-200ms (one-shot) → 1-5ms (persistent connection)
- **Concurrency**: Up to 10 simultaneous client connections
- **Memory**: ~10-20MB baseline footprint
- **Protocols**: Load custom Lua protocols once, use across all clients

**RPC Methods Available:**
- `port_list` - List available serial ports
- `port_open` - Open a serial port (returns connection_id)
- `port_close` - Close a serial port connection
- `port_send` - Send data to an open port
- `port_recv` - Receive data from an open port
- `port_subscribe` - Subscribe to real-time data push notifications
- `port_unsubscribe` - Unsubscribe from data push notifications
- `protocol_list` - List available protocols
- `protocol_load` - Load a custom protocol
- `protocol_unload` - Unload a custom protocol
- `connection_list` - List active connections
- `server_stats` - Get server statistics

---

## 📜 Lua Scripting

Serial CLI embeds a **LuaJIT** runtime for automation. Write scripts to open ports, send/receive data, encode/decode protocols, and more.

```lua
local port = serial_open("/dev/ttyUSB0", {baudrate = 115200})
serial_send(port, "AT\r\n")
local response = serial_recv(port, 1000)
print(json_encode({status = "ok", data = response}))
serial_close(port)
```

Run: `serial-cli run script.lua`

See `examples/` for protocol implementations (Modbus RTU, data logging, etc.).

> **Full API reference**: [docs/reference/lua-scripting.md](docs/reference/lua-scripting.md)

---

## 🛠️ Development

See [DEVELOPMENT.md](DEVELOPMENT.md) for full development guide (prerequisites, IDE setup, cross-compilation, release process).

**Quick commands** (requires Rust 1.75+ and [just](https://github.com/casey/just)):

```bash
just dev          # Build (debug)
just build        # Build (release)
just test         # Run tests
just check        # fmt + lint + test
just gui-dev      # Start GUI dev server
just gui-build    # Build GUI application
```

**Project structure** — see [docs/dev/ARCH.md](docs/dev/ARCH.md) for full architecture reference.

---

## 🔍 Troubleshooting

See [docs/reference/troubleshooting.md](docs/reference/troubleshooting.md) for full guide.

**Quick fixes:**

| Issue | Solution |
|-------|----------|
| Permission denied (Linux) | `sudo usermod -a -G dialout $USER` then re-login |
| Port not found | Run `serial-cli port list` to verify available ports |
| Timeout error | Check baudrate matches device, increase timeout |
| Port in use | Close other applications using the port |

**Debug mode:** `serial-cli --verbose <command>` or `RUST_LOG=debug serial-cli <command>`

---

## 📚 Documentation

| Document | Description |
|:---|:---|
| **[DEVELOPMENT.md](DEVELOPMENT.md)** | Development guide for contributors |
| **[docs/guides/getting-started.md](docs/guides/getting-started.md)** | Getting started guide |
| **[docs/ai/SERVER_MODE.md](docs/ai/SERVER_MODE.md)** | Server Mode user guide (AI/automation workflows) |
| **[docs/ai/USAGE.md](docs/ai/USAGE.md)** | AI integration guide |
| **[docs/reference/troubleshooting.md](docs/reference/troubleshooting.md)** | Troubleshooting guide |
| **[docs/README.md](docs/README.md)** | Complete documentation index |

---

## 🤝 Contributing

Contributions are welcome! Please read [DEVELOPMENT.md](DEVELOPMENT.md) for details on our code of conduct, development setup, and submission process.

---

## 📝 License

Dual-licensed under:

- [MIT License](LICENSE-MIT)
- [Apache License, Version 2.0](LICENSE-APACHE)

---

<div align="center">

**Built with ❤️ and Rust**

[GitHub](https://github.com/transmit-bug/serial_cli) • [Report Issues](https://github.com/transmit-bug/serial_cli/issues) • [Releases](https://github.com/transmit-bug/serial_cli/releases)

</div>
