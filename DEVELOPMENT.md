# Serial CLI - Development Guide

> **Quick start**: See [README.md](README.md) for build commands. See [AGENTS.md](AGENTS.md) for architecture overview.

## Platform Dependencies

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

---

## Resources

- [API Documentation](https://docs.rs/serial-cli/)
- [GitHub Issues](https://github.com/transmit-bug/serial_cli/issues)
