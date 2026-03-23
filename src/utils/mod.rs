//! 工具函数模块
//!
//! 提供各种辅助功能

pub mod encoding;
pub mod html;
pub mod regex;
pub mod file;
pub mod cover;

// 重新导出常用函数
pub use encoding::{ensure_no_bom, clean_utf8_output};
