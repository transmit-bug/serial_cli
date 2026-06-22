# PRD: Script Import via Lua require()

## Problem Statement

Scripts in the serial_cli unified script system are currently isolated — each script is a self-contained Lua file with no way to reference code from other scripts. When business-layer protocols need to reuse common protocol logic (e.g., Modbus RTU CRC/frame handling), each device driver must duplicate the shared logic. The validation system explicitly warns against `require()` usage, blocking the natural Lua module pattern.

## Solution

Enable Lua `require()` for cross-script imports within the unified script system. All scripts live in a single directory (`scripts/protocols/`), and any script can `require()` any other script by name. The system configures Lua's `package.path` to resolve modules from the protocols directory.

## User Stories

1. As a protocol developer, I want to create a reusable Modbus RTU library script with CRC and frame handling functions, so that multiple device drivers can import and reuse it without duplication
2. As a device driver author, I want to `require('modbus_rtu')` in my temperature sensor script, so that I can call `modbus.crc16()` and `modbus.build_frame()` without reimplementing them
3. As a device driver author, I want to `require('modbus_rtu')` in my power meter script, so that I can reuse the same frame handling while defining different register mappings
4. As a script author, I want to organize my scripts in `scripts/protocols/` without needing separate `libs/` and `protocols/` subdirectories, so that the structure stays simple
5. As a script author, I want `require('scriptname')` to automatically find `scripts/protocols/scriptname.lua`, so that I don't need to specify full paths
6. As a script author, I want to write library scripts that return a Lua table (e.g., `return ModbusRTU`), so that they work as standard Lua modules
7. As a script author, I want library scripts to optionally define callbacks (on_send, on_recv), so that the same script can be both imported as a library and used directly as a protocol
8. As a script author, I want the validation system to accept `require()` without warnings, so that I'm not discouraged from using the import mechanism
9. As a script author, I want `require()` to work both in protocol scripts loaded by ScriptManager and in scripts executed by SerialScriptEngine, so that imports work consistently everywhere
10. As a script author, I want `require()` to cache modules (standard Lua behavior), so that importing the same library from multiple scripts doesn't cause redundant loading
11. As a system administrator, I want `require()` to only resolve scripts within `scripts/protocols/`, so that scripts cannot import arbitrary files from the filesystem
12. As a developer, I want existing built-in scripts (line, at_command, modbus_rtu, modbus_ascii) to continue working unchanged, so that the feature is backward-compatible
13. As a developer, I want to be able to create device-specific Modbus drivers (e.g., `temp_sensor.lua`, `power_meter.lua`) that import `modbus_rtu` and define only register mappings and data parsing, so that adding a new device type requires minimal code
14. As a script author, I want to chain imports (script A requires script B which requires script C), so that complex module hierarchies are supported
15. As a script author, I want clear error messages when a `require()`d module is not found, so that I can debug import issues quickly

## Implementation Decisions

### 1. Unified Script Directory
All scripts (libraries and protocols) live in `scripts/protocols/`. No separate `libs/` directory. A script is a "library" if it returns a table without defining callbacks, and a "protocol" if it defines callbacks. The same script can be both.

### 2. Package Path Configuration
Lua's `package.path` is configured to include all directories returned by `protocols_dir_candidates()`:
- `<exe_dir>/scripts/protocols/?.lua`
- `<exe_parent>/scripts/protocols/?.lua`
- `<cwd>/scripts/protocols/?.lua`
- `~/.serial-cli/protocols/?.lua`

This ensures `require('modbus_rtu')` resolves to `scripts/protocols/modbus_rtu.lua` in all execution contexts.

### 3. Runtime Integration Points
`package.path` is configured in two places:
- **`acquire_lua()`** — For pooled Lua instances used by ScriptManager validation and extract_script_meta
- **`SerialScriptEngine::new()`** — For engine instances attached to serial ports

A shared helper function `configure_package_path(lua)` handles the configuration logic.

### 4. Validation Changes
The `validate_script_detailed()` function in ScriptManager removes the warning about `require()` usage. The warning was present because `require()` wasn't previously supported; now it is.

### 5. Module Return Pattern
Library scripts use the standard Lua module pattern:
```lua
-- modbus_rtu.lua
local ModbusRTU = {}
function ModbusRTU.crc16(data) ... end
function ModbusRTU.build_frame(...) ... end
return ModbusRTU
```

Scripts that `require()` it get the returned table:
```lua
local modbus = require('modbus_rtu')
modbus.crc16(data)
```

### 6. Backward Compatibility
- Existing scripts that don't use `require()` continue to work unchanged
- Built-in scripts (embedded via `include_str!()`) are unaffected
- External override mechanism (`scripts/protocols/<name>.lua` overrides built-in) continues to work

### 7. Security Boundary
`require()` only resolves files within `scripts/protocols/` directories. It does not grant access to arbitrary filesystem paths. The `package.path` is explicitly set to only include the protocols directories.

## Testing Decisions

### Test Seams
- **`ScriptManager::validate_script_detailed()`** — Verify no warning for scripts using `require()`
- **`ScriptRuntime::configure_package_path()`** — Verify `package.path` is set correctly
- **`acquire_lua()`** — Verify pooled instances have `package.path` configured
- **`SerialScriptEngine::new()`** — Verify engine instances support `require()`
- **Integration test** — Create two scripts in a temp directory: a library returning a table, and a protocol that `require()`s it. Verify the protocol can call library functions.

### What Makes a Good Test
- Tests should verify external behavior (can a script `require()` another and call its functions), not implementation details (how `package.path` is formatted)
- Tests should use temp directories to avoid polluting the project
- Tests should clean up after themselves

### Prior Art
- `test_load_custom_script_from_file` — Creates temp script, loads it, verifies
- `test_validate_script_detailed_require_warning` — Currently tests the warning; needs updating
- `test_line_script_on_send_appends_newline` — Integration test pattern for script behavior

## Out of Scope

- **Remote script loading** — Scripts must be local files in `scripts/protocols/`
- **Version pinning** — No version constraints on `require()`d modules
- **Circular dependency detection** — Standard Lua `require()` caching handles this naturally
- **Per-script sandboxing** — All scripts share the same Lua environment
- **GUI script management** — The Tauri GUI may need UI updates to show import relationships, but that's a separate feature
- **Script dependency visualization** — No tooling to show which scripts import which

## Further Notes

This feature lays the groundwork for a richer protocol ecosystem. Future enhancements could include:
- A `serial-cli script deps <name>` command to show import dependencies
- Template/scaffold commands for creating new device drivers from a Modbus base
- Community script sharing via a registry
