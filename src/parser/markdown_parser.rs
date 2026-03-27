//! Markdown 解析器
//!
//! 轻量级 Markdown 解析器，将 Markdown 转换为 EPUB 章节结构。
//! 不依赖外部重型 Markdown 库，专注于小说/文档转换场景。

use crate::error::Result;
use crate::model::Section;
use once_cell::sync::Lazy;
use regex::Regex;
use tracing::{debug, warn};

/// Markdown 图片资源信息
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct MarkdownImage {
    /// 图片路径
    pub path: String,
    /// 图片替代文本
    pub alt: String,
    /// 所属章节索引
    pub chapter_index: usize,
}

/// Markdown 解析器
///
/// 将 Markdown 内容解析为 EPUB 章节结构。
/// 支持常见的 Markdown 语法，包括标题、段落、列表、代码块等。
pub struct MarkdownParser {
    /// 是否收集图片资源
    collect_images: bool,
    /// 收集到的图片资源路径列表
    images: Vec<MarkdownImage>,
}

/// 预编译正则：标题行（# ~ ######）
static RE_HEADER: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^(#{1,6})\s+(.+?)(?:\s+#+\s*)?$").unwrap()
});

/// 预编译正则：图片 `![alt](src)`（必须在链接之前匹配）
static RE_IMAGE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"!\[([^\]]*)\]\(([^)]+)\)").unwrap()
});

/// 预编译正则：链接 `[text](url)`（不含图片 `!` 前缀）
static RE_LINK: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\[([^\]]+)\]\(([^)]+)\)").unwrap()
});

/// 预编译正则：行内代码 `` `code` ``
static RE_INLINE_CODE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"`([^`]+)`").unwrap()
});

/// 预编译正则：粗体 `**text**`
static RE_BOLD_ASTERISK: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\*\*(.+?)\*\*").unwrap()
});

/// 预编译正则：粗体 `__text__`
static RE_BOLD_UNDERSCORE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"__(.+?)__").unwrap()
});

/// 预编译正则：斜体 `*text*`
static RE_ITALIC_ASTERISK: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\*(.+?)\*").unwrap()
});

/// 预编译正则：斜体 `_text_`
static RE_ITALIC_UNDERSCORE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"_(.+?)_").unwrap()
});

/// 预编译正则：水平分割线
static RE_HR: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^(?:---+|\*\*\*+|___+)\s*$").unwrap()
});

