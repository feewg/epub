//! CSS 生成器
//!
//! 根据主题生成 EPUB 所需的 CSS 样式

use super::Theme;
use crate::model::{Book, TextAlignment};

/// CSS 生成器
pub struct CssGenerator;

impl CssGenerator {
    /// 创建新的 CSS 生成器
    pub fn new() -> Self {
        Self
    }

    /// 生成完整的 CSS 样式
    pub fn generate(&self, book: &Book, theme: &Theme) -> String {
        let mut css = String::new();

        // 添加 CSS 变量
        css.push_str(&theme.to_css_variables());
        css.push_str("\n\n");

        // 添加基础样式
        css.push_str(&self.generate_base_styles());

        // 添加章节样式
        css.push_str(&self.generate_chapter_styles(book, theme));

        // 添加段落样式
        css.push_str(&self.generate_paragraph_styles(book, theme));

        // 添加特殊元素样式
        css.push_str(&self.generate_special_styles(theme));

        css
    }

    /// 生成基础样式
    fn generate_base_styles(&self) -> String {
        r#"/* 基础样式 */
* {
    margin: 0;
    padding: 0;
    box-sizing: border-box;
}

body {
    font-family: var(--body-font);
    font-size: var(--base-size);
    line-height: var(--line-height);
    color: var(--text-color);
    background-color: var(--bg-color);
    padding: var(--page-padding);
    text-align: justify;
    word-wrap: break-word;
    overflow-wrap: break-word;
}

/* 链接样式 */
a {
    color: var(--accent-color);
    text-decoration: none;
}

a:hover {
    text-decoration: underline;
}

/* 图片样式 */
img {
    max-width: 100%;
    height: auto;
    display: block;
    margin: 1em auto;
}
"#.to_string()
    }

    /// 生成章节样式
    fn generate_chapter_styles(&self, book: &Book, _theme: &Theme) -> String {
        let text_align = match book.align {
            TextAlignment::Left => "left",
            TextAlignment::Center => "center",
            TextAlignment::Right => "right",
        };

        format!(
            r#"/* 章节样式 */
.chapter {{
    margin-bottom: var(--chapter-margin);
}}

h1, h2, h3, h4, h5, h6 {{
    font-family: var(--heading-font);
    font-weight: var(--heading-weight);
    color: var(--chapter-text);
    margin-top: 1.5em;
    margin-bottom: 1em;
    line-height: 1.3;
    text-align: {align};
    page-break-after: avoid;
}}

h1 {{
    font-size: 1.8em;
    background-color: var(--chapter-bg);
    padding: 0.8em;
    border-radius: 4px;
    margin-bottom: 1.5em;
}}

h2 {{
    font-size: 1.5em;
    border-bottom: 1px solid var(--border-color);
    padding-bottom: 0.3em;
}}

h3, h3.chapter-title {{
    font-size: 1.3em;
}}

h4 {{
    font-size: 1.1em;
}}

h5, h6 {{
    font-size: 1em;
}}
"#,
            align = text_align
        )
    }

    /// 生成段落样式
    fn generate_paragraph_styles(&self, _book: &Book, _theme: &Theme) -> String {
        r#"/* 段落样式 */
p {
    margin-bottom: var(--paragraph-spacing);
    text-indent: var(--paragraph-indent);
    orphans: 2;
    widows: 2;
}

p.no-indent {
    text-indent: 0;
}

/* 首字下沉（可选） */
.drop-cap::first-letter {
    float: left;
    font-size: 3em;
    line-height: 0.8;
    padding-right: 0.1em;
    font-weight: bold;
    color: var(--accent-color);
}
"#.to_string()
    }

