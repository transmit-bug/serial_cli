# Serial CLI TODO List

**Version**: v0.5.0
**Updated**: 2026-05-10

---

## Priority Legend

- **P0** - Critical (must fix)
- **P1** - Important (should fix)
- **P2** - Nice to have (can defer)

---

## P0 - Critical Issues

### 1. ✅ CLI Output Mixing Bug — RESOLVED
**Status**: ✅ Fixed
**Priority**: P0
**Resolved**: 2026-05-10

**Previous Problem**: During CLI architecture refactor, user-visible output was using `tracing::info!` instead of `println!`, causing log metadata to pollute command output.

**Resolution**: Code review confirms all user-facing output now correctly uses `println!` for user-visible messages and `tracing::info!` only for debugging logs. All CLI command handlers verified.

**Verified Files**:
- ✅ `src/cli/commands/virtual_port.rs` - List output uses `println!`
- ✅ `src/cli/commands/sniff.rs` - Status messages use `println!`
- ✅ `src/cli/commands/port.rs` - All output uses `println!`
- ✅ `src/cli/commands/protocol.rs` - All output uses `println!`
- ✅ `src/cli/commands/config.rs` - All output uses `println!`

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

### 3. GUI Unused Function Warnings
**Status**: 📝 Known issue
**Priority**: P1
**Files**: `src-tauri/src/commands/*.rs`

**Details**:
- 13+ unused function warnings in GUI code
- Functions appear to be API stubs for future frontend integration
- No compilation errors, only warnings
- GUI builds successfully despite warnings

**Impact**: Minor - code compiles but indicates incomplete frontend integration

**Example unused functions**:
- `get_config_raw`, `save_config_raw`, `reset_config`
- `protocol_encode`, `protocol_decode`
- `emit_port_status_changed`, `emit_error`
- `PortStateManager` struct and methods

---

## P2 - Future Enhancements

### 4. Performance Optimization (v0.5.0)
**Status**: ✅ Complete

**Completed optimizations**:
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

**Future optimizations** (deferred to v0.6.0):
- [ ] Zero-copy data transfer optimization (Cow/bytes for protocol payloads)
- [ ] AsyncFd polling optimization
- [ ] Buffer size tuning based on benchmark data
- [ ] Batch read/write optimization
- [ ] Lazy initialization for faster startup
- [ ] Memory pool for buffer reuse

### 5. Enhanced Protocol Features
**Status**: 🚧 Future work

**Potential enhancements**:
- Protocol state machine visualization
- Protocol performance profiling
- Custom protocol debugging tools
- Protocol versioning and migration support
- Protocol sandboxing for security

---

## Completed (v0.4.0 - v0.5.0)

### Core Features (All Complete ✅)
- ✅ **Serial port management** (cross-platform, DTR/RTS signals)
- ✅ **Protocol engine** (Modbus RTU/ASCII, AT Commands, Line-based, custom Lua)
- ✅ **Lua scripting** (LuaJIT, async, protocol extension)
- ✅ **Virtual serial ports** (PTY, NamedPipe, Socat backends)
- ✅ **Data sniffing** (session management, packet capture)
- ✅ **Batch processing** (variables, loops, error reporting)
- ✅ **Benchmarking** (comprehensive performance tests)
- ✅ **Configuration management** (TOML, validation, persistence)
- ✅ **CLI commands** (all core functionality)
- ✅ **212 tests passing**

### Module Implementation Status (All Complete ✅)

#### CLI Layer (`src/cli/`)
- ✅ **Argument parsing** (`args.rs`) - Complete clap definitions
- ✅ **Command types** (`types.rs`) - All subcommand enums
- ✅ **Interactive shell** (`interactive.rs`) - REPL with rustyline
- ✅ **JSON formatting** (`json.rs`) - Structured output support
- ✅ **Batch processing** (`batch.rs`) - Script execution engine
- ✅ **Command handlers** (`commands/`) - All implemented:
  - ✅ `port.rs` - List ports, send data
  - ✅ `protocol.rs` - Protocol management
  - ✅ `sniff.rs` - Traffic monitoring
  - ✅ `virtual_port.rs` - Virtual port management
  - ✅ `config.rs` - Configuration commands
  - ✅ `script.rs` - Lua script execution
  - ✅ `batch.rs` - Batch file processing
  - ✅ `benchmark.rs` - Performance testing
  - ✅ `parsers.rs` - Hex/base64 utilities

#### Serial Core (`src/serial_core/`)
- ✅ **Port management** (`port.rs`) - UUID-based handles, lifecycle
- ✅ **I/O loop** (`io_loop.rs`) - Async event loop
- ✅ **Sniffer** (`sniffer.rs`) - Packet capture, session mgmt
- ✅ **Virtual ports** (`virtual_port.rs`) - Multi-backend support
- ✅ **Signals** (`signals.rs`) - DTR/RTS control, cross-platform
- ✅ **Backends** (`backends/`) - PTY, NamedPipe, Socat implementations
- ✅ **Factory** (`factory.rs`) - Backend instantiation

