# PRD：架构改进与重构

**版本**：1.0  
**创建日期**：2026-06-19  
**状态**：Ready for Agent  
**优先级**：分阶段交付（Phase A → B → C）

---

## 问题陈述

Serial CLI 经过多轮迭代，积累了 12 项架构问题，分为三个层次：

1. **功能缺陷**：热重载状态在 GUI 和 CLI 之间不同步，用户看到不一致的行为
2. **测试覆盖不足**：核心功能（脚本管理、Server 通信、端口-脚本交互）缺乏测试，而边缘功能有完整测试
3. **架构摩擦**：模块深度不足（浅层门面、死代码）、代码重复（Lua 转换、Hex 解析）、理解成本高（概念分散在多个文件）

这些问题导致：
- 用户在 GUI 启用热重载后，CLI 看不到该设置
- 重构核心模块时缺少安全网，回归风险高
- 新功能开发者需要追踪 5+ 个文件才能理解一个功能
- 维护成本随代码增长而增加

---

## 解决方案

通过三阶段重构解决所有 12 项问题：

**Phase A（1-2 周）**：修复真实 bug + 补全核心测试  
**Phase B（2-3 周）**：删除浅层模块 + 代码去重 + 依赖注入  
**Phase C（持续改进）**：平台代码提取 + 别名清理 + 抽象优化

每个阶段独立交付，可单独验证和回滚。

---

## 用户故事

### Phase A：功能修复与测试补全

1. 作为 GUI 用户，我希望在设置中启用热重载后 CLI 也能看到该设置，以便我可以在不同界面切换而不丢失配置
2. 作为开发者，我希望核心 Store（脚本管理、Server 通信）有完整测试，以便我在重构时有安全网
3. 作为自动化测试工程师，我希望端口写入→脚本回调链路有集成测试，以便验证 unsafe 代码路径的正确性
4. 作为 Code reviewer，我希望 PR 提交时有明确的测试覆盖要求，以便确保新功能不引入回归
5. 作为运维工程师，我希望测试报告清晰展示核心功能的覆盖率，以便评估发布风险
6. 作为新人开发者，我希望通过测试理解脚本管理的状态流转，以便快速上手核心逻辑
7. 作为 QA 工程师，我希望热重载的端到端测试覆盖 GUI→CLI 同步，以便验证功能完整性
8. 作为性能工程师，我希望端口写入测试验证回调不阻塞主线程，以便确保实时性
9. 作为安全审计员，我希望 unsafe 代码路径有明确的测试覆盖，以便评估内存安全风险
10. 作为技术债务清理工程师，我希望每个 Phase 有明确的验收标准，以便追踪进度

### Phase B：模块简化与去重

11. 作为架构师，我希望删除 CommandService 浅层门面，以便减少一层间接调用，代码更直接
12. 作为维护者，我希望删除未使用的 SerialSniffer，以便减少代码复杂性和维护负担
13. 作为开发者，我希望 ScriptEngine 的依赖显式注入，以便可以 mock ScriptManager 进行测试
14. 作为代码质量工程师，我希望 Lua 表/字节转换函数有单一实现，以便 bug 修复传播到所有调用点
15. 作为测试工程师，我希望 Hex 解析有统一的边界情况处理，以便确保所有模块行为一致
16. 作为新人开发者，我希望删除 CommandService 后调用者直接使用管理器，以便减少理解成本
17. 作为 Code reviewer，我希望去重后的代码有明确的模块边界，以便审查时快速定位
18. 作为性能工程师，我希望依赖注入不引入额外的运行时开销，以便保持性能
19. 作为重构工程师，我希望删除模块后调用者平滑迁移，以便避免破坏现有功能
20. 作为文档维护者，我希望重构后更新架构文档，以便新人理解新结构

### Phase C：理解优化与平台抽象

