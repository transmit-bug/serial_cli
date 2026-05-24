# Frontend Rewrite Design Specification

**Version**: 0.7.0
**Date**: 2026-05-24
**Status**: Draft

---

## 1. Overview

Complete rewrite of the Serial CLI GUI frontend, built on the existing Tauri backend (36 commands). The design follows a professional tool aesthetic (VS Code / Postman style) with function-driven layouts. The main workbench page handles ~80% of daily operations.

### Design Principles

- **Function over form**: Every UI element serves a clear purpose
- **Professional tool style**: Dense information layout, keyboard-first workflow
- **Mature dependencies only**: No custom tooling or experimental libraries
- **Pure Zustand state management**: No React Context, single source of truth per domain

---

## 2. Tech Stack & Dependencies

All versions are the latest stable as of 2026-05. Package manager: **pnpm** (per CLAUDE.md).

### Core Framework

| Package | Version | Purpose |
|---------|---------|---------|
| `react` | ^19.2 | UI rendering |
| `react-dom` | ^19.2 | DOM rendering |
| `@types/react` | ^19.2 | TypeScript types |
| `@types/react-dom` | ^19.2 | TypeScript types |
| `typescript` | ^5.8 | Type safety |

### Build Toolchain

| Package | Version | Purpose |
|---------|---------|---------|
| `vite` | ^6.0 | Dev server + bundler |
| `@vitejs/plugin-react` | ^4.5 | React Fast Refresh |
| `tailwindcss` | ^4.3 | Utility-first CSS (v4 CSS-based config) |
| `@tailwindcss/vite` | ^4.3 | Tailwind Vite plugin (replaces PostCSS plugin) |
| `autoprefixer` | ^10.5 | Vendor prefixing |

### UI Components (shadcn/ui)

shadcn/ui 通过 CLI (`npx shadcn@latest init`) 初始化，自动安装所需 Radix 原语。以下是项目会用到的 Radix 组件：

| Package | Version | Usage |
|---------|---------|-------|
| `@radix-ui/react-select` | ^2.2 | Port/baud/protocol 下拉选择 |
| `@radix-ui/react-dialog` | ^1.1 | 模态弹窗（创建虚拟串口、快捷命令编辑） |
| `@radix-ui/react-tooltip` | ^1.2 | 工具提示 |
| `@radix-ui/react-tabs` | ^1.1 | 设置面板 Tab 切换 |
| `@radix-ui/react-switch` | ^1.2 | 开关（协议编解码、自动滚动） |
| `@radix-ui/react-dropdown-menu` | ^2.1 | 右键菜单、下拉菜单（导出格式） |
| `@radix-ui/react-popover` | ^1.1 | 弹出面板 |
| `@radix-ui/react-separator` | ^1.1 | 分隔线 |
| `@radix-ui/react-toggle` | ^1.1 | 格式切换按钮（HEX/ASCII/Mixed） |
| `@radix-ui/react-label` | ^2.2 | 表单标签 |
| `@radix-ui/react-slot` | ^1.1 | Button 组件内部使用 |
| `@radix-ui/react-scroll-area` | ^1.2 | 自定义滚动条（数据查看器） |
| `class-variance-authority` | ^0.7 | 组件变体样式 |
| `clsx` | ^2.1 | 条件类名合并 |
| `tailwind-merge` | ^3.6 | Tailwind 类名去重 |

### State & Data

| Package | Version | Purpose |
|---------|---------|---------|
| `zustand` | ^5.0 | 全局状态管理（纯 Zustand，不用 React Context） |
| `@tanstack/react-virtual` | ^3.13 | 虚拟滚动列表（大数据 RX 查看器） |
| `date-fns` | ^4.3 | 时间戳格式化 |

### Code Editor

| Package | Version | Purpose |
|---------|---------|---------|
| `@monaco-editor/react` | ^4.7 | Lua 脚本编辑器 |

Monaco 自动加载 editor core，无需额外安装。

### Internationalization

| Package | Version | Purpose |
|---------|---------|---------|
| `i18next` | ^26.2 | i18n 核心 |
| `react-i18next` | ^17.0 | React 绑定 |
| `i18next-browser-languagedetector` | ^8.2 | 自动检测浏览器/系统语言 |

### Forms & Validation

| Package | Version | Purpose |
|---------|---------|---------|
| `react-hook-form` | ^7.76 | 表单状态管理（设置面板、快捷命令编辑） |
| `zod` | ^4.4 | Schema 验证 |
| `@hookform/resolvers` | ^5.4 | zod ↔ react-hook-form 桥接 |

