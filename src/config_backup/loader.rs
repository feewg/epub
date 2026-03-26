//! 配置加载器
//!
//! 负责从不同来源加载和合并配置

use crate::cli::Cli;
use crate::error::{KafError, Result};
use crate::model::{Book, Language, OutputFormat, TextAlignment};
use serde_yaml::Value;
use std::fs;
use std::path::{Path, PathBuf};

/// 配置源
#[derive(Debug, Clone)]
pub enum ConfigSource {
    /// CLI 参数（最高优先级）
    Cli,
    /// 配置文件
    File(PathBuf),
    /// 默认配置
    Default,
}

/// 配置加载器
pub struct ConfigLoader {
    /// 配置文件名称列表
    config_filenames: Vec<String>,
}

impl ConfigLoader {
    /// 创建新的配置加载器
    pub fn new() -> Self {
        Self {
            config_filenames: vec![
                "kaf.yaml".to_string(),
                "kaf.yml".to_string(),
                ".kaf.yaml".to_string(),
                ".kaf.yml".to_string(),
            ],
        }
    }

    /// 从 CLI 加载配置（统一的入口）
    pub fn load_from_cli(&self, cli: &Cli) -> Result<Book> {
        let mut book = Book::default();

        // 1. 加载配置文件（如果指定）
        if let Some(config_path) = &cli.config {
            self.load_config_file(&mut book, config_path, ConfigSource::File(config_path.clone()))?;
        } else {
            // 自动查找配置文件
            if let Some(auto_config) = Self::find_config(&cli.filename) {
                self.load_config_file(&mut book, &auto_config, ConfigSource::File(auto_config.clone()))?;
            }
        }

        // 2. 应用 CLI 参数（最高优先级）
        self.apply_cli_config(&mut book, cli)?;

        // 3. 如果没有指定文件名，使用 CLI 的文件名
        if book.filename.as_os_str().is_empty() {
            if let Some(ref filename) = cli.filename {
                book.filename = filename.clone();
            }
        }

        Ok(book)
    }

    /// 查找配置文件
    pub fn find_config(filename: &Option<PathBuf>) -> Option<PathBuf> {
        let config_names = ["kaf.yaml", "kaf.yml", ".kaf.yaml", ".kaf.yml"];

        // 优先搜索文件所在目录，然后搜索当前目录
        let search_dirs = filename.as_ref()
            .and_then(|p| p.parent())
            .into_iter()
            .chain(Some(Path::new(".")));

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

    /// 加载配置文件
    fn load_config_file(&self, book: &mut Book, path: &Path, source: ConfigSource) -> Result<()> {
        let content = fs::read_to_string(path)?;
        let config: Value = serde_yaml::from_str(&content)
            .map_err(|e| KafError::ConfigParseError(e))?;

        self.merge_config(book, &config, source)
    }

    /// 合并配置
    fn merge_config(&self, book: &mut Book, config: &Value, _source: ConfigSource) -> Result<()> {
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
            book.align = Self::parse_align(s)?;
        }
        if let Some(s) = config.get("lang").and_then(|v| v.as_str()) {
            book.lang = Self::parse_lang(s)?;
        }
        if let Some(s) = config.get("format").and_then(|v| v.as_str()) {
            book.format = Self::parse_format(s)?;
        }

        // 自定义 CSS
        if let Some(ref custom_css) = config.get("custom_css").and_then(|v| v.as_str()) {
            book.custom_css = Some(custom_css.into());
        }
        if let Some(ref extended_css) = config.get("extended_css").and_then(|v| v.as_str()) {
            book.extended_css = Some(extended_css.to_string());
        }

        // 字体
        if let Some(ref font) = config.get("font").and_then(|v| v.as_str()) {
            book.font = Some(PathBuf::from(font));
        }

        // 封面
        if let Some(ref cover) = config.get("cover").and_then(|v| v.as_str()) {
            book.cover = Some(crate::model::CoverSource::Local {
                path: PathBuf::from(cover),
            });
        }

        // 输出文件名
        if let Some(ref output_name) = config.get("output_name").and_then(|v| v.as_str()) {
            book.output_name = Some(output_name.to_string());
        }

        Ok(())
    }

    /// 应用 CLI 配置
    fn apply_cli_config(&self, book: &mut Book, cli: &Cli) -> Result<()> {
        // 文件相关
        if let Some(ref filename) = cli.filename {
            book.filename = filename.clone();
        }

        // 书籍信息
        if let Some(ref bookname) = cli.bookname {
            book.bookname = Some(bookname.clone());
        }
        book.author = cli.author.clone();

        // 解析配置
        if let Some(ref chapter_match) = cli.chapter_match {
            book.chapter_match = Some(chapter_match.clone());
        }
        if let Some(ref volume_match) = cli.volume_match {
            book.volume_match = Some(volume_match.clone());
        }
        if let Some(ref exclude) = cli.exclude {
            book.exclusion_pattern = Some(exclude.clone());
        }

        // 格式配置
        book.max_title_length = cli.max_title_length;
        book.indent = cli.indent;
        book.align = Self::parse_align(&cli.align)?;
        book.format = Self::parse_format(&cli.format)?;
        book.lang = Self::parse_lang(&cli.lang)?;
        book.separate_chapter_number = cli.separate_chapter_number;

        // 样式配置
        if let Some(ref cover) = cli.cover {
            book.cover = Some(crate::model::CoverSource::Local {
                path: PathBuf::from(cover),
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

    /// 解析对齐方式
    fn parse_align(s: &str) -> Result<TextAlignment> {
        Ok(match s.to_lowercase().as_str() {
            "left" => TextAlignment::Left,
            "center" => TextAlignment::Center,
            "right" => TextAlignment::Right,
            _ => return Err(KafError::ParseError(format!("无效的对齐方式: {}", s))),
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
            _ => return Err(KafError::ParseError(format!("无效的语言: {}", s))),
        })
    }

    /// 解析输出格式
    fn parse_format(s: &str) -> Result<OutputFormat> {
        Ok(match s.to_lowercase().as_str() {
            "epub" => OutputFormat::Epub,
            "all" => OutputFormat::All,
            _ => return Err(KafError::ParseError(format!("无效的输出格式: {}", s))),
        })
    }
}

impl Default for ConfigLoader {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_loader_creation() {
        let loader = ConfigLoader::new();
        assert_eq!(loader.config_filenames.len(), 4);
    }

    #[test]
    fn test_parse_align() {
        assert!(matches!(ConfigLoader::parse_align("left"), Ok(TextAlignment::Left)));
        assert!(matches!(ConfigLoader::parse_align("center"), Ok(TextAlignment::Center)));
        assert!(matches!(ConfigLoader::parse_align("right"), Ok(TextAlignment::Right)));
        assert!(ConfigLoader::parse_align("invalid").is_err());
    }

    #[test]
    fn test_parse_lang() {
        assert!(matches!(ConfigLoader::parse_lang("zh"), Ok(Language::Zh)));
        assert!(matches!(ConfigLoader::parse_lang("en"), Ok(Language::En)));
        assert!(ConfigLoader::parse_lang("invalid").is_err());
    }

    #[test]
    fn test_parse_format() {
        assert!(matches!(ConfigLoader::parse_format("epub"), Ok(OutputFormat::Epub)));
        assert!(matches!(ConfigLoader::parse_format("all"), Ok(OutputFormat::All)));
        assert!(ConfigLoader::parse_format("invalid").is_err());
    }
}
