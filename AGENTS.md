# AGENTS.md

This file provides guidance to Codex (Codex.ai/code) when working with code in this repository.

## Project Overview

Serial CLI is a Rust-based serial port communication tool with embedded LuaJIT scripting, optimized for AI/automation workflows. It supports multiple protocols (Modbus RTU/ASCII, AT Commands, line-based, and custom Lua protocols) with structured JSON output.

Includes a Tauri-based GUI application (`src-tauri/` + `frontend/`).

## Build & Development Commands

```bash
# Development
just dev          # cargo build (debug)
just build        # cargo build --release
just run <args>   # cargo run -- <args>
just watch        # auto-rebuild on file changes
just clean        # cargo clean
just close        # kill dev processes, free ports

# Testing
just test         # cargo test
just test-one <name>  # run specific test
just test-watch   # auto-run tests on file changes

# Code quality
just check        # fmt + lint + test (all checks)
just fmt          # cargo fmt
just lint         # cargo clippy -- -D warnings

# Cross-compilation
just build-all    # Linux + macOS + Windows
just build-linux  # x86_64 + aarch64
just build-macos  # x86_64 + arm64
just release      # clean + build all platforms
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
- `src/script/` — unified script system (ScriptManager, built-in Lua scripts)
- `src/lua/` — LuaJIT integration (bindings, stdlib, executor)
- `src/config.rs` — TOML-based ConfigManager

## Key Conventions

- **Error handling**: Use `Result<T>` from `error.rs`
- **Async**: All I/O uses `tokio`
- **Lua integration**: Scripts executed via LuaEngine
- **Configuration**: TOML-based with fallback defaults
- **Script imports**: Scripts in `scripts/protocols/` can `require()` each other; library scripts should `return ModuleTable`
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
just gui-deps    # Install frontend dependencies (pnpm)
just gui-dev     # Start Tauri dev server (frontend + backend)
just gui-build   # Build GUI application for production
just gui-check   # Check all (Rust workspace + frontend biome)
just gui-fmt     # Format all code (cargo fmt + biome)
```

For frontend-only tasks, run directly in `frontend/`:
```bash
cd frontend && pnpm test        # run tests (vitest)
cd frontend && pnpm lint        # lint with biome
cd frontend && pnpm type-check  # TypeScript type check
```

**Frontend stack:** pnpm, React 19, Vite 8, Tailwind CSS 4, Zustand 5, shadcn/ui, biome, vitest

**Frontend 规范**：UI 组件优先使用 shadcn/ui（`npx shadcn@latest add <组件名>`），不要手写基础组件。

## Agent skills

### Issue tracker

GitHub Issues (via `gh` CLI). See `docs/agents/issue-tracker.md`.

### Triage labels

Default five-role vocabulary (`needs-triage`, `needs-info`, `ready-for-agent`, `ready-for-human`, `wontfix`). See `docs/agents/triage-labels.md`.

### Domain docs

Single-context layout (root `CONTEXT.md` + `docs/adr/`). See `docs/agents/domain.md`.
