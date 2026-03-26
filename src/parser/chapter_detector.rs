//! 章节检测器
//!
//! 提供智能的章节和卷识别功能

use super::scorer::{ChapterScore, ScoringFactors, ScoreCalculator};

/// 章节匹配结果
#[derive(Debug, Clone)]
pub struct ChapterMatchResult {
    /// 是否匹配
    pub is_match: bool,
    /// 匹配得分
    pub score: ChapterScore,
    /// 匹配类型
    pub match_type: MatchType,
}

/// 匹配类型
#[derive(Debug, Clone, PartialEq)]
pub enum MatchType {
    /// 卷标题
    Volume,
    /// 章节标题
    Chapter,
    /// 子章节
    SubChapter,
    /// 部分
    Part,
    /// 不匹配
    None,
}

/// 章节检测器
pub struct ChapterDetector {
    calculator: ScoreCalculator,
    /// 卷的评分阈值（比章节低一些）
    volume_threshold: f32,
}

impl ChapterDetector {
    /// 创建新的章节检测器
    pub fn new() -> Self {
        Self {
            calculator: ScoreCalculator::new(),
            volume_threshold: 0.50, // 卷标题识别阈值稍低
        }
    }

    /// 检测是否是章节标题
    pub fn detect_chapter(
        &self,
        text: &str,
        line_num: usize,
        lines: &[&str],
        custom_pattern: Option<&str>,
    ) -> Option<ChapterMatchResult> {
        // 硬性检查：章节标题必须独立成行（前面必须有空行）
        if line_num > 0 {
            let prev_line = lines[line_num - 1].trim();
            if !prev_line.is_empty() {
                // 前面没有空行，不是章节标题
                return None;
            }
        }

        // 硬性检查：排除明显是普通句子的文本
        let trimmed = text.trim();

        // 检查是否以常见句子模式开头（这些不太可能是章节标题）
        let sentence_starters = [
            "这", "那", "我", "你", "他", "她", "它",
            "是", "有", "没", "不", "在", "从", "到",
            "因为", "所以", "但是", "而且", "不过",
            "虽然", "尽管", "如果", "假如", "要是",
            "所以", "然后", "接着", "之后",
        ];

        let starts_with_sentence = sentence_starters.iter()
            .any(|starter| trimmed.starts_with(starter));

        if starts_with_sentence && !custom_pattern.is_some() {
            // 以常见句子词开头，且没有自定义模式，很可能是普通句子
            // 直接返回 None，不需要进一步检查
            return None;
        }

        // 计算评分
        let score = self.calculator.calculate_chapter_score(
            text,
            line_num,
            lines,
            custom_pattern,
            35, // 默认最大标题长度
        );

        // 判断是否达到阈值
        let is_match = score.passes_threshold(self.calculator.factors());

        if is_match {
            Some(ChapterMatchResult {
                is_match: true,
                score,
                match_type: MatchType::Chapter,
            })
        } else {
            None
        }
    }

    /// 检测是否是卷标题
    pub fn detect_volume(
        &self,
        text: &str,
        line_num: usize,
        lines: &[&str],
        custom_pattern: Option<&str>,
    ) -> Option<ChapterMatchResult> {
        let trimmed = text.trim();

        // 快速检查：包含"卷"或"部"关键词
        if !trimmed.contains("卷") && !trimmed.contains("部") {
            return None;
        }

        // 计算评分（使用卷的阈值）
        let score = self.calculator.calculate_chapter_score(
            text,
            line_num,
            lines,
            custom_pattern,
            50, // 卷标题通常比章节标题长
        );

        // 判断是否达到卷的阈值
        let is_match = score.total_score >= self.volume_threshold;

        if is_match {
            Some(ChapterMatchResult {
                is_match: true,
                score,
                match_type: MatchType::Volume,
            })
        } else {
            None
        }
    }

    /// 检测是否是子章节
    pub fn detect_subchapter(
        &self,
        text: &str,
        line_num: usize,
        lines: &[&str],
    ) -> Option<ChapterMatchResult> {
        let trimmed = text.trim();

        // 子章节通常使用缩进或数字编号
        let has_prefix = trimmed.starts_with("  ")
            || trimmed.starts_with("\t")
            || regex::Regex::new(r"^\d+\.\d+").unwrap().is_match(trimmed);

        if !has_prefix {
            return None;
        }

        let score = self.calculator.calculate_chapter_score(
            text,
            line_num,
            lines,
            None,
            35,
        );

        // 子章节阈值稍低
        if score.total_score >= 0.45 {
            Some(ChapterMatchResult {
                is_match: true,
                score,
                match_type: MatchType::SubChapter,
            })
        } else {
            None
        }
    }

