//! 批量转换增强模块
//!
//! 提供更强大的批量转换功能

use super::report::{BatchReport, ConversionSummary, ConversionStatus, ErrorDetail, FileConversionResult, ReportGenerator, ReportFormat};
use crate::converter::EpubConverter3;
use crate::error::Result;
use crate::model::Book;
use crate::parser::Parser;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Semaphore;
use tracing::{info, warn, error};

/// 批量转换增强配置
#[derive(Debug, Clone)]
pub struct BatchConfig {
    /// 输出目录
    pub output_dir: Option<PathBuf>,
    /// 遇到错误是否继续
    pub continue_on_error: bool,
    /// 最大错误数量（0 表示无限制）
    pub max_errors: usize,
    /// 是否仅解析不生成
    pub dry_run: bool,
    /// 是否显示章节信息
    pub show_chapters: bool,
    /// 并发数
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

/// 批量转换增强器
pub struct EnhancedBatchConverter {
    config: BatchConfig,
    /// 当前错误计数
    error_count: Arc<std::sync::atomic::AtomicUsize>,
    /// 是否应该停止
    should_stop: Arc<std::sync::atomic::AtomicBool>,
}

impl EnhancedBatchConverter {
    /// 创建新的批量转换增强器
    pub fn new(config: BatchConfig) -> Self {
        Self {
            config,
            error_count: Arc::new(std::sync::atomic::AtomicUsize::new(0)),
            should_stop: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        }
    }

    /// 执行批量转换
    pub async fn convert(&self, books: Vec<Book>) -> Result<BatchReport> {
        let start = Instant::now();
        let semaphore = Arc::new(Semaphore::new(self.config.concurrency));
        let mut tasks = Vec::new();

        let report = Arc::new(std::sync::Mutex::new(BatchReport::default()));

        for book in books {
            // 检查是否应该停止
            if self.should_stop.load(std::sync::atomic::Ordering::Relaxed) {
                warn!("达到最大错误数量，停止批量转换");
                break;
            }

            // 获取信号量
            let permit = semaphore.clone().acquire_owned().await.map_err(|e| {
                crate::error::KafError::Unknown(format!("获取信号量失败: {}", e))
            })?;

            // 克隆需要的数据
            let book_clone = book.clone();
            let error_count_clone = self.error_count.clone();
            let should_stop_clone = self.should_stop.clone();
            let report_clone = report.clone();
            let config_clone = self.config.clone();

            let task = tokio::spawn(async move {
                // 使用 permit 确保完成后释放
                let _permit = permit;
                let file_start = Instant::now();

                let input_file = book_clone.filename.clone();
                let input_file_str = input_file.display().to_string();
                let bookname = book_clone.bookname.clone().unwrap_or_else(|| "Unknown".to_string());

                info!("开始处理: {}", bookname);

                let result = Self::process_book(&book_clone, config_clone, error_count_clone, should_stop_clone).await;

                let duration = file_start.elapsed().as_secs_f64();

                // 更新报告
                {
                    let mut report = report_clone.lock().unwrap();
                    match result {
                        Ok(file_result) => {
                            report.files.push(file_result);
                            info!("处理完成: {} ({:.2}s)", bookname, duration);
                        }
                        Err(err) => {
                            // 创建失败的结果
                            let file_result = FileConversionResult {
                                input_file: input_file_str.clone(),
                                output_file: None,
                                status: ConversionStatus::Failed,
                                duration_secs: duration,
                                chapter_count: None,
                                file_size_bytes: 0,
                                error_message: Some(err.to_string()),
                            };
                            report.files.push(file_result);
                            error!("处理失败: {} - {}", bookname, err);
                        }
                    }
                }

                Ok::<(), crate::error::KafError>(())
            });

            tasks.push(task);
        }

        // 等待所有任务完成
        for task in tasks {
            if let Err(e) = task.await {
                error!("任务执行错误: {}", e);
            }
        }

        // 完成报告
        let final_duration = start.elapsed().as_secs_f64();
        let mut final_report = report.lock().unwrap().clone();

        // 更新汇总信息
        Self::update_summary(&mut final_report, final_duration);

        Ok(final_report)
    }

    /// 处理单个书籍
    async fn process_book(
        book: &Book,
        config: BatchConfig,
        _error_count: Arc<std::sync::atomic::AtomicUsize>,
        _should_stop: Arc<std::sync::atomic::AtomicBool>,
    ) -> Result<FileConversionResult> {
        let start = Instant::now();
        let input_file = &book.filename;
        let bookname = book.bookname.clone().unwrap_or_else(|| "Unknown".to_string());

        // 确定输出路径
        let output_path = Self::determine_output_path(input_file, &config, &bookname)?;

        // 检查文件大小
        let file_size = fs::metadata(input_file)?.len();

        // 检查是否是空文件，如果是空文件，跳过处理
        if file_size == 0 {
            return Ok(FileConversionResult {
                input_file: input_file.display().to_string(),
                output_file: if !config.dry_run {
                    Some(output_path.display().to_string())
                } else {
                    None
                },
                status: ConversionStatus::Skipped,
                duration_secs: start.elapsed().as_secs_f64(),
                chapter_count: Some(0),
                file_size_bytes: 0,
                error_message: Some("文件为空，跳过处理".to_string()),
            });
        }

        // 如果是 dry-run 模式，只解析不生成
        if config.dry_run {
            return Self::dry_run_process(book, config, &output_path, file_size, start).await;
        }

        // 正常处理：解析 + 生成
        let chapter_count = Self::normal_process(book, &output_path).await?;

        Ok(FileConversionResult {
            input_file: input_file.display().to_string(),
            output_file: Some(output_path.display().to_string()),
            status: ConversionStatus::Success,
            duration_secs: start.elapsed().as_secs_f64(),
            chapter_count: Some(chapter_count),
            file_size_bytes: file_size,
            error_message: None,
        })
    }

