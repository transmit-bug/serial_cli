# Frontend Development TODO

**Version**: v0.9.0
**Updated**: 2026-05-24

---

## P0 - 终端体验增强

### 1. 数据搜索与高亮
**Status**: ✅ Completed
**Files**: `components/terminal/RxViewer.tsx`, `stores/data.ts`
**Description**: RxViewer 支持 Ctrl+F 搜索，匹配文本高亮。区分 RX/TX 颜色标记，支持正则搜索和大小写切换。

### 2. 多端口终端
**Status**: ✅ Completed
**Files**: `components/terminal/TerminalWorkbench.tsx`, `stores/connection.ts`, `stores/data.ts`, `components/terminal/ConnectionBar.tsx`, `components/terminal/TxSender.tsx`, `components/terminal/RxViewer.tsx`
**Description**: Tab 式多端口同时连接。每个端口独立 RX/TX 缓冲区和发送区。ConnectionBar 改为端口列表，支持同时连接多个串口。


---

## P1 - 工作流与数据管理

### 5. 会话管理
**Status**: ⏳ Pending
**Files**: `stores/connection.ts`, `stores/ui.ts`, `lib/tauri-api.ts`
**Description**: 保存/恢复连接配置 + 命令历史 + 窗口布局。后端 Config 已有 connection_presets，前端需完善预设 CRUD UI，支持导入/导出预设。

### 6. 数据导出增强
**Status**: ✅ Completed
**Files**: `components/terminal/ExportControls.tsx`, `components/terminal/RightPanelHistory.tsx`, `stores/data.ts`
**Description**: 导出进度条、自定义格式模板（用户可选字段）、批量导出。支持 TXT/CSV/JSON 格式，可选择导出字段。

### 7. 命令宏/序列
**Status**: ✅ Completed
**Files**: `stores/commands.ts`, `components/terminal/SequenceEditor.tsx`, `components/terminal/SequenceList.tsx`, `components/terminal/TxSender.tsx`, `types/index.ts`
**Description**: 命令组编排：延时执行（ms 级）、条件分支（等待特定 RX 再发下一个）、循环。一键执行命令序列。TxSender 新增 Sequences tab，支持创建/编辑/执行/停止命令序列。

### 8. 连接预设同步
**Status**: ✅ Completed
**Files**: `src-tauri/src/commands/config.rs`, `stores/presets.ts`, `components/terminal/ConnectionBar.tsx`, `components/settings/SettingsPage.tsx`
**Description**: 连接预设从 localStorage 迁移到后端 Config，支持跨设备同步。前端预设编辑器（增删改排序）。

---

## P2 - 应用基础设施

### 12. Light/Dark 主题
**Status**: ⏳ Pending
**Files**: `index.css`, `stores/ui.ts`, `tailwind.config.ts`
**Description**: 完整亮色主题。CSS 变量切换（已用 `bg-surface`/`text-text` 等语义 token，改造量可控）。系统偏好自动跟随 + 手动切换。

### 13. 键盘快捷键体系
**Status**: ⏳ Pending
**Files**: `hooks/useKeyboardShortcuts.ts`
**Description**: 全局快捷键注册：Ctrl+Enter 发送、Ctrl+L 清空缓冲、Ctrl+Shift+E 切换 HEX/ASCII、Ctrl+N 新建脚本。快捷键帮助面板（Ctrl+/ 或 F1）。

### 14. 应用内日志查看器
**Status**: ⏳ Pending
**Files**: `components/settings/` (新增 LogViewer 组件), `lib/tauri-api.ts`
**Description**: 对接后端日志输出（需后端暴露日志读取 API）。前端可直接查看运行日志，支持级别过滤和搜索。

### 15. 拖拽文件导入
**Status**: ⏳ Pending
**Files**: `components/editor/EditorPage.tsx`, `components/editor/ProtocolList.tsx`
**Description**: 直接拖拽 .lua 文件到编辑器或协议列表区域导入。利用 Tauri 文件系统 API 或 web drag-and-drop。

### 16. 自动更新
**Status**: ⏳ Pending
**Files**: `App.tsx`, `lib/tauri-api.ts`
**Description**: 集成 Tauri updater 插件，发布新版本时通知用户更新。需后端配置签名和发布 URL。

---

## Completed

### v0.7.0 - Frontend Rewrite
- ✅ React 19 + Vite 8 + TypeScript + Tailwind CSS 4 + Zustand 5
- ✅ 串口终端 (连接/收发/HEX-ASCII-Mixed/导出/命令/监控/解码器)
- ✅ 虚拟串口管理 (创建/停止/捕获/CSV导出)
- ✅ 脚本引擎 (Monaco编辑器/模板/执行/验证/绑定端口)
- ✅ 协议管理 (列表/加载/卸载/编解码测试)
- ✅ 设置面板 (8标签页/配置读写/重置) + DisplayConfig 持久化
- ✅ i18n 中英文支持 + 可折叠侧边栏 + 多标签工作区

### v0.7.1 - Editor 合并
- ✅ Scripts + Protocols 合并为统一 Editor 页面
- ✅ 统一模板列表 (TemplateList)
- ✅ 面板拖拽修复 (Separator 样式 + 百分比尺寸)
- ✅ 自动加载第一个脚本内容

### v0.8.0 - Backend Hardening
- ✅ 异步 I/O 重构 (spawn_blocking + mpsc channel)
- ✅ 协议帧缓冲与流式解析
- ✅ 端口热插拔检测
- ✅ 优雅关闭与资源清理
- ✅ 端口并发与锁优化
- ✅ 协议热重载实现

### v0.9.0 - 多端口与数据管理
- ✅ 数据搜索与高亮 (Ctrl+F, 正则, 大小写)
- ✅ 多端口终端 (Tab 切换, 独立缓冲区)
- ✅ 数据导出增强 (格式选择, 字段筛选, 进度条)
- ✅ 命令宏/序列 (延时, 条件等待, 循环)
- ✅ 连接预设同步 (后端 Config, 预设编辑器)



## 超低优先级（暂不实现，等到所有功能实现之后再考虑）


### 编辑器基础优化
**Status**: ⏳ Pending
**Files**: `components/editor/EditorPage.tsx`
**Description**: 多 Tab 编辑（同时打开多个文件）、未保存标记（Tab 标题小圆点）、关闭前脏检查。不做自动补全、Diff 视图等重型功能。

### Lua API 悬停提示
**Status**: ⏳ Pending
**Files**: `components/editor/EditorPage.tsx`
**Description**: Monaco hover provider 注册 Lua API 文档（serial:write, timer:start 等），鼠标悬停显示签名和说明。不做完整 LSP。

### 外部编辑器集成
**Status**: ⏳ Pending
**Files**: `components/editor/EditorPage.tsx`
**Description**: 「在外部编辑器打开」按钮，调用系统默认编辑器打开 .lua 文件路径。支持文件系统 watcher 自动刷新内容。


### 数据可视化
**Status**: ⏳ Pending
**Files**: `components/terminal/RightPanelMonitor.tsx` (新增图表组件)
**Description**: 实时吞吐量折线图（RX/TX bytes per second）、协议字段解析树状视图。可用 lightweight 库如 uPlot 或 Recharts。

### 时间轴模式
**Status**: ⏳ Pending
**Files**: `components/terminal/` (新增 TimelineView 组件)
**Description**: 按时间线展示 RX/TX 事件，支持缩放和区间选择。便于分析通信时序。
