# Serial CLI TODO List

**Updated**: 2026-06-22

---

## 已完成

### 文档与 CI
- ✅ 全面更新项目文档（CHANGELOG、README、SERVER_MODE、ARCH、FRONTEND-REWRITE-DESIGN、events）
- ✅ CI 前端改用 pnpm（替换 npm ci --force，添加 pnpm store 缓存）
- ✅ 修复 GitHub Actions：cargo fmt 格式违规（benches + src 共 12 文件）
- ✅ 修复 GitHub Actions：benchmarks 传 --save-baseline 给 lib target 导致失败，改为逐个 bench 运行
- ✅ 修复 GitHub Actions：release.yml 中 generate_release_notes 与自定义 body 冲突，prev_tag 引用不存在的变量
- ✅ 修复 GitHub Actions：release.yml 无法被 Dependabot 解析（YAML body 中模板语法问题）
- ✅ 添加 RUSTSEC-2025-0069（daemonize unmaintained）advisory skip，暂无安全替代方案
- ✅ 添加 AGENTS.md 指导文件
- ✅ 精简文档结构，删除 CLAUDE.md 并整合内容到 AGENTS.md

### Lua 脚本导入支持（2026-06-22）
- ✅ 配置 Lua package.path 支持 require()（#24）
- ✅ 移除 require() 验证警告（#25）
- ✅ 添加 5 个集成测试验证 require() 功能（#26）
- ✅ 新增 modbus_rtu_lib.lua 工具库和 temp_sensor.lua 设备驱动示例
- ✅ 更新 script-development.md 文档添加 require() 使用说明（#27）
- ✅ 更新 AGENTS.md 添加脚本导入规范

### 测试
- ✅ Tauri 后端补充 29 个单元测试（port_state、port 解析、virtual_port、config、export 5 个模块）
- ✅ 给 RxViewer 搜索高亮逻辑补充测试用例（提取 splitHighlights 到 lib/highlight.ts，22 个测试）
- ✅ 给 ConnectionStore 补充测试用例（连接/断开/错误状态流转，17 个测试）
- ✅ DataStore 测试（包管理、搜索过滤、导出选项，16 个测试）
- ✅ PresetsStore 测试（CRUD、排序、应用预设，10 个测试）
- ✅ SettingsStore 测试（加载/更新/重置配置，6 个测试）
- ✅ Server 模式集成测试（13 个测试覆盖生命周期、连接管理、并发、空闲清理）

### 统一脚本系统（2026-06-17）
- ✅ 创建 `src/script/` 模块（ScriptManager, built-in Lua 脚本）
- ✅ 实现内置 Lua 脚本：line.lua, at_command.lua, modbus_rtu.lua
- ✅ 提取 CommandService 层（共享编排逻辑）
- ✅ 迁移 CLI/RPC/Tauri 到 ScriptManager
- ✅ 迁移 lua/bindings.rs 到 ScriptManager
- ✅ 移除 `src/protocol/` 目录（12 个文件）
- ✅ 重命名 CLI `protocol` 命令为 `script`
- ✅ 修复所有编译警告

### 跨平台
- ✅ monitoring 模块 macOS 支持（libc proc_pidinfo 获取内存/FD，windows crate 按平台条件编译）

### 文档完善
- ✅ 更新 README.md 反映新的 script 命令
- ✅ 更新 CHANGELOG.md 记录统一脚本系统变更
- ✅ 更新 docs/dev/ARCH.md 反映新架构
- ✅ 添加脚本系统综合测试（load/unload/reload, hot-reload, 自定义脚本, 错误处理）

### 功能增强
- ✅ 实现脚本热重载（文件监控，自动重载变更的脚本）
- ✅ 增强脚本验证（检查必需回调、验证返回类型、添加 linting）
- ✅ 扩展 CommandService 覆盖范围（sniff, batch, virtual port 管理）
- ✅ 添加脚本示例库（更多协议实现示例）

