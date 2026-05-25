# Getting Started

Serial CLI is a universal serial port communication tool with embedded LuaJIT scripting. It supports multiple protocols (Modbus RTU/ASCII, AT Commands, line-based, and custom Lua protocols) with structured output, optimized for both interactive use and automation.

## Installation

### Requirements

- **Rust** 1.75 or later
- **just** task runner

Install Rust via [rustup](https://rustup.rs/):

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Install `just`:

```bash
cargo install just
```

### Building

Clone the repository and build:

```bash
git clone <repository-url>
cd serial_cli
```

```bash
# Development build
just dev

# Release build (recommended for production use)
just build
```

The binary is available at `target/debug/serial-cli` (dev) or `target/release/serial-cli` (release).

Run it directly with:

```bash
just run          # cargo run
just run <args>   # cargo run -- <args>
```

### Verification

```bash
serial-cli --help
```

## Quick Start

### List Available Ports

```bash
serial-cli list-ports
```

### Send Data to a Serial Port

```bash
serial-cli send --port /dev/ttyUSB0 "AT"
serial-cli send --port COM3 --baud 9600 "AT+CGMI"
```

### Interactive Mode

Start an interactive REPL shell for serial communication. This is the default mode when no subcommand is specified:

```bash
serial-cli interactive
serial-cli   # also starts interactive mode
```

### Run a Lua Script

```bash
serial-cli run script.lua
serial-cli run script.lua arg1 arg2
```

## Global Flags

| Flag | Description |
|------|-------------|
| `--json` | Output results as formatted JSON instead of human-readable text |
| `-v, --verbose` | Enable verbose logging output (DEBUG level) |

Examples:

```bash
serial-cli --json list-ports
serial-cli --verbose --json send --port /dev/ttyUSB0 "AT"
```

## Command Reference

For detailed usage of each command, see the dedicated documentation:

| Command | Description | Docs |
|---------|-------------|------|
| `interactive` | Interactive REPL shell | [Interactive Shell Guide](interactive-shell.md) |
| `run` | Execute a Lua script | [Run Script](../commands/run-script.md) |
| `list-ports` | List available serial ports | [List Ports](../commands/list-ports.md) |
| `protocol` | Protocol management | [Protocol](../commands/protocol.md) |
| `sniff` | Traffic sniffing | [Sniff](../commands/sniff.md) |
| `batch` | Batch execution | [Batch](../commands/batch.md) |
| `config` | Configuration management | [Config](../commands/config.md) |
| `virtual` | Virtual serial ports | [Virtual](../commands/virtual.md) |
| `benchmark` | Performance benchmarks | [Benchmark](../commands/benchmark.md) |
| `server` | JSON-RPC server mode | [Server](../commands/server.md) |

## Development

```bash
just dev          # Development build
just build        # Release build
just test         # Run tests
just test-verbose # Run tests with full output
just check        # Format check + lint + test
just fmt          # Format code
just lint         # Clippy lint check
just run <args>   # Run with arguments
```

Cross-compilation:

```bash
just build-all    # Linux + macOS + Windows
just build-linux  # x86_64 + aarch64
just build-macos  # x86_64 + arm64
```

## Next Steps

- Read the [Architecture Guide](../dev/ARCH.md) for a deep dive into the project structure and design patterns.
- Explore protocol scripting with Lua in the `protocol/` documentation.
- Check the configuration reference for all available settings.
