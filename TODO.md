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

### 1. ProtocolPanel 硬编码协议列表
**Status**: 📋 Planned
**Priority**: P1
**Files**: `frontend/src/components/protocols/ProtocolPanel.tsx`
**Issue**: 内置协议列表硬编码在组件中，未调用 `list_protocols()` 后端命令获取实际可用协议。
**Fix**: 首次渲染时调用 `list_protocols()` 加载内置协议元数据，保留硬编码作为 fallback。

### 2. `validate_script` 前端 UI
**Status**: 📋 Planned
**Priority**: P1
**Files**: `frontend/src/components/scripting/ScriptPanel.tsx`
**Issue**: 后端 `validate_script` 命令已注册但 ScriptPanel 无校验按钮。
**Fix**: 在工具栏添加 "Validate" 按钮，调用 `invoke('validate_script', { script })` 并展示 ValidationError 结果。

### 3. 协议 encode/decode UI
**Status**: 📋 Planned
**Priority**: P1
**Files**: `frontend/src/components/terminal/TxSender.tsx`, `frontend/src/components/terminal/RxDataViewer.tsx`
**Issue**: `protocol_encode`/`protocol_decode` 已注册但无 UI 入口。
**Fix**: TxSender 添加编码选项（发送前通过选中协议编码），RxDataViewer 添加解码显示选项。

---

## P2 - Future Enhancements

### 4. 性能监控模块接入
**Status**: 📋 v0.6.0
**Priority**: P2
**Files**: `src/monitoring/`

### 5. 任务调度模块接入
**Status**: 📋 v0.6.0
**Priority**: P2
**Files**: `src/task/`

### 6. 事件发射器补全
**Status**: 📋 v0.6.0
**Priority**: P2
**Files**: `src-tauri/src/events/emitter.rs`

### 7. Config 原始操作 API
**Status**: 📋 v0.6.0
**Priority**: P2
**Files**: `src-tauri/src/commands/config.rs`

### 8. 后端统计接入 SidePanel
**Status**: 📋 v0.6.0
**Priority**: P2
**Files**: `frontend/src/components/terminal/SidePanel.tsx`
**Issue**: 统计数据纯前端计算，未调用 `get_port_status` 获取后端真实字节计数。

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
- ✅ Zustand stores (8 stores) + 34+ unit tests
- ✅ React contexts (6 contexts)
- ✅ TX 数据发送 — TxSender 调用 `invoke('send_data')`
- ✅ RX 数据接收 — 事件驱动，Hex/ASCII/Mixed 格式显示
- ✅ 虚拟串口管理 — 创建/列表/停止/统计/抓包全通
- ✅ 脚本编辑 — Monaco 编辑器 + 执行
- ✅ 协议管理 — 内置协议切换 + 自定义协议加载/卸载/校验（文件正确保存到后端）
- ✅ 端口详情显示 — 端口名/波特率/数据位/停止位/校验位
- ✅ 数据统计 — RX/TX 包数和字节数实时显示
- ✅ 快捷操作 — 清空 RX/TX、导出数据
- ✅ 导航系统 — 5 视图 + 键盘快捷键 + 命令面板
- ✅ 设置面板 — 5 Tab UI + 后端配置持久化
- ✅ 协议卡片 — SidePanel 显示当前活动协议
- ✅ 侧边导航 — "打开设置" 按钮正确导航
- ✅ 自定义协议文件 — `save_protocol_file` 后端命令，绝对路径传递给 validate/load

### ✅ Tauri Backend Commands (36 registered)
- ✅ Port: `list_ports`, `open_port`, `close_port`, `get_port_status`, `get_all_ports_status`, `check_port_health`
- ✅ Serial: `send_data`, `read_data`, `start_sniffing`, `stop_sniffing`
- ✅ Protocol: `list_protocols`, `load_protocol`, `unload_protocol`, `reload_protocol`, `validate_protocol`, `protocol_encode`, `protocol_decode`, `save_protocol_file`, `get_protocol_info`
- ✅ Script: `execute_script`, `validate_script`, `list_scripts`, `save_script`, `delete_script`
- ✅ Config: `get_config`, `update_config`, `reset_config`
- ✅ Window: `show_window`, `hide_window`, `toggle_window`
- ✅ Virtual Port: `create_virtual_port`, `list_virtual_ports`, `stop_virtual_port`, `get_virtual_port_stats`, `check_virtual_port_health`, `get_captured_packets`

### ✅ Tests
- ✅ 229 unit tests passing
- ✅ 9 E2E tests passing
- ✅ 34+ frontend store tests passing
- ✅ 3 integration tests passing
- ✅ 4 doc tests passing

### ✅ CI/CD Pipeline (4 phases)
- ✅ Phase 1a: Rust quality (clippy -- -D warnings, fmt check)
- ✅ Phase 1b: Frontend tests (Vitest)
- ✅ Phase 2: Unit tests
- ✅ Phase 3: Integration + E2E tests
- ✅ Code coverage reporting (cargo-llvm-cov)
- ✅ Security scanning (audit, cargo-deny, trivy)

---

## Project Status

**Release Readiness**: v0.5.0 🚢 Ready

**Blocking issues**: None
**P1 gaps**: 3 (ProtocolPanel 硬编码列表, validate_script UI, encode/decode UI)
**Test coverage**: Excellent (275+ total tests, 100% pass rate)
**Platform support**: Linux/macOS/Windows
**CI/CD**: Production-grade four-stage pipeline
