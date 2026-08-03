# AGENTS.md

Serial CLI is a Rust-based serial port communication tool with embedded LuaJIT scripting, optimized for AI/automation workflows. Supports multiple protocols (Modbus RTU/ASCII, AT Commands, line-based, custom Lua) with structured JSON output.

## Binary Architecture

Three targets share one library:

| Binary | Crate | Purpose |
|---|---|---|
| `serial-cli` | root | CLI tool — arg parsing, interactive REPL, batch execution |
| `serial-cli-tauri` | `src-tauri/` | GUI app — Tauri 2.0 IPC layer over the same library |
| (library) | `serial-cli` | Core logic: port I/O, script engine, config, server |

**Boundary:** The Tauri backend is a thin IPC layer. All real logic lives in the library (`serial_cli::serial_core`, `serial_cli::script`, `serial_cli::lua`, `serial_cli::server`). The CLI and GUI never share a process.

Sub-project details: `src-tauri/AGENTS.md`, `frontend/AGENTS.md`.

## Build Commands

```bash
just dev          # cargo build (debug)
just build        # cargo build --release
just test         # cargo test
just check        # fmt + lint + test
just gui-dev      # Tauri dev server (frontend + backend)
just gui-build    # Build GUI for production
```

Full list: `just --list`. Requirements: Rust 1.75+, just, LuaJIT dev libraries.

## Key Patterns

### Script System

Everything goes through `ScriptManager` (`src/script/`). It manages script lifecycle (load/unload/reload), protocol dispatch, and Lua execution. **Do not** bypass it to call `LuaEngine` directly from command handlers.

Scripts in `scripts/protocols/` can `require()` each other; library scripts should `return ModuleTable`.

### Command Flow

```
CLI args / Tauri invoke / JSON-RPC
    → PortManager / ScriptManager / LuaEngine
```

CLI, GUI, and embedded server call managers directly — no shared orchestration layer.

### Error Handling

`Result<T>` from `error.rs` (thiserror-based `SerialError`). Tauri commands convert to `Result<T, String>` at the IPC boundary — no custom error types cross it.

### Async

All I/O uses `tokio`. Serial reads use `spawn_blocking` because the `serialport` crate is synchronous.

## Conventions

- **Configuration:** TOML-based (`src/config.rs`), read from disk on each access (no in-memory cache)
- **Logging:** `tracing` crate, dual output (file `info` + stderr `warn`)
- **JSON output:** Structured via `src/cli/json.rs`, fields use camelCase
- **Cross-platform:** Serial backends differ per OS — use `cfg` conditional compilation, not runtime detection
- **Documentation:** NEVER create root-level `docs/*.md` files. User docs → `docs/guides/`, dev docs → `docs/dev/`, reference → `docs/reference/`
- **TODO tracking:** `TODO.md` is for high-level strategic directions only. All task planning and tracking goes through GitHub Issues. Completed items are deleted, not archived.

## Agent Skills

### Issue tracker
GitHub Issues (via `gh` CLI). See `docs/agents/issue-tracker.md`.

### Triage labels
Default five-role vocabulary (`needs-triage`, `needs-info`, `ready-for-agent`, `ready-for-human`, `wontfix`). See `docs/agents/triage-labels.md`.

### Domain docs
Single-context layout (root `CONTEXT.md` + `docs/adr/`). See `docs/agents/domain.md`.
