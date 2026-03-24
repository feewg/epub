//! 文件操作工具模块
//!
//! 提供文件操作相关的辅助函数

use crate::error::{KafError, Result};
use std::fs;
use std::path::Path;

/// 读取文件内容
#[allow(dead_code)]
pub fn read_file(path: &Path) -> Result<String> {
    if !path.exists() {
        return Err(KafError::FileNotFound(
            path.to_string_lossy().to_string(),
        ));
    }

    fs::read_to_string(path)
        .map_err(|e| KafError::FileNotFound(format!("Failed to read {}: {}", path.display(), e)))
}

/// 读取文件为字节
#[allow(dead_code)]
pub fn read_file_bytes(path: &Path) -> Result<Vec<u8>> {
    if !path.exists() {
        return Err(KafError::FileNotFound(
            path.to_string_lossy().to_string(),
        ));
    }

    fs::read(path)
        .map_err(|e| KafError::FileNotFound(format!("Failed to read {}: {}", path.display(), e)))
}

/// 写入文件
#[allow(dead_code)]
pub fn write_file(path: &Path, content: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(path, content)?;
    Ok(())
}

/// 写入文件（字节）
#[allow(dead_code)]
pub fn write_file_bytes(path: &Path, content: &[u8]) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(path, content)?;
    Ok(())
}

/// 从文件名提取书名和作者
pub fn extract_bookname_from_filename(path: &Path) -> Result<(String, Option<String>)> {
    let filename = path
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| KafError::ParseError("Invalid filename".to_string()))?;

    // 去除前缀 @ 符号（知轩藏书格式）
    let filename = filename.trim_start_matches('@');

    // 知轩藏书格式: 《希灵帝国》（校对版全本）作者：远瞳
    let re1 = regex::Regex::new(r"《(.*?)》.*作者[：:](.*?)$").unwrap();
    if let Some(caps) = re1.captures(filename) {
        let bookname = caps.get(1).map(|m| m.as_str().to_string()).unwrap();
        let author = caps.get(2).map(|m| m.as_str().to_string());
        return Ok((bookname, author));
    }

    // 简单格式：书名
    Ok((filename.to_string(), None))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_extract_bookname_simple() {
        let path = PathBuf::from("测试小说.txt");
        let (bookname, author) = extract_bookname_from_filename(&path).unwrap();
        assert_eq!(bookname, "测试小说");
        assert!(author.is_none());
    }

    #[test]
    fn test_extract_bookname_with_author() {
        let path = PathBuf::from("《希灵帝国》（校对版全本）作者：远瞳.txt");
        let (bookname, author) = extract_bookname_from_filename(&path).unwrap();
        assert_eq!(bookname, "希灵帝国");
        assert_eq!(author, Some("远瞳".to_string()));
    }

    #[test]
    fn test_extract_bookname_with_prefix() {
        let path = PathBuf::from("soushu2024@《希灵帝国》作者：远瞳.txt");
        let (bookname, author) = extract_bookname_from_filename(&path).unwrap();
        assert_eq!(bookname, "希灵帝国");
        assert_eq!(author, Some("远瞳".to_string()));
    }
}
