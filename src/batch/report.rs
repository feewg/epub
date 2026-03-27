//! 批量转换报告模块
//!
//! 生成批量转换报告

use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::PathBuf;

/// 批量转换报告
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchReport {
    /// 报告生成时间
    pub timestamp: String,
    /// 转换汇总
    pub summary: ConversionSummary,
    /// 文件详情
    pub files: Vec<FileConversionResult>,
    /// 错误详情
    pub errors: Vec<ErrorDetail>,
}

/// 转换汇总
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversionSummary {
    /// 总文件数
    pub total_files: usize,
    /// 成功转换数
    pub successful_conversions: usize,
    /// 失败转换数
    pub failed_conversions: usize,
    /// 总耗时（秒）
    pub total_duration_secs: f64,
    /// 平均耗时（秒）
    pub average_duration_secs: f64,
    /// 成功率
    pub success_rate: f64,
}

/// 文件转换结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileConversionResult {
    /// 输入文件路径
    pub input_file: String,
    /// 输出文件路径
    pub output_file: Option<String>,
    /// 状态
    pub status: ConversionStatus,
    /// 转换耗时（秒）
    pub duration_secs: f64,
    /// 章节数量
    pub chapter_count: Option<usize>,
    /// 文件大小（字节）
    pub file_size_bytes: u64,
    /// 错误信息（如果失败）
    pub error_message: Option<String>,
}

/// 转换状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ConversionStatus {
    /// 成功
    Success,
    /// 失败
    Failed,
    /// 跳过
    Skipped,
}

/// 错误详情
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorDetail {
    /// 错误类型
    pub error_type: String,
    /// 错误消息
    pub message: String,
    /// 受影响的文件
    pub affected_files: Vec<String>,
    /// 发生次数
    pub occurrence_count: usize,
}

/// 报告格式
#[derive(Debug, Clone, Copy)]
pub enum ReportFormat {
    /// JSON 格式
    Json,
    /// Markdown 格式
    Markdown,
    /// HTML 格式
    Html,
}

impl ReportFormat {
    /// 从字符串解析格式
    pub fn parse(s: &str) -> Result<Self> {
        Ok(match s.to_lowercase().as_str() {
            "json" => ReportFormat::Json,
            "markdown" | "md" => ReportFormat::Markdown,
            "html" => ReportFormat::Html,
            _ => return Err(crate::error::KafError::ParseError(
                format!("Invalid report format: {}", s)
            )),
        })
    }

    /// 文件扩展名
    pub fn extension(&self) -> &str {
        match self {
            ReportFormat::Json => "json",
            ReportFormat::Markdown => "md",
            ReportFormat::Html => "html",
        }
    }
}

/// 报告生成器
pub struct ReportGenerator {
    /// 报告格式
    format: ReportFormat,
}

impl ReportGenerator {
    /// 创建新的报告生成器
    pub fn new(format: ReportFormat) -> Self {
        Self { format }
    }

    /// 生成报告
    pub fn generate(&self, report: &BatchReport) -> Result<String> {
        match self.format {
            ReportFormat::Json => self.generate_json(report),
            ReportFormat::Markdown => self.generate_markdown(report),
            ReportFormat::Html => self.generate_html(report),
        }
    }

    /// 生成 JSON 报告
    fn generate_json(&self, report: &BatchReport) -> Result<String> {
        serde_json::to_string_pretty(report)
            .map_err(|e| crate::error::KafError::ParseError(e.to_string()))
    }

