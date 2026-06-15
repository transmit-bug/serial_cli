# Unified Script System — Design Decision

**Date**: 2026-06-15
**Status**: Accepted
**Supersedes**: Current Protocol + Hook Script separation

---

## Context

Serial CLI has three Lua execution modes that overlap in purpose:

| Mode | Module | Callbacks | Purpose |
|------|--------|-----------|---------|
| Protocol | `protocol/lua_ext.rs` → `LuaProtocol` | `on_frame`, `on_encode` | Frame encoding/decoding |
| Hook Script | `serial_core/serial_script.rs` → `SerialScriptEngine` | `on_open`, `on_send`, `on_recv`, `on_timer`, `on_close` | Port lifecycle control |
| Autonomous | `lua/bindings.rs` → `LuaBindings` | (free-form) | One-shot CLI script execution |

Protocol and Hook Script serve the same fundamental purpose — "attach behavior to a port" — but are split across two module hierarchies with separate lifecycle management (`ProtocolManager` vs `attach_script`). Users must understand the distinction to use the tool correctly. Built-in protocols (Modbus, AT Command, Line) are Rust implementations of the `Protocol` trait, bypassing the Lua engine entirely.

Additionally, the three command surfaces (CLI, JSON-RPC, Tauri) duplicate orchestration logic for port, protocol, and script operations.

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

Frame encoding/decoding (the former `Protocol` role) is implemented through `on_send`/`on_recv`:

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
├── registry.rs         # ScriptRegistry (script storage + lookup)
├── loader.rs           # ScriptLoader (Lua file loading + validation)
├── built_in/
│   ├── mod.rs          # Built-in registration
│   ├── modbus_rtu.lua  # Embedded Lua
│   ├── at_command.lua  # Embedded Lua
│   └── line.lua        # Embedded Lua
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

## Migration Plan

### Phase 1: Create `script/` Module + ScriptManager

1. Create `src/script/` with `manager.rs`, `registry.rs`, `loader.rs`, `built_in/`
2. Implement `ScriptManager` with the 5-method interface
3. Rewrite Modbus RTU as `built_in/modbus_rtu.lua` + write tests
4. Rewrite AT Command as `built_in/at_command.lua` + write tests
5. Rewrite Line as `built_in/line.lua` + write tests
6. Register built-in scripts at startup (via `include_str!()`)
7. `ScriptManager` coexists with `ProtocolManager` during migration

**Validation**: All existing protocol tests pass with Lua implementations.

### Phase 2: Extract CommandService

1. Create `src/service.rs` (or `src/service/mod.rs`)
2. Move shared port operations from CLI/RPC/Tauri into `CommandService`
3. Move shared script operations into `CommandService`
4. Move shared virtual port operations into `CommandService`
5. Refactor CLI handlers to delegate to `CommandService`
6. Refactor RPC dispatcher to delegate to `CommandService`
7. Refactor Tauri commands to delegate to `CommandService`

**Validation**: All three surfaces produce identical results for the same inputs.

### Phase 3: Remove Old Protocol Modules

1. Remove `src/protocol/` directory entirely
2. Remove `Protocol` trait
3. Remove `set_protocol` from `PortManager`
4. Update `PortManager` to use `ScriptManager` for script attachment
5. Update all imports and re-exports
6. Clean up dead code

**Validation**: Full test suite passes. No references to `Protocol` trait remain.

---

## Module Dependencies (After)

```
main.rs
  └→ cli/args (parse args)
  └→ cli/commands/* (dispatch)
       └→ service/* (CommandService — shared orchestration)
            ├→ serial_core (PortManager, VirtualSerialPair)
            ├→ script/* (ScriptManager, built-in scripts)
            └→ lua/* (ScriptRuntime, LuaBindings for autonomous)
```

---

## Files Changed (Summary)

| Action | Path | Notes |
|--------|------|-------|
| **New** | `src/script/mod.rs` | Module root |
| **New** | `src/script/manager.rs` | ScriptManager |
| **New** | `src/script/registry.rs` | ScriptRegistry |
| **New** | `src/script/loader.rs` | ScriptLoader + validation |
| **New** | `src/script/built_in/mod.rs` | Built-in registration |
| **New** | `src/script/built_in/modbus_rtu.lua` | Modbus RTU in Lua |
| **New** | `src/script/built_in/at_command.lua` | AT Command in Lua |
| **New** | `src/script/built_in/line.lua` | Line protocol in Lua |
| **New** | `src/service.rs` | CommandService |
| **Modified** | `src/serial_core/port.rs` | Remove `set_protocol`, add `attach_script` |
| **Modified** | `src/serial_core/serial_script.rs` | Adapt for unified script model |
| **Modified** | `src/cli/commands/port.rs` | Delegate to CommandService |
| **Modified** | `src/cli/commands/protocol.rs` | Rename to `script.rs`, delegate |
| **Modified** | `src/cli/commands/sniff.rs` | Delegate to CommandService |
| **Modified** | `src/cli/commands/virtual_port.rs` | Delegate to CommandService |
| **Modified** | `src/server/rpc.rs` | Delegate to CommandService |
| **Modified** | `src-tauri/src/commands/serial.rs` | Delegate to CommandService |
| **Modified** | `src-tauri/src/commands/protocol.rs` | Rename to `script.rs`, delegate |
| **Modified** | `src-tauri/src/commands/virtual_port.rs` | Delegate to CommandService |
| **Modified** | `src/cli/types.rs` | Update command types |
| **Modified** | `src/cli/args.rs` | Update CLI arg definitions |
| **Modified** | `src/config.rs` | Remove protocol config, add script config |
| **Modified** | `src/server/state.rs` | Replace ProtocolManager with ScriptManager |
| **Modified** | `src-tauri/src/state/app_state.rs` | Replace ProtocolManager with ScriptManager |
| **Deleted** | `src/protocol/` (entire directory) | Replaced by `src/script/` |