---

## Phase 6：技术债务 + 性能体验（2026-06-18 规划）

### P0：技术债务清理 ✅ 完成

- [x] **统一 `protocol` → `script` 命名** ✅ 2026-06-18
  - Tauri 命令：`src-tauri/src/commands/protocol.rs` → 整合到 `script.rs`
  - 前端 API 层：`frontend/src/lib/tauri-api.ts` 合并 protocol/script 两节
  - 前端 Store：合并 `protocol.ts` 到 `script.ts`，统一类型定义
  - RPC 方法：保持 `protocol_*` 命名（兼容性）
  - 前端类型：`ProtocolInfo`/`ScriptInfo` → `Script`/`UserScriptInfo`

- [x] **清理死代码** ✅ 2026-06-18
  - ✅ 移除 `src/task/` 模块（4 文件）
  - ✅ 移除 `src/lua/engine.rs`
  - ✅ 移除 `ScriptManager.watched_paths` 字段
  - ✅ 移除 `src/monitoring/` 模块（2 文件）
  - ✅ 移除 `batch` 命令及相关模块
  - ✅ 清理 `TaskConfig` 配置项

- [x] **补全 Server Mode 数据推送** ✅ 2026-06-18
  - 添加 DataPushEvent 结构体用于推送事件
  - 在 ServerState 中添加 broadcast channel
  - port_open 时订阅端口数据并启动转发任务
  - handle_connection 中监听推送事件并发送给订阅客户端
  - 实现 JSON-RPC 推送格式 (method: port_data)

### P1：性能与体验提升

- [x] **Lua 状态池 & 预编译** ✅ 2026-06-18
  - ✅ 实现线程本地 Lua 状态池（LuaStatePool），默认容量 10
  - ✅ 添加 acquire_lua() 和 release_lua() 便捷函数
  - ✅ 实现 ScriptCache 用于缓存已验证的脚本
  - ✅ 更新 ScriptManager 使用状态池减少 Lua 实例创建开销
  - ✅ 添加 8 个单元测试验证状态池功能

- [x] **改进错误消息和诊断** ✅ 2026-06-18
  - ScriptError 支持可选的 stack_trace 字段
  - 新增 ErrorContext 包装器，支持链式上下文添加
  - 使用 ResultExt trait 提供 context/with_port/with_script 方法
  - 7 个单元测试验证错误格式

- [x] **脚本调试支持** ✅ 2026-06-18
  - [x] 添加 `debug.traceback` 集成，脚本错误时输出完整调用栈

---

## Phase 7：前端功能补齐（2026-06-18 规划）

### 已完成 ✅

- [x] **修复虚拟串口连接 Bug** ✅ 2026-06-18
  - 问题：前端调用 `openPort` 时未传递 `isVirtual` 参数，导致 PTY 端口尝试 DTR/RTS 初始化失败
  - 修复：在 `connection.ts` 的 `connect` 方法中，从 `availablePorts` 查找端口的 `is_virtual` 属性并传递给 `tauriApi.openPort`
  - 文件：`frontend/src/stores/connection.ts`

### P0：Server 模式 GUI ✅

- [x] **Server 页面实现** ✅ 2026-06-19
  - 定位：控制面板（Monitor & Manage），不操作串口数据
  - 导航：侧边栏新增独立 "Server" 入口
  - 布局：单页多区块（顶部控制栏 + 左侧信息 + 右侧连接列表）
  
- [x] **后端：嵌入式 Server 支持** ✅ 2026-06-19
  - 在 Tauri 后端实现嵌入式 JSON-RPC Server（不 fork 子进程）
  - 复用现有 `ServerState`、`RpcHandler`、`Listener`
  - 支持 Unix Socket（macOS/Linux）
  - 通过 Tauri events 推送状态到前端
  
