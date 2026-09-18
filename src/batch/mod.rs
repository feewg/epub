//! 批量转换模块
//!
//! 处理批量转换任务

mod enhanced;
mod report;

pub use enhanced::{BatchConfig, BatchInput, EnhancedBatchConverter};
pub use report::{BatchReport, ReportFormat};

use crate::cli::Cli;
use crate::config::{load_config_tracked, AuthorSource};
use crate::error::Result;
use crate::model::Book;
use std::fs;
use std::path::{Path, PathBuf};
use tracing::warn;

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

/// 扫描结果：可用输入文件与扫描期问题。
///
/// 扫描期问题（如不可读子目录）不中断扫描，调用方可选择将其记录为
/// 逐项失败结果（见 [`FolderScanner::scan_with_cli_inputs`]）。
#[derive(Debug, Default)]
pub struct ScanOutcome {
    /// 支持的输入文件（已排序）
    pub files: Vec<PathBuf>,
    /// 扫描期问题：(路径, 错误消息)
    pub issues: Vec<(PathBuf, String)>,
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
    ///
    /// 单个子目录读取失败只记录警告并继续；根目录无效或不可读仍返回错误。
    pub fn scan(&self) -> Result<Vec<PathBuf>> {
        let outcome = self.scan_with_issues()?;
        for (path, message) in &outcome.issues {
            warn!(path = %path.display(), error = %message, "扫描目录时出错，已跳过该目录");
        }
        Ok(outcome.files)
    }

    /// 扫描文件夹并保留扫描期问题（如不可读子目录）。
    ///
    /// 与 [`FolderScanner::scan`] 不同，问题不丢失而是随文件一起返回，
    /// 供批量转换将其记录为逐项失败结果。
    pub fn scan_with_issues(&self) -> Result<ScanOutcome> {
        if !self.root.is_dir() {
            return Err(crate::error::KafError::ParseError(format!(
                "批量输入路径不是目录: {}",
                self.root.display()
            )));
        }
        let mut outcome = ScanOutcome::default();
        // 根目录读取失败视为整体错误，保持与旧 scan() 行为一致
        let entries = fs::read_dir(&self.root)?;
        self.scan_entries(entries, &self.root, &mut outcome, &|dir| fs::read_dir(dir));
        outcome.files.sort();
        Ok(outcome)
    }

