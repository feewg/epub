//! 批量转换模块
//!
//! 处理批量转换任务

mod enhanced;
mod report;

pub use enhanced::{BatchConfig, EnhancedBatchConverter};
pub use report::{BatchReport, ReportFormat};

use crate::converter::EpubConverter3;
use crate::error::Result;
use crate::model::Book;
use crate::parser::Parser;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Semaphore;
use tracing::{info, warn, error};

/// 批量转换结果
#[derive(Debug, Default)]
pub struct BatchResult {
    /// 成功转换的书籍
    pub success: Vec<PathBuf>,
    /// 失败的书籍
    pub failed: Vec<(PathBuf, String)>,
    /// 总耗时（秒）
    #[allow(dead_code)]
    pub elapsed_secs: f64,
}

/// 文件夹扫描器
pub struct FolderScanner {
    root: PathBuf,
    recursive: bool,
}

impl FolderScanner {
    /// 创建新的文件夹扫描器
    pub fn new(root: PathBuf, recursive: bool) -> Self {
        Self { root, recursive }
    }

    /// 扫描文件夹，返回所有 TXT 文件
    #[allow(dead_code)]
    pub fn scan(&self) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();
        self.scan_directory(&self.root, &mut files)?;
        Ok(files)
    }

    /// 扫描目录
    #[allow(dead_code)]
    fn scan_directory(&self, dir: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                if self.recursive {
                    self.scan_directory(&path, files)?;
                }
            } else if path.extension().is_some_and(|e| e == "txt") {
                files.push(path);
            }
        }
        Ok(())
    }

    /// 扫描文件夹并创建书籍配置
    pub fn scan_with_config(&self) -> Result<Vec<Book>> {
        let mut books = Vec::new();
        self.scan_directory_with_config(&self.root, &mut books)?;
        Ok(books)
    }

    /// 扫描目录并创建书籍配置
    fn scan_directory_with_config(&self, dir: &Path, books: &mut Vec<Book>) -> Result<()> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                if self.recursive {
                    self.scan_directory_with_config(&path, books)?;
                }
            } else if path.extension().is_some_and(|e| e == "txt") {
                if let Some(book) = self.create_book_config(&path)? {
                    books.push(book);
                }
            }
        }
        Ok(())
    }

    /// 为单个文件创建书籍配置
    fn create_book_config(&self, file_path: &Path) -> Result<Option<Book>> {
        let (bookname, author) = crate::utils::file::extract_bookname_from_filename(file_path)?;

        let mut book = Book {
            filename: file_path.to_path_buf(),
            bookname: Some(bookname),
            author: author.unwrap_or_else(|| "YSTYLE".to_string()),
            ..Default::default()
        };

        // 查找封面
        self.apply_resources(&mut book, file_path)?;

        Ok(Some(book))
    }

    /// 应用资源（封面、CSS等）
    fn apply_resources(&self, book: &mut Book, file_path: &Path) -> Result<()> {
        let dir = file_path.parent().unwrap_or_else(|| Path::new("."));

        // 查找封面
        for name in &["cover.jpg", "cover.png", "封面.jpg", "封面.png"] {
            let cover_path = dir.join(name);
            if cover_path.exists() {
                book.cover = Some(crate::model::CoverSource::Local {
                    path: cover_path,
                });
                break;
            }
        }

        // 查找页眉图片文件夹
        let header_folder = dir.join("headers");
        if header_folder.exists() && header_folder.is_dir() {
            book.chapter_header.image_folder = Some(header_folder);
            book.chapter_header.mode = crate::model::HeaderMode::Folder;
        }

        Ok(())
    }
}

/// 批量转换器
#[allow(dead_code)]
pub struct BatchConverter {
    /// 并发数
    concurrency: usize,
}

#[allow(dead_code)]
impl BatchConverter {
    /// 创建新的批量转换器
    pub fn new(concurrency: usize) -> Self {
        Self { concurrency }
    }

    /// 执行批量转换
    #[allow(dead_code)]
    pub async fn convert(&self, books: Vec<Book>) -> BatchResult {
        let start = std::time::Instant::now();
        let semaphore = Arc::new(Semaphore::new(self.concurrency));
        let mut tasks = Vec::new();

        for book in books {
            let _permit = semaphore.clone().acquire_owned().await.ok();
            let task = tokio::spawn(async move {
                let filename = book.filename.clone();
                let bookname = book.bookname.clone().unwrap_or_default();

                info!("开始转换: {}", bookname);

                match Self::convert_single(book).await {
                    Ok(output_path) => {
                        info!("转换成功: {} -> {:?}", bookname, output_path);
                        Ok((filename, output_path))
                    }
                    Err(e) => {
                        error!("转换失败: {} - {}", bookname, e);
                        Err((filename, e.to_string()))
                    }
                }
            });
            tasks.push(task);
        }

        let mut result = BatchResult {
            elapsed_secs: start.elapsed().as_secs_f64(),
            ..Default::default()
        };

        for task in tasks {
            match task.await {
                Ok(Ok((input, output))) => {
                    info!("任务完成: {:?} -> {:?}", input, output);
                    result.success.push(output);
                }
                Ok(Err((input, err))) => {
                    warn!("任务失败: {:?} - {}", input, err);
                    result.failed.push((input, err));
                }
                Err(e) => {
                    warn!("任务panic: {}", e);
                    result.failed.push((PathBuf::from("unknown"), format!("Task panicked: {}", e)));
                }
            }
        }

        result
    }

    /// 转换单个书籍
    #[allow(dead_code)]
    async fn convert_single(book: Book) -> Result<PathBuf> {
        let filename = book.filename.clone();
        let output_name = book.output_name.clone().unwrap_or_else(|| {
            book.bookname.clone().unwrap_or_else(|| "output".to_string())
        });
        let output_path = filename.parent()
            .unwrap_or_else(|| Path::new("."))
            .join(format!("{}.epub", output_name));

        // 解析文件（同步操作在阻塞线程中执行）
        let sections = tokio::task::spawn_blocking(move || {
            let mut parser = Parser::new(book);
            parser.parse()
        }).await.map_err(|e| {
            crate::error::KafError::Unknown(format!("Task join error: {}", e))
        })??;

        // 生成 EPUB
        let converter = EpubConverter3::new(Book {
            filename: filename.clone(),
            ..Default::default()
        });
        let epub_data = converter.generate(&sections).await?;

        // 写入文件
        tokio::fs::write(&output_path, epub_data).await?;

        Ok(output_path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_folder_scanner_creation() {
        let scanner = FolderScanner::new(PathBuf::from("/tmp"), true);
        assert_eq!(scanner.root, PathBuf::from("/tmp"));
        assert!(scanner.recursive);
    }

    #[test]
    fn test_batch_result_default() {
        let result = BatchResult::default();
        assert!(result.success.is_empty());
        assert!(result.failed.is_empty());
    }

    #[test]
    fn test_batch_converter_creation() {
        let converter = BatchConverter::new(4);
        assert_eq!(converter.concurrency, 4);
    }
}
