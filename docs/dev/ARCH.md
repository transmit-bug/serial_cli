# Architecture Reference

**Updated**: 2026-06-17
**Version**: 0.6.0
**Status**: Unified script system implemented, 237 tests passing

---

## Directory Layout

```
src/
├── main.rs                 # CLI entry point — arg parsing & command dispatch
├── lib.rs                  # Library root — re-exports Result, SerialError
├── error.rs                # SerialError enum (thiserror), Result<T>
├── error_handling.rs       # Error formatting & recovery helpers
├── config.rs               # ConfigManager — TOML-based, thread-safe
├── logging.rs              # tracing init (JSON / CLI format)
├── utils.rs                # Shared utility functions
├── service.rs              # CommandService — shared orchestration layer
│
├── cli/                    # CLI layer
│   ├── args.rs             # Cli, Commands clap definitions
│   ├── types.rs            # ScriptCommand, SniffCommand, BatchCommand,
│   │                       # ConfigCommand, VirtualCommand enums
│   ├── interactive.rs      # InteractiveShell REPL (rustyline)
│   ├── json.rs             # JsonFormatter, JsonResponse
│   ├── batch.rs            # BatchRunner, BatchConfig
│   └── commands/           # Command handlers (one file per group)
│       ├── script.rs       # script list/info/validate/load/unload/reload + run_lua_script
│       ├── sniff.rs        # sniff start/stop/stats/save
│       ├── batch.rs        # batch run/list
│       ├── config.rs       # config show/set/save/reset
│       ├── virtual_port.rs # virtual create/list/stop/stats + registry
│       ├── ports.rs        # list_ports, send_data
│       └── parsers.rs      # hex/base64 parsing utilities
│
├── serial_core/            # Serial port I/O
│   ├── port.rs             # PortManager, SerialConfig, PortHandle
│   ├── io_loop.rs          # Async I/O event loop
│   ├── sniffer.rs          # SerialSniffer, SnifferConfig
│   ├── virtual_port.rs     # VirtualSerialPair (pluggable backends)
│   ├── factory.rs          # BackendFactory — platform auto-detection
│   ├── backends/           # Virtual backend implementations
│   │   ├── mod.rs          # BackendType enum, factory registration
│   │   ├── trait.rs        # VirtualBackend trait
│   │   ├── pty.rs          # PTY backend (Unix/macOS)
│   │   ├── named_pipe.rs   # NamedPipe backend (Windows)
│   │   └── socat.rs        # Socat backend (cross-platform)
│   ├── signals.rs          # Platform signal control (DTR/RTS)
│   ├── serial_script.rs    # SerialScriptEngine (Hook mode Lua)
│   └── windows_signals.rs  # Windows-specific signal impl
│
├── script/                 # Unified script system (replaces protocol/)
│   ├── mod.rs              # ScriptInfo, module root
│   ├── manager.rs          # ScriptManager — load/unload/reload/list
│   └── built_in/           # Built-in Lua scripts
│       ├── mod.rs          # Built-in registration
│       ├── line.lua        # Line-based protocol
│       ├── at_command.lua  # AT Command protocol
│       └── modbus_rtu.lua  # Modbus RTU protocol
│
├── lua/                    # LuaJIT integration
│   ├── bindings.rs         # LuaBindings — Autonomous mode (CLI scripts)
│   ├── runtime.rs          # ScriptRuntime — unified tool function registration
│   ├── engine.rs           # LuaEngine — lightweight wrapper
│   ├── executor.rs         # ScriptEngine — script execution engine
│   └── ui_actions.rs       # UiAction — GUI script action bindings
│
├── task/                   # Task scheduling
│   ├── queue.rs            # TaskQueue
│   ├── executor.rs         # TaskExecutor
│   └── monitor.rs          # TaskMonitor
│
├── server/                 # Server Mode daemon (JSON-RPC 2.0 over Unix socket)
│   ├── mod.rs              # Module root
│   ├── rpc.rs              # JSON-RPC 2.0 dispatcher
│   ├── listener.rs         # Unix socket listener
│   ├── state.rs            # ServerState (shared state, similar to Tauri AppState)
│   └── session.rs          # Server session management (PID, socket path)
│
└── monitoring/             # System monitoring
    └── windows.rs          # Windows-specific monitoring
```

**Key changes from Unified Script System**:
- Removed `src/protocol/` directory (12 files) — replaced by `src/script/`
- Created `src/script/manager.rs` as ScriptManager for script lifecycle
- Built-in protocols rewritten in Lua (line.lua, at_command.lua, modbus_rtu.lua)
- Created `src/service.rs` as CommandService for shared orchestration
- CLI command renamed from `protocol` to `script`

---

## Key Design Patterns

| Pattern | Location | Description |
|---------|----------|-------------|
| ScriptManager | `script/manager.rs` | Load/unload/reload/list Lua scripts |
| CommandService | `service.rs` | Shared orchestration for CLI, RPC, Tauri |
| PortManager | `serial_core/port.rs` | UUID-based port handles, centralized open/close |
| SerialScriptEngine | `serial_core/serial_script.rs` | Hook mode Lua execution for port lifecycle |
| LuaBindings | `lua/bindings.rs` | Autonomous mode Lua execution for CLI scripts |
| ConfigManager | `config.rs` | `Arc<RwLock<Config>>` — thread-safe TOML config |
| Command Dispatch | `cli/args.rs` + `cli/commands/*` | Clap parse → match on Commands → handler fn |

---

## Module Dependencies

```
main.rs
  └→ cli/args (parse args)
  └→ cli/commands/* (dispatch)
       └→ service (CommandService — shared orchestration)
            ├→ serial_core (port I/O, sniffer, virtual ports)
            ├→ script/* (ScriptManager, built-in scripts)
            ├→ config (ConfigManager)
            └→ lua/* (script execution, runtime)
```

---

## Data Flow: Typical CLI Command

```
User input → Cli::parse() → match Commands
  → commands/<handler>::handle_*_command()
    → serial_core / protocol / config (business logic)
    → Result<()> propagated up
```

---

## GUI (Tauri + React)

```
src-tauri/          # Rust backend (workspace member)
  src/
    main.rs         # Tauri app entry
    commands/       # Tauri command handlers (52 commands across 11 modules)
    events/         # Event emitter system (8 event types)
    state/          # AppState, port state types
frontend/           # React 19 + TypeScript (rewritten)
```

See `docs/reference/protocols.md` for protocol documentation.