    /// 生成 Markdown 报告
    fn generate_markdown(&self, report: &BatchReport) -> Result<String> {
        let mut md = String::new();

        // 标题
        md.push_str("# Batch Conversion Report\n\n");
        md.push_str(&format!("Generated at: {}\n\n", report.timestamp));

        // 汇总信息
        md.push_str("## Summary\n\n");
        md.push_str(&format!("- Total files: {}\n", report.summary.total_files));
        md.push_str(&format!("- Successful conversions: {}\n", report.summary.successful_conversions));
        md.push_str(&format!("- Failed conversions: {}\n", report.summary.failed_conversions));
        md.push_str(&format!("- Total duration: {:.2} seconds\n", report.summary.total_duration_secs));
        md.push_str(&format!("- Average duration: {:.2} seconds\n", report.summary.average_duration_secs));
        md.push_str(&format!("- Success rate: {:.1}%\n\n", report.summary.success_rate * 100.0));

        // 文件详情
        md.push_str("## File Details\n\n");
        for (i, file) in report.files.iter().enumerate() {
            md.push_str(&format!("{}. {}\n\n", i + 1, file.input_file));
            md.push_str(&format!("   - Status: {:?}\n", file.status));
            if let Some(ref output) = file.output_file {
                md.push_str(&format!("   - Output: {}\n", output));
            }
            md.push_str(&format!("   - Duration: {:.2} seconds\n", file.duration_secs));
            if let Some(count) = file.chapter_count {
                md.push_str(&format!("   - Chapters: {}\n", count));
            }
            md.push_str(&format!("   - Size: {} bytes\n", file.file_size_bytes));
            if let Some(ref error) = file.error_message {
                md.push_str(&format!("   - Error: {}\n", error));
            }
            md.push('\n');
        }

        // 错误详情
        if !report.errors.is_empty() {
            md.push_str("## Error Details\n\n");
            for (i, error) in report.errors.iter().enumerate() {
                md.push_str(&format!("### {}. {}\n\n", i + 1, error.error_type));
                md.push_str(&format!("**Message**: {}\n\n", error.message));
                md.push_str(&format!("**Occurrences**: {}\n\n", error.occurrence_count));
                md.push_str("**Affected files**:\n\n");
                for file in &error.affected_files {
                    md.push_str(&format!("- {}\n", file));
                }
                md.push('\n');
            }
        }

        Ok(md)
    }

    /// 生成 HTML 报告
    fn generate_html(&self, report: &BatchReport) -> Result<String> {
        let mut html = String::new();

        // HTML 头部
        html.push_str("<!DOCTYPE html>\n");
        html.push_str("<html lang=\"en\">\n");
        html.push_str("<head>\n");
        html.push_str("<meta charset=\"UTF-8\">\n");
        html.push_str("<title>Batch Conversion Report</title>\n");
        html.push_str("<style>\n");
        html.push_str("body { font-family: Arial, sans-serif; line-height: 1.6; margin: 20px; }\n");
        html.push_str("h1 { color: #333; }\n");
        html.push_str("h2 { color: #555; border-bottom: 2px solid #eee; padding-bottom: 10px; }\n");
        html.push_str("table { border-collapse: collapse; width: 100%; margin-bottom: 20px; }\n");
        html.push_str("th, td { border: 1px solid #ddd; padding: 8px; text-align: left; }\n");
        html.push_str("th { background-color: #f2f2f2; }\n");
        html.push_str(".success { color: #28a745; }\n");
        html.push_str(".failed { color: #dc3545; }\n");
        html.push_str(".skipped { color: #ffc107; }\n");
        html.push_str("</style>\n");
        html.push_str("</head>\n");
        html.push_str("<body>\n");

        // 标题
        html.push_str("<h1>Batch Conversion Report</h1>\n");
        html.push_str(&format!("<p><strong>Generated at:</strong> {}</p>\n", report.timestamp));

        // 汇总信息
        html.push_str("<h2>Summary</h2>\n");
        html.push_str("<table>\n");
        html.push_str("<tr><th>Metric</th><th>Value</th></tr>\n");
        html.push_str(&format!("<tr><td>Total files</td><td>{}</td></tr>\n", report.summary.total_files));
        html.push_str(&format!("<tr><td>Successful conversions</td><td class=\"success\">{}</td></tr>\n", report.summary.successful_conversions));
        html.push_str(&format!("<tr><td>Failed conversions</td><td class=\"failed\">{}</td></tr>\n", report.summary.failed_conversions));
        html.push_str(&format!("<tr><td>Total duration</td><td>{:.2} seconds</td></tr>\n", report.summary.total_duration_secs));
        html.push_str(&format!("<tr><td>Average duration</td><td>{:.2} seconds</td></tr>\n", report.summary.average_duration_secs));
        html.push_str(&format!("<tr><td>Success rate</td><td>{:.1}%</td></tr>\n", report.summary.success_rate * 100.0));
        html.push_str("</table>\n");

        // 文件详情
        html.push_str("<h2>File Details</h2>\n");
        html.push_str("<table>\n");
        html.push_str("<tr><th>#</th><th>Input file</th><th>Output file</th><th>Status</th><th>Duration</th><th>Chapters</th><th>Size</th></tr>\n");
        for (i, file) in report.files.iter().enumerate() {
            let status_class = match file.status {
                ConversionStatus::Success => "success",
                ConversionStatus::Failed => "failed",
                ConversionStatus::Skipped => "skipped",
            };
            let status_text = format!("{:?}", file.status).to_lowercase();
            let chapter_text = file.chapter_count.map(|c| c.to_string()).unwrap_or_default();
            let output_text = file.output_file.as_ref().unwrap_or(&String::new()).clone();

            html.push_str(&format!("<tr><td>{}</td><td>{}</td><td>{}</td><td class=\"{}\">{}</td><td>{:.2}s</td><td>{}</td><td>{}</td></tr>\n",
                i + 1,
                file.input_file,
                output_text,
                status_class,
                status_text,
                file.duration_secs,
                chapter_text,
                file.file_size_bytes
            ));
        }
        html.push_str("</table>\n");

        // 错误详情
        if !report.errors.is_empty() {
            html.push_str("<h2>Error Details</h2>\n");
            for (i, error) in report.errors.iter().enumerate() {
                html.push_str(&format!("<h3>{}. {}</h3>\n", i + 1, error.error_type));
                html.push_str(&format!("<p><strong>Message:</strong> {}</p>\n", error.message));
                html.push_str(&format!("<p><strong>Occurrences:</strong> {}</p>\n", error.occurrence_count));
                html.push_str("<p><strong>Affected files:</strong></p>\n");
                html.push_str("<ul>\n");
                for file in &error.affected_files {
                    html.push_str(&format!("<li>{}</li>\n", file));
                }
                html.push_str("</ul>\n");
            }
        }

        // HTML 尾部
        html.push_str("</body>\n");
        html.push_str("</html>\n");

        Ok(html)
    }

