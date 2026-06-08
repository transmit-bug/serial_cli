# AGENTS.md

This file provides guidance to Codex (Codex.ai/code) when working with code in this repository.

## Project Overview

Serial CLI is a Rust-based serial port communication tool with embedded LuaJIT scripting, optimized for AI/automation workflows. It supports multiple protocols (Modbus RTU/ASCII, AT Commands, line-based, and custom Lua protocols) with structured JSON output.

Includes a Tauri-based GUI application (`src-tauri/` + `frontend/`).

## Build & Development Commands

```bash
# Development build
just dev          # cargo build

# Release build
just build        # cargo build --release

# Run tests
just test         # cargo test
just test-verbose # cargo test -- --nocapture

# Code quality
just check        # fmt-check + lint + test (all checks)
just fmt          # cargo fmt
just lint         # cargo clippy -- -D warnings

# Run application
just run <args>   # cargo run -- <args>

# Cross-compilation
just build-all    # Linux + macOS + Windows
just build-linux  # x86_64 + aarch64
just build-macos  # x86_64 + arm64
```

**Requirements:** Rust 1.75+, just task runner

## Architecture

See [`docs/dev/ARCH.md`](docs/dev/ARCH.md) for full directory layout, design patterns, module dependencies, and data flow.

For complete documentation structure, see [`docs/README.md`](docs/README.md).

**Quick reference:**

- `src/main.rs` — thin entry point (~73 lines), dispatches to `cli/commands/*`
- `src/cli/args.rs` — clap definitions (Cli, Commands)
- `src/cli/types.rs` — command enum definitions (all subcommands)
- `src/cli/commands/` — one handler file per command group
- `src/serial_core/` — port I/O, sniffer, virtual ports
- `src/protocol/` — protocol engine with Lua extensibility
- `src/lua/` — LuaJIT integration (bindings, stdlib, executor)
- `src/config.rs` — TOML-based ConfigManager

## Key Conventions

- **Error handling**: Use `Result<T>` from `error.rs`
- **Async**: All I/O uses `tokio`
- **Lua integration**: Scripts executed via LuaEngine
- **Configuration**: TOML-based with fallback defaults
- **Documentation**: Follow strict documentation hierarchy
  - **Root level only**: README.md, CHANGELOG.md, RELEASE.md (essential user-facing docs only)
  - **docs/guides/** - End-user documentation (guides, tutorials, feature explanations)
  - **docs/dev/** - Internal development docs (architecture, design decisions, technical specs)
  - **docs/reference/** - Reference material (configuration, protocols, API docs, troubleshooting)
  - **docs/commands/** - Per-command documentation (auto-generated or command-specific guides)
  - **Documentation constraints**:
    - NEVER create root-level docs/*.md files (except this AGENTS.md)
    - Avoid duplication - check existing docs before creating new ones
    - Prefer updating existing docs over creating new files
    - Design decision docs go in docs/dev/ with DECISION.md suffix
    - User-facing content goes in docs/guides/ or docs/reference/, never in docs/dev/
    - Before creating any new .md file, verify it doesn't exist in a related location
- **TODO tracking**: 发现或修复问题后，同步更新 `TODO.md` 中的待办/已完成列表。

## GUI Subproject

Tauri 2.0 GUI in `src-tauri/` (workspace member) with React 19 + TypeScript frontend in `frontend/`:

```bash
just gui-deps           # Install frontend dependencies (pnpm)
just gui-dev            # Start Tauri dev server (frontend + backend)
just gui-build          # Build GUI application for production
just gui-check          # Check all (Rust workspace + frontend biome)
just gui-check-frontend # Check frontend with biome only
just gui-type-check     # TypeScript type check
just gui-fmt            # Format all code (cargo fmt + biome)
just gui-lint           # Lint frontend with biome
just gui-test           # Run frontend tests (vitest)
just gui-test-watch     # Run frontend tests in watch mode
```

**Frontend stack:** pnpm, React 19, Vite 8, Tailwind CSS 4, Zustand 5, biome, vitest
