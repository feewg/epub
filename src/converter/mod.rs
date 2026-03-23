//! 格式转换器模块
//!
//! 提供各种格式的转换功能

pub mod epub3;

pub use epub3::EpubConverter3;

pub use crate::model::OutputFormat;

/// 根据格式选择使用相应的转换器
pub fn get_converter(book: &crate::model::Book) -> EpubConverter3 {
    epub3::EpubConverter3::new(book.clone())
}