    /// 生成特殊元素样式
    fn generate_special_styles(&self, _theme: &Theme) -> String {
        r#"/* 特殊元素样式 */

/* 引用块 */
blockquote {
    margin: 1.5em 2em;
    padding: 0.5em 1em;
    border-left: 3px solid var(--accent-color);
    background-color: var(--chapter-bg);
    color: var(--text-secondary);
    font-style: italic;
}

/* 代码块 */
pre, code {
    font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
    background-color: var(--chapter-bg);
    border-radius: 3px;
}

code {
    padding: 0.2em 0.4em;
    font-size: 0.9em;
}

pre {
    padding: 1em;
    overflow-x: auto;
    margin: 1em 0;
    line-height: 1.5;
}

pre code {
    padding: 0;
    background: none;
}

/* 表格 */
table {
    width: 100%;
    border-collapse: collapse;
    margin: 1em 0;
}

th, td {
    padding: 0.5em;
    border: 1px solid var(--border-color);
    text-align: left;
}

th {
    background-color: var(--chapter-bg);
    font-weight: bold;
}

/* 列表 */
ul, ol {
    margin: 1em 0;
    padding-left: 2em;
}

li {
    margin-bottom: 0.3em;
}

/* 强调 */
em, i {
    font-style: italic;
}

strong, b {
    font-weight: bold;
}

mark {
    background-color: var(--highlight-color);
    padding: 0.1em 0.2em;
}

/* 分隔线 */
hr {
    border: none;
    border-top: 1px solid var(--border-color);
    margin: 2em 0;
}

/* 注释/脚注 */
.footnote {
    font-size: 0.85em;
    color: var(--text-secondary);
    vertical-align: super;
}

/* 封面 */
.cover {
    text-align: center;
    padding: 3em 0;
}

.cover img {
    max-width: 80%;
    box-shadow: 0 4px 8px rgba(0,0,0,0.1);
}

/* 目录 */
.toc {
    margin: 2em 0;
}

.toc ul {
    list-style: none;
    padding: 0;
}

.toc li {
    margin: 0.5em 0;
    padding-left: 1em;
}

.toc a {
    color: var(--text-color);
    text-decoration: none;
}

.toc a:hover {
    color: var(--accent-color);
}

/* 分页控制 */
.page-break {
    page-break-before: always;
}

.no-break {
    page-break-inside: avoid;
}

/* 竖排支持（传统中文） */
.vertical {
    writing-mode: vertical-rl;
    text-orientation: mixed;
}

/*  night mode 适配（保留，用于阅读器覆盖） */
@media (prefers-color-scheme: dark) {
    /* 阅读器可能使用此媒体查询 */
}
"#.to_string()
    }

    /// 生成仅包含基础样式的 CSS（用于兼容性）
    #[allow(dead_code)]
    pub fn generate_minimal(&self, book: &Book) -> String {
        format!(
            r#"body {{
    font-family: system-ui, -apple-system, sans-serif;
    font-size: 18px;
    line-height: 1.8;
    color: #333;
    background: #fff;
    padding: 2rem;
    text-align: justify;
}}

h1, h2, h3 {{
    text-align: center;
    margin: 1.5em 0 1em;
}}

p {{
    text-indent: {}em;
    margin-bottom: 1em;
}}

img {{
    max-width: 100%;
}}"#,
            book.indent
        )
    }
}

impl Default for CssGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Book;
    use std::path::PathBuf;

    #[test]
    fn test_css_generation() {
        let generator = CssGenerator::new();
        let theme = Theme::light();
        let book = Book {
            filename: PathBuf::from("test.txt"),
            indent: 2,
            ..Default::default()
        };

        let css = generator.generate(&book, &theme);
        assert!(css.contains("--bg-color:"));
        assert!(css.contains("body {"));
        assert!(css.contains("h1 {"));
    }

    #[test]
    fn test_minimal_css() {
        let generator = CssGenerator::new();
        let book = Book {
            filename: PathBuf::from("test.txt"),
            indent: 2,
            ..Default::default()
        };

        let css = generator.generate_minimal(&book);
        assert!(css.contains("body {"));
        assert!(css.contains("text-indent: 2em"));
    }
}