### Layout & Interaction

| Package | Version | Purpose |
|---------|---------|---------|
| `react-resizable-panels` | ^4.11 | 可拖拽分割面板（RX/TX 区域、左右面板） |
| `cmdk` | ^1.1 | 命令面板（Cmd+K，VS Code 风格） |
| `sonner` | ^2.0 | Toast 通知 |
| `lucide-react` | ^1.16 | 图标库（800+ 图标，shadcn/ui 官方推荐） |

### Tauri Integration

| Package | Version | Purpose |
|---------|---------|---------|
| `@tauri-apps/api` | ^2.11 | Tauri 前端 API（invoke、event listen） |
| `@tauri-apps/plugin-dialog` | ^2.7 | 原生文件选择对话框（加载协议文件） |

### Development Quality

| Package | Version | Purpose |
|---------|---------|---------|
| `@biomejs/biome` | ^2.4 | Linting + Formatting（替代 eslint + prettier） |
| `vitest` | ^4.1 | 单元测试 |
| `@testing-library/react` | ^16.3 | React 组件测试 |
| `jsdom` | ^29.1 | 测试环境 DOM 模拟 |

### 完整安装命令

```bash
# 创建项目
pnpm create vite frontend --template react-ts
cd frontend

# Core UI
pnpm add react@^19 react-dom@^19
pnpm add -D @types/react@^19 @types/react-dom@^19 typescript@^5.8

# Build
pnpm add -D vite@^6 @vitejs/plugin-react@^4.5 tailwindcss@^4 @tailwindcss/vite@^4 autoprefixer@^10

# State & Data
pnpm add zustand@^5 @tanstack/react-virtual@^3 date-fns@^4

# UI Utilities
pnpm add clsx@^2 tailwind-merge@^3 class-variance-authority@^0.7 lucide-react@^1
pnpm add sonner@^2 cmdk@^1 react-resizable-panels@^4

# Radix UI (installed via shadcn CLI, listed for reference)
pnpm add @radix-ui/react-select @radix-ui/react-dialog @radix-ui/react-tooltip \
  @radix-ui/react-tabs @radix-ui/react-switch @radix-ui/react-dropdown-menu \
  @radix-ui/react-popover @radix-ui/react-separator @radix-ui/react-toggle \
  @radix-ui/react-label @radix-ui/react-slot @radix-ui/react-scroll-area

# Monaco Editor
pnpm add @monaco-editor/react@^4

# i18n
pnpm add i18next@^26 react-i18next@^17 i18next-browser-languagedetector@^8

# Forms
pnpm add react-hook-form@^7 zod@^4 @hookform/resolvers@^5

# Tauri
pnpm add @tauri-apps/api@^2 @tauri-apps/plugin-dialog@^2

# Dev Quality
pnpm add -D @biomejs/biome@^2 vitest@^4 @testing-library/react@^16 jsdom@^29

# Init shadcn/ui
npx shadcn@latest init
```

---

## 3. Application Structure

```
frontend/
├── index.html
├── package.json
├── pnpm-lock.yaml
├── tsconfig.json
├── vite.config.ts
├── tailwind.config.ts
├── postcss.config.js
├── components.json                  # shadcn/ui config
├── public/
└── src/
    ├── main.tsx                     # Entry point
    ├── App.tsx                      # Root: providers + router
    ├── index.css                    # Tailwind base + custom theme
    │
    ├── components/
    │   ├── ui/                      # shadcn/ui primitives (button, select, etc.)
    │   ├── layout/
    │   │   ├── AppShell.tsx         # Sidebar + content + status bar
    │   │   ├── Sidebar.tsx          # Navigation sidebar
    │   │   └── StatusBar.tsx        # Bottom status bar
    │   ├── terminal/
    │   │   ├── ConnectionBar.tsx    # Port config + connect/disconnect
    │   │   ├── RxViewer.tsx         # Received data display
    │   │   ├── TxSender.tsx         # Data input + send
    │   │   ├── QuickCommands.tsx    # Quick command bar
    │   │   └── DataExport.tsx       # Export dropdown (TXT/CSV/JSON)
    │   ├── virtual/
    │   │   └── VirtualPortsPage.tsx # Virtual port management
    │   ├── scripts/
    │   │   └── ScriptsPage.tsx      # Lua script editor + management
    │   ├── protocols/
    │   │   └── ProtocolsPage.tsx    # Protocol management
    │   └── settings/
    │       └── SettingsPage.tsx     # App configuration
    │
    ├── stores/
    │   ├── connection.ts            # Port connect/disconnect/status
    │   ├── data.ts                  # RX/TX data buffer + display format
    │   ├── virtualPort.ts           # Virtual port CRUD
    │   ├── protocol.ts              # Protocol list + active protocol
    │   ├── script.ts                # Script CRUD + execution
    │   ├── serialScript.ts          # Script attach/detach to port
    │   ├── settings.ts              # App config
    │   └── ui.ts                    # Navigation + sidebar state
    │
    ├── hooks/
    │   ├── useTauriCommand.ts       # Generic Tauri invoke wrapper
    │   ├── useTauriEvent.ts         # Tauri event listener hook
    │   └── useKeyboardShortcuts.ts  # Global keyboard shortcuts
    │
    ├── lib/
    │   ├── tauri-api.ts             # Typed Tauri command definitions
    │   └── utils.ts                 # cn() helper + formatters
    │
    ├── i18n/
    │   ├── index.ts                 # i18next init
    │   └── locales/
    │       ├── en.json
    │       └── zh.json
    │
    └── types/
        └── index.ts                 # Shared TypeScript types
```

