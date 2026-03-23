# kaf-cli (Rust) - AI Agent Skill

> Rust 实现的 TXT 转 EPUB 电子书转换器

---

## 项目概述

kaf-cli 是一个命令行工具，用于将 TXT 格式的小说文本转换为 EPUB 3.0 标准电子书。它是 Go 版本 kaf-cli 的 Rust 重构版本，具有更好的性能和跨平台支持。

### 核心功能
- ✅ 傻瓜操作模式（拖拽文件自动转换）
- ✅ 自动识别字符编码（UTF-8/GBK/Big5）
- ✅ 自动识别书名和章节
- ✅ 本地封面图片支持
- ✅ 章节页眉图片支持
- ✅ 自定义 CSS 样式
- ✅ CSS 变量注入
- ✅ 批量转换文件夹
- ✅ EPUB 3.0 标准输出（通过 epubcheck 验证）

---

## 命令行用法

### 基本转换
```bash
# 转换单个文件
kaf-cli --filename novel.txt --bookname "小说名" --author "作者"

# 自动生成输出文件名
kaf-cli -f novel.txt -b "小说名" -a "作者"
```

### 批量转换
```bash
# 批量转换整个文件夹
kaf-cli --batch ./novels/ --author "YSTYLE"

# 递归扫描子文件夹
kaf-cli --batch ./novels/ --recursive
```

### 自定义样式
```bash
# 自定义章节匹配规则
kaf-cli -f novel.txt -m "^第.{1,8}章"

# 自定义段落缩进
kaf-cli -f novel.txt --indent 4

# 自定义对齐方式
kaf-cli -f novel.txt --align left

# 添加封面
kaf-cli -f novel.txt --cover cover.jpg

# 嵌入自定义字体
kaf-cli -f novel.txt --font fonts/custom.ttf

# 设置行高
kaf-cli -f novel.txt --line-height 1.8
```

### 生成示例配置
```bash
kaf-cli --example-config > kaf.yaml
```

---

## 配置文件

### 配置优先级（从高到低）
1. 命令行参数
2. 文件所在目录的 `kaf.yaml`
3. 当前目录的 `kaf.yaml`
4. 默认值

### 配置文件示例
```yaml
# kaf.yaml
bookname: "示例小说"
author: "作者名"
chapter_match: "^第.{1,8}章"
volume_match: "^第[0-9一二三四五六七八九十零〇百千两 ]+[卷部]"
max_title_length: 35
indent: 2
align: "center"
cover: "cover.png"
format: "epub"
lang: "zh"
add_tips: true
separate_chapter_number: false

# CSS 变量
css_variables:
  --main-color: "#333"
  --bg-color: "#fff"

# 扩展 CSS
extended_css: |
  .custom-header { font-size: 1.2em; }

# 嵌入字体文件（支持 TTF/OTF 格式）
font: "fonts/custom.ttf"

# 章节页眉图片
chapter_header:
  image: "header.png"
  position: "center"
  height: "100px"
```

---

## 项目结构

```
kaf-rs/
├── src/
│   ├── main.rs           # 程序入口
│   ├── cli.rs            # CLI 参数解析
│   ├── config.rs         # 配置管理
│   ├── model.rs          # 数据模型
│   ├── parser.rs         # TXT 解析器
│   ├── converter/
│   │   ├── mod.rs
│   │   └── epub3.rs      # EPUB 3.0 生成器
│   ├── batch.rs          # 批量转换
│   ├── error.rs          # 错误处理
│   └── utils/
│       ├── mod.rs
│       ├── encoding.rs   # 编码检测
│       ├── html.rs       # HTML 处理
│       ├── regex.rs      # 正则工具
│       ├── file.rs       # 文件操作
│       └── cover.rs      # 封面处理
├── Cargo.toml
├── README.md
└── test_input.txt        # 测试文件
```

---

## 核心模块说明

### 1. Parser (解析器)

```rust
use kaf_cli::parser::Parser;
use kaf_cli::model::Book;

let book = Book::default();
let mut parser = Parser::new(book);
let sections = parser.parse()?;
```

**功能**:
- 编码检测和转换
- 章节标题识别
- 卷结构识别
- 排除规则过滤
- HTML 内容生成

### 2. EpubConverter3 (EPUB 生成器)

```rust
use kaf_cli::converter::EpubConverter3;
use kaf_cli::model::Book;

let book = Book::default();
let converter = EpubConverter3::new(book);
let epub_data = converter.generate(&sections).await?;
```

**功能**:
- EPUB 3.0 标准生成
- 目录生成 (toc.xhtml)
- CSS 样式系统
- 封面图片嵌入
- 章节页眉图片

### 3. BatchConverter (批量转换)

```rust
use kaf_cli::batch::{BatchConverter, FolderScanner};

// 扫描文件夹
let scanner = FolderScanner::new(PathBuf::from("./novels"), true);
let books = scanner.scan_with_config()?;

// 批量转换
let converter = BatchConverter::new(4); // 4 并发
let result = converter.convert(books).await;
```

---

## 数据模型

