//! 段落处理器
//!
//! 提供智能的段落处理功能

use crate::model::Book;
use crate::utils::encoding::ensure_no_bom;

/// 段落模式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[derive(Default)]
#[allow(dead_code)]
pub enum ParagraphMode {
    /// 每行独立段落
    Line,
    /// 按空行分割段落
    BlankLine,
    /// 智能判断（推荐）
    #[default]
    Smart,
}


/// 段落处理器
#[allow(dead_code)]
pub struct ParagraphProcessor {
    book: Book,
    mode: ParagraphMode,
    /// 最大段落长度
    max_paragraph_length: usize,
    /// 对话检测启用
    enable_dialogue_detection: bool,
    /// 合并阈值（行长度）
    merge_threshold: usize,
    /// 缓存的缩进字符串（避免重复创建）
    cached_indent: String,
}

impl ParagraphProcessor {
    /// 创建新的段落处理器
    pub fn new(book: Book) -> Self {
        let indent = book.indent;
        Self {
            book,
            mode: ParagraphMode::Smart,
            max_paragraph_length: 500,
            enable_dialogue_detection: true,
            merge_threshold: 30, // 短行合并阈值
            cached_indent: " ".repeat(indent),
        }
    }

    /// 设置段落模式
    #[allow(dead_code)]
    pub fn set_mode(&mut self, mode: ParagraphMode) {
        self.mode = mode;
    }

    /// 处理单个段落（返回HTML）
    pub fn process(&self, text: &str) -> String {
        if text.trim().is_empty() {
            return String::new();
        }

        let cleaned = ensure_no_bom(text.trim());

        format!("<p>{}{}</p>", self.cached_indent, cleaned)
    }

    /// 处理多行内容（智能合并）
    #[allow(dead_code)]
    pub fn process_lines(&self, lines: &[&str]) -> Vec<String> {
        match self.mode {
            ParagraphMode::Line => self.process_line_mode(lines),
            ParagraphMode::BlankLine => self.process_blank_line_mode(lines),
            ParagraphMode::Smart => self.process_smart_mode(lines),
        }
    }

    /// 行模式：每行独立成段
    #[allow(dead_code)]
    fn process_line_mode(&self, lines: &[&str]) -> Vec<String> {
        lines
            .iter()
            .filter(|line| !line.trim().is_empty())
            .map(|line| self.process(line))
            .collect()
    }

    /// 空行模式：按空行分割段落
    #[allow(dead_code)]
    fn process_blank_line_mode(&self, lines: &[&str]) -> Vec<String> {
        let mut paragraphs = Vec::new();
        let mut current_paragraph = String::new();

        for line in lines {
            let trimmed = line.trim();

            if trimmed.is_empty() {
                // 空行，结束当前段落
                if !current_paragraph.is_empty() {
                    paragraphs.push(self.process(&current_paragraph));
                    current_paragraph.clear();
                }
            } else {
                if current_paragraph.is_empty() {
                    current_paragraph.push_str(trimmed);
                } else {
                    current_paragraph.push(' ');
                    current_paragraph.push_str(trimmed);
                }
            }
        }

        // 处理最后一个段落
        if !current_paragraph.is_empty() {
            paragraphs.push(self.process(&current_paragraph));
        }

        paragraphs
    }

    /// 智能模式：智能判断段落分割
    #[allow(dead_code)]
    fn process_smart_mode(&self, lines: &[&str]) -> Vec<String> {
        let mut paragraphs = Vec::new();
        let mut current_paragraph = String::new();
        let mut previous_line_was_short = false;

        for line in lines {
            let trimmed = line.trim();

            if trimmed.is_empty() {
                // 空行，结束当前段落
                if !current_paragraph.is_empty() {
                    paragraphs.push(self.process(&current_paragraph));
                    current_paragraph.clear();
                    previous_line_was_short = false;
                }
                continue;
            }

            let line_len = trimmed.len();

            // 检查是否是对话
            let is_dialogue = self.enable_dialogue_detection && self.is_dialogue(trimmed);

            // 检查是否应该开始新段落
            let should_start_new_paragraph = self.should_start_new_paragraph(
                trimmed,
                line_len,
                &current_paragraph,
                previous_line_was_short,
                is_dialogue,
            );

            if should_start_new_paragraph {
                if !current_paragraph.is_empty() {
                    paragraphs.push(self.process(&current_paragraph));
                }
                current_paragraph.clear();
                current_paragraph.push_str(trimmed);
            } else {
                if current_paragraph.is_empty() {
                    current_paragraph.push_str(trimmed);
                } else {
                    current_paragraph.push(' ');
                    current_paragraph.push_str(trimmed);
                }
            }

            previous_line_was_short = line_len < self.merge_threshold;
        }

        // 处理最后一个段落
        if !current_paragraph.is_empty() {
            paragraphs.push(self.process(&current_paragraph));
        }

        paragraphs
    }

