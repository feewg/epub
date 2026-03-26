//! 解析器模块
//!
//! 提供智能的文本解析能力，包括章节识别、段落处理等

mod chapter_detector;
mod paragraph_processor;
mod scorer;

pub use chapter_detector::ChapterDetector;
pub use paragraph_processor::{ParagraphProcessor, ParagraphMode};
pub use scorer::ScoreCalculator;

use crate::error::{KafError, Result};
use crate::model::{Book, Section};
use crate::utils::encoding::{detect_and_convert, ensure_no_bom};
use crate::utils::regex::RegexCache;
use std::fs;
use std::io::{BufRead, BufReader, Cursor};

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

    /// 解析 TXT 文件
    pub fn parse(&mut self) -> Result<Vec<Section>> {
        // 1. 读取文件
        let bytes = fs::read(&self.book.filename)?;

        // 2. 检测并转换编码
        let content = detect_and_convert(&bytes)?;

        // 3. 解析内容
        self.parse_content(&content)
    }

    /// 流式解析 TXT 文件（适用于大文件）
    ///
    /// 此方法逐行读取文件，避免一次性加载整个文件到内存，
    /// 适用于处理大型文本文件。
    pub fn parse_streaming(&mut self) -> Result<Vec<Section>> {
        // 1. 读取文件并检测编码
        let bytes = fs::read(&self.book.filename)?;
        let content = detect_and_convert(&bytes)?;

        // 2. 使用 Cursor 进行流式读取
        let cursor = Cursor::new(content);
        let reader = BufReader::new(cursor);

        // 3. 流式解析内容
        self.parse_content_streaming(reader)
    }

    /// 解析文本内容（流式版本）
    fn parse_content_streaming<R: BufRead>(&mut self, reader: R) -> Result<Vec<Section>> {
        let mut sections = Vec::new();
        let mut current_section = Section::default();
        let mut lines_cache: Vec<String> = Vec::new();
        let mut line_num: usize = 0;

        // 预读取缓冲区（用于上下文判断）
        const LOOKAHEAD_LINES: usize = 3;
        let mut buffer_lines: Vec<String> = Vec::with_capacity(LOOKAHEAD_LINES);

        for line_result in reader.lines() {
            let line = line_result.map_err(|e| {
                KafError::Io(std::io::Error::new(std::io::ErrorKind::InvalidData, e))
            })?;

            let trimmed = line.trim();
            lines_cache.push(line.clone());
            line_num += 1;

            // 维护 lookahead 缓冲区
            if !trimmed.is_empty() {
                buffer_lines.push(trimmed.to_string());
                if buffer_lines.len() > LOOKAHEAD_LINES {
                    buffer_lines.remove(0);
                }
            }

            // 转换为 &str 引用以供检测使用
            let lines_refs: Vec<&str> = buffer_lines.iter().map(|s| s.as_str()).collect();

            // 跳过空行
            if trimmed.is_empty() {
                continue;
            }

            // 检查是否是卷标题
            if self.chapter_detector.detect_volume(
                trimmed,
                line_num,
                &lines_refs,
                self.book.volume_match.as_deref()
            ).is_some() {
                // 保存当前章节
                if !current_section.title.is_empty() {
                    sections.push(std::mem::take(&mut current_section));
                }

                // 创建新卷（确保标题无 BOM）
                current_section.title = ensure_no_bom(trimmed);
                current_section.content = String::new();
                continue;
            }

            // 检查是否是章节标题
            if self.chapter_detector.detect_chapter(
                trimmed,
                line_num,
                &lines_refs,
                self.book.chapter_match.as_deref()
            ).is_some() {
                // 检查是否被排除
                if !self.is_excluded(trimmed)? {
                    // 保存当前章节
                    if !current_section.title.is_empty() {
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

    /// 解析文本内容
    fn parse_content(&mut self, content: &str) -> Result<Vec<Section>> {
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
            if self.chapter_detector.detect_volume(
                trimmed,
                line_num,
                &lines,
                self.book.volume_match.as_deref()
            ).is_some() {
                // 保存当前章节
                if !current_section.title.is_empty() {
                    sections.push(std::mem::take(&mut current_section));
                }

                // 创建新卷（确保标题无 BOM）
                current_section.title = ensure_no_bom(trimmed);
                current_section.content = String::new();
                continue;
            }

            // 检查是否是章节标题
            if self.chapter_detector.detect_chapter(
                trimmed,
                line_num,
                &lines,
                self.book.chapter_match.as_deref()
            ).is_some() {
                // 检查是否被排除
                if !self.is_excluded(trimmed)? {
                    // 保存当前章节
                    if !current_section.title.is_empty() {
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
    fn test_parse_content() {
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
        let sections = parser.parse_content(content).unwrap();

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
        let sections = parser.parse_content(content).unwrap();

        assert_eq!(sections.len(), 4);
        assert_eq!(sections[0].title, "第一卷 开始");
        assert_eq!(sections[1].title, "第一章 开端");
        assert_eq!(sections[2].title, "第二卷 发展");
        assert_eq!(sections[3].title, "第二章 延续");
    }
}
