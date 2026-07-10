//! 解析器模块
//!
//! 提供智能的文本解析能力，包括章节识别、段落处理等。
//! 支持 TXT 和 Markdown 两种输入格式。

mod chapter_detector;
mod format_detector;
mod markdown_parser;
mod paragraph_processor;
pub mod scorer;

pub use chapter_detector::ChapterDetector;
pub use format_detector::FormatDetector;
pub use markdown_parser::MarkdownParser;
pub use paragraph_processor::ParagraphProcessor;

use crate::error::Result;
use crate::model::{Book, InputFormat, Section};
use crate::utils::encoding::{detect_and_convert, ensure_no_bom};
use crate::utils::regex::RegexCache;
use std::fs;
use tracing::{debug, info};

/// 解析器结构体
pub struct Parser {
    book: Book,
    chapter_detector: ChapterDetector,
    paragraph_processor: ParagraphProcessor,
    regex_cache: RegexCache,
}

impl Parser {
    /// 创建新的解析器
    pub fn new(book: Book) -> Self {
        let chapter_detector = ChapterDetector::new();
        let paragraph_processor = ParagraphProcessor::new(book.clone());

        Self {
            book,
            chapter_detector,
            paragraph_processor,
            regex_cache: RegexCache::new(),
        }
    }

    /// 解析文件（自动检测格式或使用指定格式）
    pub fn parse(&mut self) -> Result<Vec<Section>> {
        // 1. 读取文件
        let bytes = fs::read(&self.book.filename)?;

        // 2. 检测并转换编码
        let content = detect_and_convert(&bytes)?;

        // 3. 确定输入格式
        let format = match self.book.input_format {
            InputFormat::Auto => {
                let detected = FormatDetector::detect(&self.book.filename, &content);
                debug!("自动检测输入格式: {:?}", detected);
                detected
            }
            other => {
                debug!("使用指定输入格式: {:?}", other);
                other
            }
        };

        // 4. 根据格式选择解析器
        info!("输入格式: {:?}", format);
        match format {
            InputFormat::Markdown => self.parse_markdown(&content),
            InputFormat::Txt | InputFormat::Auto => self.parse_txt(&content),
        }
    }

    /// 解析 Markdown 文件
    fn parse_markdown(&mut self, content: &str) -> Result<Vec<Section>> {
        let mut parser = MarkdownParser::new();
        let sections = parser.parse(content)?;
        debug!("Markdown 解析完成，共 {} 个章节", sections.len());
        Ok(sections)
    }

    /// 解析 TXT 文件
    fn parse_txt(&mut self, content: &str) -> Result<Vec<Section>> {
        self.parse_content(content)
    }

    /// 兼容旧 API 的解析入口。
    ///
    /// 旧实现同样会先读取完整文件，但使用了另一套章节上下文算法，导致结果不一致。
    /// 现在统一复用主解析路径，确保相同输入得到相同章节结构。
    pub fn parse_streaming(&mut self) -> Result<Vec<Section>> {
        self.parse()
    }

    /// 解析文本内容
    #[doc(hidden)]
    pub fn parse_content(&mut self, content: &str) -> Result<Vec<Section>> {
        let mut sections = Vec::new();
        let mut current_section = Section::default();

        let lines: Vec<&str> = content.lines().collect();

        for (line_num, line) in lines.iter().enumerate() {
            let trimmed = line.trim();

            // 跳过空行
            if trimmed.is_empty() {
                continue;
            }

            // 检查是否是卷标题
            if self
                .chapter_detector
                .detect_volume(trimmed, line_num, &lines, self.book.volume_match.as_deref())
                .is_some()
            {
                // 保存当前章节或标题前的序言正文
                if !current_section.title.is_empty() || !current_section.content.is_empty() {
                    sections.push(std::mem::take(&mut current_section));
                }

                // 创建新卷（确保标题无 BOM）
                current_section.title = ensure_no_bom(trimmed);
                current_section.content = String::new();
                continue;
            }

            // 检查是否是章节标题
            if self
                .chapter_detector
                .detect_chapter(
                    trimmed,
                    line_num,
                    &lines,
                    self.book.chapter_match.as_deref(),
                )
                .is_some()
            {
                // 检查是否被排除
                if !self.is_excluded(trimmed)? {
                    // 保存当前章节或标题前的序言正文
                    if !current_section.title.is_empty() || !current_section.content.is_empty() {
                        sections.push(std::mem::take(&mut current_section));
                    }

                    // 创建新章节（确保标题无 BOM）
                    current_section.title = ensure_no_bom(trimmed);
                    current_section.content = String::new();
                    continue;
                }
            }

            // 添加内容到当前章节
            let paragraph = self.paragraph_processor.process(trimmed);
            if !paragraph.is_empty() {
                if current_section.content.is_empty() {
                    current_section.content = paragraph;
                } else {
                    current_section.content.push_str(&paragraph);
                }
            }
        }

        // 保存最后一个章节
        if !current_section.title.is_empty() || !current_section.content.is_empty() {
            sections.push(current_section);
        }

        Ok(sections)
    }

    /// 检查文本是否应该被排除
    fn is_excluded(&mut self, text: &str) -> Result<bool> {
        let pattern = self.book.exclusion_pattern.as_deref();
        self.regex_cache.is_excluded(text, pattern)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_parser_creation() {
        let book = Book {
            filename: PathBuf::from("test.txt"),
            ..Default::default()
        };
        let parser = Parser::new(book);
        assert_eq!(parser.book.filename, PathBuf::from("test.txt"));
    }

    #[test]
    fn test_parse_txt_content() {
        let content = r#"第一章 开始

这是第一章的内容。

这是第二段内容。

第二章 结束

这是第二章的内容。
"#;

        let book = Book {
            filename: PathBuf::from("test.txt"),
            ..Default::default()
        };
        let mut parser = Parser::new(book);
        let sections = parser.parse_txt(content).unwrap();

        assert_eq!(sections.len(), 2);
        assert_eq!(sections[0].title, "第一章 开始");
        assert!(sections[0].content.contains("这是第一章的内容"));
        assert_eq!(sections[1].title, "第二章 结束");
    }

    #[test]
    fn test_parse_with_volumes() {
        let content = r#"第一卷 开始

第一章 开端

这是第一章的内容。

第二卷 发展

第二章 延续

这是第二章的内容。
"#;

        let book = Book {
            filename: PathBuf::from("test.txt"),
            ..Default::default()
        };
        let mut parser = Parser::new(book);
        let sections = parser.parse_txt(content).unwrap();

        assert_eq!(sections.len(), 4);
        assert_eq!(sections[0].title, "第一卷 开始");
        assert_eq!(sections[1].title, "第一章 开端");
        assert_eq!(sections[2].title, "第二卷 发展");
        assert_eq!(sections[3].title, "第二章 延续");
    }

    #[test]
    fn test_parse_markdown_content() {
        let content = r#"# 第一章 开始

这是第一章的内容。

## 第一节

这是第一节的内容。

# 第二章 结束

这是第二章的内容。
"#;

        let book = Book {
            filename: PathBuf::from("test.md"),
            input_format: InputFormat::Markdown,
            ..Default::default()
        };
        let mut parser = Parser::new(book);
        let sections = parser.parse_markdown(content).unwrap();

        assert_eq!(sections.len(), 3);
        assert_eq!(sections[0].title, "第一章 开始");
        assert!(sections[0].content.contains("这是第一章的内容"));
        assert_eq!(sections[1].title, "第一节");
        assert_eq!(sections[2].title, "第二章 结束");
    }
}