    /// 确定输出路径
    fn determine_output_path(
        input_file: &Path,
        config: &BatchConfig,
        bookname: &str,
    ) -> Result<PathBuf> {
        let output_dir = config.output_dir.as_ref()
            .cloned()
            .unwrap_or_else(|| input_file.parent().unwrap_or(Path::new(".")).to_path_buf());

        let filename = format!("{}.epub", bookname);
        let output_path = output_dir.join(&filename);

        // 检查文件名冲突
        if output_path.exists() {
            // 添加后缀
            for i in 1..=1000 {
                let new_filename = format!("{} ({}).epub", bookname, i);
                let new_path = output_dir.join(&new_filename);
                if !new_path.exists() {
                    return Ok(new_path);
                }
            }
            return Err(crate::error::KafError::ParseError(
                format!("无法解决文件名冲突: {}", filename)
            ));
        }

        Ok(output_path)
    }

    /// Dry-run 处理
    async fn dry_run_process(
        book: &Book,
        config: BatchConfig,
        output_path: &Path,
        file_size: u64,
        start: Instant,
    ) -> Result<FileConversionResult> {
        info!("Dry-run 模式: 仅解析不生成");

        // 解析文件
        let mut parser = Parser::new(book.clone());
        let sections = parser.parse()?;

        // 显示章节信息（如果要求）
        if config.show_chapters {
            println!("\n=== {} 章节识别结果 ===", book.bookname.as_ref().unwrap_or(&"Unknown".to_string()));
            for (i, section) in sections.iter().take(20).enumerate() {
                println!("{}. {}", i + 1, section.title);
            }
            if sections.len() > 20 {
                println!("... 还有 {} 个章节", sections.len() - 20);
            }
            println!("总计: {} 个章节\n", sections.len());
        }

        Ok(FileConversionResult {
            input_file: book.filename.display().to_string(),
            output_file: Some(output_path.display().to_string()),
            status: ConversionStatus::Skipped,
            duration_secs: start.elapsed().as_secs_f64(),
            chapter_count: Some(sections.len()),
            file_size_bytes: file_size,
            error_message: None,
        })
    }

    /// 正常处理（解析 + 生成）
    async fn normal_process(book: &Book, output_path: &Path) -> Result<usize> {
        // 解析文件
        let book_clone = book.clone();
        let sections = tokio::task::spawn_blocking(move || {
            let mut parser = Parser::new(book_clone);
            parser.parse()
        }).await.map_err(|e| {
            crate::error::KafError::Unknown(format!("Task join error: {}", e))
        })??;

        // 生成 EPUB
        let book_clone = book.clone();
        let converter = EpubConverter3::new(book_clone);
        let epub_data = converter.generate(&sections).await?;

        // 写入文件
        tokio::fs::write(output_path, epub_data).await?;

        Ok(sections.len())
    }

    /// 更新汇总信息
    fn update_summary(report: &mut BatchReport, total_duration: f64) {
        let total_files = report.files.len();
        let successful = report.files.iter()
            .filter(|f| f.status == ConversionStatus::Success)
            .count();
        let failed = report.files.iter()
            .filter(|f| f.status == ConversionStatus::Failed)
            .count();
        let _skipped = report.files.iter()
            .filter(|f| f.status == ConversionStatus::Skipped)
            .count();

        let average_duration = if total_files > 0 {
            total_duration / total_files as f64
        } else {
            0.0
        };

        let success_rate = if total_files > 0 {
            successful as f64 / total_files as f64
        } else {
            0.0
        };

        report.summary = ConversionSummary {
            total_files,
            successful_conversions: successful,
            failed_conversions: failed,
            total_duration_secs: total_duration,
            average_duration_secs: average_duration,
            success_rate,
        };

        // 错误统计
        let mut error_map = std::collections::HashMap::new();
        for file in &report.files {
            if let Some(ref error) = file.error_message {
                error_map.entry(error.clone())
                    .and_modify(|e: &mut ErrorDetail| {
                        e.affected_files.push(file.input_file.clone());
                        e.occurrence_count += 1;
                    })
                    .or_insert_with(|| ErrorDetail {
                        error_type: "ConversionError".to_string(),
                        message: error.clone(),
                        affected_files: vec![file.input_file.clone()],
                        occurrence_count: 1,
                    });
            }
        }

        report.errors = error_map.into_values().collect();
    }

    /// 生成并保存报告
    pub fn generate_and_save_report(
        &self,
        report: &BatchReport,
        format: ReportFormat,
        output_dir: &Path,
    ) -> Result<PathBuf> {
        let generator = ReportGenerator::new(format);
        let filename = format!("batch_report_{}.{}", 
            chrono::Utc::now().format("%Y%m%d_%H%M%S"),
            format.extension()
        );
        let report_path = output_dir.join(filename);

        generator.save_to_file(report, &report_path)?;
        Ok(report_path)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_batch_config_default() {
        let config = BatchConfig::default();
        assert_eq!(config.concurrency, 4);
        assert!(!config.continue_on_error);
        assert_eq!(config.max_errors, 0);
    }

    #[test]
    fn test_enhanced_batch_converter_creation() {
        let config = BatchConfig::default();
        let converter = EnhancedBatchConverter::new(config);
        assert_eq!(converter.config.concurrency, 4);
    }
}