/// 预编译正则：代码块围栏
static RE_CODE_FENCE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^(`{3,}|~{3,})(\w*)\s*$").unwrap()
});

/// 预编译正则：无序列表项
static RE_UNORDERED_LIST: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^(\s*)([-*+])\s+(.*)$").unwrap()
});

/// 预编译正则：有序列表项
static RE_ORDERED_LIST: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^(\s*)(\d+)\.\s+(.*)$").unwrap()
});

/// 预编译正则：引用块
static RE_BLOCKQUOTE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^>\s?(.*)$").unwrap()
});

impl MarkdownParser {
    /// 创建新的 Markdown 解析器
    pub fn new() -> Self {
        Self {
            collect_images: true,
            images: Vec::new(),
        }
    }

    /// 设置是否收集图片资源
    #[allow(dead_code)]
    pub fn with_image_collection(mut self, collect: bool) -> Self {
        self.collect_images = collect;
        self
    }

    /// 解析 Markdown 内容为章节列表
    ///
    /// `#` 标题作为章节标题，`##` 标题作为子章节，
    /// 文本内容作为章节内容（HTML 格式）。
    pub fn parse(&mut self, content: &str) -> Result<Vec<Section>> {
        let mut sections = Vec::new();
        let mut current_section = Section::default();

        // 1. 提取并解析 YAML frontmatter
        let (content, _metadata) = self.extract_frontmatter(content);

        // 2. 按行解析
        let lines: Vec<&str> = content.lines().collect();
        let mut i = 0;

        while i < lines.len() {
            let line = lines[i].trim_end();
            let trimmed = line.trim();

            // 跳过空行
            if trimmed.is_empty() {
                i += 1;
                continue;
            }

            // 检查是否是代码块
            if let Some((lang, end_index)) = self.parse_code_block(&lines, i) {
                let code_html = self.format_code_block(&lines[i + 1..end_index], lang.as_deref());
                if current_section.content.is_empty() {
                    current_section.content = code_html;
                } else {
                    current_section.content.push_str(&code_html);
                }
                i = end_index + 1;
                continue;
            }

            // 检查是否是标题
            if let Some(caps) = RE_HEADER.captures(trimmed) {
                let level = caps[1].len();
                let title_text = caps[2].trim();

                // 保存当前章节
                if !current_section.title.is_empty() || !current_section.content.is_empty() {
                    sections.push(std::mem::take(&mut current_section));
                }

                // 根据标题级别创建新章节
                match level {
                    1 => {
                        // 一级标题：新章节
                        current_section.title = Self::escape_html(title_text);
                        current_section.content = String::new();
                    }
                    2 => {
                        // 二级标题：新章节
                        current_section.title = Self::escape_html(title_text);
                        current_section.content = String::new();
                    }
                    _ => {
                        // 三级及以下标题：作为子标题处理
                        current_section.title = Self::escape_html(title_text);
                        current_section.content = String::new();
                    }
                }
                i += 1;
                continue;
            }

            // 检查是否是水平分割线
            if RE_HR.is_match(trimmed) {
                let hr_html = "<hr/>\n";
                current_section.content.push_str(hr_html);
                i += 1;
                continue;
            }

            // 检查是否是列表块
            if RE_UNORDERED_LIST.is_match(line) || RE_ORDERED_LIST.is_match(line) {
                let (list_html, consumed) = self.parse_list_block(&lines, i);
                current_section.content.push_str(&list_html);
                i += consumed;
                continue;
            }

            // 检查是否是引用块
            if RE_BLOCKQUOTE.is_match(trimmed) {
                let (quote_html, consumed) = self.parse_blockquote(&lines, i);
                current_section.content.push_str(&quote_html);
                i += consumed;
                continue;
            }

            // 普通文本：收集连续的非空行为一个段落
            let mut paragraph_lines = Vec::new();
            while i < lines.len() {
                let next_trimmed = lines[i].trim();
                let next_line = lines[i].trim_end();

                if next_trimmed.is_empty() {
                    i += 1;
                    break;
                }

                // 如果遇到结构元素，停止收集
                if RE_HEADER.is_match(next_trimmed)
                    || RE_HR.is_match(next_trimmed)
                    || RE_CODE_FENCE.is_match(next_trimmed)
                    || RE_BLOCKQUOTE.is_match(next_trimmed)
                    || RE_UNORDERED_LIST.is_match(next_line)
                    || RE_ORDERED_LIST.is_match(next_line)
                {
                    break;
                }

                paragraph_lines.push(next_trimmed);
                i += 1;
            }

            if !paragraph_lines.is_empty() {
                let paragraph_text = paragraph_lines.join("\n");

                // 在转换为 HTML 之前收集图片资源
                if self.collect_images {
                    self.collect_images_from_markdown(&paragraph_text, sections.len());
                }

                let inline_html = self.parse_inline(&paragraph_text);
                let para_html = format!("<p>{}</p>\n", inline_html);
                current_section.content.push_str(&para_html);
            }
        }

        // 保存最后一个章节
        if !current_section.title.is_empty() || !current_section.content.is_empty() {
            sections.push(current_section);
        }

        debug!("Markdown 解析完成，共 {} 个章节", sections.len());
        Ok(sections)
    }

    /// 提取 YAML frontmatter
    fn extract_frontmatter<'a>(&self, content: &'a str) -> (&'a str, Frontmatter) {
        let mut lines = content.lines();

        // 检查文件是否以 `---` 开头
        let first_line = match lines.next() {
            Some(line) if line.trim() == "---" => line,
            _ => return (content, Frontmatter::default()),
        };

        // 收集 frontmatter 内容直到下一个 `---`
        let mut yaml_lines = Vec::new();
        let mut end_pos = first_line.len() + 1; // +1 for \n

        for line in lines {
            end_pos += line.len() + 1; // +1 for \n
            if line.trim() == "---" {
                break;
            }
            yaml_lines.push(line);
        }

        // 如果没有找到结束的 `---`，不是有效的 frontmatter
        if yaml_lines.is_empty() && content.lines().nth(1).map(|l| l.trim()) == Some("---") {
            // 只有两个 `---` 行，没有内容
            let yaml_content = "";
            let metadata = self.parse_yaml_frontmatter(yaml_content);
            debug!("提取到空的 YAML frontmatter");
            return (&content[end_pos..], metadata);
        }

        let yaml_content = yaml_lines.join("\n");
        let metadata = self.parse_yaml_frontmatter(&yaml_content);
        debug!("提取到 YAML frontmatter: {:?}", metadata);
        (&content[end_pos..], metadata)
    }

    /// 解析简单的 YAML frontmatter（不依赖 serde_yaml，保持轻量）
    fn parse_yaml_frontmatter(&self, yaml: &str) -> Frontmatter {
        let mut fm = Frontmatter::default();
        for line in yaml.lines() {
            let trimmed = line.trim();
            if let Some((key, value)) = trimmed.split_once(':') {
                let key = key.trim();
                let value = value.trim().trim_matches('"').trim_matches('\'');
                match key {
                    "title" => fm.title = Some(value.to_string()),
                    "author" => fm.author = Some(value.to_string()),
                    "lang" => fm.lang = Some(value.to_string()),
                    _ => {}
                }
            }
        }
        fm
    }

    /// 解析代码块，返回 (语言, 结束索引)
    fn parse_code_block(&self, lines: &[&str], start: usize) -> Option<(Option<String>, usize)> {
        let caps = RE_CODE_FENCE.captures(lines[start].trim())?;
        let lang = if caps[2].is_empty() { None } else { Some(caps[2].to_string()) };

        for line in &lines[(start + 1)..] {
            if line.trim().starts_with("```") || line.trim().starts_with("~~~") {
                let end_index = start + 1 + lines[(start + 1)..].iter().position(|l| l == line).unwrap();
                return Some((lang, end_index));
            }
        }

        // 未闭合的代码块，取到文件末尾
        warn!("代码块未闭合，从第 {} 行到文件末尾", start);
        Some((lang, lines.len() - 1))
    }

    /// 格式化代码块为 HTML
    fn format_code_block(&self, lines: &[&str], _lang: Option<&str>) -> String {
        let code_content: String = lines
            .iter()
            .map(|l| Self::escape_html(l))
            .collect::<Vec<_>>()
            .join("\n");

        format!("<pre><code>{}</code></pre>\n", code_content)
    }

    /// 解析列表块，返回 (HTML, 消耗的行数)
    fn parse_list_block(&self, lines: &[&str], start: usize) -> (String, usize) {
        let mut html = String::new();
        let mut i = start;
        let mut is_ordered = false;
        let mut items = Vec::new();

        // 确定列表类型
        if RE_ORDERED_LIST.is_match(lines[start].trim_end()) {
            is_ordered = true;
        }

        // 收集列表项
        while i < lines.len() {
            let line = lines[i].trim_end();
            let trimmed = line.trim();

            // 空行结束列表
            if trimmed.is_empty() {
                i += 1;
                break;
            }

            // 检查是否还是列表项
            let caps = if is_ordered {
                RE_ORDERED_LIST.captures(line)
            } else {
                RE_UNORDERED_LIST.captures(line)
            };

            if let Some(caps) = caps {
                let item_text = caps.get(3).map(|m| m.as_str()).unwrap_or("").trim();
                items.push(self.parse_inline(item_text));
                i += 1;
            } else {
                break;
            }
        }

        // 生成 HTML
        if is_ordered {
            html.push_str("<ol>\n");
        } else {
            html.push_str("<ul>\n");
        }

        for item in &items {
            html.push_str(&format!("  <li>{}</li>\n", item));
        }

        if is_ordered {
            html.push_str("</ol>\n");
        } else {
            html.push_str("</ul>\n");
        }

        (html, i - start)
    }

    /// 解析引用块，返回 (HTML, 消耗的行数)
    fn parse_blockquote(&self, lines: &[&str], start: usize) -> (String, usize) {
        let mut content_lines = Vec::new();
        let mut i = start;

        while i < lines.len() {
            let trimmed = lines[i].trim();

            if trimmed.is_empty() {
                i += 1;
                break;
            }

            if let Some(caps) = RE_BLOCKQUOTE.captures(trimmed) {
                content_lines.push(caps[1].trim().to_string());
                i += 1;
            } else {
                break;
            }
        }

        // 引用块内容也需要处理内联 Markdown
        let inner_html = content_lines
            .iter()
            .map(|l| self.parse_inline(l))
            .collect::<Vec<_>>()
            .join("</p>\n<p>");

        let html = format!("<blockquote>\n  <p>{}</p>\n</blockquote>\n", inner_html);
        (html, i - start)
    }

    /// 解析内联 Markdown 为 HTML
    ///
    /// 处理粗体、斜体、链接、图片、行内代码等内联元素。
    fn parse_inline(&self, text: &str) -> String {
        let mut result = text.to_string();

        // 处理行内代码（最先处理，避免内部被其他规则干扰）
        result = self.process_inline_code(&result);

        // 处理图片
        result = self.process_images(&result);

        // 处理链接
        result = self.process_links(&result);

        // 处理粗体
        result = self.process_bold(&result);

        // 处理斜体
        result = self.process_italic(&result);

        // 处理换行符
        result = result.replace("\n", "<br/>\n");

        result
    }

    /// 处理行内代码
    fn process_inline_code(&self, text: &str) -> String {
        RE_INLINE_CODE
            .replace_all(text, "<code>$1</code>")
            .to_string()
    }

    /// 处理图片标记
    fn process_images(&self, text: &str) -> String {
        RE_IMAGE
            .replace_all(text, "<img src=\"$2\" alt=\"$1\"/>")
            .to_string()
    }

    /// 处理链接标记（跳过图片标记 `![`）
    fn process_links(&self, text: &str) -> String {
        RE_LINK.replace_all(text, |caps: &regex::Captures| {
            let full_match = caps.get(0).unwrap().as_str();
            // 跳过图片标记（以 `![` 开头）
            if full_match.starts_with("![") {
                return full_match.to_string();
            }
            format!("<a href=\"{}\">{}</a>", &caps[2], &caps[1])
        }).to_string()
    }

    /// 处理粗体标记（`**text**` 和 `__text__`）
    fn process_bold(&self, text: &str) -> String {
        let result = RE_BOLD_ASTERISK.replace_all(text, "<strong>$1</strong>");
        let result = RE_BOLD_UNDERSCORE.replace_all(&result, "<strong>$1</strong>");
        result.to_string()
    }

    /// 处理斜体标记（`*text*` 和 `_text_`）
    ///
    /// 注意：粗体已先处理，因此 `**` 不会残留。
    /// 简单匹配 `*text*` 和 `_text_` 即可。
    fn process_italic(&self, text: &str) -> String {
        let result = RE_ITALIC_ASTERISK.replace_all(text, "<em>$1</em>");
        let result = RE_ITALIC_UNDERSCORE.replace_all(&result, "<em>$1</em>");
        result.to_string()
    }

    /// 从文本中收集图片资源（在 Markdown 转 HTML 之前调用）
    fn collect_images_from_markdown(&mut self, text: &str, chapter_index: usize) {
        for caps in RE_IMAGE.captures_iter(text) {
            let path = caps[2].to_string();
            let alt = caps[1].to_string();
            self.images.push(MarkdownImage {
                path,
                alt,
                chapter_index,
            });
        }
    }

    /// HTML 转义特殊字符
    pub fn escape_html(text: &str) -> String {
        let mut result = String::with_capacity(text.len());
        for ch in text.chars() {
            match ch {
                '&' => result.push_str("&amp;"),
                '<' => result.push_str("&lt;"),
                '>' => result.push_str("&gt;"),
                '"' => result.push_str("&quot;"),
                '\'' => result.push_str("&#39;"),
                _ => result.push(ch),
            }
        }
        result
    }

    /// 获取收集到的图片资源列表
    #[allow(dead_code)]
    pub fn images(&self) -> &[MarkdownImage] {
        &self.images
    }

    /// 清空收集到的图片资源
    #[allow(dead_code)]
    pub fn clear_images(&mut self) {
        self.images.clear();
    }
}

