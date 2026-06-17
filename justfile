# Serial CLI - Just Command Configuration
# https://github.com/casey/just

# Default: show available commands
default:
    @just --list

# ──────────────────────────────────────────────────────────────────────────────
# Development
# ──────────────────────────────────────────────────────────────────────────────

# Build (debug)
dev:
    cargo build

# Build (release)
build:
    cargo build --release

# Run with args
run *args:
    cargo run -- {{args}}

# Auto-rebuild on file changes
watch:
    cargo watch -x run

# Clean build artifacts
clean:
    cargo clean

# Kill dev server and Tauri processes to free ports
close:
    #!/usr/bin/env bash
    set -euo pipefail
    # Kill Vite dev server on port 1420
    lsof -ti:1420 | xargs kill -9 2>/dev/null || true
    # Kill serial-cli-tauri processes
    pkill -9 -f serial-cli-tauri 2>/dev/null || true
    echo "✓ Dev processes cleaned"

# ──────────────────────────────────────────────────────────────────────────────
# Testing
# ──────────────────────────────────────────────────────────────────────────────

# Run all tests
test:
    cargo test

# Run specific test: just test-one <name>
test-one name *args:
    cargo test {{name}} -- {{args}}

# Auto-run tests on file changes
test-watch:
    cargo watch -x test

# ──────────────────────────────────────────────────────────────────────────────
# Code Quality
# ──────────────────────────────────────────────────────────────────────────────

# Format code
fmt:
    cargo fmt

# Lint with clippy
lint:
    cargo clippy -- -D warnings

# Run all checks (fmt + lint + test)
check: fmt lint test

# ──────────────────────────────────────────────────────────────────────────────
# GUI (Tauri + React)
# ──────────────────────────────────────────────────────────────────────────────

# Install frontend dependencies
gui-deps:
    cd frontend && pnpm install

# Start Tauri dev (frontend + backend)
gui-dev: close
    cargo tauri dev

# Build Tauri app for production
gui-build:
    cargo tauri build

# Full check: Rust workspace + frontend
[no-cd]
gui-check:
    cargo check --workspace
    cd frontend && pnpm check

# Format all (Rust + frontend)
[no-cd]
gui-fmt:
    cargo fmt
    cd frontend && pnpm fmt

# ──────────────────────────────────────────────────────────────────────────────
# Cross-Compilation
# ──────────────────────────────────────────────────────────────────────────────

# Build for all platforms
build-all: build-linux build-macos build-windows
    @echo "✓ All platforms built"

# Build Linux (x86_64 + aarch64)
build-linux:
    #!/usr/bin/env bash
    set -euo pipefail
    echo "Building Linux x86_64..."
    cargo build --release --target x86_64-unknown-linux-gnu
    if command -v cross &>/dev/null; then
        echo "Building Linux aarch64..."
        cross build --release --target aarch64-unknown-linux-gnu
    else
        echo "⚠ cross not installed, skipping aarch64 (cargo install cross)"
    fi

# Build macOS (x86_64 + arm64)
build-macos:
    #!/usr/bin/env bash
    set -euo pipefail
    [[ "$OSTYPE" == darwin* ]] || { echo "⚠ macOS only"; exit 0; }
    echo "Building macOS x86_64..."
    cargo build --release --target x86_64-apple-darwin
    echo "Building macOS arm64..."
    cargo build --release --target aarch64-apple-darwin

# Build Windows (x86_64)
build-windows:
    #!/usr/bin/env bash
    set -euo pipefail
    if command -v cross &>/dev/null; then
        echo "Building Windows x86_64..."
        cross build --release --target x86_64-pc-windows-msvc
    else
        echo "⚠ cross not installed, skipping (cargo install cross)"
    fi

# Full release: clean + build all platforms
release: clean build-all
    @echo "✓ Release builds complete"

# ──────────────────────────────────────────────────────────────────────────────
# Documentation & Install
# ──────────────────────────────────────────────────────────────────────────────

# Generate and open docs
docs:
    cargo doc --open

# Install locally
install:
    cargo install --path .

# ──────────────────────────────────────────────────────────────────────────────
# Internal Helpers
# ──────────────────────────────────────────────────────────────────────────────

# Check Tauri CLI availability
_check-tauri-cli:
    #!/usr/bin/env bash
    cargo tauri --help &>/dev/null 2>&1 || {
        echo "❌ Tauri CLI not installed. Run: cargo install tauri-cli --version '^2.0.0'"
        exit 1
    }