21. 作为跨平台开发者，我希望平台特定代码从 open_port() 提取到独立模块，以便在单平台上阅读代码
22. 作为迁移工程师，我希望删除向后兼容的 protocol 别名，以便消除"protocol 和 script 是否相同"的困惑
23. 作为新人开发者，我希望端口-脚本绑定有明确的控制器抽象，以便通过阅读一个文件理解该功能
24. 作为架构师，我希望 Tauri/Server 的共享初始化逻辑提取为工厂，以便减少重复代码
25. 作为测试工程师，我希望平台抽象后可以为 Unix/Windows 分别编写测试，以便验证跨平台行为
26. 作为 Code reviewer，我希望别名删除有明确的迁移指南，以便下游用户平滑过渡
27. 作为开发者，我希望 PortScriptController 封装完整的生命周期，以便减少理解摩擦
28. 作为运维工程师，我希望状态容器提取后初始化逻辑集中，以便调试启动问题
29. 作为技术债务清理工程师，我希望每个改进有明确的 PR 提交，以便追踪和回滚
30. 作为项目经理，我希望三阶段交付有明确的里程碑，以便评估进度和风险

---

## 实施决策

### 总体架构决策

**决策 1：三阶段交付策略**
- Phase A 聚焦功能修复和测试补全（1-2 周）
- Phase B 聚焦模块简化和代码去重（2-3 周）
- Phase C 聚焦理解优化和平台抽象（持续改进）
- 每个阶段独立交付，可单独验证

**决策 2：优先删除浅层模块，而非深化**
- CommandService（#1）：删除，而非添加业务逻辑
- SerialSniffer（#5）：删除，而非统一实现
- 理由：这两个模块当前没有真实业务逻辑，删除比添加逻辑风险更低

**决策 3：统一使用 ConfigManager 作为真实来源**
- 热重载状态统一通过 ConfigManager 持久化
- ScriptManager 启动时从配置读取，内存标志从配置派生
- 所有界面（CLI/GUI/Server）通过 ConfigManager 读写配置

### Phase A 实施决策

**决策 4：热重载统一（#6）**
- 删除 `ScriptManager::set_hot_reload()` 的内存标志
- 添加 `ScriptManager::load_hot_reload_config()` 从 ConfigManager 读取
- Tauri 命令调用 ConfigManager 而非直接设置内存标志
- CLI 和 Server 使用相同的 ConfigManager API

**决策 5：前端 Store 测试（#10）**
- 为 `serialScript.ts` 添加测试：脚本绑定、状态转换、错误路径
- 为 `script.ts` 添加测试：mock fetch 和 Tauri API，测试脚本加载和验证
- 为 `server.ts` 添加测试：Server 状态管理、连接列表更新
- 为 `tauri-api.ts` 添加类型测试：验证命令名字符串与后端匹配

**决策 6：端口写入→回调链测试（#11）**
- 创建虚拟端口 + 脚本的集成测试
- 验证 `on_send` 回调接收到正确的字节
- 测试回调错误不阻塞写入
- 覆盖 unsafe `PortPtr` 和 `LuaSend` 的正确性

### Phase B 实施决策

**决策 7：删除 CommandService（#1）**
- 删除 `src/service.rs` 中的 CommandService
- Tauri 和 Server 直接使用 PortManager 和 ScriptManager
- CLI 直接使用管理器，或通过命令层协调
- 删除 CommandService 的测试，替换为管理器级别的集成测试

**决策 8：删除 SerialSniffer（#5）**
- 删除 `src/serial_core/sniffer.rs`
- Tauri 继续使用 `src-tauri/src/commands/serial.rs` 中的嗅探实现
- CLI 继续使用守护模式的嗅探
- 从 CommandService 和 AppState 中移除 SerialSniffer 引用

**决策 9：ScriptEngine 依赖注入（#8）**
- 删除 `ScriptEngine::execute_with_args()` 内部的 `ScriptManager::new()`
- 通过构造函数注入 ScriptManager
- 调用者负责创建和传递 ScriptManager
- 测试中可以注入 mock ScriptManager

**决策 10：Lua 转换去重（#2）**
- 创建 `src/utils/lua_conversion.rs`
- 实现 `lua_table_to_bytes()` 和 `bytes_to_lua_table()`
- 统一 `serial_core/serial_script.rs` 和 `lua/runtime.rs` 的调用点
- 添加边界情况测试：空表、混合类型表、溢出值

