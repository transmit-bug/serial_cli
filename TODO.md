# Serial CLI TODO List

**Updated**: 2026-06-06

---

## 已完成

- ✅ 修复 GitHub Actions：cargo fmt 格式违规（benches + src 共 12 文件）
- ✅ 修复 GitHub Actions：benchmarks 传递 --save-baseline 给 lib target 导致失败，改为逐个 bench 运行
- ✅ 修复 GitHub Actions：release.yml 中 generate_release_notes 与自定义 body 冲突，prev_tag 引用不存在的变量
- ✅ 修复 GitHub Actions：release.yml 无法被 Dependabot 解析（YAML body 中模板语法问题）
- ✅ 添加 RUSTSEC-2025-0069（daemonize unmaintained）advisory skip，暂无安全替代方案
- ✅ 给 RxViewer 搜索高亮逻辑补充测试用例（提取 splitHighlights 到 lib/highlight.ts，22 个测试）
- ✅ 给 ConnectionStore 补充测试用例（连接/断开/错误状态流转，17 个测试）
- ✅ DataStore 测试（包管理、搜索过滤、导出选项，16 个测试）
- ✅ PresetsStore 测试（CRUD、排序、应用预设，10 个测试）
- ✅ SettingsStore 测试（加载/更新/重置配置，6 个测试）

## 待办

（无）

---

## 跨平台改进

- ✅ monitoring 模块 macOS 支持（libc proc_pidinfo 获取内存/FD，windows crate 按平台条件编译）
