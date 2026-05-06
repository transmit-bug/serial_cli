# Serial CLI TODO List

**Version**: v0.5.0
**Updated**: 2026-05-07

---

## Priority Legend

- **P0** - Critical (must fix)
- **P1** - Important (should fix)
- **P2** - Nice to have (can defer)

---

## P0 - Critical Issues

### 1. CLI Output Mixing Bug — Refactor Regression
**Status**: 🐛 Found, needs fix
**Priority**: P0
**Files**: `src/cli/commands/*.rs`

**Problem**: During CLI architecture refactor (commit 29b513a), the fix from commit 6bd7e02 was not applied to new command files. User-visible output uses `tracing::info!` instead of `println!`, causing log metadata to pollute command output.

**Root cause**:
- 2026-04-15: Fixed output mixing (principle: user output → `println!`, logging → `tracing::info!`)
- 2026-04-17: CLI refactor deleted fixed files, created new modular structure
- New files didn't follow the established principle

**Affected files**:
- `src/cli/commands/virtual_port.rs` (lines 121-144): List output uses `tracing::info!`
- `src/cli/commands/sniff.rs` (lines 51-59): Start command uses `tracing::info!`
- `src/cli/commands/ports.rs` (send_data): Mixed use of both

**Fix**: Replace all user-visible output with `println!`, keep only debugging logs as `tracing::info!`

---

## P1 - Important Issues

### 2. CLI Command Structure Design
**Status**: 🎯 Design needed
**Priority**: P1
**Scope**: CLI interface

**Current state**:
- `list-ports` - List serial ports
- `protocol list` - List protocols
- `virtual list` - List virtual ports

**Proposed improvement**: Shorter, more intuitive commands
- Option A: `list ports`, `list protocols`, `list virtual`
- Option B: Keep current but add `list` as alias for `list-ports`

**Decision needed**: Choose unified command structure before implementation

### 3. GUI Compilation Errors
**Status**: 🐛 Known issue
**Priority**: P1
**Files**: `src-tauri/src/commands/*.rs`

**Errors**:
- 9 warnings (unused imports/variables)
- 1 compilation error in `serial.rs`

**Impact**: GUI cannot be built, but CLI works perfectly

---

## P2 - Future Enhancements

### 4. Performance Optimization (v0.5.0)
**Status**: 🚧 Partial

**Completed**:
- [x] Benchmark infrastructure
- [x] I/O throughput benchmarks
- [x] Protocol parsing benchmarks
- [x] Startup time benchmarks
- [x] Memory usage benchmarks
- [x] Concurrency benchmarks
- [x] Buffer allocation optimization
  - Modbus RTU encode: +179% (70 → 197 MB/s)
  - Modbus ASCII encode: +82% (97 → 176 MB/s)
  - Modbus ASCII parse: +17% (225 → 263 MB/s)

**Pending**:
- [ ] Zero-copy data transfer optimization (Cow/bytes for protocol payloads)
- [ ] AsyncFd polling optimization
- [ ] Buffer size tuning based on benchmark data
- [ ] Batch read/write optimization
- [ ] Lazy initialization for faster startup
- [ ] Memory pool for buffer reuse

---

## Completed (v0.4.0 - v0.5.0)

All features from v0.4.0 are fully implemented and tested:
- ✅ Serial port management (cross-platform, DTR/RTS signals)
- ✅ Protocol engine (Modbus RTU/ASCII, AT Commands, Line-based, custom Lua)
- ✅ Lua scripting (LuaJIT, async, protocol extension)
- ✅ Virtual serial ports (PTY, NamedPipe, Socat backends)
- ✅ Data sniffing (session management, packet capture)
- ✅ Batch processing (variables, loops, error reporting)
- ✅ Benchmarking (comprehensive performance tests)
- ✅ Configuration management (TOML, validation, persistence)
- ✅ CLI commands (all core functionality)
- ✅ 212 tests passing

---

## Progress Summary

| Category | Total | Completed | Partial | TODO |
|----------|-------|-----------|---------|------|
| P0 - Critical | 1 | 0 | 0 | 1 |
| P1 - Important | 2 | 0 | 0 | 2 |
| P2 - Future | 1 | 1 | 0 | 0 |
| **Total** | **4** | **1** | **0** | **3** |

**Overall Progress**: 25% complete, critical bugs found post-refactor

---

## Implementation Plan

### Phase 1 (P0 - Critical Fixes) — Current
1. 🔄 Fix CLI output mixing in all command files
2. 🔄 Verify all user output uses `println!`
3. 🔄 Test with `--verbose` and `RUST_LOG` enabled

### Phase 2 (P1 - Important) — Next
1. Design CLI command structure (get user input on preferred approach)
2. Fix GUI compilation errors
3. Implement agreed-upon command structure

### Phase 3 (P2 - Optimization)
1. Continue performance optimizations based on benchmark data
