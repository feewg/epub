//! 封面处理模块
//!
//! 处理封面图片的获取、验证、缩放和格式转换

use crate::error::{KafError, Result};
use crate::model::CoverSource;
use image::ImageFormat;
use std::path::Path;

/// 封面输出格式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum CoverOutputFormat {
    /// JPEG 格式
    Jpeg,
    /// PNG 格式
    Png,
    /// 自动保持原格式
    Auto,
}

/// 封面配置
#[derive(Debug, Clone)]
pub struct CoverConfig {
    /// 最大宽度（像素），默认 1200
    pub max_width: u32,
    /// 最大高度（像素），默认 1600
    pub max_height: u32,
    /// JPEG 质量 1-100，默认 85
    pub quality: u8,
    /// 输出格式
    pub output_format: CoverOutputFormat,
}

impl Default for CoverConfig {
    fn default() -> Self {
        Self {
            max_width: 1200,
            max_height: 1600,
            quality: 85,
            output_format: CoverOutputFormat::Auto,
        }
    }
}

/// 支持的图片扩展名列表
const IMAGE_EXTENSIONS: &[&str] = &["jpg", "jpeg", "png", "gif", "webp", "bmp", "tiff", "tif", "avif"];

/// 从封面源获取封面图片数据
#[allow(dead_code)]
pub async fn fetch_cover(cover: &CoverSource) -> Result<Vec<u8>> {
    match cover {
        CoverSource::Local { path } => fetch_local_cover(path),
        CoverSource::Data { data, .. } => Ok(data.clone()),
    }
}

/// 读取本地封面图片
#[allow(dead_code)]
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
#[allow(dead_code)]
pub fn convert_to_jpeg(data: &[u8]) -> Result<Vec<u8>> {
    let img = image::load_from_memory(data)?;

    let mut buffer = Vec::new();
    let mut cursor = std::io::Cursor::new(&mut buffer);

    img.write_to(&mut cursor, ImageFormat::Jpeg)?;

    Ok(buffer)
}

/// 通过文件扩展名检测图片格式
///
/// 支持常见图片格式的扩展名检测
#[allow(dead_code)]
fn detect_format_by_extension(path: &Path) -> Option<ImageFormat> {
    let ext = path.extension()?.to_str()?.to_lowercase();
    match ext.as_str() {
        "jpg" | "jpeg" => Some(ImageFormat::Jpeg),
        "png" => Some(ImageFormat::Png),
        "gif" => Some(ImageFormat::Gif),
        "webp" => Some(ImageFormat::WebP),
        "bmp" => Some(ImageFormat::Bmp),
        "tiff" | "tif" => Some(ImageFormat::Tiff),
        "avif" => Some(ImageFormat::Avif),
        _ => None,
    }
}

/// 通过文件头（magic bytes）检测图片格式
///
/// 用于在无扩展名或扩展名不可靠时判断图片类型
pub fn detect_image_format(data: &[u8]) -> Result<ImageFormat> {
    // PNG: 89 50 4E 47 0D 0A 1A 0A
    if data.len() >= 8 && data[..8] == [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A] {
        return Ok(ImageFormat::Png);
    }
    // JPEG: FF D8 FF
    if data.len() >= 3 && data[..3] == [0xFF, 0xD8, 0xFF] {
        return Ok(ImageFormat::Jpeg);
    }
    // GIF: 47 49 46 38
    if data.len() >= 4 && data[..4] == [0x47, 0x49, 0x46, 0x38] {
        return Ok(ImageFormat::Gif);
    }
    // WebP: RIFF ... WEBP
    if data.len() >= 12 && &data[..4] == b"RIFF" && &data[8..12] == b"WEBP" {
        return Ok(ImageFormat::WebP);
    }
    // BMP: 42 4D
    if data.len() >= 2 && data[..2] == [0x42, 0x4D] {
        return Ok(ImageFormat::Bmp);
    }
    // AVIF: ftyp at offset 4
    if data.len() >= 12 && &data[4..8] == b"ftyp" {
        let brand = &data[8..12];
        if brand == b"avif" || brand == b"avis" {
            return Ok(ImageFormat::Avif);
        }
    }

    // 如果 magic bytes 无法识别，尝试通过 image crate 解码
    image::ImageReader::new(std::io::Cursor::new(data))
        .with_guessed_format()
        .map_err(|e| KafError::CoverError(format!("无法检测图片格式: {}", e)))?
        .format()
        .ok_or_else(|| KafError::CoverError("无法识别的图片格式".to_string()))
}

