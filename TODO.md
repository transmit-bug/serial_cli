# Serial CLI TODO List

**Version**: v0.7.0 (frontend rewrite)
**Updated**: 2026-05-24

---

## Priority Legend

- **P0** - Critical (must fix before release)
- **P1** - Important (should fix)
- **P2** - Nice to have (can defer)

---

## P0 - Frontend Rewrite

### 1. 初始化新前端项目
**Status**: ⏳ Pending
**Description**: 初始化 React + Vite + TypeScript 前端项目，配置 Tailwind CSS、shadcn/ui、i18next、Zustand。

### 2. 主界面 - 串口终端
**Status**: ⏳ Pending
**Description**: 串口连接、数据收发、HEX/ASCII/Mixed 显示、数据导出。承载 80% 日常功能。

### 3. 虚拟串口管理
**Status**: ⏳ Pending
**Description**: 虚拟串口对创建/管理/数据捕获。

### 4. 脚本引擎
**Status**: ⏳ Pending
**Description**: Lua 脚本编辑器（Monaco）、执行、管理、脚本绑定。

### 5. 协议管理
**Status**: ⏳ Pending
**Description**: 协议列表、启停、编解码、自定义协议加载。

### 6. 设置面板
**Status**: ⏳ Pending
**Description**: 应用配置管理。

---

## P1 - Tauri Backend Fixes

### 7. 修复双读循环 bug
**Status**: ⏳ Pending
**Files**: `src-tauri/src/commands/port.rs`, `src-tauri/src/commands/serial.rs`
**Description**: `open_port` 和 `start_sniffing` 同时在端口上起读任务，造成数据竞争。需要统一为一个数据读取通道。

### 8. 完善事件发射器
**Status**: ⏳ Pending
**Files**: `src-tauri/src/events/emitter.rs`
**Description**: `emit_port_status_changed`、`emit_error`、`emit_virtual_port_stats_updated` 已移除 dead_code 标记，需要在对应命令中实际调用。

---

## P2 - Future Enhancements

### 9. 系统托盘
**Status**: ⏳ Pending
**Description**: 实现系统托盘功能（已移除空壳代码，待重新实现）。

### 10. UI/UX 文档
**Status**: ⏳ Pending
**Description**: 编写前端 UI/UX 设计文档。

---

## Completed Items

### ✅ v0.6.0 - CLI + Backend
- ✅ Port operation (list, send, receive, health check)
- ✅ Protocol management (list, load, unload, validate, reload, get_info)
- ✅ LuaJIT scripting (engine, cache, pool, execute, validate, list, save, delete)
- ✅ Sniff mode (start, stop, stats, save)
- ✅ Batch operations
- ✅ Virtual serial ports (PTY, socat, named_pipe)
- ✅ Server Mode MVP (JSON-RPC 2.0)
- ✅ Configuration management (TOML-based)
- ✅ Interactive shell (rustyline)
- ✅ Tauri backend commands (36 registered)
- ✅ 229 Rust unit tests + 9 E2E tests passing

### ✅ Cleanup (2026-05-24)
- ✅ 删除旧 frontend/ 目录
- ✅ 移除 PortStateManager 死代码
- ✅ 移除 tray.rs 空壳
- ✅ 清理 config.rs dead_code stubs (get_config_raw, save_config_raw, get_config_file_path)
- ✅ 清理 script.rs dead_code stub (load_script)
- ✅ 移除 emitter.rs dead_code 标记
- ✅ 移除 virtual_port.rs dead_code 标记
- ✅ 移除 tauri tray-icon feature
- ✅ 删除过时文档 (UI-DESIGN-DECISIONS.md, FINAL_REPORT.md, TEST_REPORT.md)
- ✅ 重写 TODO.md