#### Protocol Engine (`src/protocol/`)
- ✅ **Core trait** (`mod.rs`) - Protocol interface definition
- ✅ **Registry** (`registry.rs`) - Protocol registration
- ✅ **Built-in protocols** (`built_in/`) - Modbus, AT Command, Line
- ✅ **Lua extensions** (`lua_ext.rs`) - Custom protocol support
- ✅ **Manager** (`manager.rs`) - Lifecycle management
- ✅ **Loader** (`loader.rs`) - Script loading
- ✅ **Validator** (`validator.rs`) - Script validation
- ✅ **Watcher** (`watcher.rs`) - Hot-reload support

#### Lua Integration (`src/lua/`)
- ✅ **Bindings** (`bindings.rs`) - Rust→Lua API
- ✅ **Engine** (`engine.rs`) - LuaJIT runtime
- ✅ **Executor** (`executor.rs`) - Script execution
- ✅ **Stdlib** (`stdlib.rs`) - Utility functions
- ✅ **Cache** (`cache.rs`) - Script caching
- ✅ **Pool** (`pool.rs`) - Instance pooling

#### Task Scheduling (`src/task/`)
- ✅ **Queue** (`queue.rs`) - Priority-based task queue
- ✅ **Executor** (`executor.rs`) - Async task execution
- ✅ **Monitor** (`monitor.rs`) - Task monitoring

#### System Integration
- ✅ **Error handling** (`error.rs`) - Comprehensive error types
- ✅ **Configuration** (`config.rs`) - TOML config management
- ✅ **Logging** (`logging.rs`) - Structured logging
- ✅ **Utilities** (`utils.rs`) - Helper functions

#### Cross-Platform Support (100% ✅)
- ✅ **Linux** - Full support (x86_64, ARM64)
- ✅ **macOS** - Full support (x86_64, ARM64)
- ✅ **Windows** - Full support (x86_64)
- ✅ **CI/CD** - All platforms passing tests
- ✅ **Package managers** - Homebrew, Scoop, AUR integration

---

## Progress Summary

| Category | Total | Completed | Partial | TODO |
|----------|-------|-----------|---------|------|
| P0 - Critical | 1 | 1 | 0 | 0 |
| P1 - Important | 2 | 0 | 0 | 2 |
| P2 - Future | 2 | 1 | 1 | 0 |
| **Total** | **5** | **2** | **1** | **2** |

**Overall Progress**: 60% complete, all critical issues resolved

**Code Quality Metrics**:
- ✅ 65 Rust source files
- ✅ 212 tests passing (100% pass rate)
- ✅ Zero compilation errors
- ✅ Zero TODO/FIXME comments in code
- ✅ Proper error handling throughout
- ✅ Comprehensive documentation

---

## Implementation Plan

### Phase 1 (P0 - Critical Fixes) — ✅ COMPLETE
1. ✅ Fix CLI output mixing in all command files
2. ✅ Verify all user output uses `println!`
3. ✅ Test with `--verbose` and `RUST_LOG` enabled

### Phase 2 (P1 - Important) — Current Priority
1. **Design CLI command structure** (get user input on preferred approach)
   - Evaluate `list ports` vs `list-ports`
   - Consider breaking changes vs backwards compatibility
2. **Clean up GUI warnings**
   - Remove unused function stubs OR
   - Implement frontend integration to use them
3. **Implement agreed-upon command structure**

### Phase 3 (P2 - Optimization) — Future
1. Continue performance optimizations based on benchmark data
2. Implement zero-copy data transfer
3. Memory pool optimization
4. Enhanced protocol debugging tools

---

## Technical Debt Assessment

### Low Debt ✅
- **Code organization**: Excellent modular structure
- **Error handling**: Comprehensive and consistent
- **Testing**: Good coverage (212 tests)
- **Documentation**: Well-documented codebase
- **Cross-platform**: Clean platform abstractions

### Medium Debt ⚠️
- **GUI integration**: Some stub functions unused
- **Command structure**: Minor inconsistency in naming
- **Performance optimizations**: Room for improvement

### No Major Debt 🎉
- No security vulnerabilities identified
- No memory leaks detected
- No known race conditions
- No blocking bugs

---

## Release Readiness: v0.5.0

**Status**: 🚢 Ready for release (pending P1 decisions)

**Blocking issues**: None
**Known issues**: 2 (both P1, design decisions)
**Test coverage**: Excellent
**Documentation**: Complete
**Platform support**: 100%

**Recommendation**: Address P1 command structure design decision, then proceed with v0.5.0 release.