impl Default for MarkdownParser {
    fn default() -> Self {
        Self::new()
    }
}

/// YAML frontmatter 元数据
#[derive(Debug, Clone, Default)]
struct Frontmatter {
    title: Option<String>,
    author: Option<String>,
    lang: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_html() {
        assert_eq!(MarkdownParser::escape_html("<p>hello</p>"), "&lt;p&gt;hello&lt;/p&gt;");
        assert_eq!(MarkdownParser::escape_html("a & b"), "a &amp; b");
        assert_eq!(MarkdownParser::escape_html("say \"hi\""), "say &quot;hi&quot;");
    }

    #[test]
    fn test_parse_inline_bold() {
        let parser = MarkdownParser::new();
        let result = parser.parse_inline("这是**粗体**文本");
        assert!(result.contains("<strong>粗体</strong>"));
    }

    #[test]
    fn test_parse_inline_italic() {
        let parser = MarkdownParser::new();
        let result = parser.parse_inline("这是*斜体*文本");
        assert!(result.contains("<em>斜体</em>"));
    }

    #[test]
    fn test_parse_inline_link() {
        let parser = MarkdownParser::new();
        let result = parser.parse_inline("查看[链接](https://example.com)这里");
        assert!(result.contains("<a href=\"https://example.com\">链接</a>"));
    }

