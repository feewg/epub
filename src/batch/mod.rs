//! 批量转换模块
//!
//! 处理批量转换任务

mod enhanced;
mod report;

pub use enhanced::{BatchConfig, BatchInput, EnhancedBatchConverter};
pub use report::{BatchReport, ReportFormat};

use crate::cli::Cli;
use crate::config::load_config;
use crate::error::Result;
use crate::model::Book;
use std::fs;
use std::path::{Path, PathBuf};

/// 批量转换结果
#[derive(Debug, Default)]
pub struct BatchResult {
    /// 成功转换的书籍
    pub success: Vec<PathBuf>,
    /// 失败的书籍
    pub failed: Vec<(PathBuf, String)>,
    /// 跳过的书籍及原因
    pub skipped: Vec<(PathBuf, String)>,
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

    /// 扫描文件夹，返回所有支持的输入文件。
    pub fn scan(&self) -> Result<Vec<PathBuf>> {
        if !self.root.is_dir() {
            return Err(crate::error::KafError::ParseError(format!(
                "批量输入路径不是目录: {}",
                self.root.display()
            )));
        }
        let mut files = Vec::new();
        self.scan_directory(&self.root, &mut files)?;
        files.sort();
        Ok(files)
    }

    fn scan_directory(&self, dir: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let file_type = entry.file_type()?;
            let path = entry.path();
            if file_type.is_symlink() {
                continue;
            }
            if file_type.is_dir() {
                if self.recursive {
                    self.scan_directory(&path, files)?;
                }
            } else if file_type.is_file() && Self::is_supported_input(&path) {
                files.push(path);
            }
        }
        Ok(())
    }

    fn is_supported_input(path: &Path) -> bool {
        path.extension()
            .and_then(|extension| extension.to_str())
            .map(|extension| {
                matches!(
                    extension.to_ascii_lowercase().as_str(),
                    "txt" | "md" | "markdown" | "mkd"
                )
            })
            .unwrap_or(false)
    }

    /// 使用默认配置扫描文件夹。
    pub fn scan_with_config(&self) -> Result<Vec<Book>> {
        self.scan()?
            .into_iter()
            .map(|path| self.create_default_book_config(&path))
            .collect()
    }

    /// 使用与单文件模式相同的 YAML/CLI 分层规则创建每本书的配置。
    pub fn scan_with_cli(&self, cli: &Cli) -> Result<Vec<Book>> {
        self.scan()?
            .into_iter()
            .map(|path| self.create_book_from_cli(cli, &path))
            .collect()
    }

    /// 扫描批量输入，并将配置错误保留为逐文件结果。
    pub fn scan_with_cli_inputs(&self, cli: &Cli) -> Result<Vec<BatchInput>> {
        Ok(self
            .scan()?
            .into_iter()
            .map(|path| match self.create_book_from_cli(cli, &path) {
                Ok(book) => BatchInput::book(book),
                Err(error) => BatchInput::failed(path, error.to_string()),
            })
            .collect())
    }

    fn create_book_from_cli(&self, cli: &Cli, path: &Path) -> Result<Book> {
        let mut file_cli = cli.clone();
        file_cli.filename = Some(path.to_path_buf());
        file_cli.batch = None;
        let mut book = load_config(&file_cli)?;
        self.apply_filename_metadata(&mut book)?;
        self.apply_resources(&mut book, path)?;
        Ok(book)
    }

    fn create_default_book_config(&self, file_path: &Path) -> Result<Book> {
        let mut book = Book {
            filename: file_path.to_path_buf(),
            ..Default::default()
        };
        self.apply_filename_metadata(&mut book)?;
        self.apply_resources(&mut book, file_path)?;
        Ok(book)
    }

    fn apply_filename_metadata(&self, book: &mut Book) -> Result<()> {
        let (bookname, author) =
            crate::utils::file::extract_bookname_from_filename(&book.filename)?;
        if book.bookname.is_none() {
            book.bookname = Some(bookname);
        }
        if book.author == "YSTYLE" {
            if let Some(author) = author {
                book.author = author;
            }
        }
        Ok(())
    }

    /// 应用资源（封面、CSS等）
    fn apply_resources(&self, book: &mut Book, file_path: &Path) -> Result<()> {
        let dir = file_path.parent().unwrap_or_else(|| Path::new("."));

        if book.cover.is_none() {
            for name in &[
                "cover.jpg",
                "cover.jpeg",
                "cover.png",
                "封面.jpg",
                "封面.png",
            ] {
                let cover_path = dir.join(name);
                if cover_path.is_file() {
                    book.cover = Some(crate::model::CoverSource::Local { path: cover_path });
                    break;
                }
            }
        }

        if book.chapter_header.image.is_none() && book.chapter_header.image_folder.is_none() {
            let header_folder = dir.join("headers");
            if header_folder.is_dir() {
                book.chapter_header.image_folder = Some(header_folder);
                book.chapter_header.mode = crate::model::HeaderMode::Folder;
            }
        }

        Ok(())
    }
}

/// 向后兼容的批量转换器。
pub struct BatchConverter {
    concurrency: usize,
}

impl BatchConverter {
    pub fn new(concurrency: usize) -> Self {
        Self { concurrency }
    }

    pub async fn convert(&self, books: Vec<Book>) -> BatchResult {
        let inputs = books
            .iter()
            .map(|book| book.filename.clone())
            .collect::<Vec<_>>();
        let converter = EnhancedBatchConverter::new(BatchConfig {
            continue_on_error: true,
            concurrency: self.concurrency,
            ..BatchConfig::default()
        });

        match converter.convert(books).await {
            Ok(report) => {
                let mut result = BatchResult {
                    elapsed_secs: report.summary.total_duration_secs,
                    ..BatchResult::default()
                };
                for file in report.files {
                    match file.status {
                        report::ConversionStatus::Success => {
                            if let Some(output) = file.output_file {
                                result.success.push(PathBuf::from(output));
                            }
                        }
                        report::ConversionStatus::Failed => result.failed.push((
                            PathBuf::from(file.input_file),
                            file.error_message.unwrap_or_else(|| "转换失败".to_string()),
                        )),
                        report::ConversionStatus::Skipped => result.skipped.push((
                            PathBuf::from(file.input_file),
                            file.error_message.unwrap_or_else(|| "跳过转换".to_string()),
                        )),
                    }
                }
                result
            }
            Err(error) => BatchResult {
                failed: inputs
                    .into_iter()
                    .map(|input| (input, error.to_string()))
                    .collect(),
                ..BatchResult::default()
            },
        }
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
        assert!(result.skipped.is_empty());
    }

    #[test]
    fn test_batch_converter_creation() {
        let converter = BatchConverter::new(4);
        assert_eq!(converter.concurrency, 4);
    }
}
