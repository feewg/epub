//! kaf-cli 库
//!
//! 提供文本到 EPUB 的转换功能

pub mod style;  // 先声明 style，因为 model 依赖它
pub mod batch;
pub mod cli;
pub mod config;
pub mod converter;
pub mod error;
pub mod model;
pub mod parser;
pub mod utils;

// 重新导出常用类型
pub use error::{KafError, Result};
pub use model::{Book, Section, Language, TextAlignment, OutputFormat, ThemePreset};
pub use parser::{ChapterDetector, ParagraphProcessor, ParagraphMode, ScoreCalculator};
pub use style::{Theme, CssGenerator};

// 配置模块导出
pub use config::{
    load_config, validate_config,
    ConfigLoader, ConfigValidator, ConfigPreset, ConfigSource,
    generate_config_examples,
    generate_basic_config, generate_webnovel_config,
    generate_full_config, generate_minimal_config,
    generate_publication_config
};

/// 生成示例配置（向后兼容）
pub fn generate_example_config() -> String {
    config::generate_basic_config()
}