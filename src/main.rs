//! kaf-cli - 将 TXT 文本转换为 EPUB 电子书
//!
//! 主程序入口

mod batch;
mod cli;
mod config;
mod converter;
mod error;
mod model;
mod parser;
mod style;
mod utils;

use batch::{BatchConfig, EnhancedBatchConverter, ReportFormat};
use clap::Parser as ClapParser;
use config::{generate_config_examples, load_config, validate_config};
use error::{KafError, Result};
use parser::Parser;
use tracing::{error, info, warn};
use tracing_subscriber::fmt;

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    fmt::init();

    // 解析命令行参数
    let cli: cli::Cli = ClapParser::parse();

    // 处理示例配置生成
    if cli.example_config {
        let examples = generate_config_examples();
        println!("{}", examples.get("basic").unwrap_or(&String::new()));
        return Ok(());
    }

    // 处理批量转换
    if let Some(batch_dir) = &cli.batch {
        return process_batch(batch_dir, &cli).await;
    }

    // 加载配置
    let mut book = load_config(&cli)?;

    // 检查是否指定了文件名
    if book.filename.as_os_str().is_empty() {
        error!("请指定要转换的文件名，使用 --filename 参数");
        return Err(KafError::ParseError("未指定文件名".to_string()));
    }

    // 从文件名提取书名和作者（如果没有指定）
    if book.bookname.is_none() || book.author == "YSTYLE" {
        if let Ok((bookname, author)) = utils::file::extract_bookname_from_filename(&book.filename)
        {
            if book.bookname.is_none() {
                book.bookname = Some(bookname);
            }
            // 只有当作者未设置（为默认值）且能从文件名提取时，才使用提取的作者
            if book.author == "YSTYLE" {
                if let Some(a) = author {
                    book.author = a;
                }
            }
        }
    }

    // 验证配置
    if let Err(e) = validate_config(&book) {
        warn!("配置验证警告: {}", e);
        // 不中断流程，继续处理
    }

    info!("开始转换: {:?}", book.filename);
    let bookname = book.bookname.clone().unwrap_or_else(|| "Unknown".to_string());
    info!("书名: {}", bookname);
    info!("作者: {}", book.author);
    info!("输入格式: {:?}", book.input_format);

    // 解析文件
    let mut parser = Parser::new(book.clone());
    let sections = parser.parse()?;

    info!("解析完成，共 {} 个章节", sections.len());

    // 生成 EPUB（使用 EPUB 3.0 标准）
    let converter = converter::EpubConverter3::new(book.clone());
    let epub_data = converter.generate(&sections).await?;

    // 确定输出文件名并写入文件
    let output_path = format!("{}.epub", bookname);

    // 写入文件
    tokio::fs::write(&output_path, epub_data).await?;

    info!("转换完成！输出文件: {}", output_path);
    Ok(())
}

/// 处理批量转换
async fn process_batch(batch_dir: &std::path::Path, cli: &cli::Cli) -> Result<()> {
    info!("开始批量转换: {:?}", batch_dir);

    // 创建批量转换配置
    let batch_config = BatchConfig {
        output_dir: cli.output_dir.clone(),
        continue_on_error: cli.continue_on_error,
        max_errors: cli.max_errors,
        dry_run: cli.dry_run,
        show_chapters: cli.show_chapters,
        concurrency: 4, // 默认并发数
    };

    // 扫描文件夹
    let scanner = batch::FolderScanner::new(batch_dir.to_path_buf(), true);
    let books = scanner.scan_with_config()?;

    info!("找到 {} 个文件", books.len());

    if books.is_empty() {
        warn!("未找到任何 TXT 文件");
        return Ok(());
    }

    // 创建增强的批量转换器
    let converter = EnhancedBatchConverter::new(batch_config.clone());

    // 执行批量转换
    let report = converter.convert(books).await?;

    // 显示汇总信息
    print_batch_summary(&report);

    // 生成报告（如果指定）
    if let Some(ref report_format) = cli.report {
        generate_and_save_report(&report, report_format, batch_dir, &cli.output_dir)?;
    }

    // 检查是否有失败
    if report.summary.failed_conversions > 0 {
        error!("批量转换完成，但有 {} 个文件失败", report.summary.failed_conversions);
        if !batch_config.continue_on_error {
            return Err(KafError::ParseError(
                format!("批量转换失败: {} 个文件转换失败", report.summary.failed_conversions)
            ));
        }
    } else {
        info!("批量转换成功完成！");
    }

    Ok(())
}

/// 打印批量转换汇总
fn print_batch_summary(report: &batch::BatchReport) {
    println!("\n=== 批量转换汇总 ===");
    println!("总文件数: {}", report.summary.total_files);
    println!("成功转换: {}", report.summary.successful_conversions);
    println!("失败转换: {}", report.summary.failed_conversions);
    println!("总耗时: {:.2} 秒", report.summary.total_duration_secs);
    println!("平均耗时: {:.2} 秒", report.summary.average_duration_secs);
    println!("成功率: {:.1}%", report.summary.success_rate * 100.0);

    if !report.errors.is_empty() {
        println!("\n=== 错误汇总 ===");
        for error in &report.errors {
            println!("{}: {} ({} 次)", error.error_type, error.message, error.occurrence_count);
        }
    }
}

/// 生成并保存报告
fn generate_and_save_report(
    report: &batch::BatchReport,
    report_format: &str,
    batch_dir: &std::path::Path,
    output_dir: &Option<std::path::PathBuf>,
) -> Result<()> {
    // 解析报告格式
    let format = ReportFormat::parse(report_format)?;

    // 确定报告输出目录
    let report_dir = output_dir.as_ref()
        .cloned()
        .unwrap_or_else(|| batch_dir.to_path_buf());

    // 确保目录存在
    std::fs::create_dir_all(&report_dir)?;

    // 创建转换器以生成报告
    let batch_config = BatchConfig::default();
    let converter = EnhancedBatchConverter::new(batch_config);

    // 生成并保存报告
    let report_path = converter.generate_and_save_report(report, format, &report_dir)?;

    info!("报告已保存: {:?}", report_path);

    Ok(())
}
