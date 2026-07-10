//! kaf-cli 库
//!
//! 提供文本到 EPUB 的转换功能

pub mod batch;
pub mod cli;
pub mod config;
pub mod converter;
pub mod error;
pub mod model;
pub mod parser;
pub mod style;
pub mod utils;

pub use batch::{BatchConfig, BatchInput, BatchReport, EnhancedBatchConverter, ReportFormat};
pub use config::{
    generate_config_examples, load_config, validate_config, ConfigLoader, ConfigValidator,
};
pub use converter::EpubConverter3;
pub use error::{KafError, Result};
pub use model::{Book, InputFormat, Language, OutputFormat, Section, TextAlignment, ThemePreset};
pub use parser::{ChapterDetector, FormatDetector, MarkdownParser, ParagraphProcessor, Parser};
pub use style::{CssGenerator, Theme};

/// 生成示例配置（向后兼容）
pub fn generate_example_config() -> String {
    config::presets::generate_basic_config()
}
