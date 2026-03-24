//! 配置管理模块
//!
//! 负责加载和合并配置文件

use crate::cli::Cli;
use crate::error::{KafError, Result};
use crate::model::{Book, Language, OutputFormat, TextAlignment};
use serde_yaml::Value;
use std::fs;
use std::path::Path;

/// 从 CLI 加载配置
pub fn load_config(cli: &Cli) -> Result<Book> {
    let mut book = Book::default();

    // 1. 加载配置文件（如果指定）
    if let Some(config_path) = &cli.config {
        let file_config = load_config_file(config_path)?;
        merge_config(&mut book, &file_config)?;
    } else {
        // 自动查找配置文件
        if let Some(auto_config) = find_config(&cli.filename) {
            let file_config = load_config_file(&auto_config)?;
            merge_config(&mut book, &file_config)?;
        }
    }

    // 2. 应用 CLI 参数（最高优先级）
    apply_cli_config(&mut book, cli)?;

    // 3. 如果没有指定文件名，使用默认值
    if book.filename.as_os_str().is_empty() {
        if let Some(ref filename) = cli.filename {
            book.filename = filename.clone();
        }
    }

    Ok(book)
}

/// 加载配置文件
fn load_config_file(path: &Path) -> Result<Value> {
    let content = fs::read_to_string(path)?;

    serde_yaml::from_str(&content)
        .map_err(|e| KafError::ConfigParseError(e))
}

/// 查找配置文件
fn find_config(filename: &Option<std::path::PathBuf>) -> Option<std::path::PathBuf> {
    let config_names = ["kaf.yaml", "kaf.yml", ".kaf.yaml", ".kaf.yml"];

    // 优先搜索文件所在目录，然后搜索当前目录
    let search_dirs = filename.as_ref()
        .and_then(|p| p.parent())
        .into_iter()
        .chain(Some(std::path::Path::new(".")));

    for dir in search_dirs {
        for name in &config_names {
            let path = dir.join(name);
            if path.exists() {
                return Some(path);
            }
        }
    }

    None
}

/// 合并配置
fn merge_config(book: &mut Book, config: &Value) -> Result<()> {
    // 字符串字段
    if let Some(s) = config.get("bookname").and_then(|v| v.as_str()) {
        book.bookname = Some(s.to_string());
    }
    if let Some(s) = config.get("author").and_then(|v| v.as_str()) {
        book.author = s.to_string();
    }
    if let Some(s) = config.get("chapter_match").and_then(|v| v.as_str()) {
        book.chapter_match = Some(s.to_string());
    }
    if let Some(s) = config.get("volume_match").and_then(|v| v.as_str()) {
        book.volume_match = Some(s.to_string());
    }
    if let Some(s) = config.get("exclusion_pattern").and_then(|v| v.as_str()) {
        book.exclusion_pattern = Some(s.to_string());
    }
    if let Some(s) = config.get("unknown_title").and_then(|v| v.as_str()) {
        book.unknown_title = s.to_string();
    }
    if let Some(s) = config.get("paragraph_spacing").and_then(|v| v.as_str()) {
        book.paragraph_spacing = s.to_string();
    }

    // 可选字符串字段
    if let Some(s) = config.get("line_height").and_then(|v| v.as_str()) {
        book.line_height = Some(s.to_string());
    }

    // 数字字段
    if let Some(n) = config.get("max_title_length").and_then(|v| v.as_u64()) {
        book.max_title_length = n as usize;
    }
    if let Some(n) = config.get("indent").and_then(|v| v.as_u64()) {
        book.indent = n as usize;
    }

    // 布尔字段
    if let Some(b) = config.get("add_tips").and_then(|v| v.as_bool()) {
        book.add_tips = b;
    }
    if let Some(b) = config.get("separate_chapter_number").and_then(|v| v.as_bool()) {
        book.separate_chapter_number = b;
    }

    // 枚举字段
    if let Some(s) = config.get("align").and_then(|v| v.as_str()) {
        book.align = parse_align(s)?;
    }
    if let Some(s) = config.get("lang").and_then(|v| v.as_str()) {
        book.lang = parse_lang(s)?;
    }
    if let Some(s) = config.get("format").and_then(|v| v.as_str()) {
        book.format = parse_format(s)?;
    }

    // 自定义 CSS
    if let Some(ref custom_css) = config.get("custom_css").and_then(|v| v.as_str()) {
        book.custom_css = Some(custom_css.into());
    }
    if let Some(ref extended_css) = config.get("extended_css").and_then(|v| v.as_str()) {
        book.extended_css = Some(extended_css.to_string());
    }

    Ok(())
}

