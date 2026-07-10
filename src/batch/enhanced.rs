//! 增强批量转换

use super::report::{
    BatchReport, ConversionStatus, ConversionSummary, ErrorDetail, FileConversionResult,
    ReportFormat, ReportGenerator,
};
use crate::converter::EpubConverter3;
use crate::error::{KafError, Result};
use crate::model::Book;
use crate::parser::Parser;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Instant;
use tracing::{error, info, warn};

/// 批量转换配置
#[derive(Debug, Clone)]
pub struct BatchConfig {
    pub output_dir: Option<PathBuf>,
    pub continue_on_error: bool,
    /// 最大错误数量（0 表示无限制）
    pub max_errors: usize,
    pub dry_run: bool,
    pub show_chapters: bool,
    pub concurrency: usize,
}

impl Default for BatchConfig {
    fn default() -> Self {
        Self {
            output_dir: None,
            continue_on_error: false,
            max_errors: 0,
            dry_run: false,
            show_chapters: false,
            concurrency: 4,
        }
    }
}

/// 单个批量输入及其配置加载结果。
#[derive(Debug)]
pub enum BatchInput {
    Book(Box<Book>),
    Failed { input: PathBuf, error: String },
}

impl BatchInput {
    pub fn book(book: Book) -> Self {
        Self::Book(Box::new(book))
    }

    pub fn failed(input: PathBuf, error: impl Into<String>) -> Self {
        Self::Failed {
            input,
            error: error.into(),
        }
    }
}

enum PlannedJob {
    Ready {
        book: Box<Book>,
        output_path: PathBuf,
    },
    Failed {
        input: PathBuf,
        error: String,
        error_type: &'static str,
    },
}

impl PlannedJob {
    fn input(&self) -> &Path {
        match self {
            Self::Ready { book, .. } => &book.filename,
            Self::Failed { input, .. } => input,
        }
    }
}

pub struct EnhancedBatchConverter {
    config: BatchConfig,
}

impl EnhancedBatchConverter {
    pub fn new(config: BatchConfig) -> Self {
        Self { config }
    }

    /// 执行批量转换。
    pub async fn convert(&self, books: Vec<Book>) -> Result<BatchReport> {
        let inputs = books.into_iter().map(BatchInput::book).collect();
        self.convert_inputs(inputs).await
    }

    /// 转换包含逐文件配置错误的批量输入。
    pub async fn convert_inputs(&self, inputs: Vec<BatchInput>) -> Result<BatchReport> {
        if self.config.concurrency == 0 {
            return Err(KafError::ParseError("并发数必须大于 0".to_string()));
        }
        if !self.config.dry_run {
            if let Some(output_dir) = &self.config.output_dir {
                fs::create_dir_all(output_dir)?;
                if !output_dir.is_dir() {
                    return Err(KafError::ParseError(format!(
                        "输出路径不是目录: {}",
                        output_dir.display()
                    )));
                }
            }
        }

        let started = Instant::now();
        let jobs = Self::plan_jobs(inputs, &self.config);
        let mut report = BatchReport::default();
        let mut error_types = HashMap::new();
        let mut error_count = 0usize;
        let mut next_job = 0usize;
        let mut stopped = false;

        while next_job < jobs.len() && !stopped {
            if let PlannedJob::Failed {
                input,
                error,
                error_type,
            } = &jobs[next_job]
            {
                error!(path = %input.display(), error = %error, "批量预处理失败");
                error_types.insert(input.display().to_string(), *error_type);
                report.files.push(Self::failed_result(input, error.clone()));
                error_count += 1;
                next_job += 1;
                stopped = !self.config.continue_on_error
                    || (self.config.max_errors > 0 && error_count >= self.config.max_errors);
                continue;
            }

            let mut tasks = Vec::with_capacity(self.config.concurrency);
            while next_job < jobs.len() && tasks.len() < self.config.concurrency {
                let PlannedJob::Ready { book, output_path } = &jobs[next_job] else {
                    break;
                };
                let book = book.clone();
                let output_path = output_path.clone();
                let config = self.config.clone();
                let input = book.filename.clone();
                let handle =
                    tokio::spawn(
                        async move { Self::process_book(&book, &config, &output_path).await },
                    );
                tasks.push((input, handle));
                next_job += 1;
            }

            let failures_before = error_count;
            for (input, task) in tasks {
                let result = match task.await {
                    Ok(Ok(result)) => result,
                    Ok(Err(error)) => {
                        error!(path = %input.display(), %error, "批量转换失败");
                        Self::failed_result(&input, error.to_string())
                    }
                    Err(error) => {
                        error!(path = %input.display(), %error, "批量任务异常退出");
                        Self::failed_result(&input, format!("任务执行错误: {error}"))
                    }
                };
                if result.status == ConversionStatus::Failed {
                    error_count += 1;
                }
                report.files.push(result);
            }

            let batch_failed = error_count > failures_before;
            stopped = (batch_failed && !self.config.continue_on_error)
                || (self.config.max_errors > 0 && error_count >= self.config.max_errors);
        }

        if stopped && next_job < jobs.len() {
            warn!(
                remaining = jobs.len() - next_job,
                "达到停止条件，取消剩余文件"
            );
            for job in &jobs[next_job..] {
                let input = job.input();
                report.files.push(FileConversionResult {
                    input_file: input.display().to_string(),
                    output_file: None,
                    status: ConversionStatus::Skipped,
                    duration_secs: 0.0,
                    chapter_count: None,
                    file_size_bytes: fs::metadata(input)
                        .map(|metadata| metadata.len())
                        .unwrap_or(0),
                    error_message: Some("达到批量停止条件，未执行转换".to_string()),
                });
            }
        }

        report.timestamp = chrono::Utc::now().to_rfc3339();
        Self::update_summary(&mut report, started.elapsed().as_secs_f64(), &error_types);
        Ok(report)
    }

