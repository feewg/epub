//! 错误处理模块
//!
//! 定义了项目中所有可能出现的错误类型

use thiserror::Error;

/// 错误类型枚举
#[derive(Error, Debug)]
pub enum KafError {
    /// 文件不存在
    #[error("文件不存在: {0}")]
    FileNotFound(String),

    /// IO 错误
    #[error("IO 错误: {0}")]
    Io(#[from] std::io::Error),

    /// 正则表达式错误
    #[error("正则表达式错误: {0}")]
    Regex(#[from] regex::Error),

    /// 编码检测失败
    #[error("编码检测失败: {0}")]
    #[allow(dead_code)]
    Encoding(String),

    /// 章节解析失败
    #[error("章节解析失败: {0}")]
    ParseError(String),

    /// EPUB 生成失败
    #[error("EPUB 生成失败: {0}")]
    #[allow(dead_code)]
    EpubGenerationFailed(String),

    /// 配置解析失败
    #[error("配置解析失败: {0}")]
    ConfigParseError(#[from] serde_yaml::Error),

    /// 图片处理失败
    #[error("图片处理失败: {0}")]
    ImageError(#[from] image::ImageError),

    /// 序列化错误
    #[error("序列化错误: {0}")]
    #[allow(dead_code)]
    SerdeError(String),

    /// 未知错误
    #[error("未知错误: {0}")]
    Unknown(String),

    /// ZIP 错误
    #[error("ZIP 错误: {0}")]
    Zip(#[from] zip::result::ZipError),

    /// EPUB Builder 错误
    #[error("EPUB Builder 错误: {0}")]
    EpubBuilder(String),
}

/// 结果类型别名
pub type Result<T> = std::result::Result<T, KafError>;

impl From<epub_builder::Error> for KafError {
    fn from(err: epub_builder::Error) -> Self {
        KafError::EpubBuilder(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = KafError::FileNotFound("test.txt".to_string());
        assert_eq!(err.to_string(), "文件不存在: test.txt");
    }

    #[test]
    fn test_error_from_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "not found");
        let err: KafError = io_err.into();
        assert!(matches!(err, KafError::Io(_)));
    }
}