/// 获取图片尺寸（宽度和高度），无需完整解码
///
/// 利用 image crate 的 Reader 只读取元数据部分
pub fn get_image_dimensions(data: &[u8]) -> Result<(u32, u32)> {
    let reader = image::ImageReader::new(std::io::Cursor::new(data))
        .with_guessed_format()
        .map_err(|e| KafError::CoverError(format!("无法读取图片信息: {}", e)))?;

    let dimensions = reader
        .into_dimensions()
        .map_err(|e| KafError::CoverError(format!("无法获取图片尺寸: {}", e)))?;

    Ok(dimensions)
}

/// 将图片缩放到指定最大尺寸内，保持原始宽高比
///
/// 如果原图小于最大尺寸，则不进行缩放
pub fn resize_cover(data: &[u8], config: &CoverConfig) -> Result<Vec<u8>> {
    let (width, height) = get_image_dimensions(data)?;

    tracing::info!("封面原始尺寸: {}x{}", width, height);

    // 检查是否需要缩放
    if width <= config.max_width && height <= config.max_height {
        tracing::info!("封面尺寸在限制范围内，无需缩放");
        return Ok(data.to_vec());
    }

    // 计算等比缩放比例
    let scale = if width > config.max_width && height > config.max_height {
        // 宽高都超限，取较小比例
        (config.max_width as f64 / width as f64)
            .min(config.max_height as f64 / height as f64)
    } else if width > config.max_width {
        config.max_width as f64 / width as f64
    } else {
        config.max_height as f64 / height as f64
    };

    let new_width = (width as f64 * scale).round() as u32;
    let new_height = (height as f64 * scale).round() as u32;

    tracing::info!("封面缩放至: {}x{} (比例: {:.2}%)", new_width, new_height, scale * 100.0);

    let img = image::load_from_memory(data)?;
    let resized = img.resize(
        new_width,
        new_height,
        image::imageops::FilterType::Lanczos3,
    );

    // 编码输出
    let output_format = match config.output_format {
        CoverOutputFormat::Jpeg => ImageFormat::Jpeg,
        CoverOutputFormat::Png => ImageFormat::Png,
        CoverOutputFormat::Auto => {
            // 自动选择：优先保持原格式，如果原格式不支持则转 JPEG
            match detect_image_format(data)? {
                ImageFormat::Jpeg => ImageFormat::Jpeg,
                ImageFormat::Png => ImageFormat::Png,
                _ => ImageFormat::Jpeg,
            }
        }
    };

    let mut buffer = Vec::new();
    let mut cursor = std::io::Cursor::new(&mut buffer);

    if output_format == ImageFormat::Jpeg {
        // JPEG 编码需要指定质量
        let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut cursor, config.quality);
        resized.write_with_encoder(encoder)?;
    } else {
        resized.write_to(&mut cursor, output_format)?;
    }

    Ok(buffer)
}

