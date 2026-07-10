//! 文本编码检测与 UTF-8 转换

use crate::error::{KafError, Result};
use chardetng::{EncodingDetector, Iso2022JpDetection, Utf8Detection};
use encoding_rs::{Encoding, UTF_16BE, UTF_16LE, UTF_8};

/// 检测并转换文件编码为 UTF-8。
pub fn detect_and_convert(content: &[u8]) -> Result<String> {
    if content.starts_with(&[0xEF, 0xBB, 0xBF]) {
        return decode_without_replacement(UTF_8, &content[3..]);
    }
    if content.starts_with(&[0xFF, 0xFE]) {
        return decode_without_replacement(UTF_16LE, &content[2..]);
    }
    if content.starts_with(&[0xFE, 0xFF]) {
        return decode_without_replacement(UTF_16BE, &content[2..]);
    }
    if let Ok(text) = std::str::from_utf8(content) {
        return Ok(text.to_string());
    }

    let encoding = detect_legacy_encoding(content);
    decode_without_replacement(encoding, content)
}

fn decode_without_replacement(encoding: &'static Encoding, content: &[u8]) -> Result<String> {
    encoding
        .decode_without_bom_handling_and_without_replacement(content)
        .map(|text| text.into_owned())
        .ok_or_else(|| KafError::Encoding(format!("无效的 {} 字节序列", encoding.name())))
}

fn detect_legacy_encoding(content: &[u8]) -> &'static Encoding {
    let mut detector = EncodingDetector::new(Iso2022JpDetection::Allow);
    detector.feed(content, true);
    detector.guess(None, Utf8Detection::Allow)
}

/// 移除字符串开头的 Unicode BOM。
pub fn ensure_no_bom(text: &str) -> String {
    text.trim_start_matches('\u{FEFF}').to_string()
}

/// 清理文本中的非法 XML 控制字符。
pub fn clean_utf8_output(text: &str) -> String {
    crate::utils::html::remove_invalid_xml_chars(&ensure_no_bom(text))
}

/// 检测文件编码（不转换）。
pub fn detect_encoding(content: &[u8]) -> &'static Encoding {
    if content.starts_with(&[0xEF, 0xBB, 0xBF]) {
        UTF_8
    } else if content.starts_with(&[0xFF, 0xFE]) {
        UTF_16LE
    } else if content.starts_with(&[0xFE, 0xFF]) {
        UTF_16BE
    } else if std::str::from_utf8(content).is_ok() {
        UTF_8
    } else {
        detect_legacy_encoding(content)
    }
}
