use kaf_cli::batch::BatchConverter;
use kaf_cli::model::{Book, TextAlignment, ThemePreset};
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

fn temp_dir_with_file(name: &str, content: &str) -> (PathBuf, PathBuf) {
    let dir = std::env::temp_dir().join(format!(
        "kaf_batch_preserves_{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(&dir).unwrap();
    let path = dir.join(name);
    fs::write(&path, content).unwrap();
    (dir, path)
}

#[tokio::test]
async fn batch_converter_preserves_book_config() {
    let content = "第一章 开始\n\n这是第一章的内容。\n\n第二章 结束\n\n这是第二章的内容。\n";
    let (_dir, path) = temp_dir_with_file("novel.txt", content);

    let mut book = Book {
        filename: path.clone(),
        bookname: Some("BatchPreservedBook".to_string()),
        author: "Test Author".to_string(),
        output_name: Some("CustomBatchOutput".to_string()),
        align: TextAlignment::Right,
        theme: ThemePreset::Dark,
        ..Default::default()
    };
    // 用非默认值验证配置被保留
    book.lookahead_lines = 7;

    let converter = BatchConverter::new(1);
    let result = converter.convert(vec![book]).await;

    assert!(
        result.failed.is_empty(),
        "批量转换不应失败: {:?}",
        result.failed
    );
    assert_eq!(result.success.len(), 1);

    let output_path = &result.success[0];
    assert!(output_path.exists(), "输出 EPUB 应存在: {:?}", output_path);
    assert!(
        output_path
            .file_name()
            .unwrap()
            .to_string_lossy()
            .contains("CustomBatchOutput"),
        "应使用 Book 中指定的 output_name: {:?}",
        output_path
    );

    // 清理
    let _ = fs::remove_file(output_path);
}