- [x] **前端：Server 状态管理** ✅ 2026-06-19
  - 新增 `stores/server.ts` 管理 Server 状态
  - 状态：运行/停止、socket path、TCP port、max connections
  - 统计：运行时间、总请求、错误数、活跃连接数
  
- [x] **前端：Server 页面组件** ✅ 2026-06-19
  - `ServerPage.tsx` — 主页面容器
  - `ServerStatus.tsx` — 顶部状态栏（启动/停止按钮、状态指示器）
  - `ServerConfig.tsx` — 配置信息卡片
  - `ServerStats.tsx` — 实时统计卡片
  - `ServerConnections.tsx` — 活跃连接列表（connection_id、客户端地址、连接时间）
  
- [x] **前端：端口冲突处理** ✅ 2026-06-19
  - 被外部客户端占用的端口在 Terminal 页面显示为灰色 + "in use" 标记
  - 不可点击连接（避免与外部客户端冲突）
  - 从后端获取端口占用状态（通过 Server 的 `connection_list` RPC 方法）

### P1：Editor 清理与优化 ✅

- [x] **清理重复的 Protocol Templates** ✅ 2026-06-19
  - ✅ 删除 `protocol_parser` template（与内置 modbus_rtu 重复）
  - ✅ 删除 `simple_at` template（与内置 at_command 重复）
  - 保留 `custom_frame` 和新增 `modbus_ascii` template
  - 文件：`frontend/src/components/editor/TemplateList.tsx`
  
- [x] **内置协议名动态获取** ✅ 2026-06-19
  - ✅ 移除前端硬编码的 `BUILT_IN_PROTOCOLS`
  - ✅ 从后端 `list_scripts` API 动态获取内置脚本列表（`built_in: true`）
  - 文件：`frontend/src/components/editor/EditorPage.tsx`、`ProtocolList.tsx`

### P2：脚本功能增强 ✅

- [x] **脚本热重载 UI** ✅ 2026-06-19
  - ✅ 在 Settings → Lua Engine tab 添加热重载开关
  - ✅ 调用后端 API 启用/禁用热重载
  - ✅ 显示当前热重载状态
  - 后端已暴露 `enable_hot_reload()` / `disable_hot_reload()` Tauri 命令
  
- [x] **脚本详细验证展示** ✅ 2026-06-19
  - ✅ Editor 中显示详细验证结果（不仅是语法错误）
  - ✅ 检查并显示：危险函数警告（`os.execute`、`io.popen`）、必需回调缺失（`on_send`、`on_recv`）
  - ✅ 后端 `validate_script_detailed` 已集成，前端调用并展示
  - 文件：`frontend/src/components/editor/EditorPage.tsx`

### P3：其他功能补齐 ✅

- [x] **连接预设管理增强** ✅ 2026-06-19
  - ✅ 连接预设 UI 已完整（CRUD、拖拽排序）
  - 文件：`frontend/src/components/settings/presets/`

- [x] **脚本 UI Actions 完善** ✅ 2026-06-19
  - ✅ `StandaloneActions.tsx` 正确渲染脚本的 `#[ui_action]` 按钮
  - ✅ 按钮可以在 Editor 和 Right Panel 中正常显示和执行

---

## Phase 8：测试与质量保障（2026-06-19 规划）

### P0：Server 模式集成测试 ✅ 完成

- [x] **Server 生命周期测试** ✅ 2026-06-19
  - ✅ 测试启动 Server → 状态查询（running=true）
  - ✅ 测试停止 Server → 状态查询（running=false）
  - ✅ 测试 Socket 文件创建和清理
  - 文件：`tests/server_integration.rs`

- [x] **Server 错误处理测试** ✅ 2026-06-19
  - ✅ 测试重复启动（应返回已运行状态）
  - ✅ 测试停止未运行的 Server
  - ✅ 测试 Socket 文件冲突处理
  - ✅ 测试无效配置的错误响应