    /// 判断是否应该开始新段落
    #[allow(dead_code)]
    fn should_start_new_paragraph(
        &self,
        line: &str,
        line_len: usize,
        current_paragraph: &str,
        previous_line_was_short: bool,
        is_dialogue: bool,
    ) -> bool {
        // 对话总是开始新段落
        if is_dialogue {
            return true;
        }

        // 空段落总是可以开始新段落
        if current_paragraph.is_empty() {
            return false;
        }

        // 当前行是短行且上一行也是短行，合并
        if line_len < self.merge_threshold && previous_line_was_short {
            return false;
        }

        // 当前行是短行且当前段落不够长，合并
        if line_len < self.merge_threshold && current_paragraph.len() < 100 {
            return false;
        }

        // 检查当前行是否以句号、问号、感叹号结尾（可能是完整句子）
        let ends_with_punctuation = line.ends_with('。')
            || line.ends_with('！')
            || line.ends_with('？')
            || line.ends_with('.')
            || line.ends_with('!')
            || line.ends_with('?');

        // 如果当前段落已经够长，开始新段落
        if current_paragraph.len() > self.max_paragraph_length {
            return true;
        }

        // 如果当前行以标点结尾且段落已经有一定长度，开始新段落
        if ends_with_punctuation && current_paragraph.len() > 50 {
            return true;
        }

        // 检查当前行是否以特定字符开头（可能是对话或特殊格式）
        let starts_with_special = line.starts_with('「')
            || line.starts_with('『')
            || line.starts_with('"')
            || line.starts_with('"')
            || line.starts_with('"');

        if starts_with_special {
            return true;
        }

        false
    }

    /// 判断是否是对话
    #[allow(dead_code)]
    fn is_dialogue(&self, line: &str) -> bool {
        // 对话通常以引号开始
        let starts_with_quote = line.starts_with('「')
            || line.starts_with('『')
            || line.starts_with('"')
            || line.starts_with('"')
            || line.starts_with('"');

        // 对话通常包含引号
        let has_quotes = line.contains('「')
            || line.contains('』')
            || line.contains('"')
            || line.contains('"')
            || line.contains('"');

        // 对话通常较短
        let is_short = line.len() < 50;

        starts_with_quote || (has_quotes && is_short)
    }

    /// 合并短行
    #[allow(dead_code)]
    pub fn merge_short_lines(&self, lines: &[&str]) -> Vec<String> {
        let mut merged = Vec::new();
        let mut buffer = String::new();

        for line in lines {
            let trimmed = line.trim();

            if trimmed.is_empty() {
                if !buffer.is_empty() {
                    merged.push(std::mem::take(&mut buffer));
                }
                continue;
            }

            let is_short = trimmed.len() < self.merge_threshold;

            if is_short && !buffer.is_empty() {
                buffer.push(' ');
                buffer.push_str(trimmed);
            } else {
                if !buffer.is_empty() {
                    merged.push(std::mem::take(&mut buffer));
                }
                buffer.push_str(trimmed);
            }
        }

        if !buffer.is_empty() {
            merged.push(buffer);
        }

        merged
    }

    /// 设置最大段落长度
    #[allow(dead_code)]
    pub fn set_max_paragraph_length(&mut self, max_length: usize) {
        self.max_paragraph_length = max_length;
    }

    /// 设置合并阈值
    #[allow(dead_code)]
    pub fn set_merge_threshold(&mut self, threshold: usize) {
        self.merge_threshold = threshold;
    }

    /// 设置对话检测
    #[allow(dead_code)]
    pub fn set_dialogue_detection(&mut self, enabled: bool) {
        self.enable_dialogue_detection = enabled;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_paragraph_processor_creation() {
        let book = Book {
            filename: PathBuf::from("test.txt"),
            ..Default::default()
        };
        let processor = ParagraphProcessor::new(book);
        assert_eq!(processor.mode, ParagraphMode::Smart);
    }

    #[test]
    fn test_process_line() {
        let book = Book {
            filename: PathBuf::from("test.txt"),
            indent: 2,
            ..Default::default()
        };
        let processor = ParagraphProcessor::new(book);
        let result = processor.process("这是一个段落");
        assert_eq!(result, "<p>  这是一个段落</p>");
    }

    #[test]
    fn test_process_line_mode() {
        let book = Book {
            filename: PathBuf::from("test.txt"),
            indent: 2,
            ..Default::default()
        };
        let mut processor = ParagraphProcessor::new(book);
        processor.set_mode(ParagraphMode::Line);

        let lines = vec!["第一行", "第二行", "第三行"];
        let results = processor.process_lines(&lines);

        assert_eq!(results.len(), 3);
    }

    #[test]
    fn test_process_blank_line_mode() {
        let book = Book {
            filename: PathBuf::from("test.txt"),
            indent: 2,
            ..Default::default()
        };
        let mut processor = ParagraphProcessor::new(book);
        processor.set_mode(ParagraphMode::BlankLine);

        let lines = vec!["第一行", "第二行", "", "第三行"];
        let results = processor.process_lines(&lines);

        assert_eq!(results.len(), 2); // 两个段落
    }

    #[test]
    fn test_is_dialogue() {
        let book = Book {
            filename: PathBuf::from("test.txt"),
            ..Default::default()
        };
        let processor = ParagraphProcessor::new(book);

        assert!(processor.is_dialogue("「你好」"));
        assert!(processor.is_dialogue("他说道：「你好」"));
        assert!(!processor.is_dialogue("这是一个很长的普通段落内容"));
    }

    #[test]
    fn test_merge_short_lines() {
        let book = Book {
            filename: PathBuf::from("test.txt"),
            ..Default::default()
        };
        let processor = ParagraphProcessor::new(book);

        let lines = vec!["这是第一行", "这是第二行", "这是一行很长的普通段落内容"];
        let merged = processor.merge_short_lines(&lines);

        // 短行应该被合并
        assert!(merged.len() <= 3);
    }
}