**决策 11：Hex 解析去重（#3）**
- 创建 `src/utils/hex.rs`
- 实现 `hex_encode()` 和 `hex_decode()`
- 统一 `serial_core/parsers.rs`、`lua/runtime.rs`、`lua/bindings.rs` 的调用点
- 添加边界情况测试：奇数长度、无效字符、空输入、0x 前缀、混合大小写

### Phase C 实施决策

**决策 12：平台代码提取（#12）**
- 创建 `src/serial_core/platform/` 模块
- 提取 `configure_port()` 函数，分别为 Unix 和 Windows 实现
- 从 `open_port()` 中移除平台条件编译块
- 为平台配置添加独立测试

**决策 13：向后兼容别名清理（#7）**
- 删除 Lua 绑定中的 `protocol_encode`/`protocol_decode` 别名
- 删除 `SerialError::ProtocolError` 变体（如果未被使用）
- 在 CHANGELOG 中记录破坏性变更
- 提供迁移指南：将所有 `protocol_*` 调用改为 `script_*`

**决策 14：PortScriptController 抽象（#9）**
- 创建 `PortScriptController` 类型，封装端口-脚本绑定生命周期
- 包含：脚本加载、引擎创建、回调注册、状态管理
- 调用者只需与 Controller 交互，无需理解 5 层间接
- 添加集成测试验证完整生命周期

**决策 15：状态容器提取（#4）**
- 创建共享工厂函数，构建 PortManager + ScriptManager
- AppState 和 ServerState 调用工厂，然后添加各自的特定状态
- 工厂函数可以接受配置参数（脚本目录、端口列表等）
- 添加测试验证工厂构建的管理器配置正确

---

## 测试决策

### 测试原则

- **只测试外部行为，不测试实现细节**
- **优先测试 Seam（接口边界），而非内部逻辑**
- **集成测试优先于单元测试，验证跨模块协作**
- **测试覆盖率不是目标，测试质量才是**

### Phase A 测试策略

**热重载测试（#6）**
- 单元测试：ConfigManager 读写热重载配置
- 集成测试：GUI 启用后 CLI 能看到（通过共享配置文件）
- 回归测试：重启后热重载状态保持

**前端 Store 测试（#10）**
- 状态转换测试：Store 的 action 触发正确的状态更新
- Mock 测试：Tauri API 和 fetch 调用被正确 mock
- 边界情况：空数据、错误响应、并发更新

**端口写入→回调链测试（#11）**
- 集成测试：虚拟端口 + 脚本 + 回调的完整链路
- 错误路径测试：回调抛出异常不阻塞写入
- 性能测试：回调不阻塞主线程（异步验证）

### Phase B 测试策略

**删除 CommandService 后的测试（#1）**
- 集成测试：Tauri 命令直接调用 PortManager/ScriptManager
- 回归测试：CLI 命令行为不变
- 性能测试：减少间接层后性能提升（可选）

**删除 SerialSniffer 后的测试（#5）**
- 集成测试：Tauri 嗅探功能正常
- 集成测试：CLI 嗅探功能正常
- 回归测试：嗅探数据格式不变

**依赖注入测试（#8）**
- 单元测试：ScriptEngine 接受 mock ScriptManager
- 集成测试：注入的 ScriptManager 脚本对执行可见
- 回归测试：自主脚本执行行为不变

**去重测试（#2, #3）**
- 单元测试：utils 模块的边界情况
- 集成测试：所有调用点使用统一实现后行为一致
- 回归测试：现有测试全部通过

### Phase C 测试策略

**平台抽象测试（#12）**
- 单元测试：Unix 平台配置正确
- 单元测试：Windows 平台配置正确（在 Windows CI 上运行）
- 集成测试：跨平台端口打开行为一致

**别名清理测试（#7）**
- 回归测试：所有现有 Lua 脚本使用 `script_*` 正常工作
- 迁移测试：旧脚本的 `protocol_*` 调用报错并提供迁移提示

**PortScriptController 测试（#9）**
- 单元测试：Controller 的生命周期管理
- 集成测试：完整的绑定→执行→解绑流程
- 回归测试：现有端口-脚本交互行为不变

**状态容器测试（#4）**
- 单元测试：工厂函数构建正确的管理器配置
- 集成测试：AppState 和 ServerState 使用工厂后行为一致
- 回归测试：现有功能不受影响

---

## 范围外

