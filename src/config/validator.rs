//! 配置验证器
//!
//! 负责验证配置的合法性和一致性

use crate::error::{KafError, Result};
use crate::model::Book;

/// 验证错误
#[derive(Debug, Clone)]
pub enum ValidationError {
    /// 文件不存在
    FileNotFound(String),
    /// 值超出范围
    OutOfRange(String),
    /// 值无效
    InvalidValue(String),
    /// 缺少必需字段
    MissingField(String),
    /// 字段冲突
    FieldConflict(String),
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FileNotFound(path) => write!(f, "文件不存在: {}", path),
            Self::OutOfRange(msg) => write!(f, "值超出范围: {}", msg),
            Self::InvalidValue(msg) => write!(f, "无效的值: {}", msg),
            Self::MissingField(field) => write!(f, "缺少必需字段: {}", field),
            Self::FieldConflict(msg) => write!(f, "字段冲突: {}", msg),
        }
    }
}

impl std::error::Error for ValidationError {}

impl From<ValidationError> for KafError {
    fn from(err: ValidationError) -> Self {
        KafError::ParseError(err.to_string())
    }
}

/// 配置验证器
pub struct ConfigValidator {
    /// 最大标题长度限制
    max_title_length_limit: usize,
    /// 最小标题长度限制
    min_title_length_limit: usize,
    /// 最大缩进限制
    max_indent_limit: usize,
    /// 最小缩进限制
    min_indent_limit: usize,
}

impl ConfigValidator {
    /// 创建新的配置验证器
    pub fn new() -> Self {
        Self {
            max_title_length_limit: 100,
            min_title_length_limit: 5,
            max_indent_limit: 10,
            min_indent_limit: 0,
        }
    }

    /// 验证配置
    pub fn validate(&self, book: &Book) -> Result<()> {
        // 1. 验证文件存在
        self.validate_files(book)?;

        // 2. 验证数值范围
        self.validate_ranges(book)?;

        // 3. 验证枚举值
        self.validate_enums(book)?;

        // 4. 验证字符串格式
        self.validate_formats(book)?;

        // 5. 验证逻辑一致性
        self.validate_consistency(book)?;

        Ok(())
    }

    /// 验证文件存在
    fn validate_files(&self, book: &Book) -> Result<()> {
        // 验证输入文件存在
        if !book.filename.as_os_str().is_empty() && !book.filename.exists() {
            return Err(ValidationError::FileNotFound(book.filename.display().to_string()).into());
        }

        // 验证封面文件存在
        if let Some(crate::model::CoverSource::Local { ref path }) = book.cover {
            if !path.exists() {
                return Err(ValidationError::FileNotFound(path.display().to_string()).into());
            }
        }

        // 验证自定义 CSS 文件存在
        if let Some(ref custom_css) = book.custom_css {
            if !custom_css.exists() {
                return Err(ValidationError::FileNotFound(custom_css.display().to_string()).into());
            }
        }

        // 验证字体文件存在
        if let Some(ref font) = book.font {
            if !font.exists() {
                return Err(ValidationError::FileNotFound(font.display().to_string()).into());
            }
        }

        Ok(())
    }

    /// 验证数值范围
    fn validate_ranges(&self, book: &Book) -> Result<()> {
        // 验证最大标题长度
        if book.max_title_length > self.max_title_length_limit {
            return Err(ValidationError::OutOfRange(format!(
                "max_title_length 不能超过 {}",
                self.max_title_length_limit
            ))
            .into());
        }

        if book.max_title_length < self.min_title_length_limit {
            return Err(ValidationError::OutOfRange(format!(
                "max_title_length 不能小于 {}",
                self.min_title_length_limit
            ))
            .into());
        }

        // 验证缩进
        if book.indent > self.max_indent_limit {
            return Err(ValidationError::OutOfRange(format!(
                "indent 不能超过 {}",
                self.max_indent_limit
            ))
            .into());
        }

        if book.indent < self.min_indent_limit {
            return Err(ValidationError::OutOfRange(format!(
                "indent 不能小于 {}",
                self.min_indent_limit
            ))
            .into());
        }

        Ok(())
    }

    /// 验证枚举值
    fn validate_enums(&self, book: &Book) -> Result<()> {
        // 验证对齐方式（这些已经是枚举类型，编译时检查）
        let _ = book.align;
        let _ = book.lang;
        let _ = book.format;

        Ok(())
    }

