//! TXT 文本解析器模块
//!
//! 将 TXT 文件解析为章节结构

use crate::error::Result;
use crate::model::{Book, Section};
use crate::utils::encoding::{detect_and_convert, ensure_no_bom};
use crate::utils::regex::RegexCache;
use std::fs;

/// 解析器结构体
pub struct Parser {
    book: Book,
    regex_cache: RegexCache,
}

impl Parser {
    /// 创建新的解析器
    pub fn new(book: Book) -> Self {
        Self {
            book,
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

    /// 解析文本内容
    fn parse_content(&mut self, content: &str) -> Result<Vec<Section>> {
        let mut sections = Vec::new();
        let mut current_section = Section::default();

        for line in content.lines() {
            let trimmed = line.trim();

            // 跳过空行
            if trimmed.is_empty() {
                continue;
            }

            // 检查是否是卷标题
            if self.is_volume(trimmed)? {
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
            if self.is_chapter(trimmed)? && !self.is_excluded(trimmed)? {
                // 保存当前章节
                if !current_section.title.is_empty() {
                    sections.push(std::mem::take(&mut current_section));
                }

                // 创建新章节（确保标题无 BOM）
                current_section.title = ensure_no_bom(trimmed);
                current_section.content = String::new();
                continue;
            }

            // 添加内容到当前章节
            let paragraph = self.build_paragraph(trimmed);
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

    /// 检查文本是否是卷标题
    fn is_volume(&mut self, text: &str) -> Result<bool> {
        let pattern = self.book.volume_match.as_deref();
        self.regex_cache.is_volume(text, pattern)
    }

    /// 检查文本是否是章节标题
    fn is_chapter(&mut self, text: &str) -> Result<bool> {
        // 检查标题长度
        if text.len() > self.book.max_title_length {
            return Ok(false);
        }

        let pattern = self.book.chapter_match.as_deref();
        self.regex_cache.is_chapter(text, pattern)
    }

    /// 检查文本是否应该被排除
    fn is_excluded(&mut self, text: &str) -> Result<bool> {
        let pattern = self.book.exclusion_pattern.as_deref();
        self.regex_cache.is_excluded(text, pattern)
    }

    /// 构建段落 HTML
    fn build_paragraph(&self, text: &str) -> String {
        // 生成缩进
        let indent = " ".repeat(self.book.indent);

        // 清理 HTML 标签并移除 BOM
        let cleaned = ensure_no_bom(text.trim());

        // 添加段落标签
        format!("<p>{}{}</p>", indent, cleaned)
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
    fn test_build_paragraph() {
        let book = Book {
            filename: PathBuf::from("test.txt"),
            indent: 2,
            ..Default::default()
        };
        let parser = Parser::new(book);
        let paragraph = parser.build_paragraph("这是一个段落");
        assert_eq!(paragraph, "<p>  这是一个段落</p>");
    }
}