---

## 4. Page Architecture

### Navigation Structure

5 pages accessible via icon sidebar (48px wide):

| Icon | Page | Route | Description |
|------|------|-------|-------------|
| Terminal | Terminal | `/` (default) | Main workbench — 80% of usage |
| Split | Virtual Ports | `/virtual` | Virtual serial port pairs |
| Code | Scripts | `/scripts` | Lua script editor |
| Layers | Protocols | `/protocols` | Protocol management |
| Settings | Settings | `/settings` | App configuration |

No React Router — use Zustand `uiStore` for view switching (SPA, no URL routing needed in Tauri desktop app).

---

## 5. Main Workbench (Terminal Page)

The terminal page is the primary interface. It uses a vertical split layout with a collapsible right panel.

```
┌──────────────────────────────────────────────────────────────────┐
│  ConnectionBar                                                    │
│  [▼ Port] [▼ Baud] [▼ Data] [▼ Stop] [▼ Parity] [▼ Flow]       │
│  [● Connect]  Protocol: [▼ None]     Script: [▼ None]           │
├──────────────────────────────────────────────────────┬───────────┤
│                                                      │  Stats    │
│  RX Data Viewer                                      │  RX: 1.2K │
│  ┌────────────────────────────────────────────────┐  │  TX: 456  │
│  │ # │ Timestamp    │ HEX          │ ASCII        │  │  Pkts: 18 │
│  │ 1 │ 14:23:01.123 │ 48 65 6C 6C │ Hell         │  │           │
│  │ 2 │ 14:23:01.456 │ 6F 20 57 6F │ o Wo         │  │  Actions  │
│  │ 3 │ 14:23:02.789 │ 72 6C 64 21 │ rld!         │  │ [Action1] │
│  │ ...                                            │  │ [Action2] │
│  └────────────────────────────────────────────────┘  │           │
│  [HEX] [ASCII] [Mixed]  [🔍 Search]  [Clear] [Export]│  Protocol │
├──────────────────────────────────────────────────────┤  ModbusRTU │
│  TX Sender                                           │           │
│  ┌────────────────────────────────────────────────┐  │  Quick    │
│  │ Input area (multiline)                          │  │  Cmds     │
│  │                                                 │  │  [AT]     │
│  └────────────────────────────────────────────────┘  │  [Reset]  │
│  [HEX/ASCII] [Send ▶] [Send Loop ↻]                  │  [Ping]   │
├──────────────────────────────────────────────────────┴───────────┤
│  StatusBar: ● Connected /dev/ttyUSB0 @ 115200 │ RX: 1.2KB TX: 456B │
└──────────────────────────────────────────────────────────────────┘
```

### 5.1 ConnectionBar

Top toolbar spanning full width. Contains all connection parameters and quick actions.

**Components:**
- **Port selector**: Dropdown listing available ports (from `list_ports`). Shows port name + type. Auto-refresh on click open.
- **Baud rate**: Dropdown with common values (9600, 19200, 38400, 57600, 115200, 230400, 460800, 921600) + custom input.
- **Data bits**: Dropdown (5, 6, 7, 8). Default: 8.
- **Stop bits**: Dropdown (1, 2). Default: 1.
- **Parity**: Dropdown (None, Odd, Even). Default: None.
- **Flow control**: Dropdown (None, Software, Hardware). Default: None.
- **Connect/Disconnect button**: Primary action. Green when disconnected, red when connected.
- **Protocol selector**: Dropdown of loaded protocols (from `list_protocols`). "None" = raw mode. On change, calls `set_port_protocol`.
- **Script selector**: Dropdown of scripts with attach/detach (from `list_scripts`). "None" = no script.

