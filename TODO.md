# Serial CLI TODO List

**Updated**: 2026-06-17

---

## 已完成

### 文档与 CI
- ✅ 全面更新项目文档（CHANGELOG、README、SERVER_MODE、ARCH、FRONTEND-REWRITE-DESIGN、events）
- ✅ CI 前端改用 pnpm（替换 npm ci --force，添加 pnpm store 缓存）
- ✅ 修复 GitHub Actions：cargo fmt 格式违规（benches + src 共 12 文件）
- ✅ 修复 GitHub Actions：benchmarks 传递 --save-baseline 给 lib target 导致失败，改为逐个 bench 运行
- ✅ 修复 GitHub Actions：release.yml 中 generate_release_notes 与自定义 body 冲突，prev_tag 引用不存在的变量
- ✅ 修复 GitHub Actions：release.yml 无法被 Dependabot 解析（YAML body 中模板语法问题）
- ✅ 添加 RUSTSEC-2025-0069（daemonize unmaintained）advisory skip，暂无安全替代方案
- ✅ 添加 AGENTS.md 指导文件

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

---

## 待办

### Phase 4：文档与测试完善

- [x] 更新 README.md 反映新的 script 命令
- [x] 更新 CHANGELOG.md 记录统一脚本系统变更
- [x] 更新 docs/dev/ARCH.md 反映新架构
- [x] 添加脚本系统综合测试（load/unload/reload, hot-reload, 自定义脚本, 错误处理）

### Phase 5：功能增强

- [x] 实现脚本热重载（文件监控，自动重载变更的脚本）
- [x] 增强脚本验证（检查必需回调、验证返回类型、添加 linting）
- [x] 扩展 CommandService 覆盖范围（sniff, batch, virtual port 管理）
- [x] 添加脚本示例库（更多协议实现示例）

### Phase 6：性能与体验

- [ ] 脚本执行性能优化（Lua 状态池、预编译）
- [ ] 添加脚本调试支持（断点、单步执行、变量检查）
- [ ] 改进错误消息和诊断信息
- [ ] 添加脚本文档生成工具

---

## 统计

- **测试总数**: 277（全部通过）
- **源代码行数**: ~17,500
- **测试代码行数**: ~1,400
