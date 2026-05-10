# Serial CLI TODO List

**Version**: v0.5.0
**Updated**: 2026-05-10

---

## Priority Legend

- **P0** - Critical (must fix)
- **P1** - Important (should fix)
- **P2** - Nice to have (can defer)

---

## P1 - Important Issues

### 1. CLI Command Structure Design
**Status**: 🎯 Design decision needed
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

### 2. GUI Frontend Integration
**Status**: 📝 Implementation needed
**Priority**: P1
**Files**: `src-tauri/src/commands/*.rs`

**Current state**:
- GUI backend has 13+ unused function warnings
- Functions are API stubs waiting for frontend integration
- GUI compiles successfully but lacks complete frontend

**Required work**:
- Implement frontend components to use existing backend APIs
- OR remove unused stub functions if not needed
- Complete Tauri + React integration

**Unused functions include**:
- Configuration: `get_config_raw`, `save_config_raw`, `reset_config`
- Protocol: `protocol_encode`, `protocol_decode`, `validate_protocol`
- Events: `emit_port_status_changed`, `emit_error`
- State management: `PortStateManager` struct

---

## P2 - Future Enhancements

### 3. Advanced Performance Optimizations
**Status**: 📋 Deferred to v0.6.0
**Priority**: P2

**Completed optimizations** (v0.5.0):
- ✅ Benchmark infrastructure established
- ✅ Buffer allocation optimization (+179% Modbus RTU encode)
- ✅ Protocol parsing optimization (+82% Modbus ASCII encode)

**Future optimizations**:
- [ ] Zero-copy data transfer (Cow/bytes for protocol payloads)
- [ ] AsyncFd polling optimization
- [ ] Buffer size tuning based on benchmark data
- [ ] Batch read/write optimization
- [ ] Lazy initialization for faster startup
- [ ] Memory pool for buffer reuse

### 4. Enhanced Protocol Features
**Status**: 📋 Future work
**Priority**: P2

**Potential enhancements**:
- Protocol state machine visualization
- Protocol performance profiling
- Custom protocol debugging tools
- Protocol versioning and migration support
- Protocol sandboxing for security

---

## Project Status

**Release Readiness**: v0.5.0 🚢 Ready (pending P1 design decisions)

**Blocking issues**: None
**Known issues**: 2 (both P1, design/implementation decisions)
**Test coverage**: Excellent (212 tests, 100% pass rate)
**Platform support**: 100% (Linux/macOS/Windows)

**Recommendation**: Address P1 items, then proceed with v0.5.0 release.