**Behavior:**
- On connect: calls `open_port` with config, then `start_sniffing`. Stores port_id.
- On disconnect: calls `stop_sniffing` then `close_port`. Clears port_id.
- Port selector auto-refreshes when opened (debounced `list_ports` call).
- Last used config persisted in localStorage.

### 5.2 RX Data Viewer

Main data display area occupying ~65% of vertical space. Uses virtual scrolling for performance.

**Data format modes:**
- **HEX**: `48 65 6C 6C 6F 20 57 6F 72 6C 64 21`
- **ASCII**: `Hello World!`
- **Mixed**: Table with columns `# | Timestamp | HEX | ASCII`

**Features:**
- Auto-scroll to latest data (toggle-able)
- Timestamp per packet (from event payload)
- Search/filter within displayed data
- Clear buffer button
- Export dropdown (TXT, CSV, JSON) via `DataExport` component
- Packet count badge
- Protocol decode toggle: when active protocol is set and toggle enabled, shows decoded data column

**Data flow:**
- Listens to `data-received` Tauri event
- Appends to Zustand `dataStore` FIFO buffer (max configurable, default 10000 packets)
- Virtual rendering via `@tanstack/react-virtual`

### 5.3 TX Sender

Data input area at the bottom ~35% of vertical space.

**Features:**
- Multiline text input
- HEX/ASCII mode toggle (affects how input is parsed before sending)
- Send button: calls `send_data` with parsed bytes
- Send Loop: periodic send with configurable interval
- History: up/down arrow cycles through sent commands
- Protocol encode toggle: when active protocol is set, calls `protocol_encode` before sending

**HEX input parsing:**
- Accepts `48 65 6C 6C 6F` or `4865 6C6C 6F20` format
- Validates hex characters before send
- Shows byte count preview

### 5.4 Right Panel (Collapsible)

270px wide panel, collapsible via button. Contains 3 sections stacked vertically:

**Section 1: Port Statistics**
- RX/TX bytes (formatted: 1.2KB, 3.4MB)
- RX/TX packet counts
- Connection duration
- Data from `get_port_status` (polled every 2s when connected)

**Section 2: Script Actions**
- When a script is attached, lists discovered UI actions (from `list_script_actions`)
- Each action renders as a button with label/icon from UiAction
- Click calls `call_script_function`

**Section 3: Quick Commands**
- User-defined quick command buttons
- Each has: label, data payload, format (hex/ascii)
- Persisted in localStorage
- Add/edit/delete via dialog
- Click sends data via `send_data`

### 5.5 StatusBar

Full-width bar at the bottom (28px height):

- Left: Connection status indicator (dot + text: "Connected /dev/ttyUSB0 @ 115200" or "Disconnected")
- Center: Active protocol name (if any)
- Right: RX/TX byte counters (live), current timestamp

---

## 6. Virtual Ports Page

Management interface for virtual serial port pairs.

```
┌──────────────────────────────────────────────────────────────────┐
│  Virtual Serial Ports                                             │
│                                                                   │
│  [+ Create New Pair]     Backend: [▼ PTY]                        │
│                                                                   │
│  ┌────────────────────────────────────────────────────────────┐  │
│  │ Pair: vp-001    Backend: PTY    Status: ● Running          │  │
│  │ Port A: /dev/pts/3    Port B: /dev/pts/4                  │  │
│  │ Uptime: 5m 23s    Bridged: 1.2KB / 456B                   │  │
│  │ [Stop] [Capture ▶]                                          │  │
│  └────────────────────────────────────────────────────────────┘  │
│                                                                   │
│  Captured Packets (when capture active):                          │
│  ┌────────────────────────────────────────────────────────────┐  │
│  │ # │ Dir │ Timestamp    │ Data (HEX)                       │  │
│  │ 1 │ A→B │ 14:23:01.123 │ 48 65 6C 6C 6F                   │  │
│  └────────────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────────────┘
```

**Tauri commands used:**
- `create_virtual_port` — create new pair
- `list_virtual_ports` — list all pairs
- `stop_virtual_port` — stop a pair
- `get_virtual_port_stats` — statistics
- `get_captured_packets` — captured data
- `check_virtual_port_health` — health check

**Features:**
- Create pair with backend selection (PTY/socat/named_pipe)
- List all active pairs with status and stats
- Start/stop capture on a pair
- View captured packets with direction indicator
- Auto-refresh list (polling every 3s)