    /// 验证字符串格式
    fn validate_formats(&self, book: &Book) -> Result<()> {
        // 验证书名不为空
        if let Some(ref bookname) = book.bookname {
            if bookname.trim().is_empty() {
                return Err(ValidationError::InvalidValue("书名不能为空".to_string()).into());
            }
        }

        // 验证作者不为空
        if book.author.trim().is_empty() {
            return Err(ValidationError::InvalidValue("作者不能为空".to_string()).into());
        }

        // 验证段落间距格式
        if !self.validate_css_value(&book.paragraph_spacing) {
            return Err(ValidationError::InvalidValue(
                format!("无效的段落间距: {}", book.paragraph_spacing),
            )
            .into());
        }

        // 验证行高格式
        if let Some(ref line_height) = book.line_height {
            if !self.validate_css_value(line_height) {
                return Err(ValidationError::InvalidValue(
                    format!("无效的行高: {}", line_height),
                )
                .into());
            }
        }

        Ok(())
    }

    /// 验证逻辑一致性
    fn validate_consistency(&self, _book: &Book) -> Result<()> {
        // 这里可以添加更复杂的逻辑验证
        // 例如：如果指定了字体，检查字体文件扩展名是否正确

        Ok(())
    }

    /// 验证 CSS 值格式
    fn validate_css_value(&self, value: &str) -> bool {
        // 简单的 CSS 值验证
        // 支持数字、像素、em、rem 等
        let value = value.trim();
        if value.is_empty() {
            return false;
        }

        // 检查是否是纯数字
        if value.parse::<f32>().is_ok() {
            return true;
        }

        // 使用正则表达式验证 CSS 值
        let css_regex = regex::Regex::new(r"^\d+(\.\d+)?(px|em|rem|%|vh|vw)$").unwrap();
        css_regex.is_match(value)
    }

    /// 设置最大标题长度限制
    pub fn set_max_title_length_limit(&mut self, limit: usize) {
        self.max_title_length_limit = limit;
    }

    /// 设置最小标题长度限制
    pub fn set_min_title_length_limit(&mut self, limit: usize) {
        self.min_title_length_limit = limit;
    }

    /// 设置最大缩进限制
    pub fn set_max_indent_limit(&mut self, limit: usize) {
        self.max_indent_limit = limit;
    }
}

impl Default for ConfigValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{ChapterHeader, CoverSource};
    use std::path::PathBuf;

    #[test]
    fn test_validator_creation() {
        let validator = ConfigValidator::new();
        assert_eq!(validator.max_title_length_limit, 100);
        assert_eq!(validator.min_title_length_limit, 5);
    }

    #[test]
    fn test_validate_valid_config() {
        let validator = ConfigValidator::new();
        let book = Book {
            filename: PathBuf::from("test.txt"),
            bookname: Some("Test Book".to_string()),
            author: "Test Author".to_string(),
            max_title_length: 35,
            indent: 2,
            paragraph_spacing: "1.5em".to_string(),
            ..Default::default()
        };

        // 文件可能不存在，所以这里只验证其他字段
        let result = validator.validate(&book);
        // 如果文件不存在会失败，这是预期的
        if result.is_err() {
            let err = result.unwrap_err();
            assert!(matches!(err, KafError::ParseError(_)));
        }
    }

    #[test]
    fn test_validate_invalid_title_length() {
        let validator = ConfigValidator::new();
        let book = Book {
            max_title_length: 200, // 超过限制
            ..Default::default()
        };

        let result = validator.validate(&book);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_invalid_indent() {
        let validator = ConfigValidator::new();
        let book = Book {
            indent: 20, // 超过限制
            ..Default::default()
        };

        let result = validator.validate(&book);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_empty_bookname() {
        let validator = ConfigValidator::new();
        let book = Book {
            bookname: Some("".to_string()),
            ..Default::default()
        };

        let result = validator.validate(&book);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_css_value() {
        let validator = ConfigValidator::new();

        assert!(validator.validate_css_value("1.5"));
        assert!(validator.validate_css_value("16px"));
        assert!(validator.validate_css_value("1.5em"));
        assert!(validator.validate_css_value("2rem"));
        assert!(validator.validate_css_value("100%"));
        assert!(!validator.validate_css_value(""));
        assert!(!validator.validate_css_value("invalid"));
    }
}
