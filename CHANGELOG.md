# Changelog

所有显著变更都将记录在此文件中。

格式基于 [Keep a Changelog](https://keepachangelog.com/zh-CN/1.0.0/)，
并且本项目遵循 [语义化版本](https://semver.org/lang/zh-CN/)。

## [Unreleased]

## [0.3.0] - 2026-07-10

### Added
- 新增逐文件批量输入结果 `BatchInput`，配置加载失败可进入批量报告和停止条件
- 新增批量跳过计数，兼容层 `BatchResult` 会保留跳过文件及原因
- 新增 Markdown 格式支持
- 新增 6 种内置主题（light/dark/sepia/high_contrast/modern/traditional）
- 新增封面自动缩放和格式转换
- 新增字体嵌入支持（TTF/OTF/WOFF/WOFF2/TTC）
- 新增批量转换报告系统（JSON/Markdown/HTML）
- 新增 EPUBCheck 验证集成
- 新增流式解析支持大文件处理
- 新增 466 个测试用例

### Changed
- `Cli` 现在通过 clap 构造并在内部记录显式参数；依赖公开结构体字面量的调用方应改用 `Cli::parse` 或 `Cli::try_parse_from`
- `ConversionSummary` 新增 `skipped_conversions`，`BatchResult` 新增 `skipped`
- CLI 资源路径相对调用目录解析；YAML 资源路径仍相对配置文件解析
- dry-run 不再为 EPUB 转换预创建输出目录；显式请求的报告仍会写入指定目录
- 优化章节识别算法，准确率提升至 ~100%
- 优化内存使用，减少 30-40% 字符串分配
- 改进错误处理和日志输出

### Fixed
- 修复批量配置错误绕过 `continue_on_error`、`max_errors` 和报告的问题
- 修复嵌套 Markdown 标题切章导致 XHTML 标签不平衡的问题
- 恢复 YAML 枚举值的大小写兼容及 `md`、`text`、`high-contrast` 等别名
- 修复流式解析中的索引越界问题
- 修复章节检测器的边界情况处理

## [0.2.0] - 2026-03-27

### Added
- Phase 6: 完整的测试套件（466 个测试）
- Phase 5: 封面增强、Markdown 支持、多主题系统
- Phase 4: 性能优化（依赖瘦身、IO优化、内存优化）
- Phase 3: 批量转换增强（错误处理、报告系统）
- Phase 2: 配置统一化、模块重构
- Phase 1: 智能章节识别系统

### Changed
- 重构项目架构，模块化设计
- 统一配置管理（CLI + YAML）

## [0.1.0] - 2025

### Added
- 基础 TXT 到 EPUB 转换功能
- CLI 命令行界面
- 基础章节识别
- 编码自动检测

---

## 版本说明

- **MAJOR**: 不兼容的 API 变更
- **MINOR**: 向下兼容的功能新增
- **PATCH**: 向下兼容的问题修复