---

## 7. Scripts Page

Lua script management with Monaco editor.

```
┌──────────────────────────────────────────────────────────────────┐
│  Scripts                                                  [+ New] │
│  ┌──────────────┬───────────────────────────────────────────┐   │
│  │ Script List   │  Monaco Editor                            │   │
│  │              │                                           │   │
│  │ ▸ hello.lua  │  -- Lua script                            │   │
│  │   modbus.lua │  function on_recv(data)                   │   │
│  │   at_cmd.lua │    log("Received: " .. data:len())        │   │
│  │              │    return data                             │   │
│  │              │  end                                       │   │
│  │              │                                           │   │
│  │              ├───────────────────────────────────────────┤   │
│  │              │  Output Console                           │   │
│  │              │  > Script executed successfully            │   │
│  └──────────────┴───────────────────────────────────────────┘   │
│  [Validate ✓] [Run ▶] [Save 💾] [Delete 🗑]                      │
└──────────────────────────────────────────────────────────────────┘
```

**Tauri commands used:**
- `list_scripts` — list saved scripts
- `save_script` — save script content
- `delete_script` — delete script file
- `execute_script` — run standalone script
- `validate_script` — syntax check
- `list_standalone_script_actions` — discover UI actions
- `call_standalone_script_function` — execute action

**Layout:**
- Left panel (200px): Script file list with selection
- Center: Monaco editor (Lua language mode)
- Bottom (150px, collapsible): Output console for execution results
- Toolbar: Validate, Run, Save, Delete buttons

**Features:**
- Lua syntax highlighting via Monaco
- Script validation with error display
- Execute and show output in console
- Save/load scripts to/from backend `~/.serial-cli/scripts/`
- Auto-save on switch (prompt if dirty)

---

## 8. Protocols Page

Protocol management interface.

```
┌──────────────────────────────────────────────────────────────────┐
│  Protocols                                              [+ Load] │
│                                                                   │
│  Built-in Protocols                                               │
│  ┌────────────────────────────────────────────────────────────┐  │
│  │ ● Modbus RTU   Industrial communication protocol           │  │
│  │ ● Modbus ASCII  ASCII-based Modbus variant                 │  │
│  │ ● AT Commands   Hayes AT command protocol                  │  │
│  │ ● Line         Line-based text protocol                    │  │
│  └────────────────────────────────────────────────────────────┘  │
│                                                                   │
│  Custom Protocols                                                 │
│  ┌────────────────────────────────────────────────────────────┐  │
│  │ ● my_protocol  Custom Lua protocol     [Reload] [Unload]  │  │
│  └────────────────────────────────────────────────────────────┘  │
│                                                                   │
│  [+ Import Protocol File]                                         │
│  Protocol Editor                                                  │
│  ┌────────────────────────────────────────────────────────────┐  │
│  │ -- Lua protocol definition                                 │  │
│  │ function encode(data) ... end                              │  │
│  │ function parse(data) ... end                               │  │
│  └────────────────────────────────────────────────────────────┘  │
│  [Validate] [Save & Load]                                         │
└──────────────────────────────────────────────────────────────────┘
```

**Tauri commands used:**
- `list_protocols` — list all protocols
- `load_protocol` — load custom protocol from file
- `unload_protocol` — unload custom protocol
- `reload_protocol` — hot-reload protocol file
- `validate_protocol` — validate protocol file
- `save_protocol_file` — save protocol content to file
- `get_protocol_info` — protocol details

**Features:**
- List built-in and custom protocols separately
- Load custom protocol from file (file picker dialog)
- Create new protocol with in-app editor (Monaco, Lua mode)
- Validate, save, and load in one workflow
- Reload custom protocols without restart
- Unload custom protocols

---

## 9. Settings Page

Application configuration panel.

```
┌──────────────────────────────────────────────────────────────────┐
│  Settings                                                         │
│                                                                   │
│  ┌──────────────┬───────────────────────────────────────────┐   │
│  │ Serial       │  Serial Defaults                          │   │
│  │ Logging      │  Baud Rate: [115200 ▼]                    │   │
│  │ Lua Engine   │  Data Bits: [8 ▼]                         │   │
│  │ Output       │  Stop Bits: [1 ▼]                         │   │
│  │ Protocols    │  Parity: [None ▼]                         │   │
│  │ Display      │  Timeout: [1000] ms                       │   │
│  │              │                                           │   │
│  │              │  Flow Control: [None ▼]                   │   │
│  │              ├───────────────────────────────────────────┤   │
│  │              │  [Save] [Reset to Defaults]               │   │
│  └──────────────┴───────────────────────────────────────────┘   │
└──────────────────────────────────────────────────────────────────┘
```