/// 优化封面图片（主入口函数）
///
/// 自动检测格式、按需缩放、转换格式。返回 (图片数据, MIME 类型)
pub fn optimize_cover(data: &[u8], config: &CoverConfig) -> Result<(Vec<u8>, String)> {
    let format = detect_image_format(data)?;
    let (width, height) = get_image_dimensions(data)?;

    tracing::info!(
        "封面优化 - 原始格式: {:?}, 尺寸: {}x{}, 大小: {} bytes",
        format, width, height, data.len()
    );

    // 缩放（仅在需要时）
    let processed = if width > config.max_width || height > config.max_height {
        resize_cover(data, config)?
    } else {
        data.to_vec()
    };

    // 如果输出格式不是 Auto 且与原始格式不同，需要转换
    let (output_data, mime_type) = match config.output_format {
        CoverOutputFormat::Jpeg => {
            if format == ImageFormat::Jpeg {
                (processed, "image/jpeg".to_string())
            } else {
                let mut buffer = Vec::new();
                let mut cursor = std::io::Cursor::new(&mut buffer);
                let img = image::load_from_memory(&processed)?;
                let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut cursor, config.quality);
                img.write_with_encoder(encoder)?;
                (buffer, "image/jpeg".to_string())
            }
        }
        CoverOutputFormat::Png => {
            if format == ImageFormat::Png {
                (processed, "image/png".to_string())
            } else {
                let mut buffer = Vec::new();
                let mut cursor = std::io::Cursor::new(&mut buffer);
                let img = image::load_from_memory(&processed)?;
                img.write_to(&mut cursor, ImageFormat::Png)?;
                (buffer, "image/png".to_string())
            }
        }
        CoverOutputFormat::Auto => {
            let mime = format_to_mime(&format);
            (processed, mime)
        }
    };

    tracing::info!(
        "封面优化完成 - 输出格式: {}, 大小: {} bytes",
        mime_type, output_data.len()
    );

    Ok((output_data, mime_type))
}

/// 将 ImageFormat 转换为 MIME 类型字符串
pub fn format_to_mime(format: &ImageFormat) -> String {
    match format {
        ImageFormat::Jpeg => "image/jpeg".to_string(),
        ImageFormat::Png => "image/png".to_string(),
        ImageFormat::Gif => "image/gif".to_string(),
        ImageFormat::WebP => "image/webp".to_string(),
        ImageFormat::Bmp => "image/bmp".to_string(),
        ImageFormat::Tiff => "image/tiff".to_string(),
        ImageFormat::Avif => "image/avif".to_string(),
        _ => "image/jpeg".to_string(),
    }
}

/// 验证路径是否为图片文件
///
/// 通过扩展名和 magic bytes 双重检测
pub fn is_image_file(path: &Path) -> bool {
    // 首先检查扩展名
    let ext_match = path.extension()
        .and_then(|e| e.to_str())
        .map(|e| IMAGE_EXTENSIONS.contains(&e.to_lowercase().as_str()))
        .unwrap_or(false);

    if !ext_match {
        return false;
    }

    // 然后检查 magic bytes（如果文件存在）
    if path.exists() {
        if let Ok(data) = std::fs::read(path) {
            return detect_image_format(&data).is_ok();
        }
    }

    // 文件不存在但扩展名匹配，仍然返回 true（可能是尚未生成的文件）
    ext_match
}

