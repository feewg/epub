//! kaf-cli - 将 TXT/Markdown 转换为 EPUB

use clap::Parser as ClapParser;
use kaf_cli::batch::{BatchConfig, EnhancedBatchConverter, ReportFormat};
use kaf_cli::cli::Cli;
use kaf_cli::config::{generate_config_examples, load_config, validate_config};
use kaf_cli::error::{KafError, Result};
use kaf_cli::model::Book;
use kaf_cli::parser::Parser;
use kaf_cli::EpubConverter3;
use std::io::Write;
use std::path::{Path, PathBuf};
use tracing::{error, info, warn};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<()> {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::fmt().with_env_filter(env_filter).init();

    let cli = Cli::parse();
    if cli.example_config {
        let examples = generate_config_examples();
        if let Some(example) = examples.get("basic") {
            println!("{example}");
        }
        return Ok(());
    }
    if let Some(batch_dir) = &cli.batch {
        return process_batch(batch_dir, &cli).await;
    }

    process_single(&cli).await
}

async fn process_single(cli: &Cli) -> Result<()> {
    let mut book = load_config(cli)?;
    if book.filename.as_os_str().is_empty() {
        return Err(KafError::ParseError(
            "未指定文件名，请使用 --filename".to_string(),
        ));
    }
    apply_filename_metadata(&mut book)?;
    validate_config(&book)?;

    let bookname = book
        .bookname
        .clone()
        .unwrap_or_else(|| "Unknown".to_string());
    info!(input = %book.filename.display(), title = %bookname, author = %book.author, "开始转换");

    let mut parser = Parser::new(book.clone());
    let sections = parser.parse()?;
    info!(chapters = sections.len(), "解析完成");

    if cli.show_chapters {
        print_chapters(&bookname, &sections);
    }
    if cli.dry_run {
        info!("Dry-run 完成，未生成 EPUB");
        return Ok(());
    }

    let converter = EpubConverter3::new(book.clone());
    let epub_data = converter.generate(&sections).await?;
    let output_stem = book.output_name.as_deref().unwrap_or(&bookname);
    let output_path = choose_output_path(Path::new("."), output_stem)?;
    write_new_file(output_path.clone(), epub_data).await?;
    info!(output = %output_path.display(), "转换完成");
    Ok(())
}

fn apply_filename_metadata(book: &mut Book) -> Result<()> {
    if book.bookname.is_none() || book.author == "YSTYLE" {
        let (bookname, author) =
            kaf_cli::utils::file::extract_bookname_from_filename(&book.filename)?;
        if book.bookname.is_none() {
            book.bookname = Some(bookname);
        }
        if book.author == "YSTYLE" {
            if let Some(author) = author {
                book.author = author;
            }
        }
    }
    Ok(())
}

fn print_chapters(title: &str, sections: &[kaf_cli::model::Section]) {
    println!("\n=== {title} 章节识别结果 ===");
    for (index, section) in sections.iter().take(20).enumerate() {
        println!("{}. {}", index + 1, section.title);
    }
    if sections.len() > 20 {
        println!("... 还有 {} 个章节", sections.len() - 20);
    }
    println!("总计: {} 个章节\n", sections.len());
}

fn sanitize_output_name(value: &str) -> String {
    let value = value
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
    let value = value.trim_matches([' ', '.']);
    if value.is_empty() {
        "output".to_string()
    } else {
        value.to_string()
    }
}

fn choose_output_path(directory: &Path, requested_name: &str) -> Result<PathBuf> {
    let basename = sanitize_output_name(requested_name);
    for suffix in 0..=10_000usize {
        let filename = if suffix == 0 {
            format!("{basename}.epub")
        } else {
            format!("{basename} ({suffix}).epub")
        };
        let path = directory.join(filename);
        if !path.exists() {
            return Ok(path);
        }
    }
    Err(KafError::ParseError(format!(
        "无法为 {requested_name} 分配唯一输出文件名"
    )))
}

async fn write_new_file(path: PathBuf, data: Vec<u8>) -> Result<()> {
    tokio::task::spawn_blocking(move || {
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)?;
        if let Err(error) = file.write_all(&data).and_then(|_| file.sync_all()) {
            drop(file);
            let _ = std::fs::remove_file(path);
            return Err(error);
        }
        Ok::<(), std::io::Error>(())
    })
    .await
    .map_err(|error| KafError::Unknown(format!("写入任务异常: {error}")))??;
    Ok(())
}

async fn process_batch(batch_dir: &Path, cli: &Cli) -> Result<()> {
    info!(path = %batch_dir.display(), "开始批量转换");
    let batch_config = BatchConfig {
        output_dir: cli.output_dir.clone(),
        continue_on_error: cli.continue_on_error,
        max_errors: cli.max_errors,
        dry_run: cli.dry_run,
        show_chapters: cli.show_chapters,
        concurrency: usize::from(cli.concurrency),
    };

    let scanner = kaf_cli::batch::FolderScanner::new(batch_dir.to_path_buf(), true);
    let inputs = scanner.scan_with_cli_inputs(cli)?;
    info!(files = inputs.len(), "扫描完成");
    if inputs.is_empty() {
        warn!("未找到 TXT 或 Markdown 文件");
        return Ok(());
    }

    let converter = EnhancedBatchConverter::new(batch_config.clone());
    let report = converter.convert_inputs(inputs).await?;
    print_batch_summary(&report);

    if let Some(report_format) = &cli.report {
        generate_and_save_report(&report, report_format, batch_dir, &cli.output_dir)?;
    }
    if report.summary.failed_conversions > 0 && !batch_config.continue_on_error {
        error!(failed = report.summary.failed_conversions, "批量转换失败");
        return Err(KafError::ParseError(format!(
            "批量转换失败: {} 个文件转换失败",
            report.summary.failed_conversions
        )));
    }
    Ok(())
}

fn print_batch_summary(report: &kaf_cli::batch::BatchReport) {
    println!("\n=== 批量转换汇总 ===");
    println!("总文件数: {}", report.summary.total_files);
    println!("成功转换: {}", report.summary.successful_conversions);
    println!("失败转换: {}", report.summary.failed_conversions);
    println!("跳过转换: {}", report.summary.skipped_conversions);
    println!("总耗时: {:.2} 秒", report.summary.total_duration_secs);
    println!("平均耗时: {:.2} 秒", report.summary.average_duration_secs);
    println!("成功率: {:.1}%", report.summary.success_rate * 100.0);

    if !report.errors.is_empty() {
        println!("\n=== 错误汇总 ===");
        for detail in &report.errors {
            println!(
                "{}: {} ({} 次)",
                detail.error_type, detail.message, detail.occurrence_count
            );
        }
    }
}

fn generate_and_save_report(
    report: &kaf_cli::batch::BatchReport,
    report_format: &str,
    batch_dir: &Path,
    output_dir: &Option<PathBuf>,
) -> Result<()> {
    let format = ReportFormat::parse(report_format)?;
    let report_dir = output_dir
        .as_ref()
        .cloned()
        .unwrap_or_else(|| batch_dir.to_path_buf());
    std::fs::create_dir_all(&report_dir)?;
    let converter = EnhancedBatchConverter::new(BatchConfig::default());
    let report_path = converter.generate_and_save_report(report, format, &report_dir)?;
    info!(path = %report_path.display(), "报告已保存");
    Ok(())
}
