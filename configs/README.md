# kaf-cli 配置系统使用指南

## 概述

kaf-cli 提供了灵活的配置系统，支持多种配置方式和丰富的配置选项。

## 配置方式

### 1. CLI 参数（最高优先级）

```bash
kaf-cli -f input.txt -a "作者名" -b "书名"
```

### 2. 配置文件

在输入文件所在目录或当前目录创建 `kaf.yaml` 文件：

```yaml
# 书名
bookname: "示例小说"

# 作者
author: "YSTYLE"

# 章节匹配规则
chapter_match: "^第.{1,8}章"

# 输出格式
format: "epub"
```

然后直接运行：

```bash
kaf-cli -f input.txt
```

### 3. 混合配置

可以同时使用配置文件和 CLI 参数，CLI 参数会覆盖配置文件：

```bash
kaf-cli -f input.txt -a "CLI 作者"  # 覆盖配置文件中的作者
```

## 配置文件命名

支持以下文件名（按优先级）：

1. `kaf.yaml`
2. `kaf.yml`
3. `.kaf.yaml`
4. `.kaf.yml`

## 配置预设

kaf-cli 提供了 5 种配置预设，适合不同使用场景：

### 1. 基础配置 (Basic)

适合一般的小说转换。

```bash
kaf-cli --example-config > kaf.yaml
```

### 2. 网络小说配置 (WebNovel)

适合在线小说格式，标题通常较长，行间距较大。

### 3. 完整配置 (Full)

包含所有可配置项，适合高级用户。

### 4. 简约配置 (Minimal)

只包含最基本的设置，适合快速上手。

### 5. 出版物配置 (Publication)

适合正式出版书籍，格式更加规范。

## 主要配置选项

### 书籍信息

```yaml
# 书名
bookname: "小说名称"

# 作者
author: "作者名"

# 书籍语言 (zh, en, de, fr, it, es, ja, pt, ru, nl)
lang: "zh"

# 输出文件名（不含扩展名）
output_name: "custom_output"
```

### 章节识别

```yaml
# 章节匹配规则（正则表达式）
chapter_match: "^第.{1,8}章"

# 卷匹配规则（正则表达式）
volume_match: "^第[0-9一二三四五六七八九十零〇百千两 ]+[卷部]"

# 排除规则（正则表达式）
exclusion_pattern: "^第[0-9一二三四五六七八九十零〇百千两 ]+(部门|部队|部属)"

# 标题最大字数
max_title_length: 35

# 未知章节默认名称
unknown_title: "未知章节"
```

### 段落处理

```yaml
# 段落缩进字数
indent: 2

# 段落间距
paragraph_spacing: "0.5em"

# 行高
line_height: "1.8"
```

### 样式设置

```yaml
# 标题对齐方式 (left, center, right)
align: "center"

# 封面图片路径
cover: "cover.jpg"

# 自定义 CSS 文件路径
custom_css: "custom.css"

# 扩展 CSS（内联）
extended_css: |
  .content {
    font-size: 18px;
    line-height: 1.8;
  }

# 嵌入字体文件路径（支持 TTF/OTF 格式）
font: "fonts/custom.ttf"

# CSS 变量
css_variables:
  primary-color: "#333333"
  background-color: "#ffffff"
  text-color: "#333333"
  link-color: "#0000ff"
```

### 输出设置

```yaml
# 输出格式 (epub, all)
format: "all"
```

### 高级设置

```yaml
# 是否添加教程
add_tips: false

# 是否分离章节序号和标题
separate_chapter_number: false

# 章节页眉图片配置
chapter_header:
  image: "header.jpg"           # 页眉图片路径
  image_folder: "headers/"     # 页眉图片文件夹
  position: "center"           # 图片位置 (left, center, right)
  height: "100px"              # 图片高度
  width: "auto"                # 图片宽度
  mode: "single"               # 匹配模式 (single, folder)
```

## 配置验证

kaf-cli 会在转换前验证配置，检查以下内容：

- 文件存在性（输入文件、封面图片、自定义 CSS、字体文件）
- 数值范围（标题长度、缩进等）
- 格式正确性（CSS 值、语言、格式等）
- 逻辑一致性

如果配置有问题，会显示详细的错误信息。

## 使用示例

### 基本使用

```bash
# 生成基础配置
kaf-cli --example-config > kaf.yaml

# 编辑配置文件
vim kaf.yaml

# 使用配置文件转换
kaf-cli -f novel.txt
```

### 高级使用

```bash
# 使用网络小说预设配置
cp configs/kaf-webnovel.yaml kaf.yaml

# 自定义部分配置
vim kaf.yaml

# 转换网络小说
kaf-cli -f webnovel.txt
```

### CLI 覆盖配置

```bash
# 使用配置文件，但覆盖部分选项
kaf-cli -f novel.txt -a "新作者" --format epub
```

## 批量转换

批量转换时，配置文件会应用到所有文件：

```bash
# 创建配置文件
kaf-cli --example-config > kaf.yaml

# 批量转换文件夹中的所有 txt 文件
kaf-cli --batch ./novels/
```

## 故障排查

### 配置文件未找到

如果看到警告 "未找到配置文件"，请检查：

1. 配置文件名是否正确（kaf.yaml 等）
2. 配置文件是否在输入文件所在目录
3. 配置文件格式是否正确（YAML）

### 配置验证失败

如果配置验证失败，请检查：

1. 文件路径是否正确
2. 数值是否在允许范围内
3. 格式是否符合要求
4. YAML 语法是否正确

### 配置优先级

记住配置优先级：默认配置 → 配置文件 → CLI 参数

## 更多帮助

- 查看所有 CLI 参数：`kaf-cli --help`
- 生成配置示例：`kaf-cli --example-config`
- 查看版本信息：`kaf-cli --version`