### Book (书籍配置)
```rust
pub struct Book {
    pub filename: PathBuf,
    pub bookname: Option<String>,
    pub author: String,
    pub chapter_match: Option<String>,
    pub volume_match: Option<String>,
    pub exclusion_pattern: Option<String>,
    pub max_title_length: usize,
    pub indent: usize,
    pub align: TextAlignment,
    pub cover: Option<CoverSource>,
    pub font: Option<PathBuf>,
    pub line_height: Option<String>,
    pub add_tips: bool,
    pub lang: Language,
    pub separate_chapter_number: bool,
    pub custom_css: Option<PathBuf>,
    pub extended_css: Option<String>,
    pub css_variables: HashMap<String, String>,
    pub chapter_header: ChapterHeader,
}
```

### Section (章节)
```rust
pub struct Section {
    pub title: String,
    pub content: String,
    pub subsections: Vec<Section>,
}
```

---

## 正则表达式模式

### 默认章节匹配
```rust
"^第[0-9一二三四五六七八九十零〇百千两 ]+[章回节集幕卷部]|^[Ss]ection.{1,20}$|^[Cc]hapter.{1,20}$|^[Pp]age.{1,20}$|^\\d{1,4}$|^\\d+、$|^引子$|^楔子$|^章节目录|^章节|^序章|^最终章 \\w{1,20}$|^番外\\d?\\w{0,20}|^完本感言.{0,4}$"
```

### 默认卷匹配
```rust
"^第[0-9一二三四五六七八九十零〇百千两 ]+[卷部]"
```

### 默认排除规则
```rust
"^第[0-9一二三四五六七八九十零〇百千两 ]+(部门|部队|部属|部分|部件|部落|部.*：$)"
```

---

## 开发命令

### 构建
```bash
cd kaf-rs
cargo build --release
```

### 测试
```bash
# 运行所有测试
cargo test --release

# 运行特定测试
cargo test test_epub_converter

# 显示测试输出
cargo test -- --nocapture
```

### 运行
```bash
# 开发模式运行
cargo run -- --filename test.txt

# 使用发布版本
./target/release/kaf-cli --filename test.txt
```

### EPUB 验证
```bash
java -jar epubcheck/epubcheck.jar output.epub
```

---

## 常见任务

### 添加新功能
1. 在 `model.rs` 添加数据模型
2. 在对应模块实现功能
3. 添加单元测试
4. 更新 CLI 参数（如需要）

### 修改 EPUB 生成
编辑 `src/converter/epub3.rs`:
- `build_css()` - 修改 CSS 样式
- `generate_chapter_html()` - 修改章节 HTML 结构
- `generate_toc_content()` - 修改目录结构

### 添加新的编码支持
编辑 `src/utils/encoding.rs`:
- 在 `detect_and_convert()` 中添加新的编码检测逻辑

### 修改章节识别规则
编辑 `src/utils/regex.rs`:
- 修改 `DEFAULT_CHAPTER_MATCH` 常量
- 或使用自定义正则表达式

---

## 调试技巧

### 启用日志输出
```bash
RUST_LOG=info cargo run -- --filename test.txt
```

### 查看 EPUB 内容
```bash
unzip -l output.epub        # 列出文件
unzip -p output.epub OEBPS/content.opf  # 查看内容
```

### 测试正则表达式
```rust
let re = Regex::new(r"^第.{1,8}章$").unwrap();
assert!(re.is_match("第一章"));
```

---

## 性能目标

- 4000 章解析时间 < 10s
- EPUB 生成速度 > 300 章/s
- 内存占用 < 200MB

---

## 依赖库

| 库 | 用途 |
|-----|------|
| `clap` | CLI 参数解析 |
| `tokio` | 异步运行时 |
| `regex` | 正则表达式 |
| `encoding_rs` | 编码检测转换 |
| `epub-builder` | EPUB 生成 |
| `serde_yaml` | YAML 配置解析 |
| `reqwest` | HTTP 客户端 |
| `image` | 图片处理 |
| `thiserror` | 错误定义 |
| `tracing` | 日志 |

---

## 注意事项

1. **EPUB 3.0 命名空间**: 使用 `http://www.idpf.org/2007/ops` 而非 `http://www.idpf.org/2007/epub`
2. **编码处理**: 
   - 读取时：自动检测并处理 UTF-8 BOM，转换为无 BOM 的 UTF-8
   - 输出时：所有文件均为纯 UTF-8 无 BOM 格式
3. **并发控制**: 批量转换使用 Semaphore 控制并发数，避免资源耗尽
4. **CSS 优先级**: :root 变量 > 默认样式 > 自定义 CSS 文件 > 扩展 CSS

## 编码说明

### 输入文件编码支持
- **UTF-8**（有/无 BOM）- 推荐
- **GBK/GB18030**
- **Big5**

### 输出文件编码
- 所有输出文件均为 **UTF-8 无 BOM** 格式
- XML/HTML 声明: `<?xml version="1.0" encoding="UTF-8"?>`
- CSS 文件: 纯 UTF-8 无 BOM

### BOM 处理
```rust
// 读取时自动移除 BOM
let text = detect_and_convert(&bytes)?;

// 手动清理 BOM
let clean = ensure_no_bom(text);
```

---

## 相关链接

- [EPUB 3.0 规范](https://www.w3.org/publishing/epub3/)
- [epub-builder crate](https://crates.io/crates/epub-builder)
- [原 Go 版本 kaf-cli](https://github.com/ystyle/kaf-cli)
