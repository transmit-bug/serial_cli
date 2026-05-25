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

## Release Process

### Prerequisites

- Rust toolchain installed
- git-cliff: `cargo install git-cliff`
- Write access to GitHub Repository

### 1. Prepare Release

```bash
# Check readiness
./scripts/release.sh v1.2.3 --check-only

# Prepare new version (runs checks then updates versions)
./scripts/release.sh v1.2.3

# Or skip checks if you already verified
./scripts/release.sh v1.2.3 --no-checks

# Review changes
git diff
git status

# Commit version changes
git commit -am "chore: prepare release v1.2.3"
```

### 2. Create Release

```bash
# Create and push tag
git tag -a v1.2.3 -m "Release v1.2.3"
git push origin v1.2.3
```

After pushing the tag, GitHub Actions will:
1. Build binaries for all platforms
2. Create GitHub Release
3. Update Homebrew, Scoop, and AUR
4. Publish to crates.io

### 3. Verify Release

- [ ] GitHub Release created
- [ ] All platform builds successful
- [ ] crates.io publish successful
- [ ] CHANGELOG.md updated

### Rollback

If release fails or issues are found:

```bash
# Delete GitHub Release and tag
gh release delete v1.2.3 --cleanup-tag

# Delete local tag
git tag -d v1.2.3

# Fix issues and re-release
```

### Conventional Commits

Commit message format: `<type>(<scope>): <subject>`

**Types:** `feat`, `fix`, `docs`, `style`, `refactor`, `perf`, `test`, `chore`, `ci`

**Examples:**
```bash
git commit -m "feat(cli): add protocol list command"
git commit -m "fix(protocol): handle empty response correctly"
git commit -m "docs(readme): update installation instructions"
```

---

## Resources

- [Rust Guidelines](https://rust-lang.github.io/api-guidelines/)
- [API Documentation](https://docs.rs/serial-cli/)
- [README.md](README.md) - Quick start
- [GitHub Issues](https://github.com/zazac-zhang/serial_cli/issues)
