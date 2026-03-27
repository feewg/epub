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

pub use converter::EpubConverter3;
pub use error::{KafError, Result};
pub use model::{Book, Section, Language, TextAlignment, OutputFormat, InputFormat, ThemePreset};
pub use parser::{ChapterDetector, ParagraphProcessor, MarkdownParser, FormatDetector, Parser};
pub use style::{Theme, CssGenerator};
pub use batch::{BatchConfig, EnhancedBatchConverter, BatchReport, ReportFormat};
pub use config::{load_config, validate_config, ConfigLoader, ConfigValidator, generate_config_examples};

/// 生成示例配置（向后兼容）
pub fn generate_example_config() -> String {
    config::presets::generate_basic_config()
}