/// 解析对齐方式
fn parse_align(s: &str) -> Result<TextAlignment> {
    Ok(match s.to_lowercase().as_str() {
        "left" => TextAlignment::Left,
        "center" => TextAlignment::Center,
        "right" => TextAlignment::Right,
        _ => return Err(KafError::ParseError(format!("Invalid align value: {}", s))),
    })
}

/// 解析语言
fn parse_lang(s: &str) -> Result<Language> {
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
        _ => return Err(KafError::ParseError(format!("Invalid lang value: {}", s))),
    })
}

/// 解析输出格式
fn parse_format(s: &str) -> Result<OutputFormat> {
    Ok(match s.to_lowercase().as_str() {
        "epub" => OutputFormat::Epub,
        "all" => OutputFormat::All,
        _ => return Err(KafError::ParseError(format!("Invalid format value: {}", s))),
    })
}

/// 应用 CLI 配置
fn apply_cli_config(book: &mut Book, cli: &Cli) -> Result<()> {
    if let Some(ref filename) = cli.filename {
        book.filename = filename.clone();
    }

    if let Some(ref bookname) = cli.bookname {
        book.bookname = Some(bookname.clone());
    }

    book.author = cli.author.clone();

    if let Some(ref chapter_match) = cli.chapter_match {
        book.chapter_match = Some(chapter_match.clone());
    }

    if let Some(ref volume_match) = cli.volume_match {
        book.volume_match = Some(volume_match.clone());
    }

    if let Some(ref exclude) = cli.exclude {
        book.exclusion_pattern = Some(exclude.clone());
    }

    book.max_title_length = cli.max_title_length;
    book.indent = cli.indent;
    book.align = parse_align(&cli.align)?;
    book.format = parse_format(&cli.format)?;
    book.lang = parse_lang(&cli.lang)?;
    book.separate_chapter_number = cli.separate_chapter_number;

    if let Some(ref cover) = cli.cover {
        book.cover = Some(crate::model::CoverSource::Local {
            path: std::path::PathBuf::from(cover),
        });
    }

    if let Some(ref custom_css) = cli.custom_css {
        book.custom_css = Some(custom_css.clone());
    }

    if let Some(ref extended_css) = cli.extended_css {
        book.extended_css = Some(extended_css.clone());
    }

    if let Some(ref font) = cli.font {
        book.font = Some(font.clone());
    }

    if let Some(ref line_height) = cli.line_height {
        book.line_height = Some(line_height.clone());
    }

    if let Some(ref paragraph_spacing) = cli.paragraph_spacing {
        book.paragraph_spacing = paragraph_spacing.clone();
    }

    Ok(())
}

