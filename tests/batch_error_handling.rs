use kaf_cli::batch::{BatchConfig, EnhancedBatchConverter};
use kaf_cli::model::Book;
use std::path::PathBuf;

#[tokio::test]
async fn enhanced_batch_captures_failed_file_result() {
    let book = Book {
        filename: PathBuf::from("/nonexistent/path/missing_novel.txt"),
        bookname: Some("Missing".to_string()),
        ..Default::default()
    };

    let config = BatchConfig {
        concurrency: 1,
        ..Default::default()
    };
    let converter = EnhancedBatchConverter::new(config);
    let report = converter.convert(vec![book]).await.unwrap();

    assert_eq!(report.summary.total_files, 1);
    assert_eq!(report.summary.failed_conversions, 1);
    assert_eq!(report.summary.successful_conversions, 0);

    let file = report.files.first().expect("应有一个文件结果");
    assert_eq!(
        format!("{:?}", file.status).to_lowercase(),
        "failed",
        "文件状态应为 Failed"
    );
    assert!(file.error_message.is_some(), "错误信息不应为空");
}
