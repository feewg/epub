use std::fs;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn full_conversion_pipeline_produces_epub() {
    let dir = std::env::temp_dir().join(format!(
        "kaf_e2e_{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(&dir).unwrap();

    let input_path = dir.join("novel.txt");
    fs::write(
        &input_path,
        "第一章 启程\n\n这是第一章的内容。\n\n第二章 终点\n\n这是第二章的内容。\n",
    )
    .unwrap();

    let output_path = dir.join("EndToEnd.epub");

    let status = Command::new(env!("CARGO_BIN_EXE_kaf-cli"))
        .current_dir(&dir)
        .args([
            "--filename",
            input_path.to_str().unwrap(),
            "--bookname",
            "EndToEnd",
        ])
        .status()
        .expect("应能运行 kaf-cli 二进制文件");

    assert!(status.success(), "转换进程应成功退出");
    assert!(output_path.exists(), "应生成 EPUB 文件: {:?}", output_path);

    let metadata = fs::metadata(&output_path).expect("应能读取生成的 EPUB 元数据");
    assert!(metadata.len() > 0, "生成的 EPUB 不应为空");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&input_path);
    let _ = fs::remove_dir(&dir);
}