/// 生成示例配置
pub fn generate_example_config() -> String {
    let mut config = String::new();
    config.push_str("# kaf-cli 配置文件示例\n\n");
    config.push_str("# 书名\n");
    config.push_str("bookname: \"示例小说\"\n\n");
    config.push_str("# 作者\n");
    config.push_str("author: \"YSTYLE\"\n\n");
    config.push_str("# 章节匹配规则（正则表达式）\n");
    config.push_str("chapter_match: \"^第.{1,8}章\"\n\n");
    config.push_str("# 卷匹配规则（正则表达式）\n");
    config.push_str("volume_match: \"^第[0-9一二三四五六七八九十零〇百千两 ]+[卷部]\"\n\n");
    config.push_str("# 排除规则（正则表达式）\n");
    config.push_str("exclusion_pattern: \"^第[0-9一二三四五六七八九十零〇百千两 ]+(部门|部队|部属)\"\n\n");
    config.push_str("# 标题最大字数\n");
    config.push_str("max_title_length: 35\n\n");
    config.push_str("# 段落缩进字数\n");
    config.push_str("indent: 2\n\n");
    config.push_str("# 标题对齐方式 (left, center, right)\n");
    config.push_str("align: \"center\"\n\n");
    config.push_str("# 未知章节默认名称\n");
    config.push_str("unknown_title: \"未知章节\"\n\n");
    config.push_str("# 段落间距\n");
    config.push_str("paragraph_spacing: \"0.5em\"\n\n");
    config.push_str("# 行高\n");
    config.push_str("# line_height: \"1.8\"\n\n");
    config.push_str("# 是否添加教程\n");
    config.push_str("add_tips: false\n\n");
    config.push_str("# 书籍语言 (zh, en, de, fr, it, es, ja, pt, ru, nl)\n");
    config.push_str("lang: \"zh\"\n\n");
    config.push_str("# 输出格式 (epub, all)\n");
    config.push_str("format: \"all\"\n\n");
    config.push_str("# 是否分离章节序号和标题\n");
    config.push_str("separate_chapter_number: false\n\n");
    config.push_str("# 自定义 CSS 文件路径\n");
    config.push_str("# custom_css: \"custom.css\"\n\n");
    config.push_str("# 扩展 CSS（内联）\n");
    config.push_str("# extended_css: |\n");
    config.push_str("#   .content {\n");
    config.push_str("#     font-size: 18px;\n");
    config.push_str("#   }\n\n");
    config.push_str("# CSS 变量\n");
    config.push_str("# css_variables:\n");
    config.push_str("#   primary-color: \"#333333\"\n");
    config.push_str("#   background-color: \"#ffffff\"\n\n");
    config.push_str("# 嵌入字体文件路径（支持 TTF/OTF 格式）\n");
    config.push_str("# font: \"fonts/custom.ttf\"\n");

    config
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merge_config() {
        let mut book = Book::default();
        let config = serde_yaml::from_str::<Value>(
            r#"
bookname: "Test Book"
author: "Test Author"
max_title_length: 50
"#,
        )
        .unwrap();

        merge_config(&mut book, &config).unwrap();

        assert_eq!(book.bookname, Some("Test Book".to_string()));
        assert_eq!(book.author, "Test Author");
        assert_eq!(book.max_title_length, 50);
    }

    #[test]
    fn test_apply_cli_config() {
        let mut book = Book::default();

        // 手动应用一些 CLI 配置
        book.filename = std::path::PathBuf::from("test.txt");
        book.bookname = Some("CLI Book".to_string());
        book.author = "CLI Author".to_string();
        book.max_title_length = 40;
        book.indent = 3;
        book.align = crate::model::TextAlignment::Left;
        book.format = crate::model::OutputFormat::Epub;
        book.lang = crate::model::Language::En;
        book.separate_chapter_number = true;

        assert_eq!(book.filename, std::path::PathBuf::from("test.txt"));
        assert_eq!(book.bookname, Some("CLI Book".to_string()));
        assert_eq!(book.author, "CLI Author");
        assert_eq!(book.max_title_length, 40);
        assert_eq!(book.indent, 3);
        assert_eq!(book.align, TextAlignment::Left);
        assert_eq!(book.format, OutputFormat::Epub);
        assert_eq!(book.lang, Language::En);
        assert!(book.separate_chapter_number);
    }
}