    #[test]
    fn test_parse_inline_image() {
        let parser = MarkdownParser::new();
        let result = parser.parse_inline("图片![alt text](image.png)展示");
        assert!(result.contains("<img src=\"image.png\" alt=\"alt text\"/>"));
    }

    #[test]
    fn test_parse_inline_code() {
        let parser = MarkdownParser::new();
        let result = parser.parse_inline("使用`code`标记");
        assert!(result.contains("<code>code</code>"));
    }

    #[test]
    fn test_parse_inline_combined() {
        let parser = MarkdownParser::new();
        let result = parser.parse_inline("**粗体**和*斜体*和`代码`");
        assert!(result.contains("<strong>粗体</strong>"));
        assert!(result.contains("<em>斜体</em>"));
        assert!(result.contains("<code>代码</code>"));
    }

    #[test]
    fn test_parse_headers_as_chapters() {
        let mut parser = MarkdownParser::new();
        let content = r#"# 第一章 开始

这是第一章的内容。

## 第一节

这是第一节的内容。

# 第二章 结束

这是第二章的内容。
"#;
        let sections = parser.parse(content).unwrap();
        assert_eq!(sections.len(), 3);
        assert_eq!(sections[0].title, "第一章 开始");
        assert!(sections[0].content.contains("这是第一章的内容"));
        assert_eq!(sections[1].title, "第一节");
        assert_eq!(sections[2].title, "第二章 结束");
    }

