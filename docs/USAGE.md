# kaf-cli 使用指南

本文档详细介绍 kaf-cli 的各种使用场景和配置方法。

## 目录

- [快速开始](#快速开始)
- [基础用法](#基础用法)
- [进阶配置](#进阶配置)
- [批量转换](#批量转换)
- [主题和样式](#主题和样式)
- [常见问题](#常见问题)

---

## 快速开始

### 1. 安装

```bash
cargo build --release
```

### 2. 转换第一个文件

```bash
kaf-cli --filename novel.txt --bookname "我的小说" --author "作者"
```

输出：`我的小说.epub`

---

## 基础用法

### 命令行参数

#### 最简单的用法
```bash
kaf-cli -f novel.txt
```

程序会自动：
- 从文件名提取书名和作者
- 自动检测章节
- 使用默认配置生成 EPUB

#### 指定书名和作者
```bash
kaf-cli -f novel.txt -b "小说名" -a "作者名"
```

#### 选择主题
```bash
# 护眼模式
kaf-cli -f novel.txt --theme sepia

# 深色模式
kaf-cli -f novel.txt --theme dark

# 高对比度
kaf-cli -f novel.txt --theme high_contrast
```

#### 添加封面
```bash
kaf-cli -f novel.txt --cover cover.jpg
```

支持格式：PNG、JPEG、WebP、BMP、GIF

封面会自动缩放至适合 EPUB 的尺寸（最大 1200x1600）。

---

## 进阶配置

### 使用配置文件

#### 生成示例配置
```bash
kaf-cli --example-config > kaf.yaml
```

#### 基础配置示例
```yaml
# 书籍信息
bookname: "凡人修仙传"
author: "忘语"

# 章节识别
chapter_match: "^第[0-9一二三四五六七八九十零〇百千两 ]+[章回节]"
max_title_length: 35

# 排版
indent: 2
align: "center"
line_height: "1.8"
paragraph_spacing: "0.5em"

# 主题
theme: "sepia"

# 封面
cover:
  type: local
  path: "cover.jpg"
```

#### 使用配置文件
```bash
kaf-cli -f novel.txt -C kaf.yaml
```

### 自定义章节识别规则

如果你的小说使用特殊的章节标题格式：

```bash
# 匹配 "001 章节名" 格式
kaf-cli -f novel.txt -m "^\d{3}\s+"

# 匹配 "[第1章] 章节名" 格式
kaf-cli -f novel.txt -m "^\[第[0-9]+章\]"

# 匹配英文 "Chapter 1" 格式
kaf-cli -f novel.txt -m "^[Cc]hapter\s+\d+"
```

### 排除干扰行

某些行会被误识别为章节：

```bash
# 排除包含 "部门"、"部队" 的行
kaf-cli -f novel.txt -e "部门|部队"
```

---

## 批量转换

### 基础批量转换

```bash
kaf-cli --batch ./novels
```

扫描 `./novels` 目录下的所有 TXT 文件并转换。

### 指定输出目录

```bash
kaf-cli --batch ./novels --output-dir ./output
```

### 生成报告

```bash
# JSON 格式报告
kaf-cli --batch ./novels --report json

# Markdown 格式
kaf-cli --batch ./novels --report markdown

# HTML 格式
kaf-cli --batch ./novels --report html
```

报告文件会保存在输出目录中。

### 错误处理

```bash
# 遇到错误继续转换
kaf-cli --batch ./novels --continue-on-error

# 限制最大错误数
kaf-cli --batch ./novels --continue-on-error --max-errors 10
```

### Dry-run 模式

```bash
# 仅预览转换结果，不生成文件
kaf-cli --batch ./novels --dry-run

# 显示详细的章节识别结果
kaf-cli --batch ./novels --dry-run --show-chapters
```

---

## 主题和样式

### 内置主题

| 主题 | 说明 | 适用场景 |
|------|------|----------|
| `light` | 浅色主题 | 默认主题 |
| `dark` | 深色主题 | 夜间阅读 |
| `sepia` | 护眼模式 | 长时间阅读 |
| `high_contrast` | 高对比度 | 视力辅助 |
| `modern` | 现代简约 | 简洁风格 |
| `traditional` | 传统文学 | 古典风格 |

### 自定义 CSS

#### 使用外部 CSS 文件
```bash
kaf-cli -f novel.txt --custom-css custom.css
```

custom.css:
```css
body {
    font-size: 20px;
    line-height: 2.0;
}

h3.chapter-title {
    color: #8b4513;
    border-bottom: 1px solid #ddd;
}
```

#### 内联扩展 CSS
```bash
kaf-cli -f novel.txt --extended-css 'body { font-family: "Noto Serif SC", serif; }'
```

#### CSS 变量
```yaml
css_variables:
  primary-color: "#333"
  background-color: "#f5f5dc"
  font-size: "20px"
```

### 嵌入字体

```bash
kaf-cli -f novel.txt --font "NotoSerifSC-Regular.otf"
```

支持格式：
- TTF (TrueType)
- OTF (OpenType)
- WOFF/WOFF2 (Web Font)
- TTC (TrueType Collection)

---

## Markdown 支持

### 转换 Markdown 文件

```bash
kaf-cli -f readme.md -I markdown
```

或自动检测：
```bash
kaf-cli -f readme.md  # 根据 .md 扩展名自动识别
```

### 支持的 Markdown 语法

- `# 标题` - 章节标题
- `**粗体**` - 粗体文本
- `*斜体*` - 斜体文本
- `> 引用` - 引用块
- `- 列表` / `* 列表` - 无序列表
- `1. 列表` - 有序列表
- `` `代码` `` - 行内代码
- ` ```代码块``` ` - 代码块
- `[链接](url)` - 链接
- `![图片](path)` - 图片
- `---` / `***` - 水平分割线

### YAML Frontmatter

Markdown 文件可以在开头包含元数据：

```markdown
---
title: 我的文档
author: 作者名
lang: zh
---

# 第一章

内容...
```

---

## 常见问题

### Q: 章节识别不准确怎么办？

**A:** 可以尝试以下方法：

1. 调整章节匹配正则：
```bash
kaf-cli -f novel.txt -m "^第[0-9]+章"
```

2. 使用更宽松的匹配：
```bash
kaf-cli -f novel.txt --theme webnovel  # 使用网络小说预设
```

3. 手动检查章节格式：
```bash
kaf-cli -f novel.txt --dry-run --show-chapters
```

### Q: 转换后中文乱码怎么办？

**A:** 程序会自动检测文件编码，但如果检测失败：

1. 确保文件是 UTF-8 或 GBK/GB2312/GB18030 编码
2. 使用文本编辑器将文件另存为 UTF-8 编码

### Q: 如何调整 EPUB 的排版？

**A:** 使用配置文件调整排版参数：

```yaml
indent: 2                    # 段落缩进
align: "center"              # 标题对齐
line_height: "1.8"          # 行高
paragraph_spacing: "0.5em"  # 段落间距
```

### Q: 批量转换时某些文件失败怎么办？

**A:** 使用 `--continue-on-error` 参数：

```bash
kaf-cli --batch ./novels --continue-on-error --report json
```

然后查看生成的报告了解失败原因。

### Q: EPUB 在某些阅读器上显示不正常？

**A:** 可以尝试：

1. 使用更简单的主题（如 `light` 或 `modern`）
2. 不使用自定义 CSS
3. 使用 EPUBCheck 验证生成的文件

### Q: 如何提取文件名中的书名和作者？

**A:** 程序会自动识别知轩藏书格式的文件名：
- `《书名》（校对版全本）作者：作者名.txt`
- `@《书名》作者：作者名.txt`

其他格式会提取文件名作为书名。

---

## 高级技巧

### 组合多个选项

```bash
kaf-cli \
  --batch ./novels \
  --output-dir ./output \
  --theme sepia \
  --continue-on-error \
  --report markdown \
  --config kaf.yaml
```

### 创建快捷脚本

Windows (batch):
```batch
@echo off
kaf-cli.exe --config "C:\kaf\default.yaml" %*
```

Linux/macOS (shell):
```bash
#!/bin/bash
kaf-cli --config ~/.config/kaf/default.yaml "$@"
```

---

## 参考

- 更多配置示例见 `configs/` 目录
- 开发计划见 `plan/` 目录
- 问题反馈请提交 GitHub Issue
