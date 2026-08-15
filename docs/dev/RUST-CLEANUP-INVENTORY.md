# Rust/Lua cleanup inventory

Research ticket: [#79](https://github.com/transmit-bug/serial_cli/issues/79) (Rust/Lua: dead-code & unused-dependency inventory).
Map: [#75](https://github.com/transmit-bug/serial_cli/issues/75). Execution ticket (after this lands): #82.

Terminology (per map #75): **dead** = unreferenced/unreachable, verifiable by build + tooling; **stale** = still referenced but describes a superseded state; **legacy leftover** = a superseded replacement exists in-repo.

Every claim cites a source. Methods used:

- `cargo machete` (v0.9.2, installed via cargo-binstall) on the workspace; `cargo machete --with-metadata` too.
- Manual grep pass for every dependency in `src/`, `src-tauri/src/`, `tests/`, `benches/`.
- `cargo build --all-targets` and `cargo bench --no-run` (both succeeded; the only compiler warning in the whole workspace is `benches/lua_execution.rs:20`).
- Full `cargo test` run (462 tests, 0 failures; only `tests/e2e_server_tests.rs`'s 17 tests are deliberately `#[ignore]`d).
- Cross-reference script: every `pub` item in `src/` checked for references outside its own file (method-name false positives re-verified manually).
- Lua: word-boundary reference scan for all 16 protocol-script names across `tests/benches/examples/docs/src`; line-signature (Jaccard) overlap scan over `scripts/protocols/*.lua` and `examples/*.lua`; `diff` of `src/script/built_in/*.lua` vs `scripts/protocols/*.lua`.
- Frontend mock layer verified statically against `frontend/vite.config.ts`, `frontend/AGENTS.md`, `docs/dev/PDR-MOCK-LAYER.md`, and commit history (no `pnpm install` was run in the worktree).
- `cargo bench --no-run` completed in 1m14s (fresh worktree target dir). `just test` = `cargo test` (`justfile:47`). Cargo.lock is gitignored (`.gitignore:4`).

---

## 1. Unused Cargo dependencies

| # | Dependency | Declared in | Evidence | Severity | Disposition |
|---|------------|-------------|----------|----------|-------------|
| D1 | `anyhow` | root `Cargo.toml` `[dependencies]` | `cargo machete`: "serial-cli -- ./Cargo.toml: anyhow". Manual grep: zero hits for `anyhow` in `src/`, `src-tauri/src/`, `tests/`, `benches/`. | dead | **safe-delete** |
| D2 | `notify` | root `Cargo.toml` `[dependencies]` | `cargo machete`: same run flags `notify`. Manual grep: zero hits for `notify` in any `.rs` file. File watching was never wired up. | dead | **safe-delete** |
| D3 | `rand` | root `Cargo.toml` `[dev-dependencies]` | Not flagged by machete, but repo-wide `\brand\b` grep over `tests/`, `benches/`, `src/` yields zero hits. No bench/test uses it (criterion pulls its own rand). | dead | **safe-delete** |
| D4 | `rustyline` (dev-dep copy) | root `Cargo.toml` `[dev-dependencies]` (`"18.0.0"`) | `rustyline = "18.0"` already declared in `[dependencies]`; dev-dep copy is redundant. Only user is `src/cli/interactive.rs` (main dep). | redundant | **safe-delete** |
| D5 | `libc` (dev-dep copy) | root `Cargo.toml` `[dev-dependencies]` (`"0.2"`) | `libc = "0.2"` already in `[dependencies]`; dev copy redundant. | redundant | **safe-delete** |
| D6 | src-tauri dependencies | `src-tauri/Cargo.toml` | All 14 declared deps have real usage in `src-tauri/src/` (verified per-crate grep): `tauri`, `tauri-plugin-shell`, `tauri-plugin-dialog`, `serde`, `serde_json`, `tokio`, `tokio-util`, `tracing`, `tracing-subscriber`, `log`, `chrono`, `uuid`, `dirs`, `toml`, plus build-dep `tauri-build`. machete flags nothing in src-tauri. | — | keep (see T5/T6 for overlap notes) |
| D7 | `windows` (root, cfg(windows)) | root `Cargo.toml` `[target.'cfg(windows)'.dependencies]` | Genuinely used under `#[cfg(windows)]`: `src/cli/virtual_port_session.rs:153-187`, `src/cli/sniff_session.rs:106-141`, `src/serial_core/backends/socat.rs:199`, `src/serial_core/backends/named_pipe.rs:49`, and the dead `src/serial_core/windows_signals.rs:14-18`. If R2 is deleted, `Win32_Devices_Communication` feature becomes unused there (feature trimming only, crate stays). | live | keep |

Notes:
- machete requires a Cargo.lock; it was generated in the worktree (Cargo.lock is gitignored — nothing to commit).
- `criterion`/`proptest`/`tempfile` dev-deps are correctly dev-only and used (benches / `tests/protocol_parsing_tests.rs` / `tests/integration_tests.rs`).

---

## 2. Dead / duplicate Rust modules and items

### Modules / files — verifiably dead

| # | Item | Size | Evidence | Severity | Disposition |
|---|------|------|----------|----------|-------------|
| R1 | `src/error_handling.rs` | 508 lines | Zero references anywhere except its own `pub mod` declaration (`src/lib.rs:9`) and a stale doc mention (`docs/dev/ARCH.md:16`). `ErrorHandler`/`RecoveryHandler`/`ErrorCode`/`RecoveryStrategy` never used. Defines a *second* `ErrorContext` type (message/code/timestamp) distinct from the live `ErrorContext` in `src/error.rs` (operation/port/script/source) — near-duplicate name/type. Its `#[cfg(test)]` unit tests test this dead code only. | dead | **safe-delete** (update `docs/dev/ARCH.md:16` in the same change) |
| R2 | `src/serial_core/windows_signals.rs` | 154 lines | `WindowsSignalControl` is re-exported (`src/serial_core/mod.rs:16-17,30-31`) but never used anywhere; every method carries `#[allow(dead_code)]` (self-acknowledged dead). Superseded by `WindowsSignalController` in live `src/serial_core/signals.rs:294-402` (different type, same job). Two parallel Windows signal implementations — legacy leftover. | legacy leftover | **safe-delete** (verify Windows build; remove re-export from `mod.rs`) |
| R3 | `src/serial_core/port_script_controller.rs` | 280 lines | **Orphan file**: not declared in `src/serial_core/mod.rs` (module list at `mod.rs:3-18`), zero references repo-wide (grep). Never compiled; no warnings possible. `PortManager::attach_script` (`src/serial_core/port.rs:444-449, 691`) implements script attachment inline via `SerialScriptEngine`, duplicating this file's purpose. | dead / legacy leftover | **safe-delete** |
| R4 | `src/serial_core/io_loop.rs` | 576 lines (269 in `#[cfg(test)]`, 307-576) | `IoLoop`/`IoLoopConfig`/`IoEvent` are never instantiated or referenced outside this file's own tests (`IoLoop::new()` appears only at `io_loop.rs:315,330,351,380,400,420,442,453,460,482` — all inside `mod tests`). Dead re-export `pub use io_loop::IoLoop;` at `serial_core/mod.rs:20`. The "IoLoop mode" in `src/serial_core/port.rs` is port.rs's own inline background read task (`port.rs:286-314`), not this module. Note: the Tauri sniffer (`src-tauri/src/commands/serial.rs`) re-implements exactly this pump; see T2. | dead | **safe-delete** (nothing calls it; if the Tauri sniffing pump is ever promoted to the library, write it fresh or revive this — see T2) |
| R5 | `src/cli/json.rs` | 250 lines | `JsonFormatter` re-exported at `src/cli/mod.rs:15` but never called anywhere. CLI commands build JSON inline (`src/cli/commands/port.rs:42` `use serde_json::json;`, `:58` `to_string_pretty`). Only users are the module's own tests. Types `JsonResponse`/`ResponseStatus`/`ErrorDetail`/`ResponseMetadata`/`OperationStatistics` and free fns `success_response`/`error_response` are likewise unused. | dead | **safe-delete** (update `docs/dev/ARCH.md:27`) |
| R6 | `src/lua/runtime.rs` → `ScriptCache` | ~55 lines | `ScriptCache` (`runtime.rs:127-177`) is a standalone struct with no non-test usage; only `runtime.rs:623` (test) constructs it. `mark_executed`/`contains`/`clear`/`len`/`is_empty` used nowhere. | dead | **safe-delete** (the struct + its methods; keep `LuaStatePool`, `acquire_lua`/`release_lua`, `configure_package_path`, `ScriptRuntime` — all live) |
| R7 | `src/utils.rs` top-level items | ~340 of 441 lines | `AutoReconnectConfig` (`utils.rs:16`), `PortStats` (`:40`), `AutoReconnect` (`:125`), `DataFormat` (`:193`), `ProgressReporter` (`:289`) — zero references outside utils.rs; only used by the file's own `#[cfg(test)]` tests. Submodules `hex` (`utils.rs:11`) and `lua_conversion` (`:12`) are live (used by `server/client.rs:427`, `server/rpc.rs:419`, `serial_core/serial_script.rs:11`, `lua/runtime.rs:25`, `lua/bindings.rs:22`, `tests/protocol_parsing_tests.rs:8`). `DataFormat::bytes_to_hex`/`hex_dump` are a second hex formatter next to `utils/hex.rs::hex_encode_simple` — near-duplicate helper. | dead | **safe-delete** (trim to `hex` + `lua_conversion` submodules; keep `bytes_to_hex` only if referenced by docs — no doc refs found) |
| R8 | `src/serial_core/sniffer.rs` partial | methods | `get_packets`, `clear_packets`, `capture_tx` have no callers outside the module's own tests. Live parts: `SerialSniffer`, `start_sniffing`, `capture_rx` (used by `src/cli/sniff_session.rs:365,377,416`), `save_to_file`, `SnifferSession`. | dead (partial) | **safe-delete** of the 3 methods |
| R9 | `src/serial_core/backends/mock.rs` partial | — | `MockSerialPortBuilder` + methods `with_timeout`, `with_read_limit`, `written_data`, `written_len`, `clear_write_capture`, `push_read_data` — no external callers (tests only). `MockSerialPort` itself is live for tests (`server/rpc.rs:912-977`, `port.rs:633` doc ref). | dead (partial) | **safe-delete** of builder-only API after checking `tests/` (test-only consumers) |

### Public items with no callers (visibility-reduction candidates)

These are `pub` but never referenced outside their defining file. They are NOT removable as easily as the modules above (they are part of the crate's public API surface; some are `Config` field types used via type inference). Disposition: **needs-decision** — either make `pub(crate)`/private or delete, decided by the executing session:

| # | Item | Source | Note |
|---|------|--------|------|
| P1 | `load_from_path`, `get_custom_protocol`, `get_global_config_dir`; types `LuaConfig`, `OutputConfig`, `CustomProtocolConfig`, `ProtocolsConfig`, `VirtualPortsConfig`, `DisplayConfig` | `src/config.rs` | The types are live as `Config` struct fields (`config.rs:399-416`) but never *named* externally (type inference). Visibility-reduction, not deletion. `get_global_config_path` IS used (`src-tauri/src/commands/config.rs:181`) — keep. |
| P2 | `set_ioloop_enabled`, `is_ioloop_enabled`, `script_timer_interval_ms` | `src/serial_core/port.rs` | No callers; the whole "IoLoop mode" toggle is only exercised by `state_factory.rs`'s test (`state_factory.rs:56`). `PortManager::with_ioloop` + `CoreManagers::with_ioloop` (`state_factory.rs:30`) are also test-only. Production paths use `PortManager::new()` (`main.rs`, tauri `app_state.rs`). |
| P3 | `reload_modified_scripts`, `tracked_paths`, `load_with_validation`, `get_script_path`, `is_script_modified` | `src/script/manager.rs:424-530` | All only referenced by manager.rs's own tests. `statistics`/`ScriptStatistics` are used by CLI (`cli/commands/script.rs`) — keep. |
| P4 | `log_data_transfer`, `log_protocol_message`, `record_operation_duration` | `src/logging.rs:150-177` | No callers anywhere (not even internally). `LoggingConfig::from_env` is live (`logging.rs:132,142`). |
| P5 | `hex_decode_strict` | `src/utils/hex.rs:128` | Tests only (`hex.rs:256-262`). |
| P6 | `ConnectionStats` | `src/server/state.rs` | No external references. |
| P7 | `execute_command`, `set_current_port` | `src/cli/interactive.rs` | No external callers (interactive loop calls them internally? no — they are `pub`, called only by tests if at all). Verify before removing. |
| P8 | `ActionParam` | `src/lua/ui_actions.rs` | ui_actions module is live (used by `serial_core/serial_script.rs:375-395`, `port.rs:513-551`); only the `ActionParam` type is never named externally. |

### Near-duplicate code (Rust)

| # | Duplicate pair | Evidence | Severity | Disposition |
|---|----------------|----------|----------|-------------|
| R10 | `ErrorContext` in `error_handling.rs` vs `error.rs` | Two structs with the same name, different shape; one is dead (R1). | dead | safe-delete (covered by R1) |
| R11 | `PortStats` (`utils.rs:40`) vs `OperationStatistics` (`cli/json.rs`) vs `PortStatsTracker` (`src-tauri/src/state/app_state.rs`) vs `PortStats` DTO (`src-tauri/src/state/port_state.rs`) | Four port-statistics implementations; the first two are dead (R7, R5), the last two live in the Tauri layer (T4). | redundancy | needs-decision (consolidate into the library; see T4) |
| R12 | Windows signal control: `windows_signals.rs` vs `signals.rs::WindowsSignalController` | See R2. | legacy leftover | safe-delete (R2) |
| R13 | Sniffer file export (`serial_core/sniffer.rs::save_to_file`) vs Tauri `commands/export.rs` (txt/csv/json exporters) | Both hex-format serial captures to files; Tauri re-implemented it (T1). | redundancy | needs-decision (move one exporter to the library) |

### Test-file redundancy

| # | Item | Evidence | Severity | Disposition |
|---|------|----------|----------|-------------|
| R14 | `tests/server_integration.rs` (443 L) + `tests/server_integration_tests.rs` (133 L) + `tests/e2e_server_tests.rs` (851 L) | Three server test files with overlapping scope (lifecycle unit-style vs socket round-trip vs CLI e2e). | redundancy | **needs-decision** (consolidate/rename; not verifiably dead — all run under `cargo test`) |
| R15 | `tests/protocol_manager_test.rs` (singular) vs `*_tests.rs` convention | Naming inconsistency only. | cosmetic | needs-decision (rename) |

---

## 3. src-tauri IPC thinness (business logic in the Tauri layer)

AGENTS.md boundary: "All real logic lives in the library (`serial_cli::...`); the Tauri backend is a thin IPC layer." Most commands are thin wrappers (`script.rs`, `config.rs`, `virtual_port.rs`, `server.rs`, `script_ui_actions.rs`, `serial_script.rs`, `window.rs` — all delegate to library managers). Violations found:

| # | Location | What lives there | Evidence | Disposition |
|---|----------|------------------|----------|-------------|
| T1 | `src-tauri/src/commands/export.rs` (197 L) | Full txt/csv/json export logic: directory creation, hex formatting, header/timestamp writing (`export_txt`/`export_csv`/`export_json`), using `chrono::Utc`, `serde_json`. | File body; duplicates library capability `serial_core/sniffer.rs::save_to_file` and `utils::hex`. | **needs-decision** — move exporter to library (e.g. `serial_cli::export`), Tauri command becomes a 3-line wrapper |
| T2 | `src-tauri/src/commands/serial.rs` `start_sniffing` (~180 L) | The whole data pump: `spawn_blocking` read loop, mpsc channel, async event loop, disconnect sentinel, per-port stats updates. | `serial.rs:115-296`. The library's `io_loop.rs` (R4) was designed for exactly this and is dead. | **needs-decision** — promote to library (revive/adopt `io_loop`) or extract a `serial_cli::sniffer::run_pump`; the CLI's own daemon (`cli/sniff_session.rs`) is a third pump implementation |
| T3 | `src-tauri/src/commands/port.rs` | Business rules in the layer: hardware-port filtering (`!contains("debug-console") && !contains("pty.") && !contains("ttys")`, `port.rs:36-40`), DTO mapping `SerialConfig ↔ CoreSerialConfig` (`port.rs:62-73`), string→enum shims `parse_parity`/`parse_flow_control` (`port.rs:316-337`). | File body. The CLI has no equivalent parser (CLI takes different input shape), so the enum parsing is Tauri-specific — but it belongs with the library's `Parity`/`FlowControl` types (`serial_core/port.rs`). | **needs-decision** — move filtering + parsing to library; DTO structs stay at the boundary |
| T4 | `src-tauri/src/state/app_state.rs` (`PortStatsTracker`) + `state/port_state.rs` (`PortStats`/`SerialConfig` DTOs) | Third port-stats implementation (see R11) and a string-typed `SerialConfig` DTO that forces T3's parse shims. | `app_state.rs:23-55`, `port_state.rs`. | **needs-decision** — derive stats from a library stats type; consider serde-compatible library `SerialConfig` |
| T5 | `src-tauri/src/commands/remote.rs` `load_devices`/`save_devices`/`registry_path` | Remote-device registry persistence (JSON file on disk) implemented in the Tauri layer. | `remote.rs:37-66`. | **needs-decision** — small; could move to a library `remote::device_registry` |
| T6 | `src-tauri/src/main.rs` | Dual logging stack: `log` crate (`main.rs`, `commands/port.rs`, `commands/virtual_port.rs`, `commands/serial.rs`) **and** `tracing`/`tracing-subscriber` (`main.rs`, `server.rs`, `serial.rs`). Library convention is tracing only (`src/logging.rs`). | grep of `src-tauri/src/` shows `log::` in 5 files, `tracing::` in 3. Also `dirs` (tauri) vs `directories` (lib) — same job, two crates. | needs-decision — unify on tracing; drop `log` dep or alias it (cosmetic; not dead) |

---

## 4. Benches (`benches/*.rs`)

`cargo bench --no-run` **succeeds** — all 5 benches compile (1m14s, criterion v0.5.1 + rand v0.10.2 built). CI runs all 5 on every push/PR touching `src/benches/Cargo*` plus a weekly schedule (`.github/workflows/benchmark.yml:1-17, 56-60`). So benches are live tooling, not abandoned.

| # | Finding | Evidence | Severity | Disposition |
|---|---------|----------|----------|-------------|
| B1 | `benches/lua_execution.rs:20` — `let manager = ScriptManager::new();` is unused | Only warning in the entire workspace build. Would fail `cargo clippy --all-targets -- -D warnings` (CI currently runs plain `cargo clippy -- -D warnings`, `ci.yml:46`, which skips bench targets). | dead code (1 line) | **safe-delete** (drop the unused binding) |
| B2 | `benches/lua_execution.rs` is misnamed — "Lua execution benchmarks" contains no execution benchmarks (load/validate/list/get_source only) | File header + bench list (`lua_execution.rs:1-50`). | stale (naming/scope) | **needs-decision** — rename or add real `execute_string` benches |
| B3 | `benches/serial_io.rs` is misnamed — "Serial I/O throughput benchmarks" contains no serial I/O; it benchmarks `ScriptManager` `on_send`/`on_recv` roundtrips; its `bench_script_execution` duplicates `benches/protocol_parsing.rs`'s roundtrip benches | `serial_io.rs:1-70` vs `protocol_parsing.rs:8-70`; no `serialport`/`tokio-serial` import anywhere in benches (dep scan). | stale / redundancy | **needs-decision** — either add real serial I/O benches (mock backend exists: `serial_core/backends/mock.rs`) or delete/merge the file |
| B4 | `benches/startup.rs` `lua_engine_init` — creates `ScriptManager::new()` and discards; `ScriptManager::new()` does not initialize a Lua engine (engines are created lazily via `create_engine`). Bench measures nothing about Lua init. | `startup.rs:44-47`; `manager.rs` `create_engine` is where Lua init happens. | stale (benchmark measures the wrong thing) | needs-decision — drop or retarget |
| B5 | `benches/concurrency.rs` `bench_concurrent_buffer_ops` — benchmarks `vec![0u8; 4096]` + `yield_now` (trivial allocations), not app code | `concurrency.rs:31-48`. | low value | needs-decision — keep (cheap signal) or replace with real concurrent port/script ops |

---

## 5. Test stubs — `tests/lua_sandbox_tests.rs`

Finding: these are **not failing stubs** — all tests pass and document the current (non-sandboxed) runtime behavior. The "stubs" are two TODO comments plus behavior-capture tests:

- File header (`lua_sandbox_tests.rs:1-8`): states `enable_sandbox` and `memory_limit_mb` config keys are used only for static validation; runtime sandboxing is not enforced.
- `lua_sandbox_tests.rs:203` — TODO: `memory_limit_mb` — the "limit" test (`test_lua_allocation_basic`, `:215-223`) merely asserts allocation succeeds.
- `lua_sandbox_tests.rs:223` — TODO: `timeout_seconds` — `test_script_execution_completes_quickly` (`:228-243`) asserts a fast script completes; comment notes an infinite loop would block forever.
- The config keys DO exist: `config.rs:474-478` (`memory_limit_mb: usize`, `timeout_seconds: u64`, `enable_sandbox: bool`, default 128 MB).
- The rest of the file (16 tests) covers live static validation (`validate_script_detailed` dangerous-function detection) and documents default mlua stdlib exposure (`os`, `io`, `loadfile`, `dofile` available; `debug` not loaded) — genuinely useful regression coverage.

Disposition: **needs-decision** (map #75 explicitly defers this). Options for the executing session: (a) keep as-is — they document real current behavior and encode the backlog intent; (b) convert the two runtime tests to `#[ignore]`d so the TODO is explicit; (c) delete only the two runtime "limit" tests. Implementation of the sandbox itself is out of scope (map: no new features).

---

## 6. Lua protocol redundancy (`scripts/protocols/`)

| # | Finding | Evidence | Severity | Disposition |
|---|---------|----------|----------|-------------|
| L1 | `modbus_rtu.lua` (176 L) and `modbus_rtu_lib.lua` (168 L) implement the same CRC16 + frame logic twice | `modbus_rtu.lua:72-90` (inline `crc16`) vs `modbus_rtu_lib.lua:18-30` (`calculate_crc`); both append/verify CRC. `modbus_rtu_lib.lua` is live — required by `scripts/protocols/temp_sensor.lua:15` and documented in `docs/guides/script-development.md:552,563`. Both are the "two implementations of the same protocol helper" called out in the ticket. | duplicate | **needs-decision** — refactor `modbus_rtu.lua` to `require("modbus_rtu_lib")` (single CRC source of truth). Neither file is deletable as-is |
| L2 | `modbus_rtu_lib.lua` (a library, returns module table) lives in the auto-scanned protocols dir and is registered as a *protocol* | `load_external_protocols` registers **every** `.lua` in the scanned dirs without requiring `SCRIPT_META` (`src/script/manager.rs:105-160`); lib files get a fallback description "Protocol script: …" and no callbacks, so they surface in `script list` / GUI as pseudo-protocols. | structural wart | **needs-decision** — skip files without `SCRIPT_META` in the scanner, or move libs to `scripts/lib/` (and add a lib path to `protocols_dir_candidates`, `manager.rs:222-243`) |
| L3 | `i2c_uart.lua` ~ `spi_uart.lua` (Jaccard 0.32, 70 shared line-signatures) | Pairwise line-signature scan of `scripts/protocols/*.lua`. Both are UART-bridge protocols sharing a template skeleton. | near-duplicate | **needs-decision** — likely acceptable (different wire protocols); review whether a shared helper is warranted |
| L4 | `modbus_rtu.lua` ~ `pzem004t.lua` (Jaccard 0.30, 64 shared) | Same scan. PZEM power meters speak Modbus RTU; `pzem004t.lua` re-implements the CRC/frame logic instead of requiring `modbus_rtu_lib`. | near-duplicate | **needs-decision** — refactor to `require("modbus_rtu_lib")` (same as L1) |

---

## 7. `scripts/templates/` vs `examples/` overlap

| # | Finding | Evidence | Severity | Disposition |
|---|---------|----------|----------|-------------|
| S1 | `scripts/templates/` (PKGBUILD, .SRCINFO, homebrew.rb, scoop.json) is **live**, not redundant: the release pipeline consumes it directly. No overlap with `examples/` (packaging metadata vs Lua teaching scripts — different domains). | `.github/workflows/release.yml:372` (homebrew.rb), `:392` (scoop.json), `:409,411` (PKGBUILD/.SRCINFO). CHANGELOG.md:12 says templates were extracted from inlined heredocs. | live | keep |
| S2 | `scripts/update-packages.sh` re-implements the same packaging generation with inline heredocs (`update-packages.sh:36-74` PKGBUILD/.SRCINFO, `:97-115` homebrew formula, `:140-153` scoop manifest) that release.yml now does via `scripts/templates/`; update-packages.sh is **not invoked by any CI workflow**. | release.yml's `update-packages` job does template-based updates inline (`release.yml:335-414`); grep shows no workflow calls update-packages.sh. | legacy leftover (superseded by release.yml + templates/) | **needs-decision** — likely delete after confirming no manual release process relies on it |
| S3 | `scripts/release.sh` — dev-only release-prep helper, referenced by `DEVELOPMENT.md:71-77`, not CI. | grep. | live (dev tooling) | keep |
| S4 | `scripts/install.sh` / `install.ps1` — copied into release artifacts. | `.github/workflows/release.yml:143-144`. | live | keep |
| S5 | `examples/` Lua scripts partially duplicate `scripts/protocols/` logic: `examples/temperature_sensor.lua` (79 L) shares 28% of its lines with `scripts/protocols/temp_sensor.lua` (270 L); `examples/nmea_gps.lua` (88 L) 35% with `nmea0183.lua` (225 L); `examples/modbus_with_tools.lua` 12% with `modbus_rtu.lua`. | Line-signature overlap scan. `examples/` is referenced by docs (`README.md`, `docs/ai/USAGE.md`, `docs/features/ui-actions.md`). | near-duplicate (teaching copies) | **needs-decision** — likely keep (teaching value, documented); optionally have examples `require()` the protocol scripts instead of re-encoding |

---

## 8. Full Lua script audit (all 37 .lua files)

Inventory of every `.lua` file: 4 embedded built-ins (`src/script/built_in/`), 16 runtime protocols (`scripts/protocols/`), 14 examples (`examples/`), 3 test fixtures (`tests/fixtures/protocols/`). Cross-checked against `docs/reference/protocols.md`, `config/default.toml`, `tests/`, `benches/`, `examples/`, and the loader in `src/script/manager.rs`.

### 8.1 Loader behavior (context for everything below)

- `ScriptManager::new()` registers 4 built-ins from `include_str!` (`src/script/built_in/mod.rs:17-37`) with **external-file-first** override: if `scripts/protocols/<name>.lua` exists on disk (exe dir / cwd / `~/.serial-cli/protocols`, `manager.rs:222-243, 76-101`), the on-disk copy wins at runtime; the embedded copy is the fallback for installs without a scripts dir.
- `load_external_protocols` (`manager.rs:105-160`) then registers **every other** `.lua` in the scanned dirs (no `SCRIPT_META` requirement) — this is how `modbus_rtu_lib.lua` and the 12 extra protocols surface (see L2).
- Installed packages do NOT ship `scripts/` (deb assets are only the binary + systemd unit, `Cargo.toml [package.metadata.deb]`; `install.sh` copies none) — so the 12 extras only exist in dev checkouts / manual installs.

### 8.2 Per-file classification

| # | File | Line count | References (outside the file) | Verdict | Disposition |
|---|------|-----------|-------------------------------|---------|-------------|
| LA1 | `src/script/built_in/line.lua` | 25 | Embedded `include_str!` (`built_in/mod.rs:20-23`); tests `create_engine("line")` (`tests/protocol_parsing_tests.rs:275-303`); identical to `scripts/protocols/line.lua` | **product-critical** | keep (see L5 for the duplication) |
| LA2 | `src/script/built_in/at_command.lua` | 64 | Embedded (`built_in/mod.rs:24-27`); tests + benches (`protocol_parsing_tests.rs:316-326`, `benches/*`); documented (`docs/reference/protocols.md:49,127`) | **product-critical** | keep (diverged from external copy — see L5) |
| LA3 | `src/script/built_in/modbus_rtu.lua` | 114 | Embedded (`built_in/mod.rs:28-31`); tests + benches (`protocol_parsing_tests.rs:337-390`, `tests/lua_integration_tests.rs`); documented (`protocols.md:47,58`) | **product-critical** | keep (diverged — see L5) |
| LA4 | `src/script/built_in/modbus_ascii.lua` | 114 | Embedded (`built_in/mod.rs:32-35`); tests (`tests/modbus_ascii_test.rs`, `protocol_parsing_tests.rs`); documented (`protocols.md:48,88`) | **product-critical** | keep (diverged — see L5) |
| LA5 | `scripts/protocols/at_command.lua` | 63 | External override of built-in (loaded in dev/installed-with-scripts); tests + benches; documented (`protocols.md:49`) | **product-critical** (as override) | see L5 — merge with embedded copy |
| LA6 | `scripts/protocols/line.lua` | 24 | Same as LA1 (identical) | **product-critical** | see L5 |
| LA7 | `scripts/protocols/modbus_rtu.lua` | 176 | Same as LA3 (override); plus `_actions` GUI table | **product-critical** | see L5 |
| LA8 | `scripts/protocols/modbus_ascii.lua` | 165 | Same as LA4 (override) | **product-critical** | see L5 |
| LA9 | `scripts/protocols/modbus_rtu_lib.lua` | 168 | `require()`d by `scripts/protocols/temp_sensor.lua:15`; documented in `docs/guides/script-development.md:552,563` | **live** (library) | keep (see L2 for the scanner wart) |
| LA10 | `scripts/protocols/temp_sensor.lua` | 269 | Only `docs/dev/PRD-SCRIPT-IMPORT.md:25` (a design doc); no tests, no examples, no guides reference it | near-dead | **needs-decision** — document+test or remove; its purpose is duplicated by `examples/temperature_sensor.lua` |
| LA11 | `scripts/protocols/can.lua` | 391 | Zero references (word-boundary scan; "can" prose matches are false positives — no `create_engine("can")`, no doc, no example) | dead weight (auto-loaded, never referenced) | **needs-decision** — document+test or remove |
| LA12 | `scripts/protocols/dlt645.lua` | 392 | **Zero references** repo-wide (tests, benches, examples, docs, src) | dead weight | **needs-decision** — document+test or remove |
| LA13 | `scripts/protocols/dmx512.lua` | 209 | **Zero references** | dead weight | **needs-decision** — document+test or remove |
| LA14 | `scripts/protocols/mqtt_serial.lua` | 204 | **Zero references** | dead weight | **needs-decision** — document+test or remove |
| LA15 | `scripts/protocols/onewire.lua` | 236 | **Zero references** | dead weight | **needs-decision** — document+test or remove |
| LA16 | `scripts/protocols/sdi12.lua` | 186 | **Zero references** | dead weight | **needs-decision** — document+test or remove |
| LA17 | `scripts/protocols/i2c_uart.lua` | 214 | Zero references; near-duplicate of `spi_uart.lua` (Jaccard 0.32, 70 shared lines) | dead weight / duplicate pair | **needs-decision** |
| LA18 | `scripts/protocols/spi_uart.lua` | 235 | Zero references; pair with `i2c_uart.lua` | dead weight / duplicate pair | **needs-decision** |
| LA19 | `scripts/protocols/nmea0183.lua` | 224 | Zero references; `examples/nmea_gps.lua` (88 L) re-encodes the same protocol (35% line overlap) without `require()`-ing it | dead weight / duplicated by example | **needs-decision** |
| LA20 | `scripts/protocols/pzem004t.lua` | 275 | Zero references; 30% line overlap with `modbus_rtu.lua` (PZEM meters speak Modbus RTU) | dead weight / near-duplicate | **needs-decision** — could `require("modbus_rtu_lib")` (L4) |
| E1 | `examples/` (14 .lua files) | 79-275 each | Referenced only as a directory: `README.md:167`, `docs/ai/USAGE.md:340`; only `examples/at_commands.lua` by name (`docs/reference/troubleshooting.md`). Individually unreferenced; several re-encode protocol logic (`temperature_sensor` 28% overlap, `nmea_gps` 35%, `modbus_with_tools` 12%) | teaching gallery (docs-linked) | **needs-decision** — keep but index/document each file, or trim to a canonical set; do not delete blindly (docs link the dir) |
| F1 | `tests/fixtures/protocols/test_valid.lua` | — | Used: `tests/cli_functional_regression.rs:748`, `tests/lua_integration_tests.rs:120`, `tests/protocol_manager_test.rs:17-50` | **live** | keep |
| F2 | `tests/fixtures/protocols/test_syntax_error.lua` | — | Used: same three files (syntax-error cases) | **live** | keep |
| F3 | `tests/fixtures/protocols/test_missing_func.lua` | — | **Zero references** in `tests/` or anywhere (grep for `missing_func`/`test_missing`) | dead fixture | **safe-delete** |

### 8.3 Cross-check notes

- `docs/reference/protocols.md` documents only the 4 built-ins (lines 47-50: `modbus_rtu`, `modbus_ascii`, `at_command`, `line`; line 52: "Built-in script names are reserved"). The other 12 `scripts/protocols/` files are **undocumented anywhere**. If they stay, `protocols.md` needs a section; if they go, nothing to update.
- `config/default.toml` has no `[protocols]` section, and the file itself is unreferenced (see E2 below).
- `src/` loads scripts only through `ScriptManager` (embedded + dir scan + `[protocols.custom.*]` from config, `manager.rs:30-50`); no other Rust code reads `.lua` files.

### 8.4 New findings from the audit

| # | Finding | Evidence | Severity | Disposition |
|---|---------|----------|----------|-------------|
| L5 | **Four protocols exist twice with drift**: `src/script/built_in/{at_command,line,modbus_ascii,modbus_rtu}.lua` vs `scripts/protocols/` copies. `line.lua` identical; `at_command.lua` differs (31 diff lines), `modbus_ascii.lua` (52), `modbus_rtu.lua` (63). Concrete impact: the embedded `modbus_rtu.lua` lacks the `_actions` table (GUI auto-binding, added by commit a80a44b) that the external copy has — a shipped binary using the embedded fallback silently loses GUI actions. Fixes to one copy do not reach the other. | `diff` of the 4 pairs; `git log a80a44b`; override logic `manager.rs:76-101` | structural redundancy (dual source of truth) | **needs-decision** — single source of truth: embed from `scripts/protocols/` at build time, or delete the external override and keep embedded canonical, or add a sync/CI guard |
| E2 | `config/default.toml` (26 lines) is orphaned — zero references in Rust, docs, packaging, or workflows; defaults are defined in `Config::default()` (`src/config.rs`) and written to `default_config_save_path()` (`config.rs:366`); `default.toml` is never read | grep `default.toml` across `.rs/.md/.toml/.yml`; `config.rs` has no `include_str!` | dead file | **safe-delete** (it is a leftover sample; no code path reads it) |

---

## 9. Test-file redundancy audit (`tests/` + `tests/fixtures/`)

Verification basis: full `cargo test` run in the research worktree — **462 tests, 0 failures** (320 lib unittests + 142 integration). The only ignored tests: `tests/e2e_server_tests.rs` (17, all `#[ignore]`d — deliberate, run manually with `--ignored` per `docs/testing/SERVER_INTEGRATION_TESTS.md`) and 1 ignored doctest.

| # | Finding | Evidence | Severity | Disposition |
|---|---------|----------|----------|-------------|
| TA1 | All 13 `tests/*.rs` targets are discovered and pass; **no test targets are orphaned** (a file in `tests/` subdirs would be skipped by cargo's auto-discovery — `tests/fixtures/` contains only `.lua`, so nothing is hidden). | `cargo test` output: 13 integration targets ran | — | keep |
| TA2 | Three overlapping server test files: `tests/server_integration.rs` (13 unit-style tests of `ServerState`), `tests/server_integration_tests.rs` (3 socket round-trip tests), `tests/e2e_server_tests.rs` (17 `#[ignore]`d CLI/daemon e2e). Names and scope overlap. | file list + `cargo test` results | redundancy (naming/scope) | **needs-decision** — rename/merge (e.g. `server_state_tests.rs` vs `server_e2e_tests.rs`); all are live so nothing to delete outright |
| TA3 | `docs/testing/SERVER_INTEGRATION_TESTS.md` describes `tests/server_integration.rs`'s 13 tests — **all 13 still exist with matching names** (`test_server_state_creation` … `test_no_idle_connections_to_cleanup`) and pass. Doc's e2e count says "16 用例" but the file now has 17 ignored tests — minor drift. Its "后续计划" (follow-up plan) items were implemented under different names (`script_validation_tests.rs`, `port_management_tests.rs`, `benches/`) — the plan section is historical. | doc vs `grep 'async fn test_' tests/server_integration.rs` (13/13 match); `cargo test --test e2e_server_tests` shows 17 | current (minor staleness) | needs-decision — update count/plan lines when touching the doc |
| TA4 | No stale tests: everything compiles against current code and passes; nothing tests renamed/removed APIs (the `cargo build --all-targets` + `cargo test` would surface that as compile errors, which there were none). | `cargo test` all-green | — | keep |
| TA5 | Scaffolding: `tests/lua_sandbox_tests.rs` (20 tests, all pass) — see section 5. Not failing stubs; two TODO comments mark unimplemented `memory_limit_mb`/`timeout_seconds` enforcement. | section 5 + `cargo test` result | backlog markers | needs-decision (see section 5) |
| TA6 | **Unused fixture**: `tests/fixtures/protocols/test_missing_func.lua` has zero references (F3). | grep `missing_func`/`test_missing` across `tests/` | dead | **safe-delete** |
| TA7 | Naming inconsistency: `tests/protocol_manager_test.rs` (singular) vs the `*_tests.rs` convention used by the other 12 files. | `ls tests/` | cosmetic | needs-decision — rename |
| TA8 | Frontend test files (14, co-located: 13 `stores/components/lib/*.test.ts(x)` + `src/lib/mock/mock.test.ts`) run in CI via `pnpm run test` (vitest, `ci.yml:90-92`; `frontend/package.json:10`). Not executed here (no `node_modules` in the worktree — a full `pnpm install` was out of scope for a research pass); static inspection shows they import existing modules. Whether any are redundant/stale is a **frontend-scope** item for the frontend inventory ticket. | `find frontend/src -name '*.test.*'` (14 files); ci.yml frontend-tests job | frontend-scope | hand to frontend inventory |

---

## 10. Mock-layer audit

| # | Layer | Location | Evidence | Verdict | Disposition |
|---|-------|----------|----------|---------|-------------|
| M1 | **Frontend browser mock** (Rust-free `pnpm dev`) | `frontend/src/lib/mock/` (16 files, 1,639 lines: `index.ts` proxy, `interceptor.ts`, `state.ts`, `events.ts`, `dialog.ts`, `mock.test.ts`, 10 `handlers/*.ts`) | **Doc claim verified true**: `frontend/AGENTS.md` ("Mock Layer (pnpm dev without Rust)") describes Vite aliases `@/lib/tauri-api → src/lib/mock/index.ts`, `@tauri-apps/api/event → src/lib/mock/events.ts`, `@tauri-apps/plugin-dialog → src/lib/mock/dialog.ts`, switched on `process.env.TAURI_PLATFORM` — all confirmed in `frontend/vite.config.ts:12-22` (`isTauri = process.env.TAURI_PLATFORM !== undefined`, `:7`). `__MOCK_EMIT__` debug hook exists (`mock/events.ts:48`). Layer is self-tested (`mock.test.ts`, 252 L) and CI runs it. Production-safe: aliases inactive under `cargo tauri dev/build`. Commit history shows it is maintained: `77e8402` "add remote device handlers to the browser mock layer", `288d828` "resolve 9 mock/UI issues…". | **necessary** (per owner: do NOT recommend deleting) | keep — no action; confirm nothing to add |
| M2 | `docs/dev/PDR-MOCK-LAYER.md` (Draft, 2026-07-23) | — | Describes the exact design that is now implemented (proxy + per-domain handlers + in-memory state + `__MOCK_EMIT__`; verified against `mock/index.ts`, `mock/handlers/*`, `mock/events.ts`). Map #75 disposition #1: "Implemented PRD/design docs → delete (after verifying each is truly implemented)". | implemented design doc | **needs-decision** — per map disposition, delete after re-verification, or keep as history if the effort prefers (the canonical doc is `frontend/AGENTS.md`) |
| M3 | **Rust test mock** — `src/serial_core/backends/mock.rs` (`MockSerialPort`, `MockSerialPortBuilder`) | `src/serial_core/backends/mod.rs:8` (`pub mod mock;`) | Live: used by `src/server/rpc.rs` unit tests (`rpc.rs:912-977`), referenced in `port.rs:633` docs. Builder-only API partially dead (see R9). | necessary (test infrastructure) | keep; delete dead builder API (R9) |
| M4 | **Lua test fixtures** | `tests/fixtures/protocols/` (3 files) | See F1-F3: 2 live, 1 unused (F3/TA6). | mixed | safe-delete unused fixture only |
| M5 | Other test mocks | none in `tests/*.rs` | `grep -l 'mock' tests/*.rs` → no matches; no mock RPC client or mock config in tests. `examples/rust/virtual_port_demo.rs` is a runnable demo example, not a mock. | — | keep |

---

## 11. Summary by severity

**safe-delete** (verifiably dead — execution ticket #82 can land autonomously per map disposition #2):
- Deps: `anyhow`, `notify`, `rand` (dev), redundant dev copies of `rustyline` + `libc` (D1-D5).
- Modules/items: `error_handling.rs` (R1), `windows_signals.rs` (R2), `port_script_controller.rs` (R3), `io_loop.rs` (R4), `cli/json.rs` (R5), `ScriptCache` (R6), dead top-level of `utils.rs` (R7), sniffer dead methods (R8), mock builder dead API (R9), bench warning line (B1).
- Lua/fixtures: `tests/fixtures/protocols/test_missing_func.lua` (F3/TA6), `config/default.toml` (E2).
- Doc sync: `docs/dev/ARCH.md:16,27` (error_handling.rs, json.rs) updated in the same change; verify Windows build after R2.

**needs-decision** (ambiguous / refactor / policy):
- Visibility-reduction of pub-but-unused items (P1-P8).
- Library/sandbox policy: lua_sandbox test stubs (section 5 / TA5).
- src-tauri boundary refactors (T1-T6) — behavior-preserving moves, per map "behavior-changing refactors with no redundancy/cleanup win are out of scope" these are boundary-restoration, decide in #82.
- Benches retarget/rename (B2-B5).
- Lua: dual-source-of-truth drift for the 4 core protocols (L5); 12 unreferenced protocol scripts — document+test or remove (LA10-LA20: `can`, `dlt645`, `dmx512`, `i2c_uart`, `mqtt_serial`, `nmea0183`, `onewire`, `pzem004t`, `sdi12`, `spi_uart`, `temp_sensor`); dedup refactors (L1, L4); scanner wart for `modbus_rtu_lib` (L2); examples gallery indexing/trim (E1).
- Tests: server test file consolidation (TA2/TA3), naming (TA7), frontend test audit (TA8 → frontend inventory).
- Misc: `update-packages.sh` (S2), `PDR-MOCK-LAYER.md` deletion per map disposition (M2).

**Not dead (verified live)** — do not touch: all src-tauri deps (D6), `windows` dep (D7), `scripts/templates/` (S1), `scripts/install.*` (S4), `scripts/release.sh` (S3), `utils/hex.rs` + `utils/lua_conversion.rs`, `LuaStatePool`/`acquire_lua`/`release_lua`/`ScriptRuntime`, `SerialSniffer`/`sniffer::start_sniffing`/`capture_rx`, all 5 benches' *targets* (CI runs them), `get_global_config_path`, `statistics`/`ScriptStatistics`, ui_actions module, mock backend type, **the frontend mock layer (M1)** and the 4 embedded built-in scripts + their 4 external override copies (LA1-LA8, product-critical), `modbus_rtu_lib.lua` (LA9), live fixtures `test_valid.lua`/`test_syntax_error.lua` (F1/F2), and — for the record — all 462 backend tests currently pass.

---

## Appendix: commands run (all in the isolated research worktree)

```
cargo --version                                   # 1.94.1, stable only (no nightly → udeps N/A)
cargo-binstall cargo-machete -y                   # cargo-machete v0.9.2
cargo machete / cargo machete --with-metadata     # → anyhow, notify (root crate only)
cargo build --all-targets                         # OK; sole warning benches/lua_execution.rs:20
cargo bench --no-run                              # OK; 5 bench executables produced
cargo test                                        # 462 passed, 0 failed; e2e_server_tests 17 ignored (deliberate)
grep-based dependency + pub-item cross-reference  # see sections 2/3 methodology
line-signature (Jaccard) scan of scripts/protocols/*.lua and examples/*.lua vs scripts/protocols/
word-boundary reference scan for all 16 protocol script names (tests/benches/examples/docs/src)
diff of src/script/built_in/*.lua vs scripts/protocols/*.lua
static verification of frontend mock wiring (vite.config.ts aliases, AGENTS.md claims, mock/events.ts)
```
