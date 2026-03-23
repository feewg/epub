# kaf-cli - Rust 版本

将 TXT 文本转换为 EPUB 电子书的命令行工具

## 功能特性

- ✅ 傻瓜操作模式（拖拽文件自动转换）
- ✅ 自动识别书名和章节
- ✅ 自动识别字符编码（解决中文乱码）
- ✅ 自定义章节/卷标题识别规则
- ✅ 自定义 CSS 样式
- ✅ 输出 EPUB 格式
- ⚠️  批量转换文件夹（部分实现）
- ⚠️  自定义封面（基础实现）

## 快速开始

### 安装

从源代码编译：

```bash
cargo build --release
```

编译后的可执行文件位于 `target/release/kaf-cli` (Windows 上是 `kaf-cli.exe`)

### 基本用法

```bash
# 转换单个文件
kaf-cli --filename novel.txt --bookname "我的小说" --author "作者名"

# 批量转换文件夹
kaf-cli --batch ./novels

# 生成示例配置文件
kaf-cli --example-config > kaf.yaml

# 使用配置文件
kaf-cli --filename novel.txt --config kaf.yaml
```

## 命令行参数

```
-f, --filename <FILE>          txt 文件名
-b, --bookname <NAME>          书名
-a, --author <AUTHOR>          作者（默认: YSTYLE）
-m, --match <REGEX>            章节匹配规则
-v, --volume-match <REGEX>     卷匹配规则
-e, --exclude <REGEX>          排除规则
--max <NUM>                    标题最大字数（默认: 35）
-i, --indent <NUM>             段落缩进字数（默认: 2）
--align <ALIGN>                对齐方式（left, center, right）
--cover <PATH>                 封面图片
--format <FORMAT>              输出格式（epub, all）
--batch <DIR>                  批量转换文件夹
--example-config               生成示例配置
-C, --config <PATH>            指定配置文件
-l, --lang <LANG>              书籍语言（默认: zh）
--separate-chapter-number      分离章节序号和标题
--custom-css <PATH>           自定义 CSS 文件
--extended-css <CSS>          扩展 CSS（内联）
-h, --help                     显示帮助
-V, --version                  显示版本
```

## 配置文件

配置文件支持 YAML 格式，默认查找顺序：
1. 文件所在目录的 `kaf.yaml`
2. 当前目录的 `kaf.yaml`

### 配置文件示例

```yaml
# 书名
bookname: "示例小说"

# 作者
author: "YSTYLE"

# 章节匹配规则（正则表达式）
chapter_match: "^第.{1,8}章"

# 卷匹配规则（正则表达式）
volume_match: "^第[0-9一二三四五六七八九十零〇百千两 ]+[卷部]"

# 排除规则（正则表达式）
exclusion_pattern: "^第[0-9一二三四五六七八九十零〇百千两 ]+(部门|部队|部属)"

# 标题最大字数
max_title_length: 35

# 段落缩进字数
indent: 2

# 标题对齐方式 (left, center, right)
align: "center"

# 段落间距
paragraph_spacing: "0.5em"

# 是否添加教程
add_tips: false

# 书籍语言 (zh, en, de, fr, it, es, ja, pt, ru, nl)
lang: "zh"

# 输出格式 (epub, all)
format: "all"

# 是否分离章节序号和标题
separate_chapter_number: false

# 自定义 CSS 文件路径
# custom_css: "custom.css"

# 扩展 CSS（内联）
# extended_css: |
#   .content {
#     font-size: 18px;
#   }

# CSS 变量
# css_variables:
#   primary-color: "#333333"
#   background-color: "#ffffff"
```

## 项目结构

```
kaf-rs/
├── src/
│   ├── main.rs              # 程序入口
│   ├── cli.rs               # CLI 参数解析
│   ├── config.rs            # 配置管理
│   ├── model.rs             # 数据模型
│   ├── parser.rs            # TXT 解析器
│   ├── converter/            # 格式转换器
│   │   └── epub.rs        # EPUB 生成器
│   ├── utils/               # 工具函数
│   │   ├── encoding.rs     # 编码检测
│   │   ├── html.rs         # HTML 处理
│   │   ├── regex.rs        # 正则工具
│   │   ├── file.rs         # 文件操作
│   │   └── cover.rs        # 封面处理
│   ├── batch.rs             # 批量转换
│   └── error.rs             # 错误定义
├── benches/                 # 性能测试
├── docs/                    # 文档
├── assets/                  # 资源文件
└── Cargo.toml              # 项目配置
```

## 开发状态

### 已完成 (阶段一：基础架构搭建)

- ✅ 项目初始化和配置
- ✅ 数据模型实现
- ✅ 错误处理系统
- ✅ CLI 参数解析
- ✅ 配置文件管理
- ✅ 基本编码检测
- ✅ TXT 解析器（基础版本）
- ✅ EPUB 生成器（基础版本）
- ✅ 基本测试框架

### 进行中 (阶段二：核心功能完善)

- ⚠️ 编码检测优化（需要更完善的实现）
- ⚠️ 正则表达式缓存优化
- ⚠️ 测试覆盖率提升

### 待开发 (后续阶段)

- ⏳ 批量转换并发处理
- ⏳ 封面处理完善（Orly API）
- ⏳ 性能优化和基准测试
- ⏳ 文档完善
- ⏳ 交叉编译和打包

## 技术栈

- **语言**: Rust 2021 Edition
- **CLI 框架**: clap 4.5
- **异步运行时**: tokio 1.37
- **正则表达式**: regex 1.10
- **编码检测**: encoding_rs 0.8
- **EPUB 生成**: zip 0.6
- **序列化**: serde + serde_yaml
- **HTTP 客户端**: reqwest 0.12
- **图片处理**: image 0.25
- **错误处理**: anyhow + thiserror
- **日志**: tracing + tracing-subscriber

## 性能目标

- 4000 章解析时间 < 10s
- EPUB 生成速度 > 300 章/s
- 内存占用 < 200MB

## 测试

运行测试：
```bash
cargo test
```

运行性能测试：
```bash
cargo bench
```

## 开发计划

详细的开发计划请参见 `../plan/` 目录下的文档：

- `tasks.md` - 任务清单和进度追踪
- `architecture.md` - 技术架构文档
- `rust-refactor-plan.md` - 详细重构计划
- `GETTING_STARTED.md` - 开发入门指南

## 许可证

MIT License

## 贡献

欢迎贡献！请遵循以下步骤：

1. Fork 本仓库
2. 创建特性分支 (`git checkout -b feature/AmazingFeature`)
3. 提交更改 (`git commit -m 'Add some AmazingFeature'`)
4. 推送到分支 (`git push origin feature/AmazingFeature`)
5. 开启 Pull Request

## 鸣谢项目

- [kaf-cli (Go 版本)](https://github.com/ystyle/kaf-cli) - 原始 Go 实现
- [epub-builder crate](https://crates.io/crates/epub-builder) - EPUB 生成库