    fn plan_jobs(inputs: Vec<BatchInput>, config: &BatchConfig) -> Vec<PlannedJob> {
        let mut reserved = HashSet::new();
        inputs
            .into_iter()
            .map(|input| match input {
                BatchInput::Failed { input, error } => PlannedJob::Failed {
                    input,
                    error,
                    error_type: "ConfigurationError",
                },
                BatchInput::Book(book) => {
                    if let Err(error) = crate::config::validate_config(&book) {
                        PlannedJob::Failed {
                            input: book.filename.clone(),
                            error: error.to_string(),
                            error_type: "ConfigurationError",
                        }
                    } else if config.dry_run {
                        PlannedJob::Ready {
                            book,
                            output_path: PathBuf::new(),
                        }
                    } else {
                        match Self::reserve_output_path(&book, config, &mut reserved) {
                            Ok(output_path) => PlannedJob::Ready { book, output_path },
                            Err(error) => PlannedJob::Failed {
                                input: book.filename.clone(),
                                error: error.to_string(),
                                error_type: "ConversionError",
                            },
                        }
                    }
                }
            })
            .collect()
    }

    fn reserve_output_path(
        book: &Book,
        config: &BatchConfig,
        reserved: &mut HashSet<String>,
    ) -> Result<PathBuf> {
        let output_dir = config.output_dir.clone().unwrap_or_else(|| {
            book.filename
                .parent()
                .unwrap_or_else(|| Path::new("."))
                .to_path_buf()
        });
        let requested = book
            .output_name
            .as_deref()
            .or(book.bookname.as_deref())
            .unwrap_or("output");
        let basename = Self::sanitize_output_name(requested);

        for suffix in 0..=10_000usize {
            let filename = if suffix == 0 {
                format!("{basename}.epub")
            } else {
                format!("{basename} ({suffix}).epub")
            };
            let candidate = output_dir.join(filename);
            let key = Self::path_key(&candidate);
            if !candidate.exists() && reserved.insert(key) {
                return Ok(candidate);
            }
        }
        Err(KafError::ParseError(format!(
            "无法为 {} 分配唯一输出文件名",
            book.filename.display()
        )))
    }

    fn sanitize_output_name(value: &str) -> String {
        let sanitized = value
            .trim()
            .chars()
            .map(|character| {
                if character.is_control()
                    || matches!(
                        character,
                        '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*'
                    )
                {
                    '_'
                } else {
                    character
                }
            })
            .collect::<String>();
        let sanitized = sanitized.trim_matches([' ', '.']);
        if sanitized.is_empty() || matches!(sanitized, "." | "..") {
            "output".to_string()
        } else {
            sanitized.to_string()
        }
    }

    fn path_key(path: &Path) -> String {
        let value = path.to_string_lossy().into_owned();
        if cfg!(windows) {
            value.to_lowercase()
        } else {
            value
        }
    }

    async fn process_book(
        book: &Book,
        config: &BatchConfig,
        output_path: &Path,
    ) -> Result<FileConversionResult> {
        let started = Instant::now();
        crate::config::validate_config(book)?;
        let file_size = fs::metadata(&book.filename)?.len();
        if file_size == 0 {
            return Ok(FileConversionResult {
                input_file: book.filename.display().to_string(),
                output_file: None,
                status: ConversionStatus::Skipped,
                duration_secs: started.elapsed().as_secs_f64(),
                chapter_count: Some(0),
                file_size_bytes: 0,
                error_message: Some("文件为空，跳过处理".to_string()),
            });
        }

        let sections = Self::parse_book(book.clone()).await?;
        if config.show_chapters {
            Self::print_chapters(book, &sections);
        }

        if config.dry_run {
            return Ok(FileConversionResult {
                input_file: book.filename.display().to_string(),
                output_file: None,
                status: ConversionStatus::Skipped,
                duration_secs: started.elapsed().as_secs_f64(),
                chapter_count: Some(sections.len()),
                file_size_bytes: file_size,
                error_message: None,
            });
        }

        let converter = EpubConverter3::new(book.clone());
        let epub_data = converter.generate(&sections).await?;
        Self::write_new_file(output_path.to_path_buf(), epub_data).await?;
        info!(input = %book.filename.display(), output = %output_path.display(), "转换完成");

        Ok(FileConversionResult {
            input_file: book.filename.display().to_string(),
            output_file: Some(output_path.display().to_string()),
            status: ConversionStatus::Success,
            duration_secs: started.elapsed().as_secs_f64(),
            chapter_count: Some(sections.len()),
            file_size_bytes: file_size,
            error_message: None,
        })
    }

