//! 配置验证器

use crate::error::{KafError, Result};
use crate::model::{Book, CoverSource, HeaderMode};
use once_cell::sync::Lazy;
use regex::Regex;
use std::path::Path;

static CSS_VALUE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^(?:0|\d+(?:\.\d+)?(?:px|em|rem|%|vh|vw))$").expect("固定 CSS 数值正则必须有效")
});

#[derive(Debug, Clone)]
pub enum ValidationError {
    FileNotFound(String),
    OutOfRange(String),
    InvalidValue(String),
    MissingField(String),
    FieldConflict(String),
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FileNotFound(path) => write!(formatter, "文件不存在: {path}"),
            Self::OutOfRange(message) => write!(formatter, "值超出范围: {message}"),
            Self::InvalidValue(message) => write!(formatter, "无效的值: {message}"),
            Self::MissingField(field) => write!(formatter, "缺少必需字段: {field}"),
            Self::FieldConflict(message) => write!(formatter, "字段冲突: {message}"),
        }
    }
}

impl std::error::Error for ValidationError {}

impl From<ValidationError> for KafError {
    fn from(error: ValidationError) -> Self {
        KafError::ParseError(error.to_string())
    }
}

pub struct ConfigValidator {
    max_title_length_limit: usize,
    min_title_length_limit: usize,
    max_indent_limit: usize,
}

impl ConfigValidator {
    pub fn new() -> Self {
        Self {
            max_title_length_limit: 100,
            min_title_length_limit: 5,
            max_indent_limit: 10,
        }
    }

    pub fn validate(&self, book: &Book) -> Result<()> {
        self.validate_files(book)?;
        self.validate_ranges(book)?;
        self.validate_formats(book)?;
        self.validate_consistency(book)?;
        Ok(())
    }

    fn validate_files(&self, book: &Book) -> Result<()> {
        if book.filename.as_os_str().is_empty() {
            return Err(ValidationError::MissingField("filename".to_string()).into());
        }
        if !book.filename.is_file() {
            return Err(ValidationError::FileNotFound(book.filename.display().to_string()).into());
        }
        let input_parent = book.filename.parent();

        if let Some(CoverSource::Local { path }) = &book.cover {
            Self::require_file(path, input_parent, "封面")?;
        }
        if let Some(path) = &book.custom_css {
            Self::require_file(path, input_parent, "自定义 CSS")?;
        }
        if let Some(path) = &book.font {
            let resolved = Self::require_file(path, input_parent, "字体")?;
            let extension = resolved
                .extension()
                .and_then(|value| value.to_str())
                .unwrap_or_default()
                .to_ascii_lowercase();
            if !matches!(extension.as_str(), "ttf" | "otf" | "woff" | "woff2" | "ttc") {
                return Err(ValidationError::InvalidValue(format!(
                    "不支持的字体格式: {}",
                    resolved.display()
                ))
                .into());
            }
        }

        match book.chapter_header.mode {
            HeaderMode::Single => {
                if let Some(path) = &book.chapter_header.image {
                    Self::require_file(path, input_parent, "章节页眉")?;
                }
            }
            HeaderMode::Folder => {
                let folder = book.chapter_header.image_folder.as_ref().ok_or_else(|| {
                    ValidationError::MissingField("chapter_header.image_folder".to_string())
                })?;
                let resolved = crate::utils::file::resolve_resource_path(folder, input_parent)?;
                if !resolved.is_dir() {
                    return Err(ValidationError::InvalidValue(format!(
                        "章节页眉路径不是目录: {}",
                        resolved.display()
                    ))
                    .into());
                }
            }
        }
        Ok(())
    }

    fn require_file(path: &Path, base: Option<&Path>, label: &str) -> Result<std::path::PathBuf> {
        let resolved = crate::utils::file::resolve_resource_path(path, base)?;
        if !resolved.is_file() {
            return Err(ValidationError::InvalidValue(format!(
                "{label}路径不是文件: {}",
                resolved.display()
            ))
            .into());
        }
        Ok(resolved)
    }

    fn validate_ranges(&self, book: &Book) -> Result<()> {
        if !(self.min_title_length_limit..=self.max_title_length_limit)
            .contains(&book.max_title_length)
        {
            return Err(ValidationError::OutOfRange(format!(
                "max_title_length 必须在 {} 到 {} 之间",
                self.min_title_length_limit, self.max_title_length_limit
            ))
            .into());
        }
        if book.indent > self.max_indent_limit {
            return Err(ValidationError::OutOfRange(format!(
                "indent 不能超过 {}",
                self.max_indent_limit
            ))
            .into());
        }
        if book.lookahead_lines == 0 {
            return Err(
                ValidationError::OutOfRange("lookahead_lines 必须大于 0".to_string()).into(),
            );
        }
        Ok(())
    }

    fn validate_formats(&self, book: &Book) -> Result<()> {
        if book
            .bookname
            .as_ref()
            .is_some_and(|value| value.trim().is_empty())
        {
            return Err(ValidationError::InvalidValue("书名不能为空".to_string()).into());
        }
        if book.author.trim().is_empty() {
            return Err(ValidationError::InvalidValue("作者不能为空".to_string()).into());
        }
        if !Self::validate_css_value(&book.paragraph_spacing) {
            return Err(ValidationError::InvalidValue(format!(
                "无效的段落间距: {}",
                book.paragraph_spacing
            ))
            .into());
        }
        if let Some(line_height) = &book.line_height {
            let valid = line_height
                .trim()
                .parse::<f32>()
                .is_ok_and(|value| value.is_finite() && value > 0.0)
                || Self::validate_css_value(line_height);
            if !valid {
                return Err(
                    ValidationError::InvalidValue(format!("无效的行高: {line_height}")).into(),
                );
            }
        }
        for (name, value) in &book.css_variables {
            let normalized = name.strip_prefix("--").unwrap_or(name);
            if normalized.is_empty()
                || !normalized.chars().all(|character| {
                    character.is_ascii_alphanumeric() || matches!(character, '-' | '_')
                })
            {
                return Err(
                    ValidationError::InvalidValue(format!("无效的 CSS 变量名: {name}")).into(),
                );
            }
            if value.contains([';', '{', '}']) {
                return Err(ValidationError::InvalidValue(format!(
                    "CSS 变量 {name} 包含不允许的字符"
                ))
                .into());
            }
        }
        Ok(())
    }

    fn validate_consistency(&self, book: &Book) -> Result<()> {
        if let Some(output_name) = &book.output_name {
            if output_name.trim().is_empty() {
                return Err(
                    ValidationError::InvalidValue("output_name 不能为空".to_string()).into(),
                );
            }
        }
        Ok(())
    }

    fn validate_css_value(value: &str) -> bool {
        CSS_VALUE.is_match(value.trim())
    }

    pub fn set_max_title_length_limit(&mut self, limit: usize) {
        self.max_title_length_limit = limit;
    }

    pub fn set_min_title_length_limit(&mut self, limit: usize) {
        self.min_title_length_limit = limit;
    }

    pub fn set_max_indent_limit(&mut self, limit: usize) {
        self.max_indent_limit = limit;
    }
}

impl Default for ConfigValidator {
    fn default() -> Self {
        Self::new()
    }
}