**Tauri commands used:**
- `get_config` — load current config
- `update_config` — save config changes
- `reset_config` — reset to defaults

**Sections (left tab navigation):**
- Serial: Default baud rate, data bits, stop bits, parity, timeout, flow control
- Logging: Level (debug/info/warn/error), format (text/json), file path
- Lua Engine: Memory limit, timeout, sandbox toggle
- Output: JSON pretty print, show timestamps
- Protocols: Hot-reload toggle, custom protocol directory
- Display: Theme, max packets, default data format

---

## 10. State Management Design

### Store Architecture

All stores use Zustand with `subscribeWithSelector` middleware. Each store owns a single domain.

```
uiStore (navigation, sidebar)
  │
  ├── connectionStore (port lifecycle)
  │     └── port_id, port_name, status, config
  │
  ├── dataStore (RX/TX buffer)
  │     └── packets[], format, max_packets
  │
  ├── protocolStore (protocols)
  │     └── protocols[], active_protocol
  │
  ├── scriptStore (script management)
  │     └── scripts[], current_script
  │
  ├── serialScriptStore (port-script binding)
  │     └── attached_script, script_status, actions[]
  │
  ├── virtualPortStore (virtual ports)
  │     └── ports[], stats
  │
  └── settingsStore (app config)
        └── config
```

### Store Definitions

#### `connectionStore`
```typescript
interface ConnectionStore {
  // State
  portId: string | null;
  portName: string | null;
  status: 'disconnected' | 'connecting' | 'connected' | 'error';
  config: SerialConfig;
  availablePorts: PortInfo[];
  error: string | null;

  // Actions
  refreshPorts: () => Promise<void>;
  connect: (portName: string, config: SerialConfig) => Promise<void>;
  disconnect: () => Promise<void>;
  checkHealth: () => Promise<boolean>;
}
```

**Tauri commands:** `list_ports`, `open_port`, `close_port`, `start_sniffing`, `stop_sniffing`, `check_port_health`

**Critical fix**: Do NOT call both `open_port` (which spawns its own read loop) and `start_sniffing`. Use only `start_sniffing` for data reading after `open_port`. The `open_port` call should be fixed on the backend to NOT spawn a background reader — this is the double-read-loop bug.

#### `dataStore`
```typescript
interface DataPacket {
  id: number;
  direction: 'rx' | 'tx';
  timestamp: number;
  data: Uint8Array;
  decoded?: string; // protocol decode result
}

interface DataStore {
  // State
  packets: DataPacket[];
  displayFormat: 'hex' | 'ascii' | 'mixed';
  autoScroll: boolean;
  maxPackets: number;
  searchQuery: string;

  // Actions
  addPacket: (packet: DataPacket) => void;
  clearBuffer: () => void;
  setDisplayFormat: (format: DisplayFormat) => void;
  toggleAutoScroll: () => void;
  setSearchQuery: (query: string) => void;
}
```

**Event listeners:** `data-received`, `data-sent` Tauri events.

#### `protocolStore`
```typescript
interface ProtocolStore {
  // State
  protocols: ProtocolInfo[];
  activeProtocol: string | null; // protocol name set on current port
  loading: boolean;

  // Actions
  loadProtocols: () => Promise<void>;
  setActiveProtocol: (portId: string, protocolName: string | null) => Promise<void>;
  loadCustomProtocol: (path: string) => Promise<void>;
  unloadProtocol: (name: string) => Promise<void>;
  reloadProtocol: (name: string) => Promise<void>;
}
```

**Tauri commands:** `list_protocols`, `set_port_protocol`, `load_protocol`, `unload_protocol`, `reload_protocol`

#### `scriptStore`
```typescript
interface ScriptStore {
  // State
  scripts: ScriptInfo[];
  currentScript: { name: string; content: string } | null;
  isDirty: boolean;
  output: string[];

  // Actions
  loadScriptList: () => Promise<void>;
  openScript: (name: string) => Promise<void>;
  saveScript: (name: string, content: string) => Promise<void>;
  deleteScript: (name: string) => Promise<void>;
  executeScript: (content: string) => Promise<void>;
  validateScript: (content: string) => Promise<ValidationError[]>;
  newScript: () => void;
  updateContent: (content: string) => void;
}
```

**Tauri commands:** `list_scripts`, `save_script`, `delete_script`, `execute_script`, `validate_script`

