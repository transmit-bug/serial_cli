# 虚拟串口模块前端增强设计

**日期**: 2026-05-28
**状态**: 待实现
**作者**: Claude Code (spark brainstorming)

## 背景

当前虚拟串口页面（`VirtualPortsPage.tsx`）功能完整但界面单薄，存在以下问题：

1. 创建配置过于简单（仅后端选择 + 一键创建），缺少 buffer_size、监控开关、最大包数等配置
2. 创建后用户不知道"怎么用"这些虚拟端口
3. 没有可视化展示端口对之间的数据桥接关系
4. 缺少向虚拟端口发送数据的交互入口
5. 监控功能（monitor）未在界面暴露

虚拟串口的核心使用场景是：**创建一对虚拟端口 → 一端连接真实设备/程序 → 另一端用于监听和发送数据**。界面应该让这个模式一目了然。

## 目标

- 让虚拟串口的创建、管理、使用在一个页面内完成
- 直观展示端口对的桥接关系和数据流
- 集成 QuickSend 能力，让用户可以直接向虚拟端口发送数据
- 保持简洁实用，不过度设计

## 方案选择

选择 **方案 A：就地增强** —— 在现有 `VirtualPortsPage` 内做 layout 改造，而非拆分页面或完整重构。

理由：
- 改动集中，复用现有 Zustand store 和 Tauri 命令
- 虚拟端口的核心场景（监控桥接 + 发送数据）不需要完整 TerminalWorkbench
- QuickSendPanel 可提取为无状态组件复用

## 架构设计

### 整体 Layout

页面改为左右分栏：

```
┌─ 左侧 (~320px) ─────┤─ 右侧 (flex-1) ─────────────────┐
│                      │                                  │
│ [展开式创建表单]      │  [桥接数据流可视化]               │
│                      │  端口 A ════ 端口 B               │
│ 端口卡片列表          │  已桥接: 125.6 KB · 1,234 包     │
│  ● port-abc123       │                                  │
│  ● port-def456       │  [端口详细信息]                   │
│                      │  路径 / 运行时间 / 统计            │
│                      │                                  │
│                      │  [CommandSender - 快速发送]       │
│                      │  [Hex] 48 65 6C 6C 6F [发送]      │
│                      │                                  │
│                      │  [抓包数据表 (可折叠)]             │
│                      │  A→B 48 65 6C  |  B→A 52 45 53   │
│                      │                                  │
└──────────────────────┴──────────────────────────────────┘
```

### 组件拆分

| 组件 | 文件 | 职责 |
|------|------|------|
| `VirtualPortsPage` | 现有，重写 | 页面容器，管理左右分栏 layout |
| `VirtualPortList` | 新建 | 左侧卡片列表 + 创建表单 |
| `VirtualPortCard` | 新建 | 单个端口对卡片（状态、路径、快捷操作） |
| `VirtualPortDetail` | 新建 | 右侧详情面板容器 |
| `BridgeVisualization` | 新建 | 桥接数据流可视化 |
| `CommandSender` | 从 QuickSendPanel 提取 | 无状态命令发送组件 |
| `VirtualPacketTable` | 从现有代码提取 | 抓包数据表格 |

### 数据流

```
用户操作 → Zustand Store → Tauri Commands → Rust Backend
     ↓                                              ↓
  UI 刷新 ← Event System ← 状态变化 ← 桥接/抓包
```

- `virtualPortStore` 扩展：增加 `selectedPort` 持久化、实时 stats 轮询
- 保留现有的 3 秒轮询 `refreshPorts` 和 5 秒 health check
- 新增：选中端口后，以 1 秒间隔轮询 stats（用于桥接可视化实时更新）

## 详细设计

### 1. VirtualPortList（左侧列表）

**创建表单（展开式）**：

- 默认显示 "+ 创建虚拟端口对" 按钮
- 点击后展开表单区域：
  - **后端类型**：下拉选择 (PTY / socat / NamedPipe)，自动检测平台可用性，不可用的选项 disabled
  - **启用监控**：checkbox，开启后捕获桥接数据包
  - **缓冲区大小**：number input，默认 8192 bytes
  - **最大抓包数**：number input，默认 1000（0 表示不限制）
  - **端口名称**：可选 text input，用于标识此端口对
- "创建" 按钮调用 `createPort`，成功后自动在列表中选中该端口
- "取消" 按钮收起表单

