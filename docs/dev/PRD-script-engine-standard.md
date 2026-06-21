# PRD: Script Engine 标准化与协议脚本开发指南

## Problem Statement

Serial CLI 的 Lua 脚本引擎目前缺乏面向第三方开发者的标准化文档和规范。现有 4 个内置脚本（line、at_command、modbus_rtu、modbus_ascii）展示了脚本能力，但：

1. **没有统一的脚本规范** —— 元数据、回调约定、错误处理都是隐式的，第三方开发者只能通过阅读内置脚本源码来推断如何编写脚本
2. **缺少请求-响应接口** —— 现有 `serial_send` / `serial_recv` 是单向操作，脚本无法方便地实现"发送请求并等待完整响应"这一最常见模式
3. **没有测试指导** —— `script_encode` / `script_decode` 已具备测试能力，但未文档化为测试手段
4. **无法承载业务逻辑** —— 脚本只能做帧编解码（Hook 模式），无法封装完整的设备交互逻辑

## Solution

对脚本引擎进行标准化，使其成为**自包含的串口协议驱动**运行时：

1. 定义可选的 `SCRIPT_META` 元数据 table，让脚本自描述
2. 补充 `serial_query` 和 `serial_flush` 两个 I/O 接口，使脚本能实现完整的请求-响应业务逻辑
3. 编写完整的协议脚本开发指南文档
4. 更新内置脚本以符合新标准（添加 `SCRIPT_META`）

## User Stories

1. As a **第三方协议开发者**，我想要一份完整的 API 参考文档，以便我能快速编写一个新的协议脚本而不需要阅读引擎源码
2. As a **第三方协议开发者**，我想要脚本能自描述元数据（名称、版本、描述），以便宿主应用能自动发现和展示脚本信息
3. As a **第三方协议开发者**，我想要一个 `serial_query` 接口，以便我能用一行代码实现"发送请求并等待完整响应"的模式
4. As a **第三方协议开发者**，我想要一个 `serial_flush` 接口，以便在超时或校验失败后清空接收缓冲区再重试
5. As a **第三方协议开发者**，我想要明确的错误处理约定，以便我知道何时返回 nil、何时返回错误 table、何时直接 error()
6. As a **第三方协议开发者**，我想要用 `script_encode` / `script_encode` 做单元测试，以便不连设备也能验证脚本逻辑
7. As a **第三方协议开发者**，我想要内置脚本作为模板参考，以便我能基于现有实现快速改编
8. As a **宿主应用开发者**，我想要通过 `script_info` 获取脚本的 `SCRIPT_META`，以便在 UI 中展示脚本详情
9. As a **宿主应用开发者**，我想要脚本暴露 `action_*` 函数，以便我能发现和调用脚本提供的业务操作
10. As a **设备集成工程师**，我想要编写一个完整的 Modbus 设备驱动脚本，包含寄存器读写、异常处理、定时轮询，以便一个脚本就能驱动一台设备
11. As a **设备集成工程师**，我想要脚本支持 `on_timer` 回调，以便实现心跳包发送、周期性状态查询
12. As a **设备集成工程师**，我想要脚本的 `SCRIPT_META` 包含 `tags` 字段，以便按协议类型搜索和筛选脚本
13. As a **自动化测试工程师**，我想要用 mock 数据测试脚本的 `on_send` / `on_recv`，以便验证帧编解码的正确性
14. As a **自动化测试工程师**，我想要测试脚本的 `action_*` 函数，以便验证业务逻辑的正确性
15. As a **GUI 用户**，我想要通过 `script_list` 看到所有可用脚本及其描述，以便选择合适的协议
16. As a **GUI 用户**，我想要脚本的 `action_*` 函数自动出现在 UI 中，以便我无需编写前端代码就能调用设备操作

## Implementation Decisions

### 1. SCRIPT_META 嵌入式元数据

脚本通过定义全局 Lua table `SCRIPT_META` 来声明元数据。该 table **完全可选**，缺失不影响脚本运行。

**字段定义：**

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `name` | string | 否 | 唯一标识符，同名覆盖 |
| `version` | string | 否 | semver 格式 |
| `description` | string | 否 | 人类可读描述 |
| `author` | string | 否 | 作者 |
| `license` | string | 否 | 许可证 |
| `homepage` | string | 否 | 详情页 URL |
| `tags` | string[] | 否 | 用于搜索/分类 |
| `data_format` | string | 否 | `"binary"` 或 `"text"` |
| `min_frame_size` | number | 否 | 最小完整帧字节数（binary 模式） |

**Rust 侧改动：** `SerialScriptEngine` 在 `load()` 时从 Lua globals 提取 `SCRIPT_META`，存为 `Option<ScriptMeta>`。`ScriptInfo` 结构体扩展以包含这些字段。