#### `serialScriptStore`
```typescript
interface SerialScriptStore {
  // State
  attachedScript: string | null; // script source attached to current port
  scriptStatus: { has_script: boolean; timer_interval_ms: number } | null;
  actions: UiAction[];

  // Actions
  attachScript: (portId: string, scriptSource: string) => Promise<void>;
  detachScript: (portId: string) => Promise<void>;
  refreshStatus: (portId: string) => Promise<void>;
  loadActions: (portId: string) => Promise<void>;
  callAction: (portId: string, functionName: string) => Promise<string>;
}
```

**Tauri commands:** `attach_script`, `detach_script`, `has_script`, `get_script_status`, `list_script_actions`, `call_script_function`

#### `virtualPortStore`
```typescript
interface VirtualPortStore {
  // State
  ports: VirtualPortInfo[];
  selectedPort: string | null;
  capturedPackets: CapturedPacket[];

  // Actions
  refreshPorts: () => Promise<void>;
  createPort: (config: CreateVirtualPortConfig) => Promise<void>;
  stopPort: (id: string) => Promise<void>;
  getStats: (id: string) => Promise<VirtualPortStats>;
  capturePackets: (id: string) => Promise<void>;
}
```

**Tauri commands:** `create_virtual_port`, `list_virtual_ports`, `stop_virtual_port`, `get_virtual_port_stats`, `get_captured_packets`

#### `settingsStore`
```typescript
interface SettingsStore {
  // State
  config: ConfigData | null;
  loading: boolean;

  // Actions
  loadConfig: () => Promise<void>;
  updateConfig: (config: ConfigData) => Promise<void>;
  resetConfig: () => Promise<void>;
}
```

**Tauri commands:** `get_config`, `update_config`, `reset_config`

#### `uiStore`
```typescript
interface UIStore {
  // State
  currentPage: 'terminal' | 'virtual' | 'scripts' | 'protocols' | 'settings';
  sidebarCollapsed: boolean;
  rightPanelCollapsed: boolean;
  locale: 'en' | 'zh';

  // Actions
  navigateTo: (page: string) => void;
  toggleSidebar: () => void;
  toggleRightPanel: () => void;
  setLocale: (locale: 'en' | 'zh') => void;
}
```

---

## 11. Tauri Integration Layer

### Event System

| Event | Direction | Payload | Consumer |
|-------|-----------|---------|----------|
| `data-received` | Backend → Frontend | `{ port_id, data: number[], timestamp, direction: "rx" }` | `dataStore.addPacket()` |
| `data-sent` | Backend → Frontend | `{ port_id, data: number[], timestamp, direction: "tx" }` | `dataStore.addPacket()` |
| `port-status-changed` | Backend → Frontend | `{ port_id, status, timestamp }` | `connectionStore` |
| `error-occurred` | Backend → Frontend | `{ error, timestamp }` | Toast notification |
| `virtual-port-created` | Backend → Frontend | `{ port_id, port_info, timestamp }` | `virtualPortStore` |
| `virtual-port-stopped` | Backend → Frontend | `{ port_id, timestamp }` | `virtualPortStore` |

### Custom Hooks

#### `useTauriCommand<TArgs, TResult>(command: string)`
```typescript
// Generic wrapper for Tauri invoke with loading/error state
const [result, loading, error] = useTauriCommand('list_ports', { autoInvoke: false });
const { invoke, loading, error } = useTauriCommand('send_data');
```

#### `useTauriEvent(event: string, handler: (payload: any) => void)`
```typescript
// Auto-cleanup Tauri event listener
useTauriEvent('data-received', (payload) => {
  dataStore.addPacket(payload);
});
```

### TypeScript Types for Tauri API

```typescript
// types/index.ts

interface PortInfo {
  port_name: string;
  port_type: string;
  is_virtual: boolean;
  virtual_id: string | null;
}

interface SerialConfig {
  baudrate: number;
  databits: number;
  stopbits: number;
  parity: string;
  timeout_ms: number;
  flow_control: string;
}

interface PortStatus {
  id: string;
  port_name: string;
  is_open: boolean;
  config: SerialConfig | null;
  stats: PortStats;
}

interface PortStats {
  bytes_sent: number;
  bytes_received: number;
  packets_sent: number;
  packets_received: number;
  last_activity: number | null;
}

interface ProtocolInfo {
  name: string;
  description: string;
}

interface ScriptInfo {
  name: string;
  path: string;
  size: number;
  modified: number; // unix timestamp
}

interface UiAction {
  function_name: string;
  label: string;
  icon: string | null;
  group: string | null;
  confirm: boolean;
}

interface VirtualPortInfo {
  id: string;
  port_a: string;
  port_b: string;
  backend: string;
  created_at: string;
  uptime_secs: number;
  running: boolean;
}

interface ConfigData {
  serial: SerialConfigData;
  logging: LoggingConfigData;
  lua: LuaConfigData;
  task: TaskConfigData;
  output: OutputConfigData;
  protocols: ProtocolsConfigData;
  virtual_ports: VirtualPortsConfigData;
  display: DisplayConfigData;
}
```