    async fn parse_book(book: Book) -> Result<Vec<crate::model::Section>> {
        tokio::task::spawn_blocking(move || {
            let mut parser = Parser::new(book);
            parser.parse()
        })
        .await
        .map_err(|error| KafError::Unknown(format!("解析任务异常: {error}")))?
    }

    async fn write_new_file(path: PathBuf, data: Vec<u8>) -> Result<()> {
        tokio::task::spawn_blocking(move || {
            let mut file = fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&path)?;
            if let Err(error) = file.write_all(&data).and_then(|_| file.sync_all()) {
                drop(file);
                let _ = fs::remove_file(&path);
                return Err(error);
            }
            Ok::<(), std::io::Error>(())
        })
        .await
        .map_err(|error| KafError::Unknown(format!("写入任务异常: {error}")))??;
        Ok(())
    }

    fn print_chapters(book: &Book, sections: &[crate::model::Section]) {
        let title = book.bookname.as_deref().unwrap_or("Unknown");
        println!("\n=== {title} 章节识别结果 ===");
        for (index, section) in sections.iter().take(20).enumerate() {
            println!("{}. {}", index + 1, section.title);
        }
        if sections.len() > 20 {
            println!("... 还有 {} 个章节", sections.len() - 20);
        }
        println!("总计: {} 个章节\n", sections.len());
    }

    fn failed_result(input: &Path, message: String) -> FileConversionResult {
        FileConversionResult {
            input_file: input.display().to_string(),
            output_file: None,
            status: ConversionStatus::Failed,
            duration_secs: 0.0,
            chapter_count: None,
            file_size_bytes: fs::metadata(input)
                .map(|metadata| metadata.len())
                .unwrap_or(0),
            error_message: Some(message),
        }
    }

    fn update_summary(
        report: &mut BatchReport,
        total_duration: f64,
        error_types: &HashMap<String, &'static str>,
    ) {
        let total_files = report.files.len();
        let successful = report
            .files
            .iter()
            .filter(|file| file.status == ConversionStatus::Success)
            .count();
        let failed = report
            .files
            .iter()
            .filter(|file| file.status == ConversionStatus::Failed)
            .count();
        let skipped = report
            .files
            .iter()
            .filter(|file| file.status == ConversionStatus::Skipped)
            .count();
        let average_duration = if total_files == 0 {
            0.0
        } else {
            report
                .files
                .iter()
                .map(|file| file.duration_secs)
                .sum::<f64>()
                / total_files as f64
        };
        let attempted = successful + failed;
        let success_rate = if attempted == 0 {
            if skipped > 0 {
                1.0
            } else {
                0.0
            }
        } else {
            successful as f64 / attempted as f64
        };

        report.summary = ConversionSummary {
            total_files,
            successful_conversions: successful,
            failed_conversions: failed,
            skipped_conversions: skipped,
            total_duration_secs: total_duration,
            average_duration_secs: average_duration,
            success_rate,
        };

        let mut errors: HashMap<(String, String), ErrorDetail> = HashMap::new();
        for file in report
            .files
            .iter()
            .filter(|file| file.status == ConversionStatus::Failed)
        {
            if let Some(message) = &file.error_message {
                let error_type = error_types
                    .get(&file.input_file)
                    .copied()
                    .unwrap_or("ConversionError")
                    .to_string();
                errors
                    .entry((error_type.clone(), message.clone()))
                    .and_modify(|detail| {
                        detail.affected_files.push(file.input_file.clone());
                        detail.occurrence_count += 1;
                    })
                    .or_insert_with(|| ErrorDetail {
                        error_type,
                        message: message.clone(),
                        affected_files: vec![file.input_file.clone()],
                        occurrence_count: 1,
                    });
            }
        }
        report.errors = errors.into_values().collect();
        report
            .errors
            .sort_by(|left, right| left.message.cmp(&right.message));
    }

    pub fn generate_and_save_report(
        &self,
        report: &BatchReport,
        format: ReportFormat,
        output_dir: &Path,
    ) -> Result<PathBuf> {
        fs::create_dir_all(output_dir)?;
        let filename = format!(
            "batch_report_{}.{}",
            chrono::Utc::now().format("%Y%m%d_%H%M%S"),
            format.extension()
        );
        let path = output_dir.join(filename);
        ReportGenerator::new(format).save_to_file(report, &path)?;
        Ok(path)
    }
}

impl Default for FileConversionResult {
    fn default() -> Self {
        Self {
            input_file: String::new(),
            output_file: None,
            status: ConversionStatus::Success,
            duration_secs: 0.0,
            chapter_count: None,
            file_size_bytes: 0,
            error_message: None,
        }
    }
}
