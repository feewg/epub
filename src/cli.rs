//! CLI 参数解析模块
//!
//! 使用 clap 定义命令行参数解析

use clap::Parser;
use std::path::PathBuf;

/// kaf-cli - 将 TXT 文本转换为 EPUB 电子书
#[derive(Parser, Debug, Clone)]
#[command(name = "kaf-cli")]
#[command(author = "kaf-rs team")]
#[command(version = "0.1.0")]
#[command(about = "Convert txt to epub ebook", long_about = None)]
pub struct Cli {
    /// txt 文件名
    #[arg(short, long)]
    pub filename: Option<PathBuf>,

    /// 书名
    #[arg(short, long)]
    pub bookname: Option<String>,

    /// 作者
    #[arg(short, long, default_value = "YSTYLE")]
    pub author: String,

    /// 章节匹配规则
    #[arg(short = 'm', long)]
    pub chapter_match: Option<String>,

    /// 卷匹配规则
    #[arg(short, long)]
    pub volume_match: Option<String>,

    /// 排除规则
    #[arg(short, long)]
    pub exclude: Option<String>,

    /// 标题最大字数
    #[arg(short = 'M', long, default_value = "35")]
    pub max_title_length: usize,

    /// 段落缩进字数
    #[arg(short, long, default_value = "2")]
    pub indent: usize,

    /// 标题对齐方式 (left, center, right)
    #[arg(long, default_value = "center")]
    pub align: String,

    /// 封面图片
    #[arg(short, long)]
    pub cover: Option<String>,

    /// 输出格式 (epub, all)
    #[arg(long, default_value = "all")]
    pub format: String,

    /// 批量转换文件夹
    #[arg(long)]
    pub batch: Option<PathBuf>,

    /// 生成示例配置
    #[arg(long)]
    pub example_config: bool,

    /// 指定配置文件
    #[arg(short = 'C', long)]
    pub config: Option<PathBuf>,

    /// 书籍语言 (zh, en, de, fr, it, es, ja, pt, ru, nl)
    #[arg(short, long, default_value = "zh")]
    pub lang: String,

    /// 分离章节序号和标题
    #[arg(long)]
    pub separate_chapter_number: bool,

    /// 自定义 CSS 文件
    #[arg(long)]
    pub custom_css: Option<PathBuf>,

    /// 扩展 CSS（内联）
    #[arg(long)]
    pub extended_css: Option<String>,

    /// 嵌入字体文件
    #[arg(long)]
    pub font: Option<PathBuf>,

    /// 行高设置
    #[arg(long)]
    pub line_height: Option<String>,

    /// 段落间距
    #[arg(long)]
    pub paragraph_spacing: Option<String>,

    /// 批量转换输出目录
    #[arg(long)]
    pub output_dir: Option<PathBuf>,

    /// 遇到错误继续转换
    #[arg(long)]
    pub continue_on_error: bool,

    /// 生成报告文件 (json, markdown, html)
    #[arg(long)]
    pub report: Option<String>,

    /// 仅解析不生成 (dry-run)
    #[arg(long)]
    pub dry_run: bool,

    /// 最大错误数量（0 表示无限制）
    #[arg(long, default_value = "0")]
    pub max_errors: usize,

    /// 显示章节识别结果（仅 dry-run 有效）
    #[arg(long)]
    pub show_chapters: bool,

    /// 输入格式 (auto, txt, markdown)
    #[arg(short = 'I', long, default_value = "auto")]
    pub input_format: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_default() {
        let cli = Cli::try_parse_from(["kaf-cli"]).unwrap();
        assert_eq!(cli.author, "YSTYLE");
        assert_eq!(cli.max_title_length, 35);
        assert_eq!(cli.indent, 2);
        assert_eq!(cli.align, "center");
        assert_eq!(cli.format, "all");
    }

    #[test]
    fn test_cli_with_options() {
        let cli = Cli::try_parse_from([
            "kaf-cli",
            "--filename", "test.txt",
            "--bookname", "Test Book",
            "--author", "Test Author",
        ]).unwrap();
        assert_eq!(cli.filename, Some(PathBuf::from("test.txt")));
        assert_eq!(cli.bookname, Some("Test Book".to_string()));
        assert_eq!(cli.author, "Test Author");
    }

    #[test]
    fn test_cli_example_config() {
        let cli = Cli::try_parse_from(["kaf-cli", "--example-config"]).unwrap();
        assert!(cli.example_config);
    }

    #[test]
    fn test_cli_input_format_default() {
        let cli = Cli::try_parse_from(["kaf-cli"]).unwrap();
        assert_eq!(cli.input_format, "auto");
    }

    #[test]
    fn test_cli_input_format_explicit() {
        let cli = Cli::try_parse_from(["kaf-cli", "--input-format", "markdown"]).unwrap();
        assert_eq!(cli.input_format, "markdown");
    }

    #[test]
    fn test_cli_input_format_short() {
        let cli = Cli::try_parse_from(["kaf-cli", "-I", "txt"]).unwrap();
        assert_eq!(cli.input_format, "txt");
    }
}
