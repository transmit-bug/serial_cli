<div align="center">

  ![Serial CLI](https://img.shields.io/badge/Serial%20CLI-0.6.0-blue?style=for-the-badge&logo=rust)
  [![License](https://img.shields.io/badge/License-MIT%20%2F%20Apache--2.0-green?style=for-the-badge)](LICENSE-MIT)
  [![Rust](https://img.shields.io/badge/Rust-1.75%2B-orange?style=for-the-badge&logo=rust)](https://www.rust-lang.org)
  [![Tests](https://img.shields.io/badge/Tests-260%2B%20passing-success?style=for-the-badge)](https://github.com/zazac-zhang/serial_cli)
  [![GUI](https://img.shields.io/badge/GUI-Production%20Ready-brightgreen?style=for-the-badge&logo=react)](https://reactjs.org/)

  # 🚀 Serial CLI

  **A Universal Serial Port Tool with CLI & GUI - Optimized for AI Interaction**

  [Quick Start](#-quick-start) • [Documentation](#-documentation) • [Features](#-features) • [Examples](#-examples) • [Lua Scripting](#-lua-scripting) • [Development](#-development)

</div>

---

## 💡 What is Serial CLI?

Serial CLI is a powerful, cross-platform serial communication tool built with Rust. It provides **CLI interface**, **structured JSON output**, **embedded LuaJIT scripting**, **support for multiple protocols**, and a **modern GUI application** - making it perfect for both human interaction and AI/automation workflows.

**✨ CLI Production Ready** • **🖥️ GUI Production Ready** • **🔧 260+ Tests Passing** • **🌍 Linux • macOS • Windows**

---

## 🚀 Quick Start

### Installation

```bash
# Install from source
cargo install --path .

# Or download pre-built binaries
# Visit: https://github.com/zazac-zhang/serial_cli/releases

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

# List available protocols
serial-cli protocol list

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
- **📡 Protocol Support** - Built-in Modbus RTU/ASCII, AT Commands, line-based, and **custom Lua protocols**
- **🎨 Custom Protocols** - Load custom protocols from Lua scripts with hot-reload support
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

### One-Shot Commands

```bash
# Send command and receive response
serial-cli exec /dev/ttyUSB0 "send AT; sleep 100; recv 64"

# With custom baud rate
serial-cli exec /dev/ttyUSB0 --baudrate 9600 "send data"

# With protocol
serial-cli exec /dev/ttyUSB0 --protocol modbus_rtu "send 0x010300000001"

# Hex data
serial-cli exec /dev/ttyUSB0 "send 0x01020304"

# Base64 data
serial-cli exec /dev/ttyUSB0 "send base64:SGVsbG8="
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

## Lua Scripting - Modbus RTU

```lua
-- modbus_read.lua
local port_name = "/dev/ttyUSB0"
local slave_id = 1
local start_addr = 0
local reg_count = 10

-- Open port with Modbus settings
local port = serial_open(port_name, {
  baudrate = 19200,
  databits = 8,
  parity = "even",
  stopbits = 1
})

-- Build Modbus request (function 0x03 = Read Holding Registers)
local request = string.char(
  slave_id, 0x03,
  (start_addr >> 8) & 0xFF, start_addr & 0xFF,
  (reg_count >> 8) & 0xFF, reg_count & 0xFF
)

-- Calculate CRC
local crc = 0xFFFF
for i = 1, #request do
  crc = crc ~ string.byte(request, i)
  for j = 1, 8 do
    if (crc & 0x0001) ~= 0 then
      crc = (crc >> 1) ~ 0xA001
    else
      crc = crc >> 1
    end
  end
end
request = request .. string.char(crc & 0xFF, (crc >> 8) & 0xFF)

-- Send and receive
serial_send(port, request)
sleep(100)
local response = serial_recv(port, 256)

print("Response: " .. hex_encode(response))
serial_close(port)
```

**Run:** `serial-cli run modbus_read.lua`

### Data Logging

```lua
-- data_logger.lua
local port = serial_open("/dev/ttyUSB0", {baudrate = 115200})
local file = io.open("log.txt", "w")

file:write("# Data log started at " .. os.date() .. "\n")

for i = 1, 100 do
  local data = serial_recv(port, 1024)
  if #data > 0 then
    file:write(data)
    file:flush()
    print("Received " .. #data .. " bytes")
  end
  sleep(50)
end

file:close()
serial_close(port)
```

**Run:** `serial-cli run data_logger.lua`

---

## 🔧 Lua Scripting API

Serial CLI includes an embedded **LuaJIT** runtime for powerful automation:

> For the complete API reference, see [Lua Scripting Reference](docs/reference/lua-scripting.md).

### Serial Port Functions

```lua
-- Open serial port
local port = serial_open("/dev/ttyUSB0", {
    baudrate = 115200,      -- Baud rate (default: 115200)
    timeout = 1000,         -- Read timeout in ms (default: 1000)
    data_bits = 8,          -- Data bits: 5-8 (default: 8)
    parity = "none",        -- Parity: "none", "odd", "even" (default: "none")
    stop_bits = 1,          -- Stop bits: 1 or 2 (default: 1)
    flow_control = "none"   -- Flow control: "none", "hardware", "software"
})

-- Send data
serial_send(port, "Hello, World!\r\n")

-- Receive data
local data = serial_recv(port, 1000)

-- Close port
serial_close(port)
```

### Utility Functions

```lua
-- Logging
log_info("Information message")
log_warn("Warning message")
log_error("Error message")

-- JSON
local json = json_encode({key = "value"})
local obj = json_decode('{"key": "value"}')

-- Hex
local hex = hex_encode({0x48, 0x65, 0x6C, 0x6C, 0x6F})
local bytes = hex_decode("48656c6c6f")

-- Time
sleep_ms(1000)
local now = time_now()
```

### Custom Protocol Extension

Load custom protocols from Lua scripts:

```lua
-- Load custom protocol
local ok, err = protocol_load("/path/to/my_protocol.lua")
if ok then
    local encoded = protocol_encode("my_custom_protocol", "data")
    local decoded = protocol_decode("my_custom_protocol", encoded)
end
```

See `examples/` directory for complete protocol examples.

---

## 🛠️ Development

### Prerequisites

```bash
# Rust 1.75+
rustup update stable

# Just task runner (recommended)
cargo install just

# Platform dependencies
# Linux:
sudo apt-get install build-essential libudev-dev libluajit-5.1-dev

# macOS:
xcode-select --install
brew install luajit
```

### Build Commands

```bash
# Development build
just dev          # cargo build

# Release build
just build        # cargo build --release

# Run application
just run <args>   # cargo run -- <args>

# Run all checks (fmt + lint + test)
just check
```

### Testing

```bash
# Run all tests
just test

# Run specific test
just test <test_name>

# Run tests with output
just test-verbose
```

### Code Quality

```bash
# Format code
just fmt

# Check formatting
just fmt-check

# Run linter
just lint

# Cross-compilation
just build-all    # All platforms
just build-linux  # Linux (x86_64 + aarch64)
just build-macos  # macOS (x86_64 + arm64)
just build-windows # Windows (requires cross)
```

### GUI Development

```bash
# Install GUI dependencies
just gui-deps

# Start GUI development server
just gui-dev

# Build GUI application
just gui-build

# Type check frontend
just gui-type-check

# Format all code (Rust + TypeScript)
just gui-fmt

# Check Rust + TypeScript code
just gui-check
```

**GUI Features**:
- Modern Tech Stack — React 19 + Zustand 5 + shadcn/ui + Tailwind CSS 4
- Real-time Data Monitoring — Live display with virtual scrolling (10000+ packets)
- Lua Script Editor — Monaco Editor with syntax highlighting
- Protocol Management — Built-in and custom protocol loading with hot-reload
- Settings Management — Comprehensive configuration with persistence
- Data Export — TXT/CSV/JSON formats with filtering
- System Notifications — Sonner toast + OS desktop notifications
- Command Palette — Global search (⌘K) with fuzzy matching
- Keyboard Shortcuts — Full keyboard navigation and quick actions
- Internationalization — English and Chinese

**GUI Architecture**:
- State Management: Zustand stores (pure, no React Context)
- Component Library: shadcn/ui + Radix UI
- Performance: Virtual scrolling via @tanstack/react-virtual
- View Structure: 5 views (Terminal, Virtual Ports, Scripts, Protocols, Settings)

### Project Structure

```
serial_cli/
├── src/                    # Rust library (core functionality)
│   ├── main.rs             # CLI entry point
│   ├── lib.rs              # Library root
│   ├── error.rs            # Error types
│   ├── config.rs           # Configuration
│   ├── serial_core/        # Serial port I/O
│   ├── protocol/           # Protocol engine
│   ├── lua/                # LuaJIT integration
│   ├── server/             # Server Mode daemon
│   ├── task/               # Task scheduling (experimental)
│   └── cli/                # CLI interface
├── src-tauri/              # Tauri application (GUI backend)
│   ├── src/                # Tauri-specific code
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   └── build.rs
├── frontend/               # React frontend (GUI)
│   ├── src/                # React source
│   ├── components/
│   ├── index.html
│   └── package.json
├── examples/               # Lua script examples
├── tests/                  # Integration tests
├── docs/                   # Documentation
│   ├── ai/                 # AI/automation guides
│   ├── dev/                # Development docs
│   ├── reference/          # Reference material
│   └── commands/           # Per-command docs
├── justfile                # Build commands
├── Cargo.toml              # Package config
└── README.md               # This file
```

---

## 🔍 Troubleshooting

For common issues and solutions, see the [Troubleshooting Guide](docs/reference/troubleshooting.md).

**Quick fixes:**

| Issue | Solution |
|-------|----------|
| Permission denied (Linux) | `sudo usermod -a -G dialout $USER` then re-login |
| Port not found | Run `serial-cli list-ports` to verify available ports |
| Timeout error | Check baudrate matches device, increase timeout |
| Port in use | Close other applications using the port (PuTTY, Arduino IDE, etc.) |
| Lua script error | Run with `--verbose` for detailed error output |

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

[GitHub](https://github.com/zazac-zhang/serial_cli) • [Report Issues](https://github.com/zazac-zhang/serial_cli/issues) • [Releases](https://github.com/zazac-zhang/serial_cli/releases)

</div>