    /// 检测是否是部分标题
    pub fn detect_part(
        &self,
        text: &str,
        line_num: usize,
        lines: &[&str],
    ) -> Option<ChapterMatchResult> {
        let trimmed = text.trim();

        // 部分标题通常包含"篇"、"部分"等词
        if !trimmed.contains("篇") && !trimmed.contains("部分") {
            return None;
        }

        let score = self.calculator.calculate_chapter_score(
            text,
            line_num,
            lines,
            None,
            50,
        );

        if score.total_score >= 0.45 {
            Some(ChapterMatchResult {
                is_match: true,
                score,
                match_type: MatchType::Part,
            })
        } else {
            None
        }
    }

    /// 批量检测（返回所有可能的章节标题及其得分）
    pub fn detect_all_chapters(
        &self,
        lines: &[&str],
        custom_pattern: Option<&str>,
    ) -> Vec<(usize, String, ChapterMatchResult)> {
        let mut results = Vec::new();

        for (line_num, line) in lines.iter().enumerate() {
            let trimmed = line.trim();

            if trimmed.is_empty() {
                continue;
            }

            // 检测章节
            if let Some(result) = self.detect_chapter(trimmed, line_num, lines, custom_pattern) {
                results.push((line_num, trimmed.to_string(), result));
            }
        }

        results
    }

    /// 设置评分因素
    pub fn set_scoring_factors(&mut self, factors: ScoringFactors) {
        self.calculator.set_factors(factors);
    }

    /// 获取评分计算器
    pub fn calculator(&self) -> &ScoreCalculator {
        &self.calculator
    }
}

impl Default for ChapterDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chapter_detector_creation() {
        let detector = ChapterDetector::new();
        assert_eq!(detector.volume_threshold, 0.50);
    }

    #[test]
    fn test_detect_chapter() {
        let detector = ChapterDetector::new();
        let lines = vec!["", "第1章 开始", ""];

        let result = detector.detect_chapter("第1章 开始", 1, &lines, None);

        assert!(result.is_some());
        let result = result.unwrap();
        assert!(result.is_match);
        assert_eq!(result.match_type, MatchType::Chapter);
        assert!(result.score.total_score > 0.7);
    }

    #[test]
    fn test_detect_volume() {
        let detector = ChapterDetector::new();
        let lines = vec!["", "第一卷 开始", ""];

        let result = detector.detect_volume("第一卷 开始", 1, &lines, None);

        assert!(result.is_some());
        let result = result.unwrap();
        assert!(result.is_match);
        assert_eq!(result.match_type, MatchType::Volume);
    }

    #[test]
    fn test_detect_not_chapter() {
        let detector = ChapterDetector::new();
        let lines = vec!["这是一个普通的段落", "这是第二段", "这是第三段"];

        let result = detector.detect_chapter("这是一个普通的段落", 0, &lines, None);

        // 普通段落不应该被识别为章节
        assert!(result.is_none() || !result.unwrap().is_match);
    }

    #[test]
    fn test_detect_all_chapters() {
        let detector = ChapterDetector::new();
        // 章节标题必须前面有空行且以章节标记开头
        let lines = vec![
            "前言",
            "",
            "第1章 开始",
            "这是内容",
            "",
            "第2章 继续",
            "更多内容",
            "",
            "第3章 结束",
        ];

        let results = detector.detect_all_chapters(&lines, None);

        // 应该检测到3个章节（第1章、第2章、第3章）
        // "前言"不以"第"开头，不应该被识别为章节
        assert_eq!(results.len(), 3);

        // 验证章节位置
        let line_numbers: Vec<usize> = results.iter().map(|r| r.0).collect();
        assert!(line_numbers.contains(&2)); // 第1章
        assert!(line_numbers.contains(&5)); // 第2章
        assert!(line_numbers.contains(&8)); // 第3章
    }

    #[test]
    fn test_detect_short_chapter() {
        let detector = ChapterDetector::new();
        let lines = vec!["", "第1章", ""];

        let result = detector.detect_chapter("第1章", 1, &lines, None);

        assert!(result.is_some());
        assert!(result.unwrap().is_match);
    }

    #[test]
    fn test_detect_long_title() {
        let detector = ChapterDetector::new();
        let lines = vec!["", "这是一个非常长的标题绝对不可能是章节标题因为它太长了", ""];

        let result = detector.detect_chapter(
            "这是一个非常长的标题绝对不可能是章节标题因为它太长了",
            1,
            &lines,
            None,
        );

        // 长标题不应该被识别为章节
        assert!(result.is_none() || !result.unwrap().is_match);
    }
}
