//! 配置管理模块
//!
//! 负责配置的加载、验证和合并

pub mod loader;
pub mod validator;
pub mod presets;

pub use loader::ConfigLoader;
pub use validator::ConfigValidator;
pub use presets::generate_config_examples;

use crate::cli::Cli;
use crate::error::Result;
use crate::model::Book;

/// 从 CLI 加载配置（统一的入口）
pub fn load_config(cli: &Cli) -> Result<Book> {
    let loader = ConfigLoader::new();
    loader.load_from_cli(cli)
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
