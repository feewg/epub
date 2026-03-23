//! 封面处理模块
//!
//! 处理封面图片的获取和处理

use crate::error::{KafError, Result};
use crate::model::CoverSource;
use image::ImageFormat;
use std::path::Path;

/// 从封面源获取封面图片数据
pub async fn fetch_cover(cover: &CoverSource) -> Result<Vec<u8>> {
    match cover {
        CoverSource::Local { path } => fetch_local_cover(path),
    }
}

/// 读取本地封面图片
pub fn fetch_local_cover(path: &Path) -> Result<Vec<u8>> {
    if !path.exists() {
        return Err(KafError::FileNotFound(
            path.to_string_lossy().to_string(),
        ));
    }

    // 读取文件
    let bytes = std::fs::read(path)?;

    // 验证是否为有效图片
    let _ = image::load_from_memory(&bytes)?;

    Ok(bytes)
}

/// 转换图片格式为 JPEG
pub fn convert_to_jpeg(data: &[u8], _quality: u8) -> Result<Vec<u8>> {
    let img = image::load_from_memory(data)?;

    let mut buffer = Vec::new();
    let mut cursor = std::io::Cursor::new(&mut buffer);

    img.write_to(&mut cursor, ImageFormat::Jpeg)?;

    Ok(buffer)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_to_jpeg() {
        // 创建一个简单的测试图片
        let img = image::RgbImage::new(100, 100);
        let mut buffer = Vec::new();
        let mut cursor = std::io::Cursor::new(&mut buffer);
        img.write_to(&mut cursor, ImageFormat::Png).unwrap();

        // 转换为 JPEG
        let jpeg = convert_to_jpeg(&buffer, 85).unwrap();
        assert!(!jpeg.is_empty());

        // 验证是有效的 JPEG
        let _ = image::load_from_memory(&jpeg).unwrap();
    }
}
