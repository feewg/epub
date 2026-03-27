//! 格式检测器
//!
//! 根据文件扩展名或内容特征自动检测输入文件格式（TXT / Markdown）。

use crate::model::InputFormat;
use once_cell::sync::Lazy;
use regex::Regex;
use std::path::Path;
use tracing::debug;

/// 预编译正则：有序列表项
static RE_ORDERED_LIST: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^\d+\.\s").unwrap()
});

/// 预编译正则：中文小说章节标题
static RE_CHINESE_CHAPTER: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^第[一二三四五六七八九十零〇百千\d]+[章回节卷]").unwrap()
});

/// 格式检测器
pub struct FormatDetector;

impl FormatDetector {
    /// 根据文件扩展名检测格式
    ///
    /// 支持的扩展名：
    /// - `.txt`, `.text` → TXT
    /// - `.md`, `.markdown`, `.mkd` → Markdown
    /// - 其他 → Auto（需要进一步检测）
    pub fn detect_by_extension(path: &Path) -> InputFormat {
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_lowercase());

        match ext.as_deref() {
            Some("txt") | Some("text") => InputFormat::Txt,
            Some("md") | Some("markdown") | Some("mkd") => InputFormat::Markdown,
            _ => InputFormat::Auto,
        }
    }

    /// 根据内容特征检测格式
    ///
    /// 通过分析文件前 50 行内容，统计 Markdown 和 TXT 的特征得分，
    /// 得分较高的一方作为检测结果。
    pub fn detect_by_content(content: &str) -> InputFormat {
        let lines: Vec<&str> = content.lines().take(50).collect();

        let mut markdown_score: i32 = 0;
        let mut txt_score: i32 = 0;

        for line in &lines {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            // Markdown 指标
            if trimmed.starts_with("# ") || trimmed.starts_with("## ") || trimmed.starts_with("### ") {
                markdown_score += 3;
            }
            if trimmed.starts_with("> ") {
                markdown_score += 2;
            }
            if trimmed.starts_with("- ") || trimmed.starts_with("* ") || trimmed.starts_with("+ ") {
                markdown_score += 1;
            }
            if RE_ORDERED_LIST.is_match(trimmed) {
                markdown_score += 1;
            }
            if trimmed.contains("**") || trimmed.contains("__") {
                markdown_score += 1;
            }
            if trimmed.contains('[') && trimmed.contains("](") {
                markdown_score += 2;
            }
            if trimmed == "---" || trimmed == "***" || trimmed == "___" {
                markdown_score += 1;
            }
            if trimmed.starts_with("```") {
                markdown_score += 3;
            }

            // TXT 指标（中文小说模式）
            if RE_CHINESE_CHAPTER.is_match(trimmed) {
                txt_score += 5;
            }
        }

        debug!("格式检测得分 - Markdown: {}, TXT: {}", markdown_score, txt_score);

        if markdown_score > txt_score && markdown_score >= 3 {
            InputFormat::Markdown
        } else {
            InputFormat::Txt
        }
    }

    /// 综合检测格式
    ///
    /// 优先使用文件扩展名判断，若扩展名无法确定则通过内容分析。
    pub fn detect(path: &Path, content: &str) -> InputFormat {
        let ext_format = Self::detect_by_extension(path);
        if ext_format != InputFormat::Auto {
            debug!("通过扩展名检测格式: {:?}", ext_format);
            return ext_format;
        }
        let content_format = Self::detect_by_content(content);
        debug!("通过内容检测格式: {:?}", content_format);
        content_format
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_detect_by_extension_txt() {
        let path = PathBuf::from("test.txt");
        assert_eq!(FormatDetector::detect_by_extension(&path), InputFormat::Txt);
    }

    #[test]
    fn test_detect_by_extension_text() {
        let path = PathBuf::from("test.text");
        assert_eq!(FormatDetector::detect_by_extension(&path), InputFormat::Txt);
    }

    #[test]
    fn test_detect_by_extension_md() {
        let path = PathBuf::from("test.md");
        assert_eq!(FormatDetector::detect_by_extension(&path), InputFormat::Markdown);
    }

    #[test]
    fn test_detect_by_extension_markdown() {
        let path = PathBuf::from("test.markdown");
        assert_eq!(FormatDetector::detect_by_extension(&path), InputFormat::Markdown);
    }

    #[test]
    fn test_detect_by_extension_mkd() {
        let path = PathBuf::from("test.mkd");
        assert_eq!(FormatDetector::detect_by_extension(&path), InputFormat::Markdown);
    }

    #[test]
    fn test_detect_by_extension_unknown() {
        let path = PathBuf::from("test.unknown");
        assert_eq!(FormatDetector::detect_by_extension(&path), InputFormat::Auto);
    }

    #[test]
    fn test_detect_by_extension_no_ext() {
        let path = PathBuf::from("test");
        assert_eq!(FormatDetector::detect_by_extension(&path), InputFormat::Auto);
    }

    #[test]
    fn test_detect_by_content_markdown_headers() {
        let content = r#"# 标题

## 子标题

内容段落
"#;
        assert_eq!(FormatDetector::detect_by_content(content), InputFormat::Markdown);
    }

    #[test]
    fn test_detect_by_content_markdown_links() {
        let content = r#"这是一个段落

[链接文字](https://example.com)

另一个段落
"#;
        // 链接得分为 2，但 markdown_score >= 3 才判定为 Markdown
        assert_eq!(FormatDetector::detect_by_content(content), InputFormat::Txt);
    }

    #[test]
    fn test_detect_by_content_markdown_mixed() {
        let content = r#"# 标题

内容段落

- 列表项1
- 列表项2
"#;
        // # 标题: +3, - 列表项: +2 = 5 >= 3 且 > txt_score
        assert_eq!(FormatDetector::detect_by_content(content), InputFormat::Markdown);
    }

    #[test]
    fn test_detect_by_content_txt_chinese() {
        let content = r#"第一章 开始

这是第一章的内容。

第二章 发展

这是第二章的内容。
"#;
        assert_eq!(FormatDetector::detect_by_content(content), InputFormat::Txt);
    }

    #[test]
    fn test_detect_by_content_empty() {
        let content = "";
        assert_eq!(FormatDetector::detect_by_content(content), InputFormat::Txt);
    }

    #[test]
    fn test_detect_by_content_code_block() {
        let content = r#"普通文本

```
代码内容
```

更多文本
"#;
        // 代码块 +3 >= 3
        assert_eq!(FormatDetector::detect_by_content(content), InputFormat::Markdown);
    }

    #[test]
    fn test_detect_by_content_blockquote() {
        let content = r#"普通文本

> 引用内容

更多文本
"#;
        // 引用 +2 < 3，不会判定为 Markdown
        assert_eq!(FormatDetector::detect_by_content(content), InputFormat::Txt);
    }

    #[test]
    fn test_detect_combined_with_extension() {
        let path = PathBuf::from("book.md");
        let content = "纯文本内容";
        // 有扩展名优先用扩展名
        assert_eq!(FormatDetector::detect(&path, content), InputFormat::Markdown);
    }

    #[test]
    fn test_detect_combined_without_extension() {
        let path = PathBuf::from("book");
        let content = r#"# 标题

- 列表项
"#;
        // 无扩展名，通过内容检测
        assert_eq!(FormatDetector::detect(&path, content), InputFormat::Markdown);
    }

    #[test]
    fn test_detect_by_content_markdown_hr() {
        let content = r#"内容

---

更多内容
"#;
        // hr +1，但 < 3
        assert_eq!(FormatDetector::detect_by_content(content), InputFormat::Txt);
    }

    #[test]
    fn test_detect_by_content_bold_text() {
        let content = r#"普通文本

这是包含粗体文本的段落

更多文本
"#;
        // 没有明显的 Markdown 特征，应为 Txt
        assert_eq!(FormatDetector::detect_by_content(content), InputFormat::Txt);
    }

    #[test]
    fn test_detect_by_content_markdown_asterisk_list() {
        let content = r#"# 标题

* 项目一
* 项目二
* 项目三
"#;
        // # +3, * list +3 = 6
        assert_eq!(FormatDetector::detect_by_content(content), InputFormat::Markdown);
    }

    #[test]
    fn test_detect_extension_case_insensitive() {
        assert_eq!(FormatDetector::detect_by_extension(&PathBuf::from("TEST.MD")), InputFormat::Markdown);
        assert_eq!(FormatDetector::detect_by_extension(&PathBuf::from("test.TXT")), InputFormat::Txt);
    }
}
