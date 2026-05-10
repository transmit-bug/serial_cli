# Architecture Reference

**Updated**: 2026-05-10
**Version**: 0.5.0-dev
**Status**: All core modules implemented and tested (212 tests passing)

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
│
├── cli/                    # CLI layer
│   ├── args.rs             # Cli, Commands clap definitions
│   ├── types.rs            # ProtocolCommand, SniffCommand, BatchCommand,
│   │                       # ConfigCommand, VirtualCommand enums
│   ├── interactive.rs      # InteractiveShell REPL (rustyline)
│   ├── json.rs             # JsonFormatter, JsonResponse
│   ├── batch.rs            # BatchRunner, BatchConfig
│   └── commands/           # Command handlers (one file per group)
│       ├── protocol.rs     # protocol list/info/validate/load/unload/reload
│       ├── sniff.rs        # sniff start/stop/stats/save
│       ├── batch.rs        # batch run/list
│       ├── config.rs       # config show/set/save/reset
│       ├── virtual_port.rs # virtual create/list/stop/stats + registry
│       ├── ports.rs        # list_ports, send_data
│       ├── script.rs       # run_lua_script
│       └── parsers.rs      # hex/base64 parsing utilities
│
├── serial_core/            # Serial port I/O
│   ├── port.rs             # PortManager, SerialConfig, PortHandle
│   ├── io_loop.rs          # Async I/O event loop
│   ├── sniffer.rs          # SerialSniffer, SnifferConfig
│   ├── virtual_port.rs     # VirtualSerialPair (PTY backend)
│   ├── signals.rs          # Platform signal control (DTR/RTS)
│   └── windows_signals.rs  # Windows-specific signal impl
│
├── protocol/               # Protocol engine
│   ├── mod.rs              # Protocol trait definition
│   ├── registry.rs         # ProtocolRegistry, ProtocolFactory
│   ├── registration.rs     # Built-in protocol registration
│   ├── manager.rs          # ProtocolManager — load/unload/reload
│   ├── loader.rs           # ProtocolLoader — Lua script loading
│   ├── validator.rs        # ProtocolValidator — script validation
│   ├── watcher.rs          # ProtocolWatcher — hot-reload via notify
│   ├── lua_ext.rs          # Lua protocol extension
│   └── built_in/           # Modbus RTU/ASCII, AT Command, Line
│
├── lua/                    # LuaJIT integration
│   ├── bindings.rs         # LuaBindings — Rust→Lua API
│   ├── engine.rs           # LuaEngine runtime
│   ├── executor.rs         # ScriptEngine — file/args execution
│   ├── stdlib.rs           # LuaStdLib — hex/time/data utilities
│   ├── cache.rs            # ScriptCache
│   └── pool.rs             # LuaPool — instance pooling
│
├── task/                   # Task scheduling
│   ├── queue.rs            # TaskQueue
│   ├── executor.rs         # TaskExecutor
│   └── monitor.rs          # TaskMonitor
│
└── monitoring/             # System monitoring
    └── windows.rs          # Windows-specific monitoring
```

---

## Key Design Patterns

| Pattern | Location | Description |
|---------|----------|-------------|
| Protocol Trait | `protocol/mod.rs` | `parse()`, `encode()`, `reset()` — all protocols implement this |
| PortManager | `serial_core/port.rs` | UUID-based port handles, centralized open/close |
| ProtocolFactory | `protocol/registry.rs` | Factory pattern for protocol instantiation |
| LuaBindings | `lua/bindings.rs` | Registers Rust APIs into Lua globals |
| ConfigManager | `config.rs` | `Arc<RwLock<Config>>` — thread-safe TOML config |
| Command Dispatch | `cli/args.rs` + `cli/commands/*` | Clap parse → match on Commands → handler fn |

---

## Module Dependencies

```
main.rs
  └→ cli/args (parse args)
  └→ cli/commands/* (dispatch)
       ├→ serial_core (port I/O, sniffer, virtual ports)
       ├→ protocol/* (registry, built-in, Lua protocols)
       ├→ config (ConfigManager)
       └→ lua/* (script execution, stdlib)
            └→ protocol/* (custom Lua protocols)
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
    commands/       # Tauri command handlers
    config.rs       # GUI-specific config
    ...
frontend/           # React + TypeScript
  src/
    components/     # UI components (port panel, data view, etc.)
    contexts/       # React contexts (SerialPort, VirtualPort, Settings)
    hooks/          # Custom hooks
    ...
```

See `docs/features/VIRTUAL_SERIAL.md` for virtual port architecture details.
