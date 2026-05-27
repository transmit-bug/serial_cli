# Serial CLI TODO List

**Updated**: 2026-05-26

---

## 已完成

- ✅ 给 RxViewer 搜索高亮逻辑补充测试用例（提取 splitHighlights 到 lib/highlight.ts，22 个测试）
- ✅ 给 ConnectionStore 补充测试用例（连接/断开/错误状态流转，17 个测试）
- ✅ DataStore 测试（包管理、搜索过滤、导出选项，16 个测试）
- ✅ PresetsStore 测试（CRUD、排序、应用预设，10 个测试）
- ✅ SettingsStore 测试（加载/更新/重置配置，6 个测试）

## 待办

- ✅ 拖拽导入 .lua 脚本 / .json 协议文件到 Editor 页面
- Batch / Benchmark CLI 功能接入 GUI

---

## 跨平台改进

- ✅ monitoring 模块 macOS 支持（libc proc_pidinfo 获取内存/FD，windows crate 按平台条件编译）
