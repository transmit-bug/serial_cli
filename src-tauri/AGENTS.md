# AGENTS.md — Tauri Backend

Tauri 2.0 GUI backend. Thin IPC layer over the core `serial-cli` library. All real logic lives in the library (`serial_cli::serial_core`, `serial_cli::script`, `serial_cli::lua`, `serial_cli::server`).

## Architecture

### Workspace Relationship

`src-tauri/` is a Cargo workspace member. The Tauri binary (`serial-cli-tauri`) depends on the CLI library via `serial-cli = { path = ".." }`. The CLI binary and GUI binary are **separate binaries** sharing one library — they never run in the same process.

### Runtime

`src/main.rs` uses `#[tokio::main]` (not Tauri's default runtime). Tauri's `Builder` is configured inside this async main. A single `AppState` is injected via `.manage()` as shared state.

### State Management

Central `AppState` (`src/state/app_state.rs`) is `#[derive(Clone)]`:

| Field | Type | Purpose |
|---|---|---|
| `port_manager` | `Arc<Mutex<PortManager>>` | Serial port lifecycle |
| `script_manager` | `Arc<Mutex<ScriptManager>>` | Script registry |
| `active_sniffers` | `Arc<Mutex<HashMap<String, DataSniffer>>>` | Per-port background read tasks |
| `port_stats` | `Arc<Mutex<HashMap<String, Arc<PortStatsTracker>>>>` | Per-port counters (survives sniffer stop) |
| `virtual_port_registry` | `Arc<RwLock<HashMap<String, VirtualSerialPair>>>` | Virtual port pairs |
| `embedded_server` | `Arc<Mutex<Option<RunningEmbeddedServer>>>` | JSON-RPC server state |

**Locking discipline:** `port_manager.lock()` → get port handle → `port_handle.lock()` → I/O → drop locks. The sniffer pattern avoids holding `port_manager` during reads by cloning the `Arc<PortHandle>` once.

**Implicit lock order:** `port_manager` → `port_handle` → (release both) → `port_stats` → emit event. Violating this can deadlock.

### Sniffer Architecture

Two-task design per port (`src/commands/serial.rs::start_sniffing`):
1. **Blocking read task** (`spawn_blocking`): `port_handle.blocking_lock()` → read → `data_tx.blocking_send()`. Releases lock after each read so writes can interleave.
2. **Async event loop** (`tokio::spawn`): Receives from `mpsc::channel::<Vec<u8>>(256)` → updates stats → emits Tauri events.

Empty `Vec<u8>` is a **disconnect sentinel** from read task to event loop. `Arc<AtomicBool>` stop flag for coordination.

### Event System

`src/events/emitter.rs` uses Tauri 2.0 `Emitter` trait. Events are fire-and-forget — errors logged but never propagated.

**Key events:** `data-received`, `data-sent`, `port-status-changed`, `ports-changed`, `virtual-port-created`, `virtual-port-stopped`, `server-status-changed`, `error-occurred`. All carry a `timestamp` field (Unix millis).

### Embedded Server Shares State

When `start_server` is called, it creates a `ServerState` cloning the same `Arc<Mutex<PortManager>>` and `Arc<Mutex<ScriptManager>>` from `AppState`. A port opened via JSON-RPC is visible in the GUI and vice versa.

### Port Hot-Plug Monitor

Background `tokio::spawn` task polls hardware ports every 2 seconds, diffs against known ports, emits `ports-changed` with `added`/`removed` lists. Filters out `debug-console`, `pty.`, `ttys` names.

## Command Patterns

### Naming & Organization

One file per domain in `src/commands/`: `port.rs`, `serial.rs`, `script.rs`, `config.rs`, `virtual_port.rs`, `server.rs`, `export.rs`, `window.rs`, `serial_script.rs`, `script_ui_actions.rs`. All registered in `main.rs` via `tauri::generate_handler![...]`.

### Error Handling

**Every command returns `Result<T, String>`** — errors converted via `.map_err(|e| e.to_string())`. No custom error types cross the IPC boundary.

### State Access

```rust
#[tauri::command]
pub async fn some_command(
    param: String,
    app: tauri::AppHandle,        // When emitting events
    state: State<'_, AppState>,
) -> Result<SomeResponse, String> {
    let manager = state.port_manager.lock().await;
    // ... work ...
    drop(manager);  // Explicit drop before emitting events
    app.emit("event-name", payload)?;
    Ok(response)
}
```

### DTO Pattern

Commands define local `#[derive(Serialize)]` structs (`PortInfo`, `PortStatus`, `VirtualPortStats`, etc.) that mirror but are distinct from core library types. Serde `rename` attributes convert snake_case → camelCase for frontend.

## Gotchas

- **Window close is intercepted:** `on_window_event` calls `api.prevent_close()`, spawns async cleanup (stop server → stop sniffers → close ports → stop virtual ports → `exit(0)`). **Do not** remove this or the app leaks serial ports.
- **`spawn_blocking` required for serial I/O:** The `serialport` crate is synchronous. **Do not** call blocking serial I/O from async context without `spawn_blocking`. Use `port_handle.blocking_lock()` inside these tasks.
- **`virtual_port_registry` uses `RwLock`:** Reads (list, stats) are far more frequent than writes (create, stop). Other state uses `Mutex`.
- **`window.rs` commands are sync:** Unlike all other commands, they take `Window` (not `AppHandle`) and are not async.
- **Scripts directory security:** `load_script` validates canonical path starts with scripts directory — path traversal is blocked.
- **Stats tracking survives sniffer stop:** `port_stats` is separate from `active_sniffers`. When sniffer stops, `PortStatsTracker` Arc remains for history.
- **Config is read from disk on every `get_config`:** No in-memory cache. `update_config` does read-modify-write.
