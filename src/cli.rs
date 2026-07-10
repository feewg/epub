//! CLI 参数解析模块

use clap::parser::ValueSource;
use clap::{CommandFactory, FromArgMatches, Parser};
use std::collections::HashSet;
use std::path::PathBuf;

fn parse_title_length(value: &str) -> Result<usize, String> {
    let value = value
        .parse::<usize>()
        .map_err(|_| "标题最大字数必须是整数".to_string())?;
    (5..=100)
        .contains(&value)
        .then_some(value)
        .ok_or_else(|| "标题最大字数必须在 5 到 100 之间".to_string())
}

fn parse_indent(value: &str) -> Result<usize, String> {
    let value = value
        .parse::<usize>()
        .map_err(|_| "段落缩进必须是整数".to_string())?;
    (value <= 10)
        .then_some(value)
        .ok_or_else(|| "段落缩进必须在 0 到 10 之间".to_string())
}

/// kaf-cli 命令行参数。
///
/// 字段保留解析后的默认值，同时内部记录参数是否由用户显式提供，供配置分层使用。
#[derive(Debug, Clone)]
pub struct Cli {
    pub filename: Option<PathBuf>,
    pub output_name: Option<String>,
    pub bookname: Option<String>,
    pub author: String,
    pub chapter_match: Option<String>,
    pub volume_match: Option<String>,
    pub exclude: Option<String>,
    pub max_title_length: usize,
    pub indent: usize,
    pub align: String,
    pub cover: Option<String>,
    pub format: String,
    pub batch: Option<PathBuf>,
    pub example_config: bool,
    pub config: Option<PathBuf>,
    pub lang: String,
    pub separate_chapter_number: bool,
    pub custom_css: Option<PathBuf>,
    pub extended_css: Option<String>,
    pub font: Option<PathBuf>,
    pub line_height: Option<String>,
    pub paragraph_spacing: Option<String>,
    pub theme: String,
    pub output_dir: Option<PathBuf>,
    pub continue_on_error: bool,
    pub report: Option<String>,
    pub dry_run: bool,
    pub max_errors: usize,
    pub show_chapters: bool,
    pub input_format: String,
    pub concurrency: u16,
    explicit: HashSet<String>,
}

impl Cli {
    pub(crate) fn is_explicit(&self, name: &str) -> bool {
        self.explicit.contains(name)
    }
}

#[derive(Parser, Debug, Clone)]
#[command(name = "kaf-cli")]
#[command(author = "kaf-rs team")]
#[command(version)]
#[command(about = "Convert TXT or Markdown to EPUB", long_about = None)]
struct CliArgs {
    /// TXT/Markdown 文件名
    #[arg(short, long)]
    filename: Option<PathBuf>,

    /// 输出文件名（不含扩展名）
    #[arg(short, long)]
    output_name: Option<String>,

    /// 书名
    #[arg(short, long)]
    bookname: Option<String>,

    /// 作者
    #[arg(short, long, default_value = "YSTYLE")]
    author: String,

    /// 章节匹配规则
    #[arg(short = 'm', long)]
    chapter_match: Option<String>,

    /// 卷匹配规则
    #[arg(short, long)]
    volume_match: Option<String>,

    /// 排除规则
    #[arg(short, long)]
    exclude: Option<String>,

    /// 标题最大字数
    #[arg(short = 'M', long, default_value_t = 35, value_parser = parse_title_length)]
    max_title_length: usize,

    /// 段落缩进字数
    #[arg(short, long, default_value_t = 2, value_parser = parse_indent)]
    indent: usize,

    /// 标题对齐方式
    #[arg(long, default_value = "center", value_parser = ["left", "center", "right"])]
    align: String,

    /// 封面图片
    #[arg(short, long)]
    cover: Option<String>,

    /// 输出格式
    #[arg(long, default_value = "all", value_parser = ["epub", "all"])]
    format: String,

    /// 批量转换文件夹
    #[arg(long)]
    batch: Option<PathBuf>,

    /// 生成示例配置
    #[arg(long)]
    example_config: bool,

    /// 指定配置文件
    #[arg(short = 'C', long)]
    config: Option<PathBuf>,