    /// 遍历一层目录项；所有条目级错误记入 issues 并继续。
    ///
    /// 目录读取通过注入参数进行，便于外部快照副本在无法安全制造真实
    /// 权限错误的平台上验证容错路径；产品代码始终传入 [`fs::read_dir`]。
    fn scan_entries(
        &self,
        entries: fs::ReadDir,
        dir: &Path,
        outcome: &mut ScanOutcome,
        read_directory: &impl Fn(&Path) -> std::io::Result<fs::ReadDir>,
    ) {
        for entry in entries {
            let entry = match entry {
                Ok(entry) => entry,
                Err(error) => {
                    outcome.issues.push((dir.to_path_buf(), error.to_string()));
                    continue;
                }
            };
            let file_type = match entry.file_type() {
                Ok(file_type) => file_type,
                Err(error) => {
                    outcome.issues.push((entry.path(), error.to_string()));
                    continue;
                }
            };
            let path = entry.path();
            if file_type.is_symlink() {
                continue;
            }
            if file_type.is_dir() {
                if self.recursive {
                    match read_directory(&path) {
                        Ok(child) => self.scan_entries(child, &path, outcome, read_directory),
                        Err(error) => {
                            outcome.issues.push((path, error.to_string()));
                        }
                    }
                }
            } else if file_type.is_file()
                && Self::is_supported_input(&path)
                && !Self::is_own_generated_report(&path)
            {
                outcome.files.push(path);
            }
        }
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

    /// 判断路径是否为本工具生成的 Markdown 批量报告。
    ///
    /// 双重条件，避免误伤用户同名合法文件：
    /// 1. 文件名严格匹配本工具的生成命名 `batch_report_YYYYMMDD_HHMMSS[ (N)]`；
    /// 2. 文件首行字节等于 [`report::REPORT_MARKER`]（自生成证明）。
    ///
    /// 文件名命中但无法读取首行时按“非自生成”处理，交由后续转换阶段
    /// 产生可记录的错误，绝不静默排除用户文件。
    fn is_own_generated_report(path: &Path) -> bool {
        let stem_matches = path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .map(Self::matches_generated_report_stem)
            .unwrap_or(false);
        if !stem_matches {
            return false;
        }
        Self::first_line_is_report_marker(path)
    }

    /// 匹配 `batch_report_YYYYMMDD_HHMMSS` 或其 create_new 去重后缀
    /// `batch_report_YYYYMMDD_HHMMSS (N)` 形式的文件名主干。
    fn matches_generated_report_stem(stem: &str) -> bool {
        let Some(rest) = stem.strip_prefix("batch_report_") else {
            return false;
        };
        let bytes = rest.as_bytes();
        let timestamp_ok = bytes.len() >= 15
            && bytes[..8].iter().all(|byte| byte.is_ascii_digit())
            && bytes[8] == b'_'
            && bytes[9..15].iter().all(|byte| byte.is_ascii_digit());
        if !timestamp_ok {
            return false;
        }
        // 前 15 字节已验证为 ASCII，切片不会落在多字节字符边界上
        let tail = &rest[15..];
        if tail.is_empty() {
            return true;
        }
        let suffix_digits = tail
            .strip_prefix(" (")
            .and_then(|tail| tail.strip_suffix(')'))
            .unwrap_or("");
        !suffix_digits.is_empty() && suffix_digits.bytes().all(|byte| byte.is_ascii_digit())
    }

    /// 固定长度上限读取首行并比对报告标记；不把整份文件读入内存。
    fn first_line_is_report_marker(path: &Path) -> bool {
        use std::io::Read as _;
        let Ok(mut file) = fs::File::open(path) else {
            return false;
        };
        let mut buffer = vec![0u8; report::REPORT_MARKER.len() + 2];
        let read = match file.read(&mut buffer) {
            Ok(read) => read,
            Err(_) => return false,
        };
        buffer.truncate(read);
        String::from_utf8_lossy(&buffer)
            .lines()
            .next()
            .is_some_and(|line| line == report::REPORT_MARKER)
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
    ///
    /// 扫描期问题（如不可读子目录）同样转换为逐项失败结果，进入批量
    /// 报告而不是中断整个批次。
    pub fn scan_with_cli_inputs(&self, cli: &Cli) -> Result<Vec<BatchInput>> {
        let outcome = self.scan_with_issues()?;
        let mut inputs = Vec::with_capacity(outcome.files.len() + outcome.issues.len());
        for path in outcome.files {
            match self.create_book_from_cli(cli, &path) {
                Ok(book) => inputs.push(BatchInput::book(book)),
                Err(error) => inputs.push(BatchInput::failed(path, error.to_string())),
            }
        }
        for (path, message) in outcome.issues {
            warn!(path = %path.display(), error = %message, "扫描目录时出错");
            inputs.push(BatchInput::failed(path, format!("扫描失败: {message}")));
        }
        Ok(inputs)
    }

    fn create_book_from_cli(&self, cli: &Cli, path: &Path) -> Result<Book> {
        let mut file_cli = cli.clone();
        file_cli.filename = Some(path.to_path_buf());
        file_cli.batch = None;
        // 跟踪作者来源：显式 CLI/YAML（包括显式 "YSTYLE"）必须优先于
        // 《书名》作者：... 的文件名兜底，与单文件模式一致。
        let loaded = load_config_tracked(&file_cli)?;
        let mut book = loaded.book;
        self.apply_filename_metadata(&mut book, loaded.author_source)?;
        self.apply_resources(&mut book, path)?;
        Ok(book)
    }

    fn create_default_book_config(&self, file_path: &Path) -> Result<Book> {
        let mut book = Book {
            filename: file_path.to_path_buf(),
            ..Default::default()
        };
        self.apply_filename_metadata(&mut book, AuthorSource::Default)?;
        self.apply_resources(&mut book, file_path)?;
        Ok(book)
    }

    fn apply_filename_metadata(&self, book: &mut Book, author_source: AuthorSource) -> Result<()> {
        let (bookname, author) =
            crate::utils::file::extract_bookname_from_filename(&book.filename)?;
        if book.bookname.is_none() {
            book.bookname = Some(bookname);
        }
        // 仅当作者未被显式配置时才允许文件名作者兜底（见 config::apply_filename_author）
        crate::config::apply_filename_author(book, author_source, author);
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