**端口卡片**：

每个卡片显示：
- 运行状态指示灯（绿色=运行，灰色=停止，红色=健康检查失败）
- 端口对 ID（或自定义名称）
- 后端类型标签
- 缩略流量指示（有流量时显示字节数/s）
- 端口 A 路径（截断，hover 显示完整路径）
- 点击卡片 → 设为 selectedPort，右侧展示详情

卡片操作（hover 显示）：
- 复制端口 A 路径到剪贴板
- 复制端口 B 路径到剪贴板
- 停止端口对

**空状态**：

无端口时显示引导文案：
- "虚拟串口可以创建一对互联的串行端口"
- "一端连接真实设备，另一端用于监听和发送数据"
- 配简单示意图：`[设备] ─── A ◄══► B ─── [你的程序]`

### 2. BridgeVisualization（桥接可视化）

简洁风格的桥接展示：

```
┌───────────────────────────────────────────────┐
│  /dev/pts/3              /dev/pts/4           │
│   ┌───┐                  ┌───┐                │
│   │ A │ ════════════════ │ B │                │
│   └───├──── 15.2 KB/s ──►└───┘                │
│         ◄──── 8.1 KB/s ──┤                    │
│                                               │
│   已桥接: 125.6 KB  ·  1,234 包  ·  0 错误    │
└───────────────────────────────────────────────┘
```

- A 和 B 两个方框，中间一条连接线（CSS border）
- 两个方向的箭头 + 实时字节/秒（从 throughput 计算）
- 底部显示累计统计（bytes_bridged / packets_bridged / bridge_errors）
- 如果有错误（bridge_errors > 0），显示最近错误信息
- 如果 health check 失败，连接线变红色

### 3. CommandSender（从 QuickSendPanel 提取）

**当前 QuickSendPanel 的依赖问题**：

```
QuickSendPanel
  ├── useConnectionStore (需要 activePortId + connected status)
  ├── useCommandStore (命令列表 CRUD)
  ├── useDataStore (addPacket)
  └── sendCommand(index, portId, addPacket)
```

**提取后的 CommandSender**：

```typescript
interface CommandSenderProps {
  portId: string;
  sendFn: (data: string, format: "hex" | "ascii") => Promise<void>;
  onSent?: (data: string, format: "hex" | "ascii") => void;
}
```

- 复用 QuickSendPanel 的命令列表 UI（grid 按钮、编辑、删除、hotkey）
- `sendFn` 由调用方提供 —— TerminalWorkbench 中调用 Tauri serial_send，虚拟端口页面中同样调用 serial_send 但传入虚拟端口 B 的路径
- 命令列表仍通过 `useCommandStore` 管理（全局共享预设命令）

**在虚拟端口中的使用**：

```tsx
<CommandSender
  portId={selectedPort.port_b}
  sendFn={async (data, format) => {
    // 调用 Tauri serial_send，发送到端口 B
    await tauriApi.sendData(selectedPort.port_b, data, format);
  }}
  onSent={(data, format) => {
    // 可选：记录到抓包或显示
  }}
/>
```

### 4. VirtualPacketTable（抓包数据表）

从现有 VirtualPortsPage 底部提取为独立组件：

- 方向过滤：All / A→B / B→A 三个按钮
- 表格列：序号 / 方向 / 时间 / 数据（hex）
- 实时滚动：新数据自动 scroll 到底部
- CSV 导出
- 可折叠（默认展开，用户可折叠收起）
- 显示包数统计："共 N 个包，A→B: X，B→A: Y"

### 5. Zustand Store 扩展

`virtualPortStore` 新增/修改：

```typescript
interface VirtualPortStore {
  // 现有
  ports: VirtualPortInfo[];
  selectedPort: string | null;
  capturedPackets: CapturedPacket[];
  healthMap: Record<string, boolean>;
  loading: boolean;

  // 新增
  statsMap: Record<string, VirtualPortStats>;       // 缓存 stats
  throughputMap: Record<string, number>;            // 实时吞吐量
  prevBytesMap: Record<string, number>;             // 上一轮 bytes，用于计算 delta
  createFormOpen: boolean;                          // 创建表单展开状态

  // 新增 actions
  setSelectedPort: (id: string | null) => void;
  setCreateFormOpen: (open: boolean) => void;
  startStatsPolling: (id: string) => void;          // 开始 1s stats 轮询
  stopStatsPolling: (id: string) => void;           // 停止 stats 轮询

  // 现有 actions (保持不变)
  refreshPorts: () => Promise<void>;
  createPort: (config: CreateVirtualPortConfig) => Promise<void>;
  stopPort: (id: string) => Promise<void>;
  getStats: (id: string) => Promise<VirtualPortStats | null>;
  checkHealth: (id: string) => Promise<boolean>;
  loadCapturedPackets: (id: string) => Promise<void>;
}
```

