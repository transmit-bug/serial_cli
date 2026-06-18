# Serial CLI TODO List

**Updated**: 2026-06-18

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

### 测试
- ✅ Tauri 后端补充 29 个单元测试（port_state、port 解析、virtual_port、config、export 5 个模块）
- ✅ 给 RxViewer 搜索高亮逻辑补充测试用例（提取 splitHighlights 到 lib/highlight.ts，22 个测试）
- ✅ 给 ConnectionStore 补充测试用例（连接/断开/错误状态流转，17 个测试）
- ✅ DataStore 测试（包管理、搜索过滤、导出选项，16 个测试）
- ✅ PresetsStore 测试（CRUD、排序、应用预设，10 个测试）
- ✅ SettingsStore 测试（加载/更新/重置配置，6 个测试）

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

- [ ] **脚本调试支持（分阶段）**
  - Phase 6a：添加 `debug.traceback` 集成，脚本错误时输出完整调用栈
  - Phase 6b：实现 `debug.sethook` 断点/单步（可选，复杂度高）

- [ ] **脚本文档生成工具**
  - 解析 Lua 脚本头部注释和函数签名
  - 生成 Markdown 格式 API 文档（`serial-cli script doc <script.lua>`）

---

## 统计

- **测试总数**: 277+（全部通过）
- **源代码行数**: ~17,500
- **测试代码行数**: ~1,400