- [x] **Server 并发连接测试** ✅ 2026-06-19
  - ✅ 模拟多个连接同时存在
  - ✅ 验证连接状态实时更新
  - ✅ 测试统计计数正确性（total_requests、total_errors）
  - ✅ 测试连接 ID 生成和去重

### P1：其他模块集成测试 ✅

- [x] **脚本验证集成测试** ✅ 2026-06-19
  - ✅ 10 个测试全部通过
  - 验证详细验证 API 返回值
  - 测试警告生成逻辑（回调缺失、危险函数）
  - 文件：`tests/script_validation_tests.rs`

- [x] **端口管理集成测试** ✅ 2026-06-19
  - ✅ 9 个测试全部通过
  - 测试端口冲突检测
  - 验证 serverOccupiedPorts 状态同步
  - 文件：`tests/port_management_tests.rs`

---

## Phase 9：协议扩展与代码质量（2026-06-19）✅ 完成

### P0：Modbus ASCII 协议支持 ✅

- [x] **后端实现** ✅ 2026-06-19
  - ✅ 创建 `src/script/built_in/modbus_ascii.lua`
  - ✅ 实现 LRC 校验算法
  - ✅ 实现 HEX 编码/解码
  - ✅ 帧格式处理（起始符 `:`、结束符 CR LF）

- [x] **集成测试** ✅ 2026-06-19
  - ✅ 4 个测试全部通过
  - ✅ 测试 on_send 编码（二进制 → ASCII）
  - ✅ 测试 on_recv 解码（ASCII → 二进制）
  - ✅ 测试往返编码一致性
  - ✅ 测试 LRC 校验失败情况

- [x] **前端支持** ✅ 2026-06-19
  - ✅ 添加国际化文本（en.json、zh.json）
  - ✅ 创建前端协议模板（TemplateList.tsx）
  - ✅ 提供代码示例和说明

### P1：TypeScript 类型修复 ✅

- [x] **修复类型错误** ✅ 2026-06-19
  - ✅ ExportControls.tsx: 修复 `fields` 对象展开问题
  - ✅ commands.ts: 更新 `sendCommand` 函数签名，匹配 `addPacket` 参数
  - ✅ RightPanelHistory.tsx: 修复 `clearBuffer` 调用
  - ✅ vite.config.ts: 改用 `vitest/config` 导入
  - ✅ ShortcutsHelp.test.tsx: 添加 `vi` mock 函数导入
  - ✅ data.ts: 将 `ExportOptions.fields` 改为 `Partial<ExportFields>`

### 成果

- **测试总数**: 279 个（全部通过，1 个忽略）
- **TypeScript 错误**: 全部修复（非测试文件）
- **新增协议**: Modbus ASCII（完整支持）

---

## Phase 10：Benchmark 现代化更新（2026-06-19）✅ 完成

### P0：Benchmark API 迁移 ✅

- [x] **修复所有 benchmark 文件的编译错误** ✅ 2026-06-19
  - ✅ `benches/concurrency.rs`: 移除 LuaEngine，改用 ScriptManager
  - ✅ `benches/lua_execution.rs`: 更新为使用 ScriptRuntime 和新的 API
  - ✅ `benches/protocol_parsing.rs`: 移除已删除的 Protocol trait 使用
  - ✅ `benches/serial_io.rs`: 更新为使用 ScriptManager 创建引擎
  - ✅ `benches/startup.rs`: 修复 ScriptRuntime::new() 调用

### 成果

- **Benchmark 测试**: 全部通过（30+ 个 benchmark）
- **API 一致性**: 所有 benchmark 使用新的 ScriptManager/ScriptRuntime API
- **性能基线**: 为后续优化提供完整的性能测试基线

---

## 统计

- **测试总数**: 279 passed, 1 ignored（全部通过）
- **Benchmark 数量**: 30+（全部通过）
- **源代码行数**: ~17,500
- **测试代码行数**: ~1,400

