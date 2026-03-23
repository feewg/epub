//! 编码检测模块
//!
//! 使用 encoding_rs 进行字符编码检测和转换

use crate::error::Result;
use encoding_rs::{Encoding, GBK, BIG5, UTF_8};

/// 检测并转换文件编码为 UTF-8
pub fn detect_and_convert(content: &[u8]) -> Result<String> {
    // 1. 检测 BOM 并去除
    if content.starts_with(&[0xEF, 0xBB, 0xBF]) {
        let (text, _) = UTF_8.decode_with_bom_removal(content);
        return Ok(text.to_string());
    }

    // 2. 尝试 UTF-8
    if let Ok(text) = std::str::from_utf8(content) {
        // 检查是否有无效字符
        if !text.contains('\u{FFFD}') {
            return Ok(text.to_string());
        }
    }

    // 3. 检测 GBK/GB18030
    let mut gbk_decoder = GBK.new_decoder();
    let mut gbk_text = String::new();
    let (gbk_result, _, gbk_used) = gbk_decoder.decode_to_str_without_replacement(content, &mut gbk_text, false);
    if gbk_result == encoding_rs::DecoderResult::InputEmpty && gbk_used == content.len() && !gbk_text.contains('\u{FFFD}') {
        return Ok(gbk_text);
    }

    // 4. 检测 Big5
    let mut big5_decoder = BIG5.new_decoder();
    let mut big5_text = String::new();
    let (big5_result, _, big5_used) = big5_decoder.decode_to_str_without_replacement(content, &mut big5_text, false);
    if big5_result == encoding_rs::DecoderResult::InputEmpty && big5_used == content.len() && !big5_text.contains('\u{FFFD}') {
        return Ok(big5_text);
    }

    // 5. 尝试所有编码
    for encoding in [GBK, BIG5, UTF_8] {
        let mut decoder = encoding.new_decoder();
        let mut text = String::new();
        let (result, _, used) = decoder.decode_to_str_without_replacement(content, &mut text, false);
        if result == encoding_rs::DecoderResult::InputEmpty && used == content.len() && !text.contains('\u{FFFD}') {
            return Ok(text);
        }
    }

    // 6. 最后的回退：UTF-8 with replacement
    let (text, _) = UTF_8.decode_with_bom_removal(content);
    Ok(text.to_string())
}

/// 确保字符串无 BOM
/// 
/// 移除 UTF-8 BOM (EF BB BF) 和其他可能的 BOM
pub fn ensure_no_bom(text: &str) -> String {
    // UTF-8 BOM 是三个字节: EF BB BF
    // 在 Rust 字符串中，BOM 表现为 \u{FEFF}
    text.trim_start_matches('\u{FEFF}').to_string()
}

/// 清理文本，确保输出是纯 UTF-8 无 BOM
pub fn clean_utf8_output(text: &str) -> String {
    let no_bom = ensure_no_bom(text);
    // 确保没有其他的控制字符（保留正常换行和制表）
    no_bom.chars()
        .filter(|&c| c == '\n' || c == '\r' || c == '\t' || (!c.is_control() || c.is_whitespace()))
        .collect()
}

/// 检测文件编码（不转换）
pub fn detect_encoding(content: &[u8]) -> &'static Encoding {
    // 检测 BOM
    if content.starts_with(&[0xEF, 0xBB, 0xBF]) {
        return UTF_8;
    }

    // 尝试 UTF-8
    if let Ok(text) = std::str::from_utf8(content) {
        if !text.contains('\u{FFFD}') {
            return UTF_8;
        }
    }

    // 检测 GBK/GB18030
    let mut gbk_decoder = GBK.new_decoder();
    let mut gbk_text = String::new();
    let (_, _, gbk_used) = gbk_decoder.decode_to_str_without_replacement(content, &mut gbk_text, false);
    if gbk_used == content.len() && !gbk_text.contains('\u{FFFD}') {
        return GBK;
    }

    // 检测 Big5
    let mut big5_decoder = BIG5.new_decoder();
    let mut big5_text = String::new();
    let (_, _, big5_used) = big5_decoder.decode_to_str_without_replacement(content, &mut big5_text, false);
    if big5_used == content.len() && !big5_text.contains('\u{FFFD}') {
        return BIG5;
    }

    // 默认返回 UTF-8
    UTF_8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_utf8() {
        let content = "你好，世界！".as_bytes();
        let result = detect_and_convert(content).unwrap();
        assert_eq!(result, "你好，世界！");
    }

    #[test]
    fn test_detect_utf8_with_bom() {
        let mut content = vec![0xEF, 0xBB, 0xBF];
        content.extend_from_slice("你好，世界！".as_bytes());
        let result = detect_and_convert(&content).unwrap();
        assert_eq!(result, "你好，世界！");
        // 确保结果没有 BOM
        assert!(!result.starts_with('\u{FEFF}'));
    }

    #[test]
    fn test_ensure_no_bom() {
        let with_bom = "\u{FEFF}你好，世界！";
        let result = ensure_no_bom(with_bom);
        assert_eq!(result, "你好，世界！");
        assert!(!result.starts_with('\u{FEFF}'));
    }

    #[test]
    fn test_clean_utf8_output() {
        let with_control = "\u{FEFF}Hello\x00World\x01!";
        let result = clean_utf8_output(with_control);
        assert!(result.contains("Hello"));
        assert!(result.contains("World"));
        assert!(!result.starts_with('\u{FEFF}'));
    }

    #[test]
    fn test_detect_encoding() {
        let content = "Hello, World!".as_bytes();
        let encoding = detect_encoding(content);
        assert_eq!(encoding.name(), "UTF-8");
    }
}
