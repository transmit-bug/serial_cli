# Tauri Backend Investigation (`src-tauri/`)

## 1. Architecture & Patterns

### Workspace Relationship
- `src-tauri/` is a **Cargo workspace member** of the root `serial-cli` crate (root `Cargo.toml` line: `members = ["src-tauri"]`).
- The Tauri binary (`serial-cli-tauri`) depends on the CLI library via `serial-cli = { path = ".." }` — it imports core types from `serial_cli::serial_core`, `serial_cli::script`, `serial_cli::lua`, `serial_cli::config`, `serial_cli::server`, and `serial_cli::state_factory`.
- The CLI binary (`serial-cli`) and the GUI binary (`serial-cli-tauri`) are **separate binaries** sharing one library. They never run in the same process.

### Entry Point & Runtime
- `src/main.rs` uses `#[tokio::main]` (not Tauri's default runtime) — the entire app runs on a single Tokio runtime.
- Tauri's `Builder` is configured inside this async main, with `.manage(app_state)` injecting a single `AppState` as shared state.
- **All** `#[tauri::command]` functions are `async` (except `window.rs` which is sync for simple window ops).

### State Management (`src/state/app_state.rs`)
The central `AppState` struct is `#[derive(Clone)]` and contains:

| Field | Type | Purpose |
|---|---|---|
| `port_manager` | `Arc<Mutex<PortManager>>` | Core serial port lifecycle (from `CoreManagers`) |
| `script_manager` | `Arc<Mutex<ScriptManager>>` | Unified script registry (from `CoreManagers`) |
| `active_sniffers` | `Arc<Mutex<HashMap<String, DataSniffer>>>` | Per-port background read tasks |
| `port_stats` | `Arc<Mutex<HashMap<String, Arc<PortStatsTracker>>>>` | Per-port byte/packet counters (survives sniffer stop) |
| `virtual_port_registry` | `Arc<RwLock<HashMap<String, VirtualSerialPair>>>` | Active virtual port pairs |
| `embedded_server` | `Arc<Mutex<Option<RunningEmbeddedServer>>>` | Optional JSON-RPC server state |
| `scripts_dir` | `Option<PathBuf>` | User script storage directory |

**Key pattern**: `CoreManagers` (from `src/state_factory.rs` in the library) creates the `PortManager` and `ScriptManager` as `Arc<Mutex<...>>`. The Tauri `AppState` clones these Arcs — the same pattern is used by the embedded server (`serial_cli::server::ServerState`) so the GUI and server share the exact same port/script managers.

**Locking discipline**: Commands acquire `port_manager.lock()` → get a port handle → `port_handle.lock()` → do I/O → drop locks. The `DataSniffer` pattern avoids holding `port_manager` during reads by cloning the `Arc<PortHandle>` once.

### Sniffer Architecture (`src/commands/serial.rs::start_sniffing`)
Two-task design per port:
1. **Blocking read task** (`spawn_blocking`): Calls `port_handle.blocking_lock()` → reads → `data_tx.blocking_send()`. Releases lock after each read so writes can interleave.
2. **Async event loop** (`tokio::spawn`): Receives from `mpsc` channel → updates `PortStatsTracker` atomics → emits Tauri events via `app.emit()`.

Communication: `tokio::sync::mpsc::channel::<Vec<u8>>(256)` + `Arc<AtomicBool>` stop flag.
Empty `Vec<u8>` is a **disconnect sentinel** from the read task to the event loop.

### Event System (`src/events/emitter.rs`)
- Uses Tauri 2.0 `Emitter` trait: `app.emit("event-name", payload)`.
- All events carry a `timestamp` field (Unix millis).
- Events are fire-and-forget — errors are logged but never propagated to callers.
- `setup_event_system()` calls `app.listen()` for debugging, but the real consumers are frontend `listen()` calls.

**Event catalog** (emitted from Rust → consumed by frontend):
| Event | Payload | Trigger |
|---|---|---|
| `data-received` | `{port_id, data: Vec<u8>, timestamp, direction: "rx"}` | Sniffer read |
| `data-sent` | `{port_id, data: Vec<u8>, timestamp, direction: "tx"}` | `send_data` command |
| `port-status-changed` | `{port_id, status, timestamp}` | open/close |
| `ports-changed` | `{added: Vec<String>, removed: Vec<String>, timestamp}` | Hot-plug monitor (2s poll) |
| `virtual-port-created` | `{port_id, port_info, timestamp}` | `create_virtual_port` |
| `virtual-port-stopped` | `{port_id, timestamp}` | `stop_virtual_port` |
| `virtual-port-stats-updated` | `{port_id, stats, timestamp}` | `get_virtual_port_stats` |
| `server-status-changed` | `{running, socket_path, timestamp}` | `start_server`/`stop_server` |
| `error-occurred` | `{error, timestamp}` | Disconnects, fatal errors |

### Bridge to Core Library
The Tauri backend is a thin IPC layer. All real logic lives in the library:
- `serial_cli::serial_core::PortManager` — port open/close/list/read/write
- `serial_cli::serial_core::VirtualSerialPair` — virtual port pairs
- `serial_cli::script::ScriptManager` — script lifecycle
- `serial_cli::lua::LuaBindings` — standalone Lua execution
- `serial_cli::serial_core::serial_script::SerialScriptEngine` — port-attached scripts
- `serial_cli::server::listener::run_socket_server` — embedded JSON-RPC server
- `serial_cli::config` — TOML config load/save

### Port Hot-Plug Monitor (`src/main.rs::spawn_port_monitor`)
A background `tokio::spawn` task polls hardware ports every 2 seconds, diffs against a `HashSet<String>` of known port names, and emits `ports-changed` events with `added`/`removed` lists. Filters out `debug-console`, `pty.`, `ttys` port names in `list_ports`.

## 2. Key Conventions

### Command Naming
- **snake_case** function names matching the frontend's `invoke()` calls.
- Module organization: one file per domain (`port.rs`, `serial.rs`, `script.rs`, `config.rs`, `virtual_port.rs`, `server.rs`, `export.rs`, `window.rs`, `serial_script.rs`, `script_ui_actions.rs`).
- All commands registered in `main.rs` via `tauri::generate_handler![...]` — flat list, ~50 commands.

### Error Handling Pattern
**Every command returns `Result<T, String>`** — errors are converted to strings via `.map_err(|e| e.to_string())` or `.map_err(|e| format!("...", e))`. No custom error types cross the IPC boundary. This is universal — no command returns a `SerialError` directly.

### State Access Pattern
```rust
#[tauri::command]
pub async fn some_command(
    param: String,                    // Frontend args
    app: tauri::AppHandle,           // When emitting events
    state: State<'_, AppState>,      // Shared state
) -> Result<SomeResponse, String> {
    let manager = state.port_manager.lock().await;
    // ... do work ...
    drop(manager);  // Explicit drop before emitting events
    // ... emit events ...
    Ok(response)
}
```

### DTO Pattern
Commands define local `#[derive(Serialize)]` structs (e.g., `PortInfo`, `PortStatus`, `VirtualPortStats`, `ServerStatus`, `ConfigData`) that mirror but are distinct from core library types. This decouples the IPC API from internal representations. Serde `rename` attributes convert snake_case → camelCase for the frontend (e.g., `#[serde(rename = "defaultBaudrate")]`).

### Config Commands
Config is read/written directly via `serial_cli::config::load_config()` and `toml::to_string_pretty()` — no in-memory config cache. Each `get_config` call re-reads from disk. Each `update_config` does read-modify-write.

## 3. Non-Obvious Rules & Pitfalls

### Window Close is Intercepted
`on_window_event` calls `api.prevent_close()` on `CloseRequested`, then spawns an async cleanup task (`shutdown()`) that: stops embedded server → stops all sniffers → closes all ports → stops virtual ports → calls `app_handle.exit(0)`. **Do not** remove this or the app will leak serial ports.

### Lock Ordering
The implicit lock order is: `port_manager` → `port_handle` → (release both) → `port_stats` → emit event. Violating this can deadlock. The sniffer pattern specifically avoids holding `port_manager` during the read loop.

### `spawn_blocking` for Serial I/O
The underlying `serialport` crate is synchronous. All serial reads happen in `spawn_blocking` tasks. The `port_handle.blocking_lock()` is used inside these tasks (not `.lock().await`). **Do not** call blocking serial I/O from an async context without `spawn_blocking`.

### Stats Tracking Survives Sniffer Stop
`port_stats` is a separate HashMap from `active_sniffers`. When a sniffer stops, its `PortStatsTracker` Arc remains in `port_stats` so history is preserved. The sniffer's `stats` field is `#[allow(dead_code)]` — it's kept for reference but the canonical stats are in `AppState::port_stats`.

### Virtual Port Registry Uses `RwLock`, Not `Mutex`
`virtual_port_registry` uses `Arc<RwLock<...>>` because reads (list, stats, health check) are far more frequent than writes (create, stop). Other state uses `Mutex`.

### Embedded Server Shares State
When `start_server` is called, it creates a `ServerState` that clones the same `Arc<Mutex<PortManager>>` and `Arc<Mutex<ScriptManager>>` from `AppState`. The server and GUI operate on the **exact same** port/script managers. This means a port opened via the JSON-RPC server is visible in the GUI and vice versa.

### `window.rs` Commands are Sync
Unlike all other commands, `show_window`/`hide_window`/`toggle_window` are synchronous (not async). They take `Window` (not `AppHandle`) and operate directly on the window.

### Scripts Directory Security
`load_script` validates that the canonical path starts with the canonical scripts directory — path traversal is blocked.

## 4. Build & Dev

### Cargo Workspace
```toml
[workspace]
members = ["src-tauri"]
resolver = "2"
```
The Tauri crate is `serial-cli-tauri` (distinct from the library crate `serial-cli`).

### Tauri-Specific Config
- `tauri.conf.json`: Tauri 2.0 schema, identifier `com.serial-cli.gui`
- Frontend: `devUrl: http://localhost:1420`, `frontendDist: ../frontend/dist`
- `beforeDevCommand: cd frontend && pnpm dev`
- `beforeBuildCommand: cd frontend && pnpm build`
- `withGlobalTauri: true` — exposes `window.__TAURI__` globally (no explicit import needed)
- Plugins: `tauri-plugin-shell`, `tauri-plugin-dialog`
- Capabilities: single `default.json` granting event listen, path, window, app, dialog, shell permissions

### Build Differences from CLI
- `just gui-dev` runs `cargo tauri dev` (requires `tauri-cli` installed)
- `just gui-build` runs `cargo tauri build` (produces platform bundles)
- `just gui-check` runs `cargo check` + `biome` lint on frontend
- The `custom-protocol` feature is default-enabled for production builds
- `build.rs` just calls `tauri_build::build()` (generates TypeScript bindings for commands)

### Logging
Dual-layer tracing: file (`info` level) + stderr (`warn` level). Log file at `{data_local_dir}/serial-cli/logs/serial-cli.log`. The `log` crate is also used in some command files (via `log::{debug, info, error}`) — this goes through the same tracing subscriber.

## 5. Project Structure

```
src-tauri/
├── build.rs                    # tauri_build::build() — generates command IPC bindings
├── Cargo.toml                  # Crate: serial-cli-tauri
├── tauri.conf.json             # Tauri 2.0 app config (window, security, bundle)
├── capabilities/
│   └── default.json            # Permission allowlist (event, path, window, dialog, shell)
├── icons/                      # App icons (multi-platform)
├── gen/                        # Auto-generated Tauri bindings (gitignored)
├── docs/                       # (empty — no Tauri-specific docs yet)
└── src/
    ├── main.rs                 # Entry point: tokio runtime, Tauri builder, shutdown, port monitor
    ├── state/
    │   ├── mod.rs
    │   ├── app_state.rs        # AppState, DataSniffer, PortStatsTracker, RunningEmbeddedServer
    │   └── port_state.rs       # DTOs: PortStatus, SerialConfig, PortStats (serde types)
    ├── commands/
    │   ├── mod.rs
    │   ├── port.rs             # list_ports, open_port, close_port, get_port_status, health checks
    │   ├── serial.rs           # send_data, read_data, start_sniffing, stop_sniffing
    │   ├── serial_script.rs    # attach/detach/has_script, script status, UI actions on ports
    │   ├── script.rs           # Script lifecycle (load/unload/reload), validation, encode/decode, hot reload
    │   ├── script_ui_actions.rs # Standalone script UI action listing/execution
    │   ├── config.rs           # get/update/reset config, connection presets, log reading
    │   ├── export.rs           # Export packet data (txt/csv/json)
    │   ├── virtual_port.rs     # Create/list/stop virtual ports, stats, captured packets
    │   ├── server.rs           # Start/stop/status for embedded JSON-RPC server
    │   └── window.rs           # show/hide/toggle window (sync commands)
    └── events/
        ├── mod.rs
        └── emitter.rs          # Event emission functions + setup_event_system
```

## Start Here

Open `src-tauri/src/state/app_state.rs` first — it defines `AppState` which is the single source of truth for all shared state. Every command file depends on it. Then read `src-tauri/src/main.rs` to understand the runtime setup, event system initialization, port monitor, and shutdown sequence. After that, `src-tauri/src/commands/serial.rs` demonstrates the most complex pattern (sniffer with dual tasks, channel-based data flow, stats tracking).
