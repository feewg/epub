//! 配置加载器
//!
//! 负责从默认值、YAML 文件和显式 CLI 参数中按优先级合并配置。

use crate::cli::Cli;
use crate::config::parsers;
use crate::error::Result;
use crate::model::{Book, ChapterHeader, CoverSource};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// 配置源
#[derive(Debug, Clone)]
pub enum ConfigSource {
    Cli,
    File(PathBuf),
    Default,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum ConfigCover {
    Path(PathBuf),
    Source(CoverSource),
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct FileConfig {
    filename: Option<PathBuf>,
    bookname: Option<String>,
    author: Option<String>,
    chapter_match: Option<String>,
    volume_match: Option<String>,
    exclusion_pattern: Option<String>,
    max_title_length: Option<usize>,
    indent: Option<usize>,
    align: Option<String>,
    unknown_title: Option<String>,
    cover: Option<ConfigCover>,
    font: Option<PathBuf>,
    paragraph_spacing: Option<String>,
    line_height: Option<String>,
    add_tips: Option<bool>,
    lang: Option<String>,
    format: Option<String>,
    output_name: Option<String>,
    separate_chapter_number: Option<bool>,
    custom_css: Option<PathBuf>,
    extended_css: Option<String>,
    css_variables: Option<HashMap<String, String>>,
    chapter_header: Option<ChapterHeader>,
    theme: Option<String>,
    input_format: Option<String>,
    lookahead_lines: Option<usize>,
}

/// 配置加载器
pub struct ConfigLoader;

impl ConfigLoader {
    /// 创建新的配置加载器
    pub fn new() -> Self {
        Self
    }

    /// 从 CLI 加载配置（默认值 < 配置文件 < 显式 CLI 参数）
    pub fn load_from_cli(&self, cli: &Cli) -> Result<Book> {
        let mut book = Book::default();

        let config_path = cli
            .config
            .clone()
            .or_else(|| Self::find_config(&cli.filename));
        if let Some(path) = config_path {
            self.load_config_file(&mut book, &path)?;
        }

        self.apply_cli_config(&mut book, cli)?;
        Ok(book)
    }

    /// 查找配置文件
    pub fn find_config(filename: &Option<PathBuf>) -> Option<PathBuf> {
        let config_names = ["kaf.yaml", "kaf.yml", ".kaf.yaml", ".kaf.yml"];
        let input_dir = filename.as_ref().and_then(|path| path.parent());

        for dir in input_dir.into_iter().chain(std::iter::once(Path::new("."))) {
            for name in config_names {
                let path = dir.join(name);
                if path.is_file() {
                    return Some(path);
                }
            }
        }
        None
    }

    fn load_config_file(&self, book: &mut Book, path: &Path) -> Result<()> {
        let absolute_path = if path.is_absolute() {
            path.to_path_buf()
        } else {
            std::env::current_dir()?.join(path)
        };
        let absolute_path = std::fs::canonicalize(absolute_path)?;
        let content = fs::read_to_string(&absolute_path)?;
        let config: FileConfig = serde_yaml::from_str(&content)?;
        self.merge_config(book, config, absolute_path.parent())
    }

    fn merge_config(
        &self,
        book: &mut Book,
        config: FileConfig,
        config_dir: Option<&Path>,
    ) -> Result<()> {
        if let Some(value) = config.filename {
            book.filename = Self::resolve_config_path(value, config_dir);
        }
        if let Some(value) = config.bookname {
            book.bookname = Some(value);
        }
        if let Some(value) = config.author {
            book.author = value;
        }
        if let Some(value) = config.chapter_match {
            book.chapter_match = Some(value);
        }
        if let Some(value) = config.volume_match {
            book.volume_match = Some(value);
        }
        if let Some(value) = config.exclusion_pattern {
            book.exclusion_pattern = Some(value);
        }
        if let Some(value) = config.max_title_length {
            book.max_title_length = value;
        }
        if let Some(value) = config.indent {
            book.indent = value;
        }
        if let Some(value) = config.align {
            book.align = parsers::parse_align(&value)?;
        }
        if let Some(value) = config.unknown_title {
            book.unknown_title = value;
        }
        if let Some(value) = config.cover {
            book.cover = Some(match value {
                ConfigCover::Path(path) => CoverSource::Local {
                    path: Self::resolve_config_path(path, config_dir),
                },
                ConfigCover::Source(CoverSource::Local { path }) => CoverSource::Local {
                    path: Self::resolve_config_path(path, config_dir),
                },
                ConfigCover::Source(source) => source,
            });
        }
        if let Some(value) = config.font {
            book.font = Some(Self::resolve_config_path(value, config_dir));
        }
        if let Some(value) = config.paragraph_spacing {
            book.paragraph_spacing = value;
        }
        if let Some(value) = config.line_height {
            book.line_height = Some(value);
        }
        if let Some(value) = config.add_tips {
            book.add_tips = value;
        }
        if let Some(value) = config.lang {
            book.lang = parsers::parse_lang(&value)?;
        }
        if let Some(value) = config.format {
            book.format = parsers::parse_format(&value)?;
        }
        if let Some(value) = config.output_name {
            book.output_name = Some(value);
        }
        if let Some(value) = config.separate_chapter_number {
            book.separate_chapter_number = value;
        }
        if let Some(value) = config.custom_css {
            book.custom_css = Some(Self::resolve_config_path(value, config_dir));
        }
        if let Some(value) = config.extended_css {
            book.extended_css = Some(value);
        }
        if let Some(value) = config.css_variables {
            book.css_variables = value;
        }
        if let Some(mut value) = config.chapter_header {
            value.image = value
                .image
                .map(|path| Self::resolve_config_path(path, config_dir));
            value.image_folder = value
                .image_folder
                .map(|path| Self::resolve_config_path(path, config_dir));
            book.chapter_header = value;
        }
        if let Some(value) = config.theme {
            book.theme = parsers::parse_theme(&value)?;
        }
        if let Some(value) = config.input_format {
            book.input_format = parsers::parse_input_format(&value)?;
        }
        if let Some(value) = config.lookahead_lines {
            book.lookahead_lines = value;
        }
        Ok(())
    }

    fn resolve_config_path(path: PathBuf, config_dir: Option<&Path>) -> PathBuf {
        if path.is_absolute() {
            path
        } else {
            config_dir.unwrap_or_else(|| Path::new(".")).join(path)
        }
    }

    fn apply_cli_config(&self, book: &mut Book, cli: &Cli) -> Result<()> {
        if let Some(value) = &cli.filename {
            book.filename = value.clone();
        }
        if let Some(value) = &cli.output_name {
            book.output_name = Some(value.clone());
        }
        if let Some(value) = &cli.bookname {
            book.bookname = Some(value.clone());
        }
        if cli.is_explicit("author") {
            book.author = cli.author.clone();
        }
        if let Some(value) = &cli.chapter_match {
            book.chapter_match = Some(value.clone());
        }
        if let Some(value) = &cli.volume_match {
            book.volume_match = Some(value.clone());
        }
        if let Some(value) = &cli.exclude {
            book.exclusion_pattern = Some(value.clone());
        }
        if cli.is_explicit("max_title_length") {
            book.max_title_length = cli.max_title_length;
        }
        if cli.is_explicit("indent") {
            book.indent = cli.indent;
        }
        if cli.is_explicit("align") {
            book.align = parsers::parse_align(&cli.align)?;
        }
        if cli.is_explicit("format") {
            book.format = parsers::parse_format(&cli.format)?;
        }
        if cli.is_explicit("lang") {
            book.lang = parsers::parse_lang(&cli.lang)?;
        }
        if cli.is_explicit("separate_chapter_number") {
            book.separate_chapter_number = cli.separate_chapter_number;
        }
        if cli.is_explicit("theme") {
            book.theme = parsers::parse_theme(&cli.theme)?;
        }
        if cli.is_explicit("input_format") {
            book.input_format = parsers::parse_input_format(&cli.input_format)?;
        }

        if let Some(value) = &cli.cover {
            book.cover = Some(CoverSource::Local {
                path: Self::resolve_cli_path(Path::new(value))?,
            });
        }
        if let Some(value) = &cli.custom_css {
            book.custom_css = Some(Self::resolve_cli_path(value)?);
        }
        if let Some(value) = &cli.extended_css {
            book.extended_css = Some(value.clone());
        }
        if let Some(value) = &cli.font {
            book.font = Some(Self::resolve_cli_path(value)?);
        }
        if let Some(value) = &cli.line_height {
            book.line_height = Some(value.clone());
        }
        if let Some(value) = &cli.paragraph_spacing {
            book.paragraph_spacing = value.clone();
        }
        Ok(())
    }

    fn resolve_cli_path(path: &Path) -> Result<PathBuf> {
        if path.is_absolute() {
            Ok(path.to_path_buf())
        } else {
            Ok(std::env::current_dir()?.join(path))
        }
    }
}

impl Default for ConfigLoader {
    fn default() -> Self {
        Self::new()
    }
}