    /// 书籍语言
    #[arg(short, long, default_value = "zh", value_parser = ["zh", "en", "de", "fr", "it", "es", "ja", "pt", "ru", "nl"])]
    lang: String,

    /// 分离章节序号和标题
    #[arg(long)]
    separate_chapter_number: bool,

    /// 自定义 CSS 文件
    #[arg(long)]
    custom_css: Option<PathBuf>,

    /// 扩展 CSS（内联）
    #[arg(long)]
    extended_css: Option<String>,

    /// 嵌入字体文件
    #[arg(long)]
    font: Option<PathBuf>,

    /// 行高设置
    #[arg(long)]
    line_height: Option<String>,

    /// 段落间距
    #[arg(long)]
    paragraph_spacing: Option<String>,

    /// 主题
    #[arg(long, default_value = "light", value_parser = ["light", "dark", "sepia", "high_contrast", "high-contrast", "modern", "traditional"])]
    theme: String,

    /// 批量转换输出目录
    #[arg(long)]
    output_dir: Option<PathBuf>,

    /// 遇到错误继续转换
    #[arg(long)]
    continue_on_error: bool,

    /// 生成报告文件
    #[arg(long, value_parser = ["json", "markdown", "md", "html"])]
    report: Option<String>,

    /// 仅解析不生成
    #[arg(long)]
    dry_run: bool,

    /// 最大错误数量（0 表示无限制）
    #[arg(long, default_value_t = 0)]
    max_errors: usize,

    /// 显示章节识别结果（仅 dry-run 有效）
    #[arg(long)]
    show_chapters: bool,

    /// 输入格式
    #[arg(short = 'I', long, default_value = "auto", value_parser = ["auto", "txt", "text", "markdown", "md"])]
    input_format: String,

    /// 批量转换并发数
    #[arg(long, default_value_t = 4, value_parser = clap::value_parser!(u16).range(1..))]
    concurrency: u16,
}

const TRACKED_OPTIONS: &[&str] = &[
    "author",
    "max_title_length",
    "indent",
    "align",
    "format",
    "lang",
    "separate_chapter_number",
    "theme",
    "input_format",
];

impl Cli {
    fn from_matches(matches: &clap::ArgMatches) -> Result<Self, clap::Error> {
        let explicit = TRACKED_OPTIONS
            .iter()
            .filter(|name| matches.value_source(name) == Some(ValueSource::CommandLine))
            .map(|name| (*name).to_string())
            .collect();
        let raw = <CliArgs as FromArgMatches>::from_arg_matches(matches)?;
        Ok(Self {
            filename: raw.filename,
            output_name: raw.output_name,
            bookname: raw.bookname,
            author: raw.author,
            chapter_match: raw.chapter_match,
            volume_match: raw.volume_match,
            exclude: raw.exclude,
            max_title_length: raw.max_title_length,
            indent: raw.indent,
            align: raw.align,
            cover: raw.cover,
            format: raw.format,
            batch: raw.batch,
            example_config: raw.example_config,
            config: raw.config,
            lang: raw.lang,
            separate_chapter_number: raw.separate_chapter_number,
            custom_css: raw.custom_css,
            extended_css: raw.extended_css,
            font: raw.font,
            line_height: raw.line_height,
            paragraph_spacing: raw.paragraph_spacing,
            theme: raw.theme,
            output_dir: raw.output_dir,
            continue_on_error: raw.continue_on_error,
            report: raw.report,
            dry_run: raw.dry_run,
            max_errors: raw.max_errors,
            show_chapters: raw.show_chapters,
            input_format: raw.input_format,
            concurrency: raw.concurrency,
            explicit,
        })
    }
}

impl CommandFactory for Cli {
    fn command() -> clap::Command {
        CliArgs::command()
    }

    fn command_for_update() -> clap::Command {
        CliArgs::command_for_update()
    }
}

impl FromArgMatches for Cli {
    fn from_arg_matches(matches: &clap::ArgMatches) -> Result<Self, clap::Error> {
        Self::from_matches(matches)
    }

    fn update_from_arg_matches(&mut self, matches: &clap::ArgMatches) -> Result<(), clap::Error> {
        *self = Self::from_matches(matches)?;
        Ok(())
    }
}

impl Parser for Cli {}