### 6. Tauri 后端改动

**需要新增的 Tauri 命令**：

```rust
/// 向指定虚拟端口发送数据
#[tauri::command]
pub async fn send_to_virtual_port(
    id: String,
    port_end: String,  // "a" or "b"
    data: Vec<u8>,
) -> Result<(), String>
```

这个命令从虚拟端口注册表中找到对应的 `VirtualSerialPair`，向指定端写入数据。

**现有命令无需修改**：create/list/stop/stats/health/get_captured_packets 已满足需求。

**现有 Tauri 命令 `send_data`（用于物理端口）是否可以复用？**

如果现有的 `send_data` 命令接受设备路径（string）而非 portId（UUID），则可以直接复用，无需新增命令。需要确认现有实现。

## 文件变更清单

### 新建文件

| 文件 | 说明 |
|------|------|
| `frontend/src/components/virtual/VirtualPortList.tsx` | 左侧列表 + 创建表单 |
| `frontend/src/components/virtual/VirtualPortCard.tsx` | 端口卡片组件 |
| `frontend/src/components/virtual/VirtualPortDetail.tsx` | 右侧详情面板容器 |
| `frontend/src/components/virtual/BridgeVisualization.tsx` | 桥接数据流可视化 |
| `frontend/src/components/virtual/VirtualPacketTable.tsx` | 抓包数据表格 |
| `frontend/src/components/shared/CommandSender.tsx` | 从 QuickSendPanel 提取的无状态发送组件 |

### 修改文件

| 文件 | 改动 |
|------|------|
| `frontend/src/components/virtual/VirtualPortsPage.tsx` | 重写为左右分栏 layout，组合子组件 |
| `frontend/src/stores/virtualPort.ts` | 扩展 store（statsMap、throughputMap、createFormOpen、stats polling） |
| `frontend/src/components/terminal/QuickSendPanel.tsx` | 内部改用 CommandSender，保持现有 API 兼容 |
| `frontend/src/lib/tauri-api.ts` | 可能需要新增 `sendToVirtualPort` 方法 |
| `src-tauri/src/commands/virtual_port.rs` | 可能需要新增 `send_to_virtual_port` 命令 |
| `frontend/src/i18n/locales/en.json` | 新增虚拟端口相关 i18n key |
| `frontend/src/i18n/locales/zh.json` | 新增虚拟端口相关 i18n key |

## 实施顺序

1. **CommandSender 提取**：从 QuickSendPanel 拆分出无状态组件，验证 QuickSendPanel 仍正常工作
2. **Store 扩展**：添加 statsMap、throughputMap、createFormOpen、stats polling
3. **VirtualPortList**：实现左侧列表 + 创建表单
4. **BridgeVisualization**：实现简洁风格桥接可视化
5. **VirtualPacketTable**：提取抓包表格为独立组件
6. **VirtualPortDetail**：组合 BridgeVisualization + CommandSender + VirtualPacketTable
7. **VirtualPortsPage 重写**：组合所有子组件，实现左右分栏
8. **Tauri 后端（如需要）**：新增 send_to_virtual_port 命令
9. **i18n**：补充所有新增文案的中英文翻译
10. **测试和打磨**：验证端到端流程

## 风险与约束

- **QuickSendPanel 兼容性**：提取 CommandSender 后必须确保 TerminalWorkbench 中的 QuickSendPanel 行为不变
- **Tauri 发送能力**：需要确认虚拟端口是否可以直接通过 serial_send 写入数据，还是需要新增专用命令
- **性能**：1 秒间隔的 stats 轮询 + 3 秒端口刷新 + 5 秒 health check，多个虚拟端口同时运行时轮询次数可控（N*3 次/秒，N 通常 < 5）
- **平台限制**：PTY 后端在 macOS/Linux 可用，NamedPipe 在 Windows 可用，socat 需要外部依赖。创建表单需自动 disable 不可用选项
