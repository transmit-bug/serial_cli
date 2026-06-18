# Unified Script System — Design Decision

**Date**: 2026-06-15
**Status**: Accepted (Implemented)
**Supersedes**: Current Protocol + Hook Script separation

---

## Context

Serial CLI had three Lua execution modes that overlap in purpose:

| Mode | Module | Callbacks | Purpose |
|------|--------|-----------|---------|
| Protocol | `protocol/lua_ext.rs` → `LuaProtocol` | `on_frame`, `on_encode` | Frame encoding/decoding |
| Hook Script | `serial_core/serial_script.rs` → `SerialScriptEngine` | `on_open`, `on_send`, `on_recv`, `on_timer`, `on_close` | Port lifecycle control |
| Autonomous | `lua/bindings.rs` → `LuaBindings` | (free-form) | One-shot CLI script execution |

Protocol and Hook Script served the same fundamental purpose — "attach behavior to a port" — but were split across two module hierarchies with separate lifecycle management. Users had to understand the distinction to use the tool correctly. Built-in protocols (Modbus, AT Command, Line) were Rust implementations of the `Protocol` trait, bypassing the Lua engine entirely.

Additionally, the three command surfaces (CLI, JSON-RPC, Tauri) duplicated orchestration logic for port, protocol, and script operations.

## Decision

### 1. Merge Protocol and Hook Script into a Unified Script System

A **Script** is a Lua program that defines any combination of lifecycle callbacks:

```lua
-- Unified callback model
function on_open(port)     -- port opened
function on_send(data)     -- before sending (modify/block)
function on_recv(data)     -- on receiving (modify/block/auto-reply)
function on_timer()        -- periodic timer
function on_close()        -- port closing
```

Frame encoding/decoding (the former Protocol role) is implemented through `on_send`/`on_recv`:

```lua
-- Example: Modbus RTU protocol as a Script
function on_send(data)
    local frame = add_slave_id(data)
    frame = add_crc(frame)
    return frame
end

function on_recv(data)
    local frame = buffer_accumulate(data)
    if not check_crc(frame) then return nil end
    return strip_headers(frame)
end
```

The `Protocol` trait, `ProtocolManager`, `ProtocolRegistry`, `ProtocolLoader`, `ProtocolValidator`, and `ProtocolWatcher` are removed. The `protocol/` directory is replaced by `script/`.

### 2. Rewrite Built-in Protocols in Lua

All built-in protocols become Lua scripts bundled with the binary:

| Current (Rust) | New (Lua) | Lines (approx) |
|----------------|-----------|-----------------|
| `ModbusProtocol` | `built_in/modbus_rtu.lua` | ~120 |
| `AtCommandProtocol` | `built_in/at_command.lua` | ~50 |
| `LineProtocol` | `built_in/line.lua` | ~30 |

Embedded at compile time via `include_str!()`. Users can read, learn from, and modify copies.

**Trade-off**: Lua frame parsing is slower than Rust, but serial I/O (not parsing) is the bottleneck at typical baud rates.

### 3. Unified ScriptManager

One deep module replaces the five protocol-lifecycle modules:

```rust
// src/script/manager.rs
pub struct ScriptManager { /* ... */ }

impl ScriptManager {
    pub async fn load(&mut self, path: &Path) -> Result<ScriptInfo>;
    pub async fn unload(&mut self, name: &str) -> Result<()>;
    pub async fn reload(&mut self, name: &str) -> Result<()>;
    pub async fn list(&self) -> Vec<ScriptInfo>;
    pub async fn get(&self, name: &str) -> Result<SerialScriptEngine>;
}
```

**Module structure**:

```
src/script/
├── mod.rs              # Re-exports
├── manager.rs          # ScriptManager (load/unload/reload/list/get)
└── built_in/
    ├── mod.rs          # Built-in registration
    ├── modbus_rtu.lua  # Embedded Lua
    ├── at_command.lua  # Embedded Lua
    └── line.lua        # Embedded Lua
```

**Removed modules** (replaced by `script/`):
- `src/protocol/mod.rs`
- `src/protocol/manager.rs`
- `src/protocol/registry.rs`
- `src/protocol/loader.rs`
- `src/protocol/validator.rs`
- `src/protocol/watcher.rs`
- `src/protocol/lua_ext.rs`
- `src/protocol/registration.rs`
- `src/protocol/built_in/` (entire directory)

**Retained modules** (unchanged or lightly modified):
- `src/serial_core/serial_script.rs` — `SerialScriptEngine` kept as the unified runtime
- `src/lua/bindings.rs` — `LuaBindings` kept for autonomous mode
- `src/lua/runtime.rs` — `ScriptRuntime` kept for tool function registration

### 4. Unified Port Attachment

`set_protocol` is removed. One method attaches any script to a port:

```rust
// PortManager
pub async fn attach_script(&self, port_id: &str, script_name: &str) -> Result<()>;
```

Internally: looks up script in `ScriptManager` → creates `SerialScriptEngine` → attaches to port handle.

### 5. CommandService Layer

A shared service layer extracts common orchestration from the three command surfaces:

```
┌─────────────────────────────────────────────┐
│  Surface Adapters (thin)                     │
│  CLI: parse args + format output             │
│  RPC: parse JSON-RPC + manage connections    │
│  Tauri: parse params + emit events           │
├─────────────────────────────────────────────┤
│  CommandService (medium)                     │
│  Shared operations:                          │
│  - port_open / port_close / port_send        │
│  - script_load / script_unload / attach      │
│  - virtual_create / virtual_stop             │
│  - sniff_start / sniff_stop                  │
├─────────────────────────────────────────────┤
│  Core Modules (deep)                         │
│  - PortManager                               │
│  - ScriptManager (new)                       │
│  - VirtualSerialPair                         │
└─────────────────────────────────────────────┘
```

**Surface-specific logic stays in adapters**:
- CLI daemon mode for sniffing (PID management, session files)
- Tauri event emission (data-sent, virtual-port-stopped, etc.)
- RPC connection context (session tracking, idle cleanup)

**Shared logic moves to CommandService**:
- Port open/close/send/recv orchestration
- Script load/unload/attach coordination
- Virtual port create/stop lifecycle

### 6. Autonomous Script Stays Separate

`LuaBindings` (autonomous mode) is a different execution model — the script actively controls the port, rather than reacting to port events. It remains independent from `ScriptManager`.

```
ScriptManager   → hook model: port triggers callbacks
LuaBindings     → autonomous model: script drives the port
```

---

## Implementation Summary

The unified script system was implemented in phases:

1. **Phase 1**: Created `src/script/` module with ScriptManager, rewrote built-in protocols in Lua
2. **Phase 2**: Extracted CommandService layer for shared orchestration
3. **Phase 3**: Removed `src/protocol/` directory entirely, updated all references

**Migration completed**: 2026-06-17. All tests passing (237+ tests).

---

## See Also

- [Protocol Reference](../reference/protocols.md) — Current script system documentation
- [Architecture](ARCH.md) — Full system architecture
