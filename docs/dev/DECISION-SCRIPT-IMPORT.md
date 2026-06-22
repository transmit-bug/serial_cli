# ADR: Script Import via Lua require()

## Status

Accepted

## Context

Scripts in the serial_cli unified script system are currently isolated — each script is a self-contained Lua file with no way to reference code from other scripts. This creates a problem when business-layer protocols need to reuse common protocol logic:

- **Modbus RTU** provides CRC16, frame building, and frame parsing
- Different devices (temperature sensors, power meters, PLCs) use Modbus but with different register mappings
- Each device driver currently must duplicate Modbus frame handling logic

The current validation explicitly warns against `require()` usage:
```
"Script uses 'require()' which may not be available in all contexts"
```

## Decision

Support Lua `require()` for cross-script imports within the unified script system.

### Key Design Decisions

1. **No lib/protocol directory split** — All scripts live in `scripts/protocols/`. Any script can `require()` any other script by name. Scripts that define callbacks (on_send, on_recv) work as protocol scripts; scripts that are `require()`d return their module table.

2. **Configure `package.path`** — Set Lua's `package.path` to include `scripts/protocols/` directories (same candidates as `protocols_dir_candidates()`). This makes `require('modbus_rtu')` resolve to `scripts/protocols/modbus_rtu.lua`.

3. **Remove `require()` warning** — The validation warning about `require()` is removed since it's now a supported feature.

4. **`package.path` set on all Lua instances** — Both pooled instances (via `acquire_lua()`) and engine instances (via `SerialScriptEngine::new()`) get `package.path` configured.

## Consequences

- Scripts can import reusable logic from other scripts
- No new directory structure or concepts needed
- Existing scripts continue to work unchanged
- Code duplication across protocol scripts is eliminated