/// 验证并解析图片路径
///
/// - 如果是相对路径，则根据 base_dir（输入文件所在目录）进行解析
/// - 检查文件是否存在
/// - 验证是否为有效图片
pub fn validate_image_path(path: &Path, base_dir: Option<&Path>) -> Result<std::path::PathBuf> {
    let resolved = if path.is_absolute() {
        path.to_path_buf()
    } else if let Some(base) = base_dir {
        base.join(path)
    } else {
        path.to_path_buf()
    };

    // 检查文件是否存在
    if !resolved.exists() {
        return Err(KafError::CoverError(format!(
            "封面图片不存在: {}",
            resolved.display()
        )));
    }

    // 检查是否为图片
    if !is_image_file(&resolved) {
        return Err(KafError::CoverError(format!(
            "文件不是有效的图片: {}",
            resolved.display()
        )));
    }

    Ok(resolved)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 创建一个测试用 PNG 图片
    fn create_test_png(width: u32, height: u32) -> Vec<u8> {
        let img = image::RgbImage::new(width, height);
        let mut buffer = Vec::new();
        let mut cursor = std::io::Cursor::new(&mut buffer);
        img.write_to(&mut cursor, ImageFormat::Png).unwrap();
        buffer
    }

    /// 创建一个测试用 JPEG 图片
    fn create_test_jpeg(width: u32, height: u32) -> Vec<u8> {
        let img = image::RgbImage::new(width, height);
        let mut buffer = Vec::new();
        let mut cursor = std::io::Cursor::new(&mut buffer);
        img.write_to(&mut cursor, ImageFormat::Jpeg).unwrap();
        buffer
    }

    #[test]
    fn test_convert_to_jpeg() {
        let png = create_test_png(100, 100);
        let jpeg = convert_to_jpeg(&png).unwrap();
        assert!(!jpeg.is_empty());
        // 验证是有效的 JPEG
        assert!(jpeg.starts_with(&[0xFF, 0xD8, 0xFF]));
    }

    #[test]
    fn test_detect_image_format_png() {
        let png = create_test_png(10, 10);
        assert_eq!(detect_image_format(&png).unwrap(), ImageFormat::Png);
    }

    #[test]
    fn test_detect_image_format_jpeg() {
        let jpeg = create_test_jpeg(10, 10);
        assert_eq!(detect_image_format(&jpeg).unwrap(), ImageFormat::Jpeg);
    }

    #[test]
    fn test_detect_image_format_invalid() {
        let garbage = vec![0x00, 0x01, 0x02, 0x03];
        assert!(detect_image_format(&garbage).is_err());
    }

    #[test]
    fn test_detect_image_format_empty() {
        assert!(detect_image_format(&[]).is_err());
    }

    #[test]
    fn test_get_image_dimensions() {
        let png = create_test_png(200, 300);
        let (w, h) = get_image_dimensions(&png).unwrap();
        assert_eq!(w, 200);
        assert_eq!(h, 300);
    }

    #[test]
    fn test_get_image_dimensions_invalid() {
        assert!(get_image_dimensions(&[0x00, 0x01]).is_err());
    }

    #[test]
    fn test_resize_cover_no_resize_needed() {
        let small_png = create_test_png(100, 100);
        let config = CoverConfig::default();
        let result = resize_cover(&small_png, &config).unwrap();
        // 小于最大尺寸，不缩放，直接返回原数据
        assert_eq!(result.len(), small_png.len());
    }

    #[test]
    fn test_resize_cover_downscale() {
        // 创建一张大图（超过默认最大尺寸）
        let large_png = create_test_png(2000, 3000);
        let config = CoverConfig::default();
        let result = resize_cover(&large_png, &config).unwrap();

        let (w, h) = get_image_dimensions(&result).unwrap();
        assert!(w <= config.max_width);
        assert!(h <= config.max_height);
    }

    #[test]
    fn test_resize_cover_maintains_aspect_ratio() {
        // 16:9 的图片
        let img = create_test_png(1600, 900);
        let config = CoverConfig {
            max_width: 800,
            max_height: 800,
            ..Default::default()
        };
        let result = resize_cover(&img, &config).unwrap();

        let (w, h) = get_image_dimensions(&result).unwrap();
        let ratio = w as f64 / h as f64;
        let original_ratio = 1600.0 / 900.0;
        // 宽高比应保持不变（允许浮点误差）
        assert!((ratio - original_ratio).abs() < 0.01);
    }

    #[test]
    fn test_optimize_cover_auto_format() {
        let png = create_test_png(100, 100);
        let config = CoverConfig::default();
        let (data, mime) = optimize_cover(&png, &config).unwrap();

        assert_eq!(mime, "image/png");
        assert!(!data.is_empty());
    }

    #[test]
    fn test_optimize_cover_force_jpeg() {
        let png = create_test_png(100, 100);
        let config = CoverConfig {
            output_format: CoverOutputFormat::Jpeg,
            quality: 90,
            ..Default::default()
        };
        let (data, mime) = optimize_cover(&png, &config).unwrap();

        assert_eq!(mime, "image/jpeg");
        assert!(data.starts_with(&[0xFF, 0xD8, 0xFF]));
    }

    #[test]
    fn test_optimize_cover_resize_and_format() {
        // 大 PNG -> 缩放 + 转 JPEG
        let large_png = create_test_png(2000, 3000);
        let config = CoverConfig {
            output_format: CoverOutputFormat::Jpeg,
            quality: 80,
            ..Default::default()
        };
        let (data, mime) = optimize_cover(&large_png, &config).unwrap();

        assert_eq!(mime, "image/jpeg");
        let (w, h) = get_image_dimensions(&data).unwrap();
        assert!(w <= 1200);
        assert!(h <= 1600);
    }

    #[test]
    fn test_format_to_mime() {
        assert_eq!(format_to_mime(&ImageFormat::Jpeg), "image/jpeg");
        assert_eq!(format_to_mime(&ImageFormat::Png), "image/png");
        assert_eq!(format_to_mime(&ImageFormat::Gif), "image/gif");
    }

    #[test]
    fn test_validate_image_path_absolute() {
        // 使用 tempdir 创建临时图片
        let dir = std::env::temp_dir().join("kaf_test_cover");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("test.png");
        let png = create_test_png(50, 50);
        std::fs::write(&path, &png).unwrap();

        let result = validate_image_path(&path, None).unwrap();
        assert_eq!(result, path);

        // 清理
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_validate_image_path_relative() {
        let dir = std::env::temp_dir().join("kaf_test_cover_rel");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("cover.png");
        let png = create_test_png(50, 50);
        std::fs::write(&path, &png).unwrap();

        // 相对路径 + base_dir
        let relative = Path::new("cover.png");
        let result = validate_image_path(relative, Some(&dir)).unwrap();
        assert_eq!(result, path);

        // 清理
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_validate_image_path_not_found() {
        let result = validate_image_path(Path::new("nonexistent.png"), None);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("封面图片不存在"));
    }

    #[test]
    fn test_validate_image_path_not_image() {
        let dir = std::env::temp_dir().join("kaf_test_cover_ni");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("not_image.png");
        std::fs::write(&path, b"not an image").unwrap();

        let result = validate_image_path(&path, None);
        assert!(result.is_err());

        // 清理
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_is_image_file_by_extension() {
        assert!(is_image_file(Path::new("photo.jpg")));
        assert!(is_image_file(Path::new("photo.jpeg")));
        assert!(is_image_file(Path::new("photo.png")));
        assert!(is_image_file(Path::new("photo.gif")));
        assert!(is_image_file(Path::new("photo.webp")));
        assert!(!is_image_file(Path::new("document.pdf")));
        assert!(!is_image_file(Path::new("script.js")));
        assert!(!is_image_file(Path::new("no_extension")));
    }

    #[test]
    fn test_is_image_file_with_magic_bytes() {
        let dir = std::env::temp_dir().join("kaf_test_is_image");
        std::fs::create_dir_all(&dir).unwrap();

        // 有效的 PNG 文件
        let png = create_test_png(10, 10);
        let png_path = dir.join("real.png");
        std::fs::write(&png_path, &png).unwrap();
        assert!(is_image_file(&png_path));

        // 假 PNG（扩展名对但内容不对）
        let fake_path = dir.join("fake.png");
        std::fs::write(&fake_path, b"not really a png").unwrap();
        assert!(!is_image_file(&fake_path));

        // 清理
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_cover_config_default() {
        let config = CoverConfig::default();
        assert_eq!(config.max_width, 1200);
        assert_eq!(config.max_height, 1600);
        assert_eq!(config.quality, 85);
        assert_eq!(config.output_format, CoverOutputFormat::Auto);
    }

    #[test]
    fn test_detect_format_by_extension() {
        assert_eq!(
            detect_format_by_extension(Path::new("photo.jpg")),
            Some(ImageFormat::Jpeg)
        );
        assert_eq!(
            detect_format_by_extension(Path::new("photo.JPEG")),
            Some(ImageFormat::Jpeg)
        );
        assert_eq!(
            detect_format_by_extension(Path::new("photo.png")),
            Some(ImageFormat::Png)
        );
        assert_eq!(
            detect_format_by_extension(Path::new("photo.webp")),
            Some(ImageFormat::WebP)
        );
        assert_eq!(detect_format_by_extension(Path::new("photo.txt")), None);
        assert_eq!(detect_format_by_extension(Path::new("photo")), None);
    }

    #[test]
    fn test_fetch_cover_data_variant() {
        let png = create_test_png(10, 10);
        let source = CoverSource::Data {
            data: png.clone(),
            format: "image/png".to_string(),
        };
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(fetch_cover(&source)).unwrap();
        assert_eq!(result, png);
    }
}
