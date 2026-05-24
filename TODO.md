# Serial CLI TODO List

**Version**: v0.8.0 (backend hardening)
**Updated**: 2026-05-24

---

## P0 - I/O 架构

### 1. 异步 I/O 重构
**Status**: ✅ Done
**Files**: `src-tauri/src/commands/serial.rs`, `src-tauri/src/state/app_state.rs`
**Description**: sniffer 改为 `spawn_blocking` 阻塞读取 + `mpsc` channel 异步事件分发。数据到达即处理，消除 50ms 轮询。使用 `AtomicBool` 停止信号替代 oneshot channel。

### 2. 协议帧缓冲与流式解析
**Status**: ✅ Done
**Files**: `src/serial_core/port.rs`
**Description**: `SerialPortHandle` 新增 `frame_buffer` 字段，跨读取累积数据后再调用 `parse()`。解决部分协议帧被静默丢弃的问题。64KB 安全上限防止缓冲区无限增长。

---

## P1 - 可靠性与健壮性

### 3. 端口热插拔检测
**Status**: ✅ Done
**Description**: 后台任务每 2s 轮询硬件端口列表变化，自动发射 `ports-changed` 事件（added/removed 端口名列表）到前端。

### 4. 优雅关闭与资源清理
**Status**: ✅ Done
**Files**: `src-tauri/src/main.rs`
**Description**: Tauri `on_window_event(CloseRequested)` 拦截关闭，依次停止 sniffers → 关闭端口（含脚本 detach）→ 停止虚拟端口对，完成后 `app.exit(0)`。

### 5. 端口并发与锁优化
**Status**: ✅ Done
**Files**: `src/serial_core/port.rs`, `src-tauri/src/commands/serial.rs`
**Description**: 修复 `open_port` 的重复打开检查（原先 `contains_key` 对 uuid key 无效，改为遍历 values 匹配 port name）。sniffer 不再每次读取都锁 PortManager。

---

## P2 - 协议与脚本引擎

### 6. 协议热重载实现
**Status**: ✅ Done
**Files**: `src/protocol/manager.rs`, `src/protocol/watcher.rs`
**Description**: `enable_hot_reload` 现在真正创建 `ProtocolWatcher`，监视已加载协议的目录，文件变化时自动重新加载并更新注册表。

### 7. Lua 引擎统一
**Status**: ⏳ Pending
**Files**: `src/lua/`, `src/serial_core/serial_script.rs`, `src/protocol/lua_ext.rs`
**Description**: 存在三个独立的 Lua 引擎实现（SerialScriptEngine / LuaBindings / LuaProtocol），各自创建 Lua state 和注册 API 方式不同。`LuaProtocol` 每次回调重建 Lua state，无持久状态且性能差。需统一为共享 Lua 引擎架构。同时修复 Lua 中 `virtual_create` 立即销毁资源的问题。

### 8. Lua 脚本 API 扩展
**Status**: ⏳ Pending
**Description**: Lua 运行时缺少文件 I/O（受沙箱限制但无安全替代）、多定时器管理、串口控制 API。`sleep_ms` 使用 `std::thread::sleep` 会阻塞 Tauri 异步线程。

---

## P2 - 数据管理与导出

### 9. 会话持久化与数据导出
**Status**: ⏳ Pending
**Description**: 无后端会话保存/加载机制。`SerialSniffer::save_to_file` 存在于核心库但未暴露为 Tauri 命令。需增加：会话日志保存、数据导出（CSV/JSON/Hex dump）、端口配置收藏夹持久化。

### 10. GUI 日志系统
**Status**: ⏳ Pending
**Files**: `src-tauri/src/main.rs`
**Description**: Tauri 模式日志仅输出到 stderr（GUI 中不可见）。需配置日志文件输出、日志轮转、可选的应用内日志查看器。

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
- ✅ Tauri backend commands (47 registered)
- ✅ 227 Rust unit tests + 9 E2E tests passing

### ✅ v0.7.0 - Frontend + Backend Integration
- ✅ React 19 + Vite 8 + TypeScript + Tailwind CSS 4 + Zustand 5
- ✅ 串口终端 (连接/收发/HEX-ASCII-Mixed/导出/命令/监控/解码器)
- ✅ 虚拟串口管理 (创建/停止/捕获/CSV导出)
- ✅ 脚本引擎 (Monaco编辑器/模板/执行/验证/绑定端口)
- ✅ 协议管理 (列表/加载/卸载/编解码测试)
- ✅ 设置面板 (8标签页/配置读写/重置) + DisplayConfig 持久化
- ✅ i18n 中英文支持 + 可折叠侧边栏 + 多标签工作区
- ✅ get_all_ports_status 修复：查询实际已打开端口状态
- ✅ 事件发射器接入：port-status-changed / error-occurred / virtual-port-stats-updated
