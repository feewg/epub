# kaf-cli - Rust 版本

> 将 TXT/Markdown 文本转换为 EPUB 电子书的高性能命令行工具

[![Rust](https://img.shields.io/badge/rust-2021-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Tests](https://img.shields.io/badge/tests-466%20passing-brightgreen.svg)]()

## ✨ 功能特性

### 核心功能
- 📖 **智能章节识别** - 自动识别中文小说章节标题（准确率 >95%）
- 🔤 **多格式支持** - 支持 TXT 和 Markdown 输入，自动格式检测
- 🎨 **多种主题** - 内置 6 种阅读主题（浅色/深色/护眼/高对比/现代/传统）
- 🖼️ **封面处理** - 自动缩放封面图片，支持 PNG/JPEG/WebP 等多种格式
- 🔤 **字体嵌入** - 支持 TTF/OTF/WOFF/WOFF2 字体文件嵌入
- 📦 **EPUB 3.0 输出** - 符合标准的 EPUB 3.0 格式

### 批量处理
- 📁 **批量转换** - 支持文件夹批量转换，递归扫描子目录
- ⚡ **并发处理** - 多文件并行转换，提升效率
- 📊 **详细报告** - JSON/Markdown/HTML 格式转换报告
- 🔍 **Dry-run 模式** - 预览转换结果，不生成实际文件

### 高级功能
- 🎯 **智能段落处理** - 3 种段落模式（逐行/空行/智能合并）
- 🏷️ **卷/章层级** - 支持卷-章两级结构
- 📝 **自定义样式** - 支持自定义 CSS 和 CSS 变量
- 🔧 **灵活配置** - YAML 配置文件 + CLI 参数双重配置

## 🚀 快速开始

### 安装

#### 从源代码编译
```bash
git clone https://github.com/your-username/kaf-cli
cd kaf-cli
cargo build --release
```

编译后的可执行文件：`target/release/kaf-cli` (Windows: `kaf-cli.exe`)

#### 系统要求
- Rust 1.75+ 
- 可选: Java 8+ (用于 EPUBCheck 验证)

### 基本用法

#### 转换单个文件
```bash
# 基础转换
kaf-cli --filename novel.txt --bookname "我的小说" --author "作者名"

# 指定主题
kaf-cli --filename novel.txt --theme sepia

# 自定义封面
kaf-cli --filename novel.txt --cover cover.jpg

# Markdown 转换
kaf-cli --filename readme.md --input-format markdown
```

#### 批量转换
```bash
# 批量转换文件夹
kaf-cli --batch ./novels --output-dir ./output

# 批量转换 + 生成报告
kaf-cli --batch ./novels --report json

# 批量转换 + 遇到错误继续
kaf-cli --batch ./novels --continue-on-error
```

#### 生成配置文件
```bash
# 生成示例配置
kaf-cli --example-config > kaf.yaml

# 使用配置文件转换
kaf-cli --filename novel.txt --config kaf.yaml
```

## 📖 使用指南

### 配置文件示例

创建 `kaf.yaml` 文件：

```yaml
# 书籍信息
bookname: "示例小说"
author: "作者名"

# 章节识别配置
chapter_match: "^第[0-9一二三四五六七八九十零〇百千两 ]+[章回节]"
volume_match: "^第[0-9一二三四五六七八九十零〇百千两 ]+[卷部]"
max_title_length: 35

# 排版设置
indent: 2                    # 段落缩进（字符数）
align: "center"              # 章节标题对齐 (left/center/right)
line_height: "1.8"          # 行高
paragraph_spacing: "0.5em"  # 段落间距

# 主题设置
theme: "light"              # 主题: light/dark/sepia/high_contrast/modern/traditional

# 输入格式
input_format: "auto"        # auto/txt/markdown

# 语言
lang: "zh"

# 高级选项
separate_chapter_number: false  # 分离章节序号和标题
add_tips: false                # 添加转换提示
```

### 命令行参数完整列表

```
文件选项:
  -f, --filename <FILE>           输入文件路径
  -o, --output-name <NAME>        输出文件名（不含扩展名）
  
书籍信息:
  -b, --bookname <NAME>           书名
  -a, --author <AUTHOR>           作者（默认: YSTYLE）
  
章节识别:
  -m, --chapter-match <REGEX>     章节匹配正则表达式
  -v, --volume-match <REGEX>      卷匹配正则表达式
  -e, --exclude <REGEX>           排除规则正则表达式
  -M, --max-title-length <NUM>    标题最大字数（默认: 35）
  
排版设置:
  -i, --indent <NUM>              段落缩进字数（默认: 2）
      --align <ALIGN>             对齐方式（left/center/right，默认: center）
      --line-height <VALUE>       行高（默认: 1.8）
      --paragraph-spacing <VALUE> 段落间距（默认: 0.5em）
      
输入格式:
  -I, --input-format <FORMAT>     输入格式（auto/txt/markdown，默认: auto）
  -l, --lang <LANG>               书籍语言（默认: zh）
  
样式设置:
      --theme <THEME>             主题（light/dark/sepia/high_contrast/modern/traditional）
      --cover <PATH>              封面图片路径
      --font <PATH>               嵌入字体文件路径
      --custom-css <PATH>         自定义 CSS 文件
      --extended-css <CSS>        扩展 CSS（内联）
      
批量转换:
      --batch <DIR>               批量转换文件夹
      --output-dir <DIR>          批量转换输出目录
      --continue-on-error         遇到错误继续转换
      --max-errors <NUM>          最大错误数量
      --report <FORMAT>           生成报告（json/markdown/html）
      --dry-run                   仅解析不生成
      --show-chapters             显示章节识别结果（dry-run 时有效）
      
配置:
  -C, --config <PATH>             指定配置文件
      --example-config            生成示例配置
      
其他:
  -h, --help                      显示帮助
  -V, --version                   显示版本
```

## 🏗️ 项目结构

```
kaf-rs/
├── src/
│   ├── main.rs              # 程序入口
│   ├── cli.rs               # CLI 参数解析
│   ├── lib.rs               # 库导出
│   ├── config/              # 配置管理
│   │   ├── loader.rs        # 配置加载
│   │   ├── validator.rs     # 配置验证
│   │   └── presets.rs       # 配置预设
│   ├── model.rs             # 数据模型
│   ├── parser/              # 解析器模块
│   │   ├── mod.rs           # 主解析器
│   │   ├── chapter_detector.rs  # 章节识别
│   │   ├── paragraph_processor.rs # 段落处理
│   │   ├── scorer.rs        # 评分机制
│   │   ├── markdown_parser.rs     # Markdown 解析
│   │   └── format_detector.rs     # 格式检测
│   ├── converter/           # 格式转换器
│   │   └── epub3.rs         # EPUB 3.0 生成器
│   ├── batch/               # 批量转换
│   │   ├── mod.rs
│   │   ├── enhanced.rs      # 增强批量转换
│   │   └── report.rs        # 报告生成
│   ├── style/               # 样式系统
│   │   ├── theme.rs         # 主题定义
│   │   └── css_generator.rs # CSS 生成
│   ├── utils/               # 工具函数
│   │   ├── encoding.rs      # 编码检测
│   │   ├── html.rs          # HTML 处理
│   │   ├── regex.rs         # 正则工具
│   │   ├── file.rs          # 文件操作
│   │   └── cover.rs         # 封面处理
│   └── error.rs             # 错误定义
├── tests/                   # 集成测试（gitignored）
├── benches/                 # 性能测试
├── docs/                    # 文档
├── plan/                    # 开发计划
└── configs/                 # 配置示例
```

## 📊 开发状态

### ✅ 已完成

#### Phase 1: 核心稳定性
- ✅ 智能章节识别系统（多维度评分机制，准确率 ~100%）
- ✅ 智能段落处理（3 种模式）
- ✅ 卷/章两级结构支持
- ✅ HTML/XHTML 安全增强

#### Phase 2: 配置与架构
- ✅ 统一配置管理系统
- ✅ CLI 参数规范化
- ✅ 模块拆分与重构
- ✅ 5 种配置预设

#### Phase 3: 批量转换
- ✅ 完整批量流程
- ✅ 错误处理增强
- ✅ 报告系统（JSON/Markdown/HTML）
- ✅ Dry-run 模式

#### Phase 4: 性能优化
- ✅ 依赖瘦身（移除不必要的依赖）
- ✅ IO 优化（流式解析）
- ✅ 内存优化（减少字符串分配）

#### Phase 5: 功能增强
- ✅ 样式系统增强（6 种主题）
- ✅ 封面与资源增强（自动缩放、格式转换）
- ✅ 多格式支持（Markdown 解析）

#### Phase 6: 测试与质量
- ✅ 单元测试增强（129 个测试）
- ✅ 集成测试（30 个测试）
- ✅ 质量监控（23 个测试）
- ✅ EPUBCheck 验证集成
- ✅ **总计 466 个测试，全部通过**

### 🚧 Phase 7: 开源与可用性（进行中）
- ⏳ 文档完善
- ⏳ 使用示例
- ⏳ 社区建设

## 🧪 测试

### 运行测试
```bash
# 所有测试
cargo test

# 仅单元测试
cargo test --lib

# 集成测试
cargo test --test test_phase52_cover
cargo test --test test_phase53_format
cargo test --test test_phase61_unit
cargo test --test test_phase62_integration
cargo test --test test_phase63_quality
cargo test --test test_epubcheck_validation

# 带输出
cargo test -- --nocapture
```

### 代码质量检查
```bash
# Clippy 检查
cargo clippy --lib  # 库代码 0 警告

# 格式化检查
cargo fmt --check
```

## 📈 性能指标

| 指标 | 目标 | 实际 |
|------|------|------|
| 章节识别准确率 | >95% | ~100% |
| 大文件处理（13MB） | <2分钟 | <2分钟 |
| 测试覆盖率 | >80% | 全面覆盖 |
| Clippy 警告 | 0 | 0（库代码） |

## 🔧 技术栈

- **语言**: Rust 2021 Edition
- **CLI**: clap 4.5
- **异步**: tokio 1.37
- **正则**: regex 1.10
- **编码**: encoding_rs 0.8
- **图片**: image 0.25
- **EPUB**: epub-builder 0.8 + zip 0.6
- **序列化**: serde + serde_yaml + serde_json
- **日志**: tracing + tracing-subscriber

## 🤝 贡献指南

欢迎贡献！请遵循以下步骤：

1. **Fork** 本仓库
2. **创建分支** (`git checkout -b feature/AmazingFeature`)
3. **提交更改** (`git commit -m 'Add some AmazingFeature'`)
4. **推送分支** (`git push origin feature/AmazingFeature`)
5. **开启 PR**

### 开发规范
- 遵循 Rust 官方风格指南
- 所有代码通过 `cargo fmt` 格式化
- 所有代码通过 `cargo clippy` 检查
- 新功能需包含测试
- 中文注释

## 📄 许可证

MIT License

## 🙏 鸣谢

- [kaf-cli (Go 版本)](https://github.com/ystyle/kaf-cli) - 原始 Go 实现
- [epub-builder](https://crates.io/crates/epub-builder) - EPUB 生成库
- [clap](https://crates.io/crates/clap) - 强大的 CLI 解析器

## 📞 联系

如有问题或建议，欢迎通过以下方式联系：
- GitHub Issues
- 邮件

---

**Happy Reading! 📚**
