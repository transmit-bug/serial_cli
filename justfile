# Serial CLI - Just Command Configuration
# https://github.com/casey/just

# Default list (run `just` to show)
default:
    @just --list

# =============================================================================
# Development Commands
# =============================================================================

# Development build (unoptimized)
dev:
    cargo build

# Start frontend dev server only (no Tauri backend)
dev-frontend:
    cd frontend && pnpm dev

# Release build (optimized)
build:
    cargo build --release

# Run application (development mode)
run *args:
    cargo run -- {{args}}

# =============================================================================
# Testing Commands
# =============================================================================

# Run all tests
test:
    cargo test

# Run tests with verbose output
test-verbose:
    cargo test -- --nocapture

# Run tests on file changes
test-watch:
    cargo watch -x test

# Run specific test
test-test name *args:
    cargo test {{name}} -- {{args}}

# =============================================================================
# Code Quality
# =============================================================================

# Run Clippy linter
lint:
    cargo clippy -- -D warnings

# Format code
fmt:
    cargo fmt

# Check code formatting
fmt-check:
    cargo fmt -- --check

# Run all checks (format + lint + test)
check: fmt-check lint test

# =============================================================================
# Clean Commands
# =============================================================================

# Clean build artifacts
clean:
    cargo clean

# Clean all (including target/)
clean-all:
    rm -rf target/

# =============================================================================
# Documentation Commands
# =============================================================================

# Generate and open documentation
docs:
    cargo doc --open

# Generate documentation
docs-build:
    cargo doc

# =============================================================================
# Cross-Compilation
# =============================================================================

# Build for all platforms
build-all: build-linux build-macos build-windows
    @echo "✓ All platforms built successfully"

# Build for Linux (x86_64 + aarch64)
build-linux:
    #!/usr/bin/env bash
    set -euo pipefail
    echo "Building Linux x86_64..."
    cargo build --release --target x86_64-unknown-linux-gnu
    echo "Building Linux aarch64..."
    if command -v cross &> /dev/null; then
        cross build --release --target aarch64-unknown-linux-gnu
    else
        echo "⚠ Warning: 'cross' not installed. Install with: cargo install cross"
        echo "Skipping aarch64 build"
    fi

# Build for macOS (x86_64 + arm64)
build-macos:
    #!/usr/bin/env bash
    set -euo pipefail
    if [[ "$OSTYPE" == "darwin"* ]]; then
        echo "Building macOS x86_64..."
        cargo build --release --target x86_64-apple-darwin
        echo "Building macOS arm64..."
        cargo build --release --target aarch64-apple-darwin
    else
        echo "⚠ macOS builds can only be performed on macOS"
        echo "Skipping macOS build"
    fi

# Build for Windows (requires cross)
build-windows:
    #!/usr/bin/env bash
    set -euo pipefail
    echo "Building Windows x86_64..."
    if command -v cross &> /dev/null; then
        cross build --release --target x86_64-pc-windows-msvc
    else
        echo "⚠ Warning: 'cross' not installed. Install with: cargo install cross"
        echo "Skipping Windows build"
    fi

# Full release build (clean + all platforms)
release: clean-all build-all
    @echo "✓ Release builds complete"

# =============================================================================
# GUI Commands (Tauri + React Frontend)
# =============================================================================

# Install all development dependencies (Rust + frontend)
install-deps:
    #!/usr/bin/env bash
    set -euo pipefail
    echo "Installing development dependencies..."

    # Check and install cross
    if ! command -v cross &> /dev/null; then
        echo "Installing cross..."
        cargo install cross
    else
        echo "✓ cross already installed"
    fi

    # Check and install Tauri CLI
    if ! cargo tauri --help &> /dev/null 2>&1; then
        echo "Installing Tauri CLI..."
        cargo install tauri-cli --version '^2.0.0'
    else
        echo "✓ Tauri CLI already installed"
    fi

    # Install frontend dependencies
    if [ -d "frontend" ] && [ -f "frontend/package.json" ]; then
        echo "Installing frontend dependencies..."
        cd frontend && pnpm install
        echo "✓ Frontend dependencies installed"
    else
        echo "⚠  Frontend directory not found, skipping pnpm install"
    fi

    echo ""
    echo "✓ All development dependencies installed"

# Check Tauri CLI availability
_check-tauri-cli:
    #!/usr/bin/env bash
    if ! cargo tauri --help &> /dev/null 2>&1; then
        echo "❌ Error: Tauri CLI not installed"
        echo "Install with: cargo install tauri-cli --version '^2.0.0'"
        echo "Or run: just install-deps"
        exit 1
    fi

# Install frontend dependencies only
gui-deps:
    cd frontend && pnpm install

# Start Tauri GUI development (launches both frontend dev server and Rust backend)
gui-dev: _check-tauri-cli
    cargo tauri dev

# Build GUI application for production
gui-build: _check-tauri-cli
    cargo tauri build

# Start frontend dev server only (without Tauri backend)
gui-dev-frontend:
    cd frontend && pnpm dev

# Build frontend only
gui-build-frontend:
    cd frontend && pnpm build

# Preview production frontend build
gui-preview:
    cd frontend && pnpm preview

# Type check frontend TypeScript
gui-type-check:
    cd frontend && pnpm type-check

# Run frontend tests
gui-test:
    cd frontend && pnpm test

# Run frontend tests in watch mode
gui-test-watch:
    cd frontend && pnpm test:watch

# Check frontend with biome (lint + format check)
gui-check-frontend:
    cd frontend && pnpm check

# Lint frontend with biome
gui-lint:
    cd frontend && pnpm lint

# Check all (Rust + frontend)
gui-check:
    cargo check --workspace
    cd frontend && pnpm check

# Format all code (Rust + frontend)
gui-fmt:
    cargo fmt
    cd frontend && pnpm fmt

# Clean GUI artifacts
gui-clean:
    rm -rf frontend/dist
    rm -rf frontend/node_modules
    rm -rf src-tauri/target

# =============================================================================
# Installation
# =============================================================================

# Install locally (development version)
install:
    cargo install --path .

# Install locally (Release version)
install-release: build
    cargo install --path .