---

## 12. Tauri Backend Fixes

### 12.1 Fix Double Read Loop Bug

**Problem:** `open_port` (port.rs:109-153) spawns a background read task AND `start_sniffing` (serial.rs:115-194) spawns another. Both compete for the same port's data.

**Fix:** Remove the background read task from `open_port`. Data reading should be exclusively handled by `start_sniffing` (the sniffer approach with proper event emission).

The `open_port` command should only:
1. Open the port with config
2. Return the port_id
3. NOT spawn any background reader

The frontend workflow becomes:
1. `open_port(port_name, config)` → get port_id
2. `start_sniffing(port_id)` → begin data capture + event emission

### 12.2 Activate Event Emitters

Wire up the previously-stubbed event emitters:

- `emit_port_status_changed`: Call in `close_port` and when sniffer detects port disconnection
- `emit_error`: Call in error paths of commands (wrap with error handler)
- `emit_virtual_port_stats_updated`: Call periodically or on stats change

### 12.3 Other Fixes

- Remove `HashMap` import from `port_state.rs` (no longer needed after PortStateManager removal)
- Ensure `open_port_virtual` is properly used for both hardware and virtual ports

---

## 13. Keyboard Shortcuts

| Shortcut | Action | Context |
|----------|--------|---------|
| `Cmd/Ctrl + 1-5` | Navigate to page 1-5 | Global |
| `Cmd/Ctrl + \` | Toggle right panel | Global |
| `Cmd/Ctrl + Enter` | Send data | Terminal |
| `Cmd/Ctrl + L` | Clear RX buffer | Terminal |
| `Cmd/Ctrl + R` | Refresh port list | Terminal |
| `Cmd/Ctrl + S` | Save current script | Scripts |
| `Cmd/Ctrl + Shift + Enter` | Execute script | Scripts |
| `Cmd/Ctrl + ,` | Open settings | Global |
| `Escape` | Close dialog/modal | Global |

---

## 14. i18n Strategy

- Default locale: `zh` (Chinese)
- Supported: `zh`, `en`
- All user-visible strings go through `t('key')`
- Locale switcher in sidebar footer
- Store preference in localStorage
- Translation files: `src/i18n/locales/{zh,en}.json`

**Structure of translation keys:**
```json
{
  "common": { "connect": "连接", "disconnect": "断开", ... },
  "terminal": { "rxViewer": "接收数据", "txSender": "发送数据", ... },
  "virtual": { ... },
  "scripts": { ... },
  "protocols": { ... },
  "settings": { ... }
}
```

---

## 15. Theme

Professional dark theme inspired by VS Code, avoiding the previous "Cyber-Industrial" aesthetic.

**Color palette:**
- Background: `#1e1e2e` (base), `#181825` (deeper), `#313244` (elevated)
- Foreground: `#cdd6f4` (primary text), `#a6adc8` (secondary text)
- Accent: `#89b4fa` (blue), `#a6e3a1` (green for success/connected)
- Danger: `#f38ba8` (red)
- Warning: `#f9e2af` (yellow)
- Border: `#45475a`

**Fonts:**
- UI: System font stack (`-apple-system, BlinkMacSystemFont, "Segoe UI", ...`)
- Code: `"JetBrains Mono", "Fira Code", monospace`

**Key principles:**
- Low contrast between surface layers (subtle elevation)
- Blue accent for interactive elements
- Green/red for status indicators
- No decorative elements — every pixel serves a function

---

## 16. Build Configuration

### vite.config.ts
```typescript
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import path from "path";

export default defineConfig({
  plugins: [react()],
  resolve: {
    alias: { "@": path.resolve(__dirname, "./src") },
  },
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
  },
  envPrefix: ["VITE_", "TAURI_"],
  build: {
    target: "esnext",
  },
});
```

### tauri.conf.json updates needed
```json
{
  "build": {
    "beforeDevCommand": "cd ../frontend && pnpm dev",
    "beforeBuildCommand": "cd ../frontend && pnpm build",
    "devUrl": "http://localhost:1420",
    "frontendDist": "../frontend/dist"
  }
}
```
