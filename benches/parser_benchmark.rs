//! 解析器性能基准测试

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use kaf_cli::parser::Parser;
use kaf_cli::model::Book;
use std::path::PathBuf;

fn bench_parse_simple(c: &mut Criterion) {
    let content = r#"第一章 开始

这是第一章的内容。

这是第二段内容。

第二章 结束

这是第二章的内容。

第三章 继续

这是第三章的内容。
"#;

    c.bench_function("parse_simple", |b| {
        b.iter(|| {
            let book = Book {
                filename: PathBuf::from("test.txt"),
                ..Default::default()
            };
            let mut parser = Parser::new(book);
            black_box(parser.parse_content(content).unwrap())
        });
    });
}

criterion_group!(benches, bench_parse_simple);
criterion_main!(benches);
