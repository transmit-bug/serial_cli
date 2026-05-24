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
**Status**: ✅ Done
**Description**: 初始化 React + Vite + TypeScript 前端项目，配置 Tailwind CSS、shadcn/ui、i18next、Zustand。

### 2. 主界面 - 串口终端
**Status**: ✅ Done
**Description**: 串口连接、数据收发、HEX/ASCII/Mixed 显示、数据导出。承载 80% 日常功能。

### 3. 虚拟串口管理
**Status**: ✅ Done
**Description**: 虚拟串口对创建/管理/数据捕获。

### 4. 脚本引擎
**Status**: ✅ Done
**Description**: Lua 脚本编辑器（Monaco）、执行、管理、脚本绑定。

### 5. 协议管理
**Status**: ✅ Done
**Description**: 协议列表、启停、编解码、自定义协议加载。

### 6. 设置面板
**Status**: ✅ Done
**Description**: 应用配置管理。

---

## P1 - Backend Fixes (前后端审查)

### 7. 修复 `get_all_ports_status` 实现不完整
**Status**: ✅ Done
**Files**: `src/serial_core/port.rs`, `src-tauri/src/commands/port.rs`
**Description**: 新增 `PortManager::list_open_ports()` 方法枚举已打开端口。重写 `get_all_ports_status` 查询实际已打开端口状态，合并 `port_stats` 统计和 config。

### 8. 修复 `get_config`/`update_config` display 字段硬编码
**Status**: ✅ Done
**Files**: `src/config.rs`, `src-tauri/src/commands/config.rs`
**Description**: 在核心 Config 中新增 `DisplayConfig`（theme/format/max_packets/show_timestamp），`get_config` 从真实字段读取，`update_config` 回写 display 字段。

### 9. 完善事件发射器
**Status**: ✅ Done
**Files**: `src-tauri/src/events/emitter.rs`, `src-tauri/src/commands/port.rs`, `src-tauri/src/commands/serial.rs`, `src-tauri/src/commands/virtual_port.rs`
**Description**: 已在对应场景中调用：
  - `emit_port_status_changed`: open_port/close_port 时发射
  - `emit_error`: sniffer 端口断开时发射
  - `emit_virtual_port_stats_updated`: get_virtual_port_stats 时发射

---

## P2 - Integration & Polish

### 10. 接入 `checkVirtualPortHealth`
**Status**: ⏳ Pending
**Files**: `frontend/src/lib/tauri-api.ts`, `frontend/src/components/virtual/VirtualPortsPage.tsx`
**Description**: API 已定义但未使用。应在虚拟端口页面的端口列表中展示健康状态。

### 11. 接入 standalone script actions 系统
**Status**: ⏳ Pending
**Files**: `frontend/src/lib/tauri-api.ts`, `frontend/src/components/editor/EditorPage.tsx`
**Description**: `listStandaloneScriptActions` / `callStandaloneScriptFunction` 已在后端实现，前端 API 已封装，但没有 UI 入口。可接入编辑器中，让未绑定端口的脚本也能展示和调用 UI 动作。

### 12. 清理未使用的 API 包装
**Status**: ⏳ Pending
**Files**: `frontend/src/lib/tauri-api.ts`
**Description**: `getProtocolInfo`、`hasScript` 已定义但无调用方。要么接入功能，要么移除死代码。

### 13. 连接预设同步到后端配置
**Status**: ⏳ Pending
**Files**: `frontend/src/components/terminal/ConnectionBar.tsx`, `src-tauri/src/commands/config.rs`
**Description**: 连接预设仅存 localStorage，不同步到后端 Config。需扩展 Config 结构，使预设可跨设备共享。

### 14. 系统托盘
**Status**: ⏳ Pending
**Description**: 实现系统托盘功能（已移除空壳代码，待重新实现）。

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
- ✅ Tauri backend commands (45 registered)
- ✅ 229 Rust unit tests + 9 E2E tests passing

### ✅ v0.7.0 - Frontend Rewrite
- ✅ React 19 + Vite 8 + TypeScript + Tailwind CSS 4 + Zustand 5
- ✅ 串口终端 (连接/收发/HEX-ASCII-Mixed/导出/命令/监控/解码器)
- ✅ 虚拟串口管理 (创建/停止/捕获/CSV导出)
- ✅ 脚本引擎 (Monaco编辑器/模板/执行/验证/绑定端口)
- ✅ 协议管理 (列表/加载/卸载/编解码测试)
- ✅ 设置面板 (8标签页/配置读写/重置)
- ✅ i18n 中英文支持
- ✅ 可折叠侧边栏 + 多标签工作区

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