### 2. 新增 I/O 接口

在 `LuaBindings` 中注册两个新的全局函数：

**`serial_query(port_id, data, timeout_ms) -> response | nil`**
- 发送 `data`（byte table），然后等待 `on_recv` 返回非 nil
- 超时返回 nil
- 实现方式：在独立线程中循环调用 `serial_recv`，每次收到数据后传入 `on_recv` 回调，回调返回非 nil 即为响应
- 语义依赖 `on_recv` 的帧缓冲逻辑判断"完整帧"

**`serial_flush(port_id)`**
- 清空指定端口的接收缓冲区
- 用途：超时/校验失败后，清掉残余字节再重试

### 3. 回调函数规范（不变，仅文档化）

现有回调保持不变，仅在文档中明确语义：

| 回调 | 输入 | 返回值 | 语义 |
|------|------|--------|------|
| `on_open(port, config)` | port_name, config_table | 忽略 | 端口打开时触发 |
| `on_send(data)` | byte table | byte table / nil | 修改/拦截发送数据 |
| `on_recv(data)` | byte table | byte table / nil | 修改/拦截/缓冲接收数据 |
| `on_close()` | 无 | 忽略 | 端口关闭时触发 |
| `on_timer()` | 无 | interval_ms | 定时器，返回下一次间隔 |

### 4. 错误处理约定

| 错误级别 | 处理方式 | 示例 |
|----------|---------|------|
| 帧级错误（可恢复） | `log_warn()` + 返回 nil | CRC 校验失败、帧太短 |
| 业务级错误（需暴露） | 返回 `{ _error = "xxx" }` table | 设备返回异常码、超时 |
| 致命错误（脚本 bug） | 直接 `error()` | 引擎捕获并报告 |

### 5. 测试模式

脚本测试使用现有的 `script_encode` / `script_decode` 接口，不需要新增测试框架：

```lua
-- 测试 on_send 编码
local encoded = script_encode("my_protocol", {0x01, 0x03, 0x00, 0x00, 0x00, 0x01})
assert(#encoded == 8, "Expected 8 bytes with CRC")

-- 测试 on_recv 解码
local decoded = script_decode("my_protocol", encoded)
assert(decoded ~= nil, "Roundtrip failed")
```

### 6. UI Actions（不变，仅文档化）

脚本中以 `action_` 前缀命名的函数会被自动发现，暴露为宿主可调用的业务操作。

### 7. 文档结构

文档位于 `docs/guides/script-development.md`，结构如下：

1. 概述
2. 快速开始（最小可运行脚本）
3. SCRIPT_META（可选元数据）
4. 生命周期回调
5. I/O 接口
6. 工具函数
7. UI Actions
8. 错误处理约定
9. 测试
10. 内置脚本参考
11. 附录 A: API 速查表
12. 附录 B: 脚本模板

### 8. 内置脚本更新

现有 4 个内置脚本（line、at_command、modbus_rtu、modbus_ascii）添加 `SCRIPT_META` 元数据，作为符合新标准的参考实现。

## Testing Decisions

1. **Lua 侧测试**（新增）：编写 Lua 测试脚本，使用 `script_encode` / `script_decode` 验证内置脚本的编解码正确性
2. **Rust 侧测试**（扩展现有）：
   - `serial_query` 接口的集成测试：模拟发送-响应流程
   - `serial_flush` 接口测试
   - `SCRIPT_META` 提取测试：验证有/无 meta 的脚本都能正常加载
   - `ScriptInfo` 扩展字段的序列化测试
3. **内置脚本回归测试**：确保添加 `SCRIPT_META` 后现有功能不变

## Out of Scope

1. **网络获取脚本** —— 本次不实现，仅确保 `SCRIPT_META` 预留了足够的字段支持未来网络分发
2. **Lua 基础教程** —— 文档假设读者已具备 Lua 基础
3. **脚本签名/安全沙箱** —— 网络获取场景的安全机制留待后续
4. **`serial_read_until` / `serial_read_bytes`** —— 帧边界逻辑统一由 `on_recv` 回调处理，引擎不提供绕过回调的读取接口
5. **`serial_drain`** —— 串口发送是同步阻塞的，不需要
6. **重试机制** —— 重试策略因协议而异，属于脚本业务逻辑

## Further Notes

- `SCRIPT_META` 的设计为未来网络获取脚本预留了扩展点：`name` + `version` 可作为唯一标识，`tags` 支持分类搜索，`homepage` 指向详情页
- `serial_query` 的实现依赖 `on_recv` 回调的帧缓冲语义，这意味着"完整帧"的定义完全由脚本控制，引擎不做假设
- 文档使用中文编写，与项目现有文档风格一致