    #[test]
    fn test_parse_paragraphs() {
        let mut parser = MarkdownParser::new();
        let content = r#"# 标题

第一段内容。

第二段内容。

第三段内容。
"#;
        let sections = parser.parse(content).unwrap();
        assert_eq!(sections.len(), 1);
        let content = &sections[0].content;
        assert!(content.contains("<p>第一段内容。</p>"));
        assert!(content.contains("<p>第二段内容。</p>"));
        assert!(content.contains("<p>第三段内容。</p>"));
    }

    #[test]
    fn test_parse_unordered_list() {
        let mut parser = MarkdownParser::new();
        let content = r#"# 标题

- 第一项
- 第二项
- 第三项
"#;
        let sections = parser.parse(content).unwrap();
        assert!(sections[0].content.contains("<ul>"));
        assert!(sections[0].content.contains("<li>第一项</li>"));
        assert!(sections[0].content.contains("<li>第二项</li>"));
        assert!(sections[0].content.contains("<li>第三项</li>"));
        assert!(sections[0].content.contains("</ul>"));
    }

    #[test]
    fn test_parse_ordered_list() {
        let mut parser = MarkdownParser::new();
        let content = r#"# 标题

1. 第一项
2. 第二项
3. 第三项
"#;
        let sections = parser.parse(content).unwrap();
        assert!(sections[0].content.contains("<ol>"));
        assert!(sections[0].content.contains("<li>第一项</li>"));
        assert!(sections[0].content.contains("</ol>"));
    }

    #[test]
    fn test_parse_blockquote() {
        let mut parser = MarkdownParser::new();
        let content = r#"# 标题

> 这是一段引用
> 多行引用内容
"#;
        let sections = parser.parse(content).unwrap();
        assert!(sections[0].content.contains("<blockquote>"));
        assert!(sections[0].content.contains("这是一段引用"));
        assert!(sections[0].content.contains("多行引用内容"));
    }

    #[test]
    fn test_parse_code_block() {
        let mut parser = MarkdownParser::new();
        let content = r#"# 标题

```
fn main() {
    println!("Hello");
}
```
"#;
        let sections = parser.parse(content).unwrap();
        assert!(sections[0].content.contains("<pre><code>"));
        assert!(sections[0].content.contains("fn main()"));
        assert!(sections[0].content.contains("</code></pre>"));
    }

    #[test]
    fn test_parse_horizontal_rule() {
        let mut parser = MarkdownParser::new();
        let content = r#"# 标题

上面内容

---

下面内容
"#;
        let sections = parser.parse(content).unwrap();
        assert!(sections[0].content.contains("<hr/>"));
    }

