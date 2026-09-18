//! 配置管理模块
//!
//! 负责配置的加载、验证和合并

pub mod loader;
pub mod parsers;
pub mod presets;
pub mod validator;

pub use loader::{AuthorSource, ConfigLoader, LoadedConfig};
pub use parsers::{parse_align, parse_format, parse_input_format, parse_lang, parse_theme};
pub use presets::generate_config_examples;
pub use validator::ConfigValidator;

use crate::cli::Cli;
use crate::error::Result;
use crate::model::Book;

/// 从 CLI 加载配置（统一的入口）
pub fn load_config(cli: &Cli) -> Result<Book> {
    let loader = ConfigLoader::new();
    loader.load_from_cli(cli)
}

/// 加载配置并跟踪作者来源（单文件与批量转换共用）
///
/// 与 [`load_config`] 加载规则完全一致，额外返回 [`AuthorSource`]，
/// 供调用方在应用"文件名作者兜底"前判断作者是否被显式配置。
pub fn load_config_tracked(cli: &Cli) -> Result<LoadedConfig> {
    let loader = ConfigLoader::new();
    loader.load_from_cli_tracked(cli)
}

/// 应用文件名作者兜底
///
/// 仅当作者来源为 [`AuthorSource::Default`]（未被 CLI/YAML 显式配置）且
/// 文件名中提取出的作者非空白时，才用它覆盖内置默认值。
/// 显式配置的作者（包括显式指定的 "YSTYLE"）一律原样保留，交由
/// [`validate_config`] 做后续校验（如空白作者应报错而不是被文件名"修复"）。
///
/// 最终优先级：显式 CLI > YAML > 文件名 > 默认 "YSTYLE"。
pub fn apply_filename_author(
    book: &mut Book,
    source: AuthorSource,
    filename_author: Option<String>,
) {
    if source != AuthorSource::Default {
        return;
    }
    if let Some(author) = filename_author {
        let trimmed = author.trim();
        if !trimmed.is_empty() {
            book.author = trimmed.to_string();
        }
    }
}

/// 验证配置
pub fn validate_config(book: &Book) -> Result<()> {
    let validator = ConfigValidator::new();
    validator.validate(book)
}

#[allow(dead_code)]
/// 查找配置文件
pub fn find_config_file(filename: &Option<std::path::PathBuf>) -> Option<std::path::PathBuf> {
    ConfigLoader::find_config(filename)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_config_file() {
        // 在当前目录测试
        let result = find_config_file(&None);
        // 可能找到或不找到，取决于是否存在配置文件
        println!("Config file search result: {:?}", result);
    }

    #[test]
    fn test_generate_config_examples() {
        let examples = generate_config_examples();
        assert!(examples.contains_key("basic"));
        assert!(examples.contains_key("webnovel"));
        assert!(examples.contains_key("full"));
    }
}
