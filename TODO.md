# Serial CLI TODO List

**Version**: v0.5.0
**Updated**: 2026-05-13

---

## Priority Legend

- **P0** - Critical (must fix before release)
- **P1** - Important (should fix)
- **P2** - Nice to have (can defer)

---

## P1 - Important

### 1. SettingsPanel 配置持久化到后端
**Status**: 🔴 TODO
**Priority**: P1
**Files**: `frontend/src/components/settings/SettingsPanel.tsx`, `frontend/src/contexts/SettingsContext.tsx`
**Issue**: SettingsPanel 使用 `SettingsContext` 仅存储到 localStorage，不调用后端 `get_config`/`update_config`/`reset_config`。用户修改的串口参数（波特率、校验等）不会写入 `~/.serial-cli.toml`。
**Fix**: SettingsPanel 改用 `useSettingsStore`（Zustand store）直接调用后端配置命令，或修改 SettingsContext 内部使用 store。

### 2. SidePanel 协议卡片连接 protocolStore
**Status**: 🔴 TODO
**Priority**: P1
**Files**: `frontend/src/components/terminal/SidePanel.tsx`, `frontend/src/stores/protocolStore.ts`
**Issue**: SidePanel 协议卡片硬编码"无活动协议"，未连接到 `useProtocolStore`。
**Fix**: 从 `useProtocolStore` 读取当前激活协议名称并展示。

### 3. Protocol encode/decode 命令注册
**Status**: 🔴 TODO
**Priority**: P1
**Files**: `src-tauri/src/main.rs`, `src-tauri/src/commands/protocol.rs`
**Issue**: `protocol_encode`/`protocol_decode` 已实现但未注册到 invoke_handler，前端无法使用协议编解码功能。
**Fix**: 注册到 main.rs，并移除 `#[allow(dead_code)]`。

---

## P2 - Future Enhancements

### 4. 性能监控模块接入
**Status**: 📋 v0.6.0
**Priority**: P2
**Files**: `src/monitoring/`
**Issue**: `PerformanceMonitor`/`ResourceMonitor` 完整实现但 `#[allow(dead_code)]`。

### 5. 任务调度模块接入
**Status**: 📋 v0.6.0
**Priority**: P2
**Files**: `src/task/`
**Issue**: `TaskExecutor`/`TaskQueue`/`TaskMonitor` 完整实现但 `#[allow(dead_code)]`。

### 6. 事件发射器补全
**Status**: 📋 v0.6.0
**Priority**: P2
**Files**: `src-tauri/src/events/emitter.rs`
**Issue**: `emit_port_status_changed`/`emit_error`/`emit_virtual_port_stats_updated` 未被调用。

### 7. Config 原始操作 API
**Status**: 📋 v0.6.0
**Priority**: P2
**Files**: `src-tauri/src/commands/config.rs`
**Issue**: `get_config_raw`/`save_config_raw`/`get_config_file_path` `#[allow(dead_code)]`。

---

## Completed Items (v0.5.0)

### ✅ CLI Core Features
- ✅ Port operation (list, send, receive, health check)
- ✅ Protocol management (list, load, unload, validate, reload, get_info)
- ✅ LuaJIT scripting (engine, cache, pool, execute, validate, list, save, delete)
- ✅ Sniff mode (start, stop, stats, save)
- ✅ Batch operations
- ✅ Virtual serial ports (PTY, socat, named_pipe) — full CRUD + stats + capture
- ✅ Server Mode MVP (JSON-RPC 2.0, 10 methods, E2E tested)
- ✅ Server response line-based framing (`\n` separator)
- ✅ Benchmark infrastructure established
- ✅ Configuration management (TOML-based, get/update/reset)
- ✅ Interactive shell (rustyline)

### ✅ GUI (Tauri + React)
- ✅ UI component library (shadcn/ui) — 33 components
- ✅ Zustand stores (8 stores) + 34 unit tests
- ✅ React contexts (6 contexts)
- ✅ 38 Tauri commands registered in invoke_handler
- ✅ TX 数据发送 — TxSender 调用 `invoke('send_data')`
- ✅ RX 数据接收 — 事件驱动，Hex/ASCII/Mixed 格式显示
- ✅ 虚拟串口管理 — 创建/列表/停止/统计/抓包全通
- ✅ 脚本编辑 — Monaco 编辑器 + 执行
- ✅ 协议管理 — 内置协议切换 + 自定义协议加载/卸载/校验
- ✅ 端口详情显示 — 端口名/波特率/数据位/停止位/校验位
- ✅ 数据统计 — RX/TX 包数和字节数实时显示
- ✅ 快捷操作 — 清空 RX/TX、导出数据
- ✅ 导航系统 — 5 视图 + 键盘快捷键 + 命令面板
- ✅ 设置面板 — 5 Tab UI（localStorage，待接入后端）
- ✅ 系统托盘 + 通知系统

### ✅ Tauri Backend Commands (38 registered)
- ✅ Port: `list_ports`, `open_port`, `close_port`, `get_port_status`, `get_all_ports_status`, `check_port_health`
- ✅ Serial: `send_data`, `read_data`, `start_sniffing`, `stop_sniffing`
- ✅ Protocol: `list_protocols`, `load_protocol`, `unload_protocol`, `reload_protocol`, `validate_protocol`, `get_protocol_info`
- ✅ Script: `execute_script`, `validate_script`, `list_scripts`, `save_script`, `delete_script`
- ✅ Config: `get_config`, `update_config`, `reset_config`
- ✅ Window: `show_window`, `hide_window`, `toggle_window`
- ✅ Virtual Port: `create_virtual_port`, `list_virtual_ports`, `stop_virtual_port`, `get_virtual_port_stats`, `check_virtual_port_health`, `get_captured_packets`

### ✅ Tests
- ✅ 229 unit tests passing
- ✅ 9 E2E tests passing
- ✅ 34 frontend store tests passing
- ✅ 3 integration tests passing
- ✅ 4 doc tests passing

### ✅ CI/CD Pipeline (4 phases)
- ✅ Phase 1a: Rust quality (clippy -- -D warnings, fmt check)
- ✅ Phase 1b: Frontend tests (Vitest, 34/34 通过)
- ✅ Phase 2: Unit tests (229/229 通过)
- ✅ Phase 3: Integration + E2E tests (9/9 通过)
- ✅ Code coverage reporting (cargo-llvm-cov)
- ✅ Security scanning (audit, cargo-deny, trivy)

---

## Project Status

**Release Readiness**: v0.5.0 🚢 Ready (CLI core + Server MVP + GUI functional)

**Blocking issues**: None
**P1 gaps**: 3 (Settings 持久化, Protocol 卡片, Protocol 编解码注册)
**Test coverage**: Excellent (275 total tests, 100% pass rate)
**Platform support**: Linux/macOS/Windows
**CI/CD**: Production-grade four-stage pipeline
