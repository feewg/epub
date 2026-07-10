//! HTML 处理模块
//!
//! 智能过滤和清理 HTML 标签

use once_cell::sync::Lazy;
use regex::Regex;

/// 允许的 HTML 标签列表
#[allow(dead_code)]
const ALLOWED_TAGS: &[&str] = &[
    // 单标签
    "img", "br", "hr", // 文本标签
    "p", "span", "div", // 格式标签
    "b", "i", "u", "s", "strong", "em", // 链接标签
    "a",  // 表格标签
    "table", "tr", "td", "th",
];

/// 预编译的允许标签正则表达式
#[allow(dead_code)]
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

/// 清理 HTML 标签，只保留允许的标签
#[allow(dead_code)]
pub fn sanitize_html_tags(input: &str) -> String {
    if !input.contains('<') {
        return input.to_string();
    }

    let mut result = String::new();
    let mut last_end = 0;

    for mat in ALLOWED_TAG_REGEX.find_iter(input) {
        // 转义匹配前的文本
        let before = &input[last_end..mat.start()];
        result.push_str(&escape_xml(before));

        result.push_str(mat.as_str());
        last_end = mat.end();
    }

    // 转义剩余文本
    let remaining = &input[last_end..];
    result.push_str(&escape_xml(remaining));

    result
}

/// 移除 XML 1.0 不允许的控制字符。
pub fn remove_invalid_xml_chars(input: &str) -> String {
    input
        .chars()
        .filter(|character| {
            matches!(*character, '\u{9}' | '\u{A}' | '\u{D}')
                || matches!(*character as u32, 0x20..=0xD7FF | 0xE000..=0xFFFD | 0x10000..=0x10FFFF)
        })
        .collect()
}

/// 转义 XML 特殊字符（用于 XML 内容和属性）。
pub fn escape_xml<T: AsRef<str>>(input: T) -> String {
    remove_invalid_xml_chars(input.as_ref().trim_start_matches('\u{FEFF}'))
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
    fn test_escape_xml() {
        let input = "<tag>value</tag>";
        let result = escape_xml(input);
        assert!(result.contains("&lt;tag&gt;"));
    }
}
