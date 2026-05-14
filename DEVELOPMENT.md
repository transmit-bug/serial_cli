# Serial CLI - Development Guide

## Development Setup

### Prerequisites

```bash
# Rust 1.75+
rustup update stable
rustup component add rustfmt clippy

# Just task runner (recommended)
cargo install just
```

### Platform Dependencies

**Linux:**
```bash
sudo apt-get install build-essential libudev-dev libluajit-5.1-dev
sudo usermod -a -G dialout $USER  # Serial port access
```

**macOS:**
```bash
xcode-select --install
brew install luajit
```

**Windows:**
- Install Visual Studio Build Tools with C++ tools
- Install USB-to-serial drivers (FTDI, CP210x, CH340)

### IDE Setup

**VS Code Recommended Extensions:**
- rust-analyzer
- CodeLLDB
- Even Better TOML
- Error Lens

**.vscode/settings.json:**
```json
{
  "rust-analyzer.cargo.features": "all",
  "rust-analyzer.checkOnSave.command": "clippy",
  "rust-analyzer.cargo.loadOutDirsFromCheck": true
}
```

---

## GUI Development

### Prerequisites

```bash
# Node.js 20+
node --version

# Rust + Tauri CLI
cargo install tauri-cli
```

### Commands

```bash
just gui-deps       # Install frontend dependencies
just gui-dev        # Start development server (hot reload)
just gui-build      # Build GUI application
just gui-type-check # Type check frontend
just gui-check      # Check Rust + TypeScript code
just gui-fmt        # Format all code (Rust + TypeScript + CSS)
just gui-clean      # Clean build artifacts
```

See README.md for GUI features and architecture.

---

## Cross-Compilation

### Prerequisites

```bash
cargo install cross
# Docker required (cross uses containers)
```

### Build Commands

```bash
just build-all      # All platforms
just build-linux    # x86_64 + aarch64
just build-macos    # x86_64 + arm64 (macOS only)
just build-windows  # x86_64 (requires cross)
just release        # Full release build (clean + all platforms)
```

---

## Contributing

### Pull Request Process

```bash
git checkout -b feature/your-feature-name
just check
git commit -m "Add: Your feature description"
```

### Commit Message Format

```
<type>: <short description>
```

**Types:** `Add:`, `Fix:`, `Update:`, `Refactor:`, `Docs:`, `Test:`, `Chore:`, `Perf:`

### Code Style

- Use `cargo fmt` for formatting
- Fix all `clippy` warnings
- Write unit tests for new features
- Add comments for complex logic

---

## Debugging

```bash
# Debug logging
RUST_LOG=debug cargo run -- list-ports
RUST_LOG=trace cargo run -- list-ports

# Profiling
cargo install flamegraph
cargo flamegraph --bin serial-cli -- list-ports

# Benchmark
cargo bench
```

---

## Resources

- [Rust Guidelines](https://rust-lang.github.io/api-guidelines/)
- [API Documentation](https://docs.rs/serial-cli/)
- [README.md](README.md) - Quick start
- [GitHub Issues](https://github.com/zazac-zhang/serial_cli/issues)
