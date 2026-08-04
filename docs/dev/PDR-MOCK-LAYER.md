# PDR: Frontend Mock Layer

**Status**: Draft
**Created**: 2026-07-23

---

## Background

Tauri 后端（Rust）每次编译占用大量内存和 CPU。前端开发（UI 调整、样式、交互逻辑）必须依赖 `cargo tauri dev` 才能运行，无法独立迭代。

目标：让 `pnpm dev` 能独立启动前端，不编译 Rust，不依赖 Tauri 运行时。

## Design Goals

1. **零侵入**：现有代码（store / component / hook / tauri-api.ts）零改动
2. **生产安全**：`cargo tauri dev` / `cargo tauri build` 路径上一个字节不变
3. **MSW 风格**：handler 声明式、按领域拆分、内存状态、未匹配透传
4. **可调试**：开发者可在浏览器 console 手动触发后端事件
5. **可复用**：mock 模块可在 vitest 测试和 Storybook 中复用

## Architecture

```
pnpm dev (Vite)
    ↓ alias 重定向
    import { tauriApi } from "@/lib/tauri-api"
    → 解析到 src/lib/mock/index.ts
    → Proxy 拦截方法调用
    → camelCase → snake_case 转换
    → interceptor 分发到对应 handler
    → handler 读写 MockState
    → 返回 Promise<result>

    import { listen } from "@tauri-apps/api/event"
    → 解析到 src/lib/mock/events.ts
    → 注册到内存事件总线
    → __MOCK_EMIT__ 可手动触发
```

```
cargo tauri dev
    ↓ TAURI_PLATFORM 存在，alias 不生效
    import { tauriApi } from "@/lib/tauri-api"
    → 解析到 src/lib/tauri-api.ts（原文件）
    → invoke() 走真实 Tauri IPC
```

## Directory Structure

```
src/lib/mock/
├── index.ts              # 入口：注册所有 handler，导出 tauriApi (Proxy)
├── interceptor.ts        # handler 注册表 + 分发逻辑
├── state.ts              # MockState：内存状态模型
├── events.ts             # mock 事件总线（listen + __MOCK_EMIT__）
└── handlers/
    ├── port.ts           # list_ports, open_port, close_port, get_port_status, check_port_health
    ├── serial.ts         # send_data, start_sniffing, stop_sniffing
    ├── script.ts         # list_scripts, load/unload/reload, validate, user scripts
    ├── serial-script.ts  # attach/detach, get_script_status, list/call actions
    ├── config.ts         # get/update/reset_config, presets
    ├── server.ts         # start/stop/get_server_status
    ├── virtual-port.ts   # create/list/stop, stats, captured packets
    ├── export.ts         # export_data
    └── log.ts            # read_logs, clear_logs
```

## Interface Contract

### tauriApi（Proxy 模式）

`mock/index.ts` 导出一个 `tauriApi` 对象，与 `src/lib/tauri-api.ts` 导出**完全相同的 shape**。

通过 `Proxy` 实现自动方法名映射：

```typescript
// tauriApi.listPorts() → handleInvoke("list_ports", {})
// tauriApi.openPort(name, config) → handleInvoke("open_port", { portName, config })
```

调用方（store）代码不变，Vite alias 在模块解析阶段完成重定向。

### Handler 签名

```typescript
type Handler = (args: Record<string, unknown>, state: MockState) => unknown;
```

每个 handler 接收 Tauri invoke 的参数对象和全局 MockState，返回结果。

### Events

```typescript
// mock/events.ts — 导出与 @tauri-apps/api/event 相同的 listen 签名
export function listen<T>(event: string, handler: (e: { payload: T }) => void): Promise<() => void>

// 开发调试
(window as any).__MOCK_EMIT__ = (event: string, payload: any) => void
```

## State Model

`MockState` 维护以下内存状态：

| 状态 | 初始值 | 操作 |
|---|---|---|
| `ports: Map<string, PortEntry>` | 2 个假端口（closed） | open/close 改变 status |
| `scripts: Script[]` | 3 个内置脚本 | load/unload 增删 |
| `userScripts: Map<string, UserScriptEntry>` | 空 | save/delete 增删 |
| `attachedScripts: Map<string, string>` | 空 | attach/detach |
| `config: ConfigData` | 默认配置 | update 覆盖 |
| `presets: ConnectionPreset[]` | 空 | save/delete |
| `server: ServerStatus` | stopped | start/stop 切换 |
| `virtualPorts: Map<string, VirtualPortEntry>` | 空 | create/stop |
| `logs: string[]` | 空 | read/clear |

状态变更是**即时生效**的（同步 Map/Object 操作），模拟后端的即时响应。

## Switching Mechanism

```typescript
// vite.config.ts
const isTauri = process.env.TAURI_PLATFORM !== undefined;

resolve: {
  alias: {
    "@/": "./src/",  // 基础 alias（已有）
    // mock 模式下重定向（仅 pnpm dev 生效）
    ...(isTauri ? {} : {
      "@/lib/tauri-api": path.resolve(__dirname, "./src/lib/mock"),
      "@tauri-apps/api/event": path.resolve(__dirname, "./src/lib/mock/events.ts"),
    }),
  }
}
```

**为什么用 `TAURI_PLATFORM`**：`cargo tauri dev` 会自动注入此环境变量。`pnpm dev` 时不存在。无需额外配置。

## Non-Goals

- ❌ 不模拟真实的延迟/错误（开发时不需要）
- ❌ 不模拟真实的数据流（sniffer 推送用 `__MOCK_EMIT__` 手动触发）
- ❌ 不替代集成测试（这是开发辅助工具）
- ❌ 不修改任何现有文件（除 `vite.config.ts` 加 alias）

## Implementation Checklist

- [ ] `src/lib/mock/interceptor.ts` — handler 注册 + 分发
- [ ] `src/lib/mock/state.ts` — MockState 类
- [ ] `src/lib/mock/events.ts` — listen + __MOCK_EMIT__
- [ ] `src/lib/mock/handlers/port.ts`
- [ ] `src/lib/mock/handlers/serial.ts`
- [ ] `src/lib/mock/handlers/script.ts`
- [ ] `src/lib/mock/handlers/serial-script.ts`
- [ ] `src/lib/mock/handlers/config.ts`
- [ ] `src/lib/mock/handlers/server.ts`
- [ ] `src/lib/mock/handlers/virtual-port.ts`
- [ ] `src/lib/mock/handlers/export.ts`
- [ ] `src/lib/mock/handlers/log.ts`
- [ ] `src/lib/mock/index.ts` — 入口 + Proxy
- [ ] `vite.config.ts` — 添加 alias
- [ ] 验证 `pnpm dev` 独立启动
- [ ] 验证 `cargo tauri dev` 不受影响
