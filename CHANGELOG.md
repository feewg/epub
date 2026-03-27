# Changelog

所有显著变更都将记录在此文件中。

格式基于 [Keep a Changelog](https://keepachangelog.com/zh-CN/1.0.0/)，
并且本项目遵循 [语义化版本](https://semver.org/lang/zh-CN/)。

## [Unreleased]

### Added
- 新增 Markdown 格式支持
- 新增 6 种内置主题（light/dark/sepia/high_contrast/modern/traditional）
- 新增封面自动缩放和格式转换
- 新增字体嵌入支持（TTF/OTF/WOFF/WOFF2/TTC）
- 新增批量转换报告系统（JSON/Markdown/HTML）
- 新增 EPUBCheck 验证集成
- 新增流式解析支持大文件处理
- 新增 466 个测试用例

### Changed
- 优化章节识别算法，准确率提升至 ~100%
- 优化内存使用，减少 30-40% 字符串分配
- 改进错误处理和日志输出

### Fixed
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