---

## 项目完成状态 ✅

所有计划任务已完成：
- ✅ Phase 7: 前端功能补齐（Server 模式、脚本热重载、详细验证等）
- ✅ Phase 8: 测试与质量保障（集成测试、单元测试）
- ✅ Phase 9: 协议扩展与代码质量（Modbus ASCII、TypeScript 修复）
- ✅ Phase 10: Benchmark 现代化更新（API 迁移、性能基线）

---

## Phase 11：架构改进与重构（2026-06-19 规划）

**PRD 文档**: `docs/dev/PRD-ARCHITECTURE-IMPROVEMENT.md`

### Phase A：功能修复与测试补全（1-2 周）

- [x] **P0-1：统一热重载系统** ✅ 2026-06-19
  - 删除 `ScriptManager::set_hot_reload()` 的内存标志
  - 添加 `ScriptManager::load_hot_reload_config()` 从 ConfigManager 读取
  - Tauri 命令调用 ConfigManager 而非直接设置内存标志
  - 集成测试验证 GUI→CLI 状态同步
  - 验收标准：GUI 启用后 CLI 能看到，重启后状态保持

- [x] **P0-2：前端核心 Store 测试补全** ✅ 2026-06-19
  - `serialScript.ts`：脚本绑定、状态转换、错误路径测试（19 个测试）
  - `script.ts`：mock fetch 和 Tauri API，测试脚本加载和验证（24 个测试）
  - `server.ts`：Server 状态管理、连接列表更新测试（12 个测试）
  - `tauri-api.ts`：命令名字符串类型测试
  - 验收标准：核心 Store 测试覆盖率 > 80% ✅

- [x] **P0-3：端口写入→回调链集成测试** ✅ 2026-06-19
  - 创建虚拟端口 + 脚本的集成测试（`tests/port_script_callback_tests.rs`）
  - 验证 `on_send` 回调接收到正确的字节
  - 测试回调错误不阻塞写入
  - 覆盖 unsafe `PortPtr` 和 `LuaSend` 的正确性
  - 21 个测试全部通过 ✅
  - 验收标准：关键数据路径有端到端测试覆盖 ✅

### Phase B：模块简化与代码去重（2-3 周）

- [x] **P1-1：清理 SerialSniffer 死代码引用** ✅ 2026-06-19
  - ✅ 从 `VirtualSerialPair` 中移除未使用的 `sniffer` 字段和 `sniffer()` 方法
  - ✅ 删除 `PacketCapture` 结构体（从未使用）
  - ✅ 清理 `serial_core/mod.rs` 中的导出
  - ✅ 保留 `SerialSniffer` 供 CLI 嗅探守护进程使用（`src/cli/sniff_session.rs`）
  - ✅ 保留 Tauri 后端的独立嗅探实现（`DataSniffer`）
  - 验收标准：移除死代码引用，嗅探功能正常 ✅

- [x] **P1-2：删除 CommandService** ✅ 2026-06-19
  - ✅ 删除 `src/service.rs` 文件（335 行浅层门面）
  - ✅ 从 `src/lib.rs` 移除模块声明
  - ✅ CLI 和 Server 直接使用 PortManager 和 ScriptManager
  - ✅ 删除 CommandService 相关测试
  - 验收标准：所有命令行为不变，间接层减少 ✅

- [x] **P1-3：ScriptEngine 依赖注入** ✅ 2026-06-19
  - ✅ 修改 `ScriptEngine::new()` 接受 `Arc<Mutex<ScriptManager>>` 参数
  - ✅ 添加 `ScriptEngine::default()` 实现，创建独立的 ScriptManager
  - ✅ 更新 `run_lua_script()` 使用共享的 ScriptManager
  - ✅ 更新 main.rs 中的调用点
  - ✅ 更新集成测试使用 `ScriptEngine::default()`
  - ✅ 所有单元测试和集成测试通过
  - 验收标准：依赖显式，可测试性提升 ✅

