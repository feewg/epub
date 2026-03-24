# kaf-cli (Rust) - TXT 转 EPUB 电子书转换器

> 快速将 TXT 格式小说转换为 EPUB 3.0 标准电子书

---

## 功能特性

- ✅ **傻瓜操作模式** - 拖拽文件自动转换
- ✅ **智能编码识别** - 自动识别 UTF-8/GBK/Big5 编码
- ✅ **自动章节识别** - 支持中文、英文常见章节格式
- ✅ **自定义样式** - CSS 样式、字体、对齐、缩进等
- ✅ **封面支持** - 本地封面图片和章节页眉图片
- ✅ **批量转换** - 批量处理整个文件夹
- ✅ **配置文件** - YAML 配置支持，可复用设置
- ✅ **EPUB 3.0 标准** - 完全符合 EPUB 3.3 规范，通过 epubcheck 验证

---

## 下载安装

### 从 GitHub Releases 下载

访问 [https://github.com/feewg/epub/releases](https://github.com/feewg/epub/releases) 下载对应平台的二进制文件：

| 平台 | 文件名 |
|-------|--------|
| Linux x86_64 | `kaf-cli-linux-x86_64.tar.gz` |
| macOS Intel | `kaf-cli-macos-x86_64.tar.gz` |
| macOS Apple Silicon | `kaf-cli-macos-aarch64.tar.gz` |
| Windows x86_64 | `kaf-cli-windows-x86_64.zip` |
| Windows ARM | `kaf-cli-windows-aarch64.zip` |

### Linux / macOS 安装

```bash
# 解压下载的文件
tar xzvf kaf-cli-linux-x86_64.tar.gz

# 移动到系统路径
sudo mv kaf-cli /usr/local/bin/

# 验证安装
kaf-cli --version
```

### Windows 安装

```powershell
# 解压下载的 zip 文件
# 将 kaf-cli.exe 添加到 PATH 环境变量

# 验证安装
kaf-cli.exe --version
```

---

## 快速开始

### 最简单的使用方式

```bash
# 基础转换
kaf-cli --filename novel.txt

# 指定书名和作者
kaf-cli --filename novel.txt --bookname "我的小说" --author "作者名"
```

输出：`我的小说.epub`

---

## 命令行参数

### 必需参数

| 参数 | 简写 | 说明 |
|-----|-------|------|
| `--filename <FILE>` | `-f` | 输入的 TXT 文件路径 |

### 常用参数

| 参数 | 简写 | 默认值 | 说明 |
|-----|-------|---------|------|
| `--bookname <NAME>` | `-b` | 从文件名提取 | 书名 |
| `--author <AUTHOR>` | `-a` | `YSTYLE` | 作者名 |
| `--batch <DIR>` | 无 | - | 批量转换文件夹 |

### 样式控制参数

| 参数 | 简写 | 默认值 | 说明 |
|-----|-------|---------|------|
| `--indent <NUM>` | `-i` | `2` | 段落缩进字数 |
| `--align <ALIGN>` | 无 | `center` | 对齐方式：`left`, `center`, `right` |
| `--max <NUM>` | `-M` | `35` | 标题最大字数 |
| `--line-height <VALUE>` | 无 | `1.5` | 行高（如 `1.8`） |
| `--paragraph-spacing <VALUE>` | 无 | `0.5em` | 段落间距 |

### 高级参数

| 参数 | 简写 | 默认值 | 说明 |
|-----|-------|---------|------|
| `--match <REGEX>` | `-m` | 内置正则 | 章节匹配规则（正则表达式） |
| `--volume-match <REGEX>` | `-v` | 内置正则 | 卷匹配规则 |
| `--exclude <REGEX>` | `-e` | 内置正则 | 排除规则 |
| `--separate-chapter-number` | 无 | `false` | 分离章节序号和标题 |

### 资源参数

| 参数 | 简写 | 说明 |
|-----|-------|------|
| `--cover <PATH>` | 无 | 封面图片路径 |
| `--font <PATH>` | 无 | 字体文件路径（TTF/OTF） |
| `--custom-css <PATH>` | 无 | 自定义 CSS 文件路径 |
| `--extended-css <CSS>` | 无 | 内联扩展 CSS |

### 其他参数

| 参数 | 简写 | 说明 |
|-----|-------|------|
| `--config <PATH>` | `-C` | 指定配置文件 |
| `--example-config` | 无 | 生成示例配置文件 |
| `--format <FORMAT>` | 无 | 输出格式：`epub`, `all` |
| `--lang <LANG>` | `-l` | 书籍语言：`zh`, `en`, `ja` 等 |
| `--help` | `-h` | 显示帮助 |
| `--version` | `-V` | 显示版本 |

---

## 使用示例

### 1. 基础转换

```bash
# 自动识别书名和作者
kaf-cli --filename novel.txt

# 指定书名和作者
kaf-cli -f novel.txt -b "我的小说" -a "张三"
```

### 2. 自定义样式

```bash
# 调整段落缩进为 4 个字符
kaf-cli -f novel.txt --indent 4

# 设置标题左对齐
kaf-cli -f novel.txt --align left

# 自定义行高和段落间距
kaf-cli -f novel.txt --line-height 1.8 --paragraph-spacing 1em
```

### 3. 添加封面

```bash
# 使用本地封面图片
kaf-cli -f novel.txt --cover cover.jpg

# 封面图片支持格式：JPG, PNG, WEBP, GIF
```

### 4. 自定义字体

```bash
# 嵌入自定义字体
kaf-cli -f novel.txt --font fonts/NotoSansCJK.ttf

# 字体格式：TTF, OTF
```

### 5. 批量转换

```bash
# 转换整个文件夹
kaf-cli --batch ./novels/

# 批量转换时指定默认作者
kaf-cli --batch ./novels/ --author "YSTYLE"
```

### 6. 使用配置文件

```bash
# 生成示例配置
kaf-cli --example-config > kaf.yaml

# 使用配置文件转换
kaf-cli -f novel.txt --config kaf.yaml
```

---

## 配置文件

### 配置优先级

从高到低：
1. 命令行参数
2. 文件所在目录的 `kaf.yaml`
3. 当前目录的 `kaf.yaml`
4. 默认值

### 配置文件示例

```yaml
# kaf.yaml

# 基本信息
bookname: "我的小说"
author: "作者名"

# 章节识别规则（正则表达式）
chapter_match: "^第.{1,8}章"
volume_match: "^第[0-9一二三四五六七八九十零〇百千两 ]+[卷部]"
exclusion_pattern: "^第[0-9一二三四五六七八九十零〇百千两 ]+(部门|部队|部属)"

# 样式设置
max_title_length: 35
indent: 2
align: "center"
paragraph_spacing: "0.5em"
line_height: "1.8"

# 功能开关
separate_chapter_number: false
add_tips: false

# 语言和格式
lang: "zh"
format: "epub"

# 资源文件
cover: "cover.png"
font: "fonts/custom.ttf"

# CSS 变量
css_variables:
  --primary-color: "#333333"
  --background-color: "#ffffff"
  --font-family: "Microsoft YaHei"

# 扩展 CSS（内联）
extended_css: |
  .chapter-title {
    color: #333;
    border-bottom: 2px solid #666;
  }
  .chapter-content {
    font-family: "Microsoft YaHei", serif;
  }

# 章节页眉图片
chapter_header:
  image: "header.png"
  position: "center"
  height: "100px"
  width: "100%"
  mode: "single"
```

---

## 常见章节格式

kaf-cli 内置支持以下章节格式：

### 中文格式
- 第一章、第二章、第三十章...
- 第1章、第100章...
- 卷一、第一卷、卷一百...
- 引子、楔子、序章...
- 番外、番外1、番外篇...
- 完本感言...

### 英文格式
- Chapter 1, Chapter 2...
- Section 1.1, Section 2.5...
- Page 1, Page 100...

### 纯数字格式
- 1、2、3...
- 第1、第2、第3...

### 自定义章节匹配

如果内置格式不满足需求，可以使用正则表达式自定义：

```bash
# 匹配 "001-标题" 格式
kaf-cli -f novel.txt --match "^\d{3}-(.+)$"

# 匹配 "[章节名]" 格式
kaf-cli -f novel.txt --match "^\[(.+)\]$"
```

---

## 高级功能

### 1. 章节页眉图片

为每个章节添加页眉图片：

```yaml
# 所有章节使用同一张图片
chapter_header:
  image: "header.png"
  position: "center"
  mode: "single"
```

```yaml
# 根据章节名匹配不同图片
chapter_header:
  image_folder: "headers/"
  mode: "folder"
  # headers/001.png 用于第一章
  # headers/002.png 用于第二章
```

### 2. 分离章节序号

将章节序号和标题分开显示：

```bash
kaf-cli -f novel.txt --separate-chapter-number
```

效果：
```
第一章
-------
序言内容
```

### 3. 多语言支持

支持多种书籍语言：

```bash
# 中文
kaf-cli -f novel.txt --lang zh

# 英文
kaf-cli -f novel.txt --lang en

# 日文
kaf-cli -f novel.txt --lang ja

# 德语、法语、意大利语、西班牙语、葡萄牙语、俄语、荷兰语等
```

---

## 输出文件

### 文件名规则

- 未指定书名时：从文件名提取
- 指定书名时：使用指定名称
- 扩展名：`.epub`

### 目录结构

生成的 EPUB 文件包含：
```
epub文件
├── mimetype                    # EPUB MIME 类型
├── META-INF/
│   ├── container.xml           # 容器信息
│   └── com.apple.ibooks.display-options.xml  # iBooks 兼容
└── OEBPS/
    ├── content.opf             # 元数据和清单
    ├── nav.xhtml              # EPUB 3 导航
    ├── toc.ncx               # 传统目录
    ├── toc.xhtml             # 目录页面
    ├── stylesheet.css         # 样式表
    ├── chapter_0.xhtml       # 第1章
    ├── chapter_1.xhtml       # 第2章
    └── ...
```

---

## 兼容性

### 验证标准

✅ 通过 **EPUB 3.3** 标准验证（使用 EPUBCheck）

### 支持的阅读器

- ✅ Apple Books (iBooks)
- ✅ Adobe Digital Editions
- ✅ Calibre
- ✅ Readium
- ✅ Aldiko
- ✅ Moon+ Reader
- ✅ FBReader
- ✅ Amazon Kindle (需先转换为 AZW3/MOBI)

### 操作系统

- ✅ Linux (x86_64)
- ✅ macOS (x86_64, Apple Silicon M1/M2/M3)
- ✅ Windows (x86_64, ARM)

---

## 常见问题

### Q: 中文显示乱码怎么办？

A: kaf-cli 会自动检测文件编码（UTF-8/GBK/Big5），如果仍有问题，可以手动转换文件编码为 UTF-8。

### Q: 章节识别不准确？

A: 使用 `--match` 参数自定义章节匹配规则，或创建配置文件设置正则表达式。

### Q: 如何调整阅读体验？

A: 使用配置文件设置 `indent`（缩进）、`line-height`（行高）、`paragraph-spacing`（段落间距）等参数。

### Q: 批量转换时如何控制并发数？

A: 当前版本固定使用合理的并发数，未来版本可能会添加并发控制参数。

### Q: 生成的 EPUB 在 Kindle 上无法打开？

A: Kindle 不直接支持 EPUB 格式，需要使用 Calibre 或其他工具转换为 AZW3/MOBI 格式。

---

## 性能参考

| 操作 | 性能指标 |
|-----|---------|
| 章节解析 | > 400 章/秒 |
| EPUB 生成 | > 300 章/秒 |
| 内存占用 | < 200MB |
| 编码检测 | 毫秒级 |

---

## 完整示例

### 示例 1：转换单本小说

```bash
kaf-cli \
  --filename "三体.txt" \
  --bookname "三体" \
  --author "刘慈欣" \
  --cover "cover.jpg" \
  --font "fonts/NotoSansCJK.ttf" \
  --lang zh
```

### 示例 2：使用配置文件

创建 `kaf.yaml`：
```yaml
bookname: "盗墓笔记"
author: "南派三叔"
chapter_match: "^第.{1,8}章"
indent: 3
align: "center"
line-height: "1.8"
extended_css: |
  .chapter-title {
    color: #8B4513;
    font-weight: bold;
  }
```

执行转换：
```bash
kaf-cli --filename "盗墓笔记.txt" --config kaf.yaml
```

### 示例 3：批量转换

```bash
# 转换 novels 文件夹下所有 TXT 文件
kaf-cli --batch ./novels/ --author "网络文学"

# 为每本书查找同目录的封面图片
# 封面文件名：cover.jpg, cover.png, 封面.jpg, 封面.png
```

---

## 获取帮助

```bash
# 查看完整帮助
kaf-cli --help

# 查看版本
kaf-cli --version

# 生成配置文件模板
kaf-cli --example-config
```

---

## 相关链接

- **GitHub 仓库**: https://github.com/feewg/epub
- **下载页面**: https://github.com/feewg/epub/releases
- **问题反馈**: https://github.com/feewg/epub/issues
- **原 Go 版本**: https://github.com/ystyle/kaf-cli
- **EPUB 3.0 规范**: https://www.w3.org/publishing/epub3/
