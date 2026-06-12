use kaf_cli::model::Book;
use kaf_cli::parser::Parser;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

fn write_temp_novel() -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "kaf_lookahead_{}",
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()
    ));
    fs::create_dir_all(&dir).unwrap();
    let path = dir.join("novel.txt");
    fs::write(
        &path,
        "第一章\n\n这是第一章的内容。\n\n第二章\n\n这是第二章的内容。\n",
    )
    .unwrap();
    path
}

#[test]
fn custom_lookahead_value_works() {
    let path = write_temp_novel();

    for lookahead in [1usize, 5usize] {
        let book = Book {
            filename: path.clone(),
            lookahead_lines: lookahead,
            ..Default::default()
        };
        let mut parser = Parser::new(book);
        let sections = parser.parse_streaming().expect("流式解析不应失败");
        assert!(!sections.is_empty(), "lookahead={} 时应解析出章节", lookahead);
        assert_eq!(sections[0].title, "第一章");
    }
}