- [x] **P1-4：Lua 转换函数去重** ✅ 2026-06-19
  - ✅ 创建 `src/utils/lua_conversion.rs`
  - ✅ 实现 `lua_table_to_bytes()` 和 `bytes_to_lua_table()`
  - ✅ 统一 `serial_core/serial_script.rs` 和 `lua/runtime.rs` 调用点
  - ✅ 添加边界情况测试
  - 验收标准：单一实现，行为一致 ✅

- [x] **P1-5：Hex 解析去重** ✅ 2026-06-19
  - ✅ 创建 `src/utils/hex.rs`
  - ✅ 实现 `hex_encode()` 和 `hex_decode()`
  - ✅ 统一 `serial_core/parsers.rs`、`lua/runtime.rs`、`lua/bindings.rs` 调用点
  - ✅ 添加边界情况测试
  - 验收标准：单一实现，行为一致 ✅

### Phase C：理解优化与平台抽象（持续改进）

- [x] **P2-1：平台代码提取** ✅ 2026-06-20
  - ✅ 创建 `src/serial_core/platform/mod.rs`
  - ✅ 提取 `configure_port()` 函数，分别为 Unix 和 Windows 实现
  - ✅ 从 `open_port()` 中移除平台条件编译块（~85 行）
  - ✅ 移除 port.rs 中 `set_dtr_on_fd`、`set_rts_on_fd`、`map_serial_error` 重复代码
  - ✅ 所有测试通过（238 单元 + 全部集成测试）
  - 验收标准：单平台代码可读性提升 ✅

- [x] **P2-2：向后兼容别名清理** ✅ 2026-06-20
  - ✅ 删除 Lua 绑定中的 `protocol_encode`/`protocol_decode` 别名
  - ✅ 删除 `SerialError::Protocol` 变体和 `ProtocolError` 枚举
  - ✅ 删除 `ErrorCode::ProtocolError`
  - ✅ 所有测试通过（236 个）
  - 验收标准：命名一致性，混淆消除 ✅

- [x] **P2-3：PortScriptController 抽象** ✅ 2026-06-20
  - ✅ 创建 `src/serial_core/port_script_controller.rs`
  - ✅ 封装脚本引擎的完整生命周期（attach/load/detach）
  - ✅ 管理 on_open/on_recv/on_send/on_close 回调
  - ✅ 管理定时器状态
  - ✅ 包含单元测试（6 个）
  - 验收标准：理解"附加脚本"只需阅读 1 个文件 ✅

- [x] **P2-4：状态容器提取** ✅ 2026-06-20
  - ✅ 创建 `src/state_factory.rs` 模块
  - ✅ 定义 `CoreManagers` 结构体封装 PortManager + ScriptManager
  - ✅ 更新 `ServerState::new()` 使用 CoreManagers
  - ✅ 更新 `AppState::new()` 使用 CoreManagers
  - ✅ 所有测试通过（239 单元 + 全部集成测试）
  - 验收标准：初始化逻辑集中，重复代码减少 ✅

### 交付计划

- **Week 1-2**：Phase A（P0-1, P0-2, P0-3）
- **Week 3-5**：Phase B（P1-1 到 P1-5）
- **Week 6+**：Phase C（P2-1 到 P2-4）

### 验收标准

**Phase A**
- 热重载状态在 GUI 和 CLI 之间同步
- 核心 Store 测试覆盖率 > 80%
- 端口写入→回调链有集成测试
- 所有现有测试通过

**Phase B**
- CommandService 和 SerialSniffer 完全删除
- Lua 转换和 Hex 解析有单一实现
- ScriptEngine 依赖显式注入
- 代码行数减少 > 5%

**Phase C**
- 平台特定代码从 open_port() 提取
- protocol 别名完全删除
- PortScriptController 封装完整生命周期
- 状态容器有共享工厂