### 不在本 PRD 范围内

1. **新功能开发**：本 PRD 聚焦架构改进，不添加新功能
2. **性能优化**：除非重构带来的性能提升，不专门做性能优化
3. **UI 改进**：前端 UI 改进不在本 PRD 范围内
4. **文档重构**：除非架构变更需要的文档更新，不做专门的文档重构
5. **依赖升级**：不升级 Rust 或前端依赖版本
6. **CI/CD 改进**：除非测试需要，不修改 CI 流程
7. **向后兼容层**：除了明确的别名清理，不添加额外的向后兼容层

### 推迟到后续迭代

1. **监控和指标**：架构改进的监控指标（如模块复杂度、测试覆盖率趋势）
2. **自动化工具**：代码质量检查工具（如 SonarQube、CodeClimate）
3. **文档自动化**：从代码生成文档的工具
4. **国际化测试**：前端 Store 的多语言测试

---

## 进一步说明

### 风险评估

**Phase A 风险**
- **热重载统一**：可能影响现有用户的配置，需要迁移脚本
- **测试补全**：可能发现隐藏的 bug，需要额外时间修复
- **缓解措施**：每个改进独立 PR，可单独回滚

**Phase B 风险**
- **删除 CommandService**：调用者迁移可能引入 bug
- **依赖注入**：构造函数变更可能影响多个调用点
- **缓解措施**：分步骤删除，先标记废弃，再删除实现

**Phase C 风险**
- **平台抽象**：跨平台测试需要多环境 CI
- **别名清理**：下游 Lua 脚本可能需要迁移
- **缓解措施**：提供清晰的迁移指南和错误提示

### 成功标准

**Phase A**
- 热重载状态在 GUI 和 CLI 之间同步
- 核心 Store 测试覆盖率 > 80%
- 端口写入→回调链有集成测试
- 所有现有测试通过

**Phase B**
- CommandService 和 SerialSniffer 完全删除
- Lua 转换和 Hex 解析有单一实现
- ScriptEngine 依赖显式注入
- 代码行数减少 > 5%（删除重复和死代码）

**Phase C**
- 平台特定代码从 open_port() 提取
- protocol 别名完全删除
- PortScriptController 封装完整生命周期
- 状态容器有共享工厂
- 新人开发者理解"附加脚本"功能只需阅读 1 个文件

### 交付计划

**Week 1-2（Phase A）**
- PR 1：热重载统一
- PR 2：前端 Store 测试
- PR 3：端口写入→回调链测试

**Week 3-5（Phase B）**
- PR 4：删除 SerialSniffer
- PR 5：删除 CommandService
- PR 6：ScriptEngine 依赖注入
- PR 7：Lua 转换去重
- PR 8：Hex 解析去重

**Week 6+（Phase C）**
- PR 9：平台代码提取
- PR 10：向后兼容别名清理
- PR 11：PortScriptController 抽象
- PR 12：状态容器提取

### 依赖关系

- Phase A 无依赖，可立即开始
- Phase B 依赖 Phase A 完成（测试覆盖后再重构）
- Phase C 依赖 Phase B 完成（删除模块后再提取抽象）

### 沟通计划

- 每个 PR 提交时更新 TODO.md
- 每个 Phase 完成后更新 CHANGELOG.md
- Phase B 完成后更新架构文档（ARCH.md）
- Phase C 完成后更新领域语言文档（CONTEXT.md）

---

## 附录

### 架构审查报告

详细的架构审查报告已生成，包含每个问题的前后对比图和收益分析。  
报告路径：`/var/folders/qn/n_s4jvmx4m331yczcdm60ysr0000gn/T/architecture-review-20260619-001.html`

### 领域语言

本 PRD 使用以下领域术语（来自 CONTEXT.md）：
- **Port**：设备上的串口接口
- **Connection**：打开的端口句柄
- **Script**：通信协议规范（替代"protocol"）
- **Frame**：协议数据单元
- **Capture**：串口数据记录
- **Virtual Port**：模拟的串口对

### 参考文档

- `docs/dev/ARCH.md`：当前架构概述
- `CONTEXT.md`：领域语言定义
- `TODO.md`：项目进度追踪
- `CHANGELOG.md`：变更历史
