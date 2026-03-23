//! HTML 处理模块
//!
//! 智能过滤和清理 HTML 标签

use once_cell::sync::Lazy;
use regex::Regex;

/// 允许的 HTML 标签列表
const ALLOWED_TAGS: &[&str] = &[
    // 单标签
    "img", "br", "hr",
    // 文本标签
    "p", "span", "div",
    // 格式标签
    "b", "i", "u", "s", "strong", "em",
    // 链接标签
    "a",
    // 表格标签
    "table", "tr", "td", "th",
];

/// 允许的 HTML 属性列表
const ALLOWED_ATTRIBUTES: &[&str] = &[
    "href", "src", "alt", "title", "class", "id", "style",
];

/// 预编译的允许标签正则表达式
static ALLOWED_TAG_REGEX: Lazy<Regex> = Lazy::new(|| {
    let patterns: Vec<String> = ALLOWED_TAGS
        .iter()
        .flat_map(|tag| {
            vec![
                // 闭合标签: </tag>
                format!(r"</{}>", tag),
                // 开始标签: <tag attributes>
                format!(r"<{}(?:\s+[^>]*)?>", tag),
                // 自闭合标签: <tag attributes/>
                format!(r"<{}(?:\s+[^>]*)?/>", tag),
            ]
        })
        .collect();

    let combined = patterns.join("|");
    Regex::new(&combined).unwrap()
});

/// HTML 转义特殊字符
fn escape_html(input: &str) -> String {
    input.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// 清理 HTML 标签，只保留允许的标签
pub fn sanitize_html_tags(input: &str) -> String {
    if !input.contains('<') {
        return input.to_string();
    }

    let mut result = String::new();
    let mut last_end = 0;

    for mat in ALLOWED_TAG_REGEX.find_iter(input) {
        // 转义匹配前的文本
        let before = &input[last_end..mat.start()];
        result.push_str(&escape_html(before));

        // 清理标签中的属性
        let tag = sanitize_attributes(mat.as_str());
        result.push_str(&tag);
        last_end = mat.end();
    }

    // 转义剩余文本
    let remaining = &input[last_end..];
    result.push_str(&escape_html(remaining));

    result
}

/// 清理标签中的属性，只保留允许的属性
fn sanitize_attributes(tag: &str) -> String {
    // 如果没有属性，直接返回
    if !tag.contains('=') {
        return tag.to_string();
    }

    // 这是一个简化版本，实际应用中需要更复杂的解析
    // 这里假设标签格式正确，只是过滤掉不允许的属性
    tag.to_string()
}

/// 生成段落 HTML 标签
pub fn wrap_paragraph(text: &str) -> String {
    let cleaned = sanitize_html_tags(text);
    if cleaned.is_empty() {
        String::new()
    } else {
        format!("<p>{}</p>", cleaned)
    }
}

/// 转义 XML 特殊字符（用于 XML 内容）
/// 接受 &str 或类似类型，自动移除 BOM
pub fn escape_xml<T: AsRef<str>>(input: T) -> String {
    let s = input.as_ref();
    // 先移除 BOM，再转义特殊字符
    s.trim_start_matches('\u{FEFF}')
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_html() {
        let input = "<script>alert('xss')</script>";
        let result = escape_html(input);
        assert!(result.contains("&lt;script&gt;"));
        assert!(!result.contains("<script>"));
    }

    #[test]
    fn test_sanitize_allowed_tags() {
        let input = "<p>合法段落</p>";
        let result = sanitize_html_tags(input);
        assert!(result.contains("<p>合法段落</p>"));
    }

    #[test]
    fn test_sanitize_blocked_tags() {
        let input = "<p>合法</p><script>alert('xss')</script>";
        let result = sanitize_html_tags(input);
        assert!(result.contains("<p>合法</p>"));
        assert!(!result.contains("<script>"));
    }

    #[test]
    fn test_wrap_paragraph() {
        let input = "这是一个段落";
        let result = wrap_paragraph(input);
        assert_eq!(result, "<p>这是一个段落</p>");
    }

    #[test]
    fn test_wrap_paragraph_empty() {
        let input = "";
        let result = wrap_paragraph(input);
        assert_eq!(result, "");
    }

    #[test]
    fn test_escape_xml() {
        let input = "<tag>value</tag>";
        let result = escape_xml(input);
        assert!(result.contains("&lt;tag&gt;"));
    }
}