    #[test]
    fn test_parse_frontmatter() {
        let mut parser = MarkdownParser::new();
        let content = r#"---
title: 测试书籍
author: 测试作者
---

# 第一章

内容。
"#;
        let sections = parser.parse(content).unwrap();
        assert_eq!(sections.len(), 1);
        assert_eq!(sections[0].title, "第一章");
    }

    #[test]
    fn test_parse_with_inline_formatting_in_paragraph() {
        let mut parser = MarkdownParser::new();
        let content = r#"# 标题

这是**粗体**和*斜体*混合的段落，还有[链接](http://example.com)和`代码`。
"#;
        let sections = parser.parse(content).unwrap();
        let content = &sections[0].content;
        assert!(content.contains("<strong>粗体</strong>"));
        assert!(content.contains("<em>斜体</em>"));
        assert!(content.contains("<a href=\"http://example.com\">链接</a>"));
        assert!(content.contains("<code>代码</code>"));
    }

    #[test]
    fn test_image_collection() {
        let mut parser = MarkdownParser::new();
        let content = r#"# 第一章

这是图片：

![风景图](images/cover.jpg)

更多内容。

# 第二章

![另一张](images/chapter2.png)
"#;
        parser.parse(content).unwrap();
        let images = parser.images();
        assert_eq!(images.len(), 2);
        assert_eq!(images[0].path, "images/cover.jpg");
        assert_eq!(images[0].alt, "风景图");
        assert_eq!(images[0].chapter_index, 0);
        assert_eq!(images[1].path, "images/chapter2.png");
        assert_eq!(images[1].chapter_index, 1);
    }

    #[test]
    fn test_empty_content() {
        let mut parser = MarkdownParser::new();
        let sections = parser.parse("").unwrap();
        assert!(sections.is_empty());
    }

    #[test]
    fn test_no_headers_content() {
        let mut parser = MarkdownParser::new();
        let content = "这是一段没有标题的内容。\n\n这是第二段。";
        let sections = parser.parse(content).unwrap();
        assert_eq!(sections.len(), 1);
        assert!(sections[0].title.is_empty());
        assert!(sections[0].content.contains("这是一段没有标题的内容"));
    }

    #[test]
    fn test_bold_with_underscores() {
        let parser = MarkdownParser::new();
        let result = parser.parse_inline("这是__下划线粗体__文本");
        assert!(result.contains("<strong>下划线粗体</strong>"));
    }

    #[test]
    fn test_hr_variations() {
        let mut parser = MarkdownParser::new();
        let content = r#"# 标题

内容1

***

内容2

___

内容3
"#;
        let sections = parser.parse(content).unwrap();
        // 应该有 hr 标签
        let count = sections[0].content.matches("<hr/>").count();
        assert!(count >= 2, "期望至少 2 个 hr，实际: {}", count);
    }

    #[test]
    fn test_code_block_with_language() {
        let mut parser = MarkdownParser::new();
        let content = r#"# 标题

```rust
fn hello() {}
```
"#;
        let sections = parser.parse(content).unwrap();
        assert!(sections[0].content.contains("<pre><code>"));
        assert!(sections[0].content.contains("fn hello()"));
    }

    #[test]
    fn test_list_with_inline_formatting() {
        let mut parser = MarkdownParser::new();
        let content = r#"# 标题

- **粗体项**
- *斜体项*
- `代码项`
"#;
        let sections = parser.parse(content).unwrap();
        let content = &sections[0].content;
        assert!(content.contains("<strong>粗体项</strong>"));
        assert!(content.contains("<em>斜体项</em>"));
        assert!(content.contains("<code>代码项</code>"));
    }

    #[test]
    fn test_deeply_nested_inline() {
        let parser = MarkdownParser::new();
        // 注意：这个简化解析器不支持嵌套格式（如粗体中的斜体）
        let result = parser.parse_inline("**粗体中的*斜体***");
        // 至少应该有粗体
        assert!(result.contains("<strong>"));
    }

    #[test]
    fn test_escape_html_in_code_block() {
        let mut parser = MarkdownParser::new();
        let content = r#"# 标题

```
<div class="test">Hello & World</div>
```
"#;
        let sections = parser.parse(content).unwrap();
        assert!(sections[0].content.contains("&lt;div"));
        assert!(sections[0].content.contains("&amp;"));
    }
}