    /// 保存报告到文件
    pub fn save_to_file(&self, report: &BatchReport, path: &PathBuf) -> Result<()> {
        let content = self.generate(report)?;
        let mut file = fs::File::create(path)?;
        file.write_all(content.as_bytes())?;
        Ok(())
    }
}

impl Default for BatchReport {
    fn default() -> Self {
        Self {
            timestamp: chrono::Utc::now().to_rfc3339(),
            summary: ConversionSummary::default(),
            files: Vec::new(),
            errors: Vec::new(),
        }
    }
}

impl Default for ConversionSummary {
    fn default() -> Self {
        Self {
            total_files: 0,
            successful_conversions: 0,
            failed_conversions: 0,
            total_duration_secs: 0.0,
            average_duration_secs: 0.0,
            success_rate: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_report_format_from_str() {
        assert!(matches!(ReportFormat::parse("json"), Ok(ReportFormat::Json)));
        assert!(matches!(ReportFormat::parse("markdown"), Ok(ReportFormat::Markdown)));
        assert!(matches!(ReportFormat::parse("html"), Ok(ReportFormat::Html)));
        assert!(ReportFormat::parse("invalid").is_err());
    }

    #[test]
    fn test_report_format_extension() {
        assert_eq!(ReportFormat::Json.extension(), "json");
        assert_eq!(ReportFormat::Markdown.extension(), "md");
        assert_eq!(ReportFormat::Html.extension(), "html");
    }

    #[test]
    fn test_batch_report_default() {
        let report = BatchReport::default();
        assert_eq!(report.files.len(), 0);
        assert_eq!(report.errors.len(), 0);
        assert_eq!(report.summary.total_files, 0);
    }

    #[test]
    fn test_conversion_summary_default() {
        let summary = ConversionSummary::default();
        assert_eq!(summary.total_files, 0);
        assert_eq!(summary.successful_conversions, 0);
        assert_eq!(summary.failed_conversions, 0);
    }

    #[test]
    fn test_file_conversion_result_status() {
        let result = FileConversionResult {
            status: ConversionStatus::Success,
            ..Default::default()
        };
        assert_eq!(result.status, ConversionStatus::Success);
    }
}
