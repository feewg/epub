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
mod utils;

use clap::Parser as ClapParser;
use config::{generate_example_config, load_config};
use converter::EpubConverter3;
use error::{KafError, Result};
use parser::Parser;
use tracing::{error, info};
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    tracing_subscriber::fmt::init();

    // 解析命令行参数
    let cli: cli::Cli = ClapParser::parse();

    // 处理示例配置生成
    if cli.example_config {
        println!("{}", generate_example_config());
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

    // 从文件名提取书名（如果没有指定）
    if book.bookname.is_none() {
        if let Ok((bookname, _author)) = utils::file::extract_bookname_from_filename(&book.filename)
        {
            book.bookname = Some(bookname);
        }
    }

    info!("开始转换: {:?}", book.filename);
    let bookname = book.bookname.clone().unwrap_or_else(|| "Unknown".to_string());
    info!("书名: {}", bookname);
    info!("作者: {}", book.author);

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
async fn process_batch(batch_dir: &std::path::Path, _cli: &cli::Cli) -> Result<()> {
    info!("开始批量转换: {:?}", batch_dir);

    // 扫描文件夹
    let scanner = batch::FolderScanner::new(batch_dir.to_path_buf(), true);
    let books = scanner.scan_with_config()?;

    info!("找到 {} 个文件", books.len());

    let mut result = batch::BatchResult::default();

    for book in books {
        let filename = book.filename.clone();
        let bookname = book.bookname.clone();
        let mut output_name = bookname
            .clone()
            .unwrap_or_else(|| "output".to_string());
        if let Some(ref output) = book.output_name {
            output_name.clone_from(output);
        }

        info!("处理: {:?}", filename);

        // 解析
        let mut parser = Parser::new(book.clone());
        match parser.parse() {
            Ok(sections) => {
                info!("  解析完成: {} 个章节", sections.len());

                // 生成 EPUB
                let converter = converter::EpubConverter3::new(book.clone());
                match converter.generate(&sections).await {
                    Ok(epub_data) => {
                        let output_path = format!("{}.epub", output_name);

                        // 写入文件
                        if let Err(e) = tokio::fs::write(&output_path, epub_data).await {
                            error!(" 写入失败: {}", e);
                            result
                                .failed
                                .push((filename, format!("写入失败: {}", e)));
                        } else {
                            info!("  转换完成: {}", output_path);
                            result.success.push(std::path::PathBuf::from(output_path));
                        }
                    }
                    Err(e) => {
                        error!(" EPUB 生成失败: {}", e);
                        result.failed.push((filename, format!("EPUB 生成失败: {}", e)));
                    }
                }
            }
            Err(e) => {
                error!(" 解析失败: {}", e);
                result.failed.push((filename, format!("解析失败: {}", e)));
            }
        }
    }

    // 打印汇总
    info!("批量转换完成！");
    info!("成功: {} 个", result.success.len());
    info!("失败: {} 个", result.failed.len());

    if !result.failed.is_empty() {
        error!("失败的文件:");
        for (path, error) in &result.failed {
            error!("  {:?}: {}", path, error);
        }
    }

    Ok(())
}
