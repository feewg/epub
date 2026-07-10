//! Shared enum parsers used by configuration loading and CLI handling.
//!
//! Centralising the parsing logic avoids duplicating the same `match` blocks
//! across the codebase and makes it easy to add new variants.

use crate::error::{KafError, Result};
use crate::model::{InputFormat, Language, OutputFormat, TextAlignment, ThemePreset};

/// Parse a text alignment value.
pub fn parse_align(s: &str) -> Result<TextAlignment> {
    Ok(match s.to_lowercase().as_str() {
        "left" => TextAlignment::Left,
        "center" => TextAlignment::Center,
        "right" => TextAlignment::Right,
        _ => return Err(KafError::ParseError(format!("无效的对齐方式: {}", s))),
    })
}

/// Parse a language value.
pub fn parse_lang(s: &str) -> Result<Language> {
    Ok(match s.to_lowercase().as_str() {
        "zh" => Language::Zh,
        "en" => Language::En,
        "de" => Language::De,
        "fr" => Language::Fr,
        "it" => Language::It,
        "es" => Language::Es,
        "ja" => Language::Ja,
        "pt" => Language::Pt,
        "ru" => Language::Ru,
        "nl" => Language::Nl,
        _ => return Err(KafError::ParseError(format!("无效的语言: {}", s))),
    })
}

/// Parse an output format value.
pub fn parse_format(s: &str) -> Result<OutputFormat> {
    Ok(match s.to_lowercase().as_str() {
        "epub" => OutputFormat::Epub,
        "all" => OutputFormat::All,
        _ => return Err(KafError::ParseError(format!("无效的输出格式: {}", s))),
    })
}

/// Parse an input format value.
pub fn parse_input_format(s: &str) -> Result<InputFormat> {
    Ok(match s.to_lowercase().as_str() {
        "auto" => InputFormat::Auto,
        "txt" | "text" => InputFormat::Txt,
        "markdown" | "md" => InputFormat::Markdown,
        _ => return Err(KafError::ParseError(format!("无效的输入格式: {}", s))),
    })
}

/// Parse a theme preset value.
pub fn parse_theme(s: &str) -> Result<ThemePreset> {
    Ok(match s.to_lowercase().as_str() {
        "light" => ThemePreset::Light,
        "dark" => ThemePreset::Dark,
        "sepia" => ThemePreset::Sepia,
        "high_contrast" | "high-contrast" | "highcontrast" => ThemePreset::HighContrast,
        "modern" => ThemePreset::Modern,
        "traditional" => ThemePreset::Traditional,
        _ => return Err(KafError::ParseError(format!("无效的主题: {}", s))),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_align() {
        assert_eq!(parse_align("left").unwrap(), TextAlignment::Left);
        assert_eq!(parse_align("CENTER").unwrap(), TextAlignment::Center);
        assert_eq!(parse_align("Right").unwrap(), TextAlignment::Right);
        assert!(parse_align("invalid").is_err());
    }

    #[test]
    fn test_parse_lang() {
        assert_eq!(parse_lang("zh").unwrap(), Language::Zh);
        assert_eq!(parse_lang("EN").unwrap(), Language::En);
        assert_eq!(parse_lang("Ja").unwrap(), Language::Ja);
        assert!(parse_lang("invalid").is_err());
    }

    #[test]
    fn test_parse_format() {
        assert_eq!(parse_format("epub").unwrap(), OutputFormat::Epub);
        assert_eq!(parse_format("ALL").unwrap(), OutputFormat::All);
        assert!(parse_format("invalid").is_err());
    }

    #[test]
    fn test_parse_input_format() {
        assert_eq!(parse_input_format("auto").unwrap(), InputFormat::Auto);
        assert_eq!(parse_input_format("txt").unwrap(), InputFormat::Txt);
        assert_eq!(parse_input_format("text").unwrap(), InputFormat::Txt);
        assert_eq!(parse_input_format("md").unwrap(), InputFormat::Markdown);
        assert!(parse_input_format("invalid").is_err());
    }

    #[test]
    fn test_parse_theme() {
        assert_eq!(parse_theme("light").unwrap(), ThemePreset::Light);
        assert_eq!(parse_theme("Dark").unwrap(), ThemePreset::Dark);
        assert_eq!(
            parse_theme("high-contrast").unwrap(),
            ThemePreset::HighContrast
        );
        assert_eq!(
            parse_theme("high_contrast").unwrap(),
            ThemePreset::HighContrast
        );
        assert!(parse_theme("invalid").is_err());
    }
}
