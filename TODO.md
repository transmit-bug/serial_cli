# Serial CLI TODO List

**Version**: v0.8.0 (backend hardening)
**Updated**: 2026-05-24

---

## P0 - I/O 架构

### 1. 异步 I/O 重构
**Status**: ✅ Done
**Description**: sniffer 改为 `spawn_blocking` 阻塞读取 + `mpsc` channel 异步事件分发。数据到达即处理，消除 50ms 轮询。

### 2. 协议帧缓冲与流式解析
**Status**: ✅ Done
**Description**: `SerialPortHandle` 新增 `frame_buffer`，跨读取累积数据后再调用 `parse()`。64KB 安全上限。

---

## P1 - 可靠性与健壮性

### 3. 端口热插拔检测
**Status**: ✅ Done
**Description**: 后台任务每 2s 轮询硬件端口列表变化，自动发射 `ports-changed` 事件到前端。

### 4. 优雅关闭与资源清理
**Status**: ✅ Done
**Description**: Tauri `on_window_event(CloseRequested)` 依次停止 sniffers → 关闭端口 → 停止虚拟端口对。

### 5. 端口并发与锁优化
**Status**: ✅ Done
**Description**: 修复 `open_port` 重复打开检查（遍历 values 匹配 port name）。sniffer 不再每次读取都锁 PortManager。

---

## P2 - 协议与脚本引擎

### 6. 协议热重载实现
**Status**: ✅ Done
**Description**: `enable_hot_reload` 真正创建 `ProtocolWatcher`，监视目录并自动重新加载。

### 7. Lua 引擎统一
**Status**: ✅ Done
**Description**: `LuaProtocol` 缓存 Lua VM（首次调用初始化，后续复用），支持跨调用状态保持。修复 `virtual_create` bug（改为返回明确错误）。

### 8. Lua 脚本 API 扩展
**Status**: ✅ Done
**Description**: `execute_script` 改为 `spawn_blocking` 执行，避免 `sleep_ms` 阻塞 tokio 异步线程。

---

## P2 - 数据管理与导出

### 9. 会话持久化与数据导出
**Status**: ✅ Done
**Description**: 新增 `export_data` Tauri 命令，支持 TXT/CSV/JSON 格式导出数据到文件。

### 10. GUI 日志系统
**Status**: ✅ Done
**Description**: 日志双写到 `~/.local/share/serial-cli/logs/serial-cli.log`（INFO+）和 stderr（WARN+）。

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
- ✅ Tauri backend commands (48 registered)
- ✅ 227 Rust unit tests + 9 E2E tests passing

### ✅ v0.7.0 - Frontend + Backend Integration
- ✅ React 19 + Vite 8 + TypeScript + Tailwind CSS 4 + Zustand 5
- ✅ 串口终端 + 虚拟串口管理 + 脚本引擎 + 协议管理 + 设置面板
- ✅ i18n 中英文支持 + 可折叠侧边栏 + 多标签工作区

### ✅ v0.8.0 - Backend Hardening
- ✅ 异步 I/O：spawn_blocking + mpsc channel 替代 50ms 轮询
- ✅ 协议帧缓冲：跨读取累积数据，解决部分帧静默丢弃
- ✅ 端口热插拔：2s 轮询检测端口变化，发射 ports-changed 事件
- ✅ 优雅关闭：CloseRequested 拦截，有序清理所有资源
- ✅ 锁优化：修复 open_port 重复检查，sniffer 解耦 PortManager
- ✅ 协议热重载：ProtocolWatcher 接入 ProtocolManager
- ✅ LuaProtocol VM 缓存：消除每次回调重建 Lua state
- ✅ Lua execute_script 安全化：spawn_blocking 执行
- ✅ 数据导出：export_data 命令（TXT/CSV/JSON）
- ✅ GUI 日志：双写文件 + stderr
