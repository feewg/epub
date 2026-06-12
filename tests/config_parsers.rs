use kaf_cli::config::parsers;
use kaf_cli::model::{InputFormat, Language, OutputFormat, TextAlignment, ThemePreset};

#[test]
fn shared_parser_align() {
    assert_eq!(parsers::parse_align("left").unwrap(), TextAlignment::Left);
    assert_eq!(parsers::parse_align("CENTER").unwrap(), TextAlignment::Center);
    assert_eq!(parsers::parse_align("Right").unwrap(), TextAlignment::Right);
    assert!(parsers::parse_align("unknown").is_err());
}

#[test]
fn shared_parser_lang() {
    assert_eq!(parsers::parse_lang("zh").unwrap(), Language::Zh);
    assert_eq!(parsers::parse_lang("en").unwrap(), Language::En);
    assert_eq!(parsers::parse_lang("Ja").unwrap(), Language::Ja);
    assert!(parsers::parse_lang("xx").is_err());
}

#[test]
fn shared_parser_format() {
    assert_eq!(parsers::parse_format("epub").unwrap(), OutputFormat::Epub);
    assert_eq!(parsers::parse_format("ALL").unwrap(), OutputFormat::All);
    assert!(parsers::parse_format("pdf").is_err());
}

#[test]
fn shared_parser_input_format() {
    assert_eq!(parsers::parse_input_format("auto").unwrap(), InputFormat::Auto);
    assert_eq!(parsers::parse_input_format("txt").unwrap(), InputFormat::Txt);
    assert_eq!(parsers::parse_input_format("text").unwrap(), InputFormat::Txt);
    assert_eq!(
        parsers::parse_input_format("md").unwrap(),
        InputFormat::Markdown
    );
    assert!(parsers::parse_input_format("docx").is_err());
}

#[test]
fn shared_parser_theme() {
    assert_eq!(parsers::parse_theme("light").unwrap(), ThemePreset::Light);
    assert_eq!(parsers::parse_theme("Dark").unwrap(), ThemePreset::Dark);
    assert_eq!(
        parsers::parse_theme("high-contrast").unwrap(),
        ThemePreset::HighContrast
    );
    assert_eq!(
        parsers::parse_theme("high_contrast").unwrap(),
        ThemePreset::HighContrast
    );
    assert!(parsers::parse_theme("neon").is_err());
}
