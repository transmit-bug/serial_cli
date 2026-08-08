<div align="center">

  ![Serial CLI](https://img.shields.io/badge/Serial%20CLI-0.6.0-blue?style=for-the-badge&logo=rust)
  [![License](https://img.shields.io/badge/License-MIT%20%2F%20Apache--2.0-green?style=for-the-badge)](LICENSE-MIT)
  [![Rust](https://img.shields.io/badge/Rust-1.75%2B-orange?style=for-the-badge&logo=rust)](https://www.rust-lang.org)
  [![Tests](https://img.shields.io/badge/Tests-237%2B%20passing-success?style=for-the-badge)](https://github.com/transmit-bug/serial_cli)
  [![GUI](https://img.shields.io/badge/GUI-Production%20Ready-brightgreen?style=for-the-badge&logo=react)](https://reactjs.org/)

  # 🚀 Serial CLI

  **A Universal Serial Port Tool with CLI & GUI - Optimized for AI Interaction**

  [Quick Start](#-quick-start) • [Features](#-features) • [Usage](#-usage) • [Lua Scripting](#-lua-scripting) • [GUI](#-gui) • [Documentation](#-documentation)

</div>

---

## 💡 What is Serial CLI?

Serial CLI is a powerful, cross-platform serial communication tool built with Rust. It provides a **CLI**, a **modern desktop GUI**, **structured JSON output**, **embedded LuaJIT scripting**, and **multiple protocol support** — built from the ground up for both human interaction and AI/automation workflows.

**✨ CLI Production Ready** • **🖥️ GUI Production Ready** • **🔧 237+ Tests Passing** • **🌍 Linux • macOS • Windows**

**At a glance:**

- **Human-friendly** — interactive REPL shell plus a full desktop GUI (Tauri), with i18n (en/zh)
- **AI/automation-ready** — structured JSON output, a persistent JSON-RPC server daemon, and LAN remote access
- **Deeply scriptable** — embedded LuaJIT with `require()` cross-script imports, hot-reload, custom protocols
- **No-hardware testing** — virtual serial port pairs (PTY / NamedPipe / Socat) and traffic sniffing

---

## ✨ Features

<div align="center">

| 🎯 **Universal** | 🤖 **AI-Optimized** | ⚡ **Scriptable** | 🌍 **Cross-Platform** |
|:---:|:---:|:---:|:---:|
| Works with any serial device | Structured JSON output | Embedded LuaJIT runtime | Linux • macOS • Windows |

| 📡 **Protocols** | 🔍 **Sniff Sessions** | 🖥️ **GUI Available** | 🚀 **Server Mode** |
|:---:|:---:|:---:|:---:|
| Modbus • AT Commands • Custom | Start/stop/stats/save | Tauri-based GUI | Daemon with JSON-RPC (12 methods) |

</div>

### Core Capabilities

- **🔌 Serial Port Management** — list, open, configure, and manage serial ports
- **📜 Lua Scripting** — automate tasks with embedded LuaJIT, including `require()` cross-script imports and hot-reload
- **📡 Protocol Support** — built-in Modbus RTU/ASCII, AT Commands, line-based, and **custom Lua scripts**
- **🤖 AI-Friendly** — JSON output mode for easy integration with AI systems
- **🔍 Sniff Sessions** — start/stop/stats/save serial traffic with a background daemon and session management
- **🚀 Server Mode** — persistent daemon with a JSON-RPC 2.0 interface for AI/automation workflows:
  - 10-100x latency improvement (50-200ms → 1-5ms with persistent connections)
  - Protocol persistence (load once, use globally) and multi-client support (up to 10 connections)
  - **LAN remote access** — operate any device's Daemon from another machine on the LAN (`server call --remote <ip:port>`), plus a Remote Devices page in the GUI
  - Unix socket IPC (Unix), named pipes (Windows), daemon auto-start on boot (systemd / launchd / Task Scheduler)
- **🖥️ GUI Application** — modern Tauri-based GUI with:
  - Real-time data monitoring with virtual scrolling and remote device management
  - Monaco script editor and protocol management with hot-reload
  - Multi-format data export (TXT/CSV/JSON), system notifications, keyboard shortcuts, i18n (en/zh)
- **🔌 Virtual Serial Ports** — pluggable backends: **PTY** (Unix/macOS), **NamedPipe** (Windows), **Socat** (cross-platform), with platform auto-detection and runtime backend selection (CLI flag or config)

---

## 🚀 Quick Start

### Installation

```bash
# Auto-installer (detects OS/arch, verifies SHA-256)
curl -fsSL https://github.com/transmit-bug/serial_cli/releases/latest/download/serial-cli-install.sh | sh
# Windows (PowerShell):
#   Invoke-WebRequest https://github.com/transmit-bug/serial_cli/releases/latest/download/serial-cli-install.ps1 -OutFile install.ps1; .\install.ps1

# Or install the .deb (Debian/Ubuntu/Raspberry Pi) — also ships a systemd unit
wget https://github.com/transmit-bug/serial_cli/releases/latest/download/serial-cli-linux-x86_64.deb
sudo apt install ./serial-cli-linux-x86_64.deb

# Or build from source (Rust 1.75+)
cargo install --path .
```

### 开机自启 (Daemon auto-start on boot)

```bash
# Register the daemon to start on boot (systemd / launchd / Task Scheduler)
serial-cli server service install [--port 23333] [--bind 0.0.0.0]

# Linux: the .deb already ships a systemd unit — enable it directly
sudo systemctl enable --now serial-cli

# Remove auto-start
serial-cli server service uninstall
```

### First Steps

```bash
# List available serial ports
serial-cli port list

# Send data to a device
serial-cli port send --port /dev/ttyUSB0 "AT"

# Interactive REPL (also the default when no subcommand is given)
serial-cli

# Run a Lua script
serial-cli run script.lua

# Start the JSON-RPC server daemon (for AI/automation + LAN access)
serial-cli server start
```

---

## 📖 Usage

Full command references live in [docs/commands](docs/commands/); here is what each capability looks like:

| Capability | Example | Docs |
|------------|---------|------|
| Interactive shell | `serial-cli` | [interactive](docs/commands/interactive.md) |
| Send / receive data | `serial-cli port send --port /dev/ttyUSB0 "AT"` | [list-ports](docs/commands/list-ports.md) |
| Lua scripts | `serial-cli run modbus_read.lua` | [run-script](docs/commands/run-script.md) |
| Protocol scripts | `serial-cli script list` | [script](docs/commands/script.md) |
| Traffic sniffing | `serial-cli sniff start -p /dev/ttyUSB0` | [sniff](docs/commands/sniff.md) |
| Virtual ports | `serial-cli virtual create` | [virtual](docs/commands/virtual.md) |
| Configuration | `serial-cli config show` | [config](docs/commands/config.md) |
| Server / daemon | `serial-cli server start` | [server](docs/commands/server.md) |

### Server Mode — AI/Automation & LAN Remote Access

```bash
serial-cli server start                                   # daemon (Unix socket + TCP 0.0.0.0:23333)
serial-cli server status                                  # check status
serial-cli server call port_open '{"port":"/dev/ttyUSB0","baudrate":115200}'
serial-cli server call --remote 192.168.1.50:23333 port_list '{}'   # LAN remote
serial-cli server stop
```

- **AI agents** — persistent connections cut latency 10-100x (50-200ms → 1-5ms)
- **Multi-client** — up to 10 concurrent clients share port connections
- **Remote debugging** — operate a target device's serial ports from your workstation; the GUI's Remote Devices page does the same over a UI
- **RPC methods** — `port_list`, `port_open/close/send/recv`, `port_subscribe/unsubscribe`, `script_list/load/unload`, `connection_list`, `server_stats`
- **Server options** — `--port` (default 23333), `--bind`, `--no-tcp`, `--socket-path`, `--log`, `--max-connections`

Full guide: [docs/ai/SERVER_MODE.md](docs/ai/SERVER_MODE.md)

---

## 📜 Lua Scripting

Serial CLI embeds a **LuaJIT** runtime for automation. Write scripts to open ports, send/receive data, encode/decode protocols, and more. Scripts can `require()` each other and hot-reload on change.

```lua
local port = serial_open("/dev/ttyUSB0", {baudrate = 115200})
serial_send(port, "AT\r\n")
local response = serial_recv(port, 1000)
print(json_encode({status = "ok", data = response}))
serial_close(port)
```

Run: `serial-cli run script.lua` — see `examples/` and `scripts/protocols/` for ready-made protocol scripts.

**Full API reference**: [docs/reference/lua-scripting.md](docs/reference/lua-scripting.md)

---

## 🖥️ GUI

A modern Tauri-based desktop app (Windows / macOS / Linux):

- Real-time data monitoring with virtual scrolling and remote device management (LAN Daemons)
- Monaco script editor, protocol management with hot-reload
- Multi-format data export (TXT/CSV/JSON), system notifications, keyboard shortcuts, i18n (en/zh)

Build from source: `just gui-build` — see [DEVELOPMENT.md](DEVELOPMENT.md) for prerequisites.

---

## 📚 Documentation

| Document | Description |
|:---|:---|
| **[docs/README.md](docs/README.md)** | Complete documentation index |
| **[docs/guides/getting-started.md](docs/guides/getting-started.md)** | Getting started guide |
| **[docs/guides/script-development.md](docs/guides/script-development.md)** | Lua script development guide |
| **[docs/ai/SERVER_MODE.md](docs/ai/SERVER_MODE.md)** | Server Mode user guide (AI/automation workflows) |
| **[docs/ai/USAGE.md](docs/ai/USAGE.md)** | AI integration guide |
| **[docs/reference/lua-scripting.md](docs/reference/lua-scripting.md)** | Lua API reference |
| **[docs/reference/troubleshooting.md](docs/reference/troubleshooting.md)** | Troubleshooting guide |
| **[DEVELOPMENT.md](DEVELOPMENT.md)** | Development guide for contributors |

---

## 🤝 Contributing

Contributions are welcome! Please read [DEVELOPMENT.md](DEVELOPMENT.md) for our code of conduct, development setup, and submission process.

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
