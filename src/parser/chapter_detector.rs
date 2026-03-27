//! 章节检测器

use super::scorer::{ChapterScore, ScoringFactors, ScoreCalculator};
use once_cell::sync::Lazy;

static VOLUME_RE: Lazy<regex::Regex> = Lazy::new(|| {
    regex::Regex::new(r"^(第[0-9一二三四五六七八九十零〇百千万两]+[卷部]|[卷部][0-9一二三四五六七八九十零〇百千万两]+)")
        .expect("卷标题正则编译失败")
});

static CHAPTER_RE: Lazy<regex::Regex> = Lazy::new(|| {
    regex::Regex::new(r"^(第[0-9一二三四五六七八九十零〇百千万两]+[章回节]|[Cc][Hh][Aa][Pp][Tt][Ee][Rr]\s*\d+)")
        .expect("章节标题正则编译失败")
});

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ChapterMatchResult {
    pub is_match: bool,
    pub score: ChapterScore,
    pub match_type: MatchType,
}

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum MatchType {
    Volume,
    Chapter,
    SubChapter,
    Part,
    None,
}

pub struct ChapterDetector {
    calculator: ScoreCalculator,
    volume_threshold: f32,
}

impl ChapterDetector {
    pub fn new() -> Self {
        Self {
            calculator: ScoreCalculator::new(),
            volume_threshold: 0.50,
        }
    }

    pub fn detect_chapter(
        &self,
        text: &str,
        line_num: usize,
        lines: &[&str],
        custom_pattern: Option<&str>,
    ) -> Option<ChapterMatchResult> {
        let trimmed = text.trim();

        let has_chapter_format = if let Some(pattern) = custom_pattern {
            regex::Regex::new(pattern).map(|re| re.is_match(trimmed)).unwrap_or(false)
        } else {
            CHAPTER_RE.is_match(trimmed)
        };

        if !has_chapter_format {
            return None;
        }

        if line_num > 0 && line_num < lines.len() {
            let prev_line = lines[line_num - 1].trim();
            if !prev_line.is_empty() {
                let sentence_endings = ['。', '！', '？', '.', '!', '?', '"', '”'];
                if !prev_line.ends_with(sentence_endings) {
                    return None;
                }
            }
        }

        let score = self.calculator.calculate_chapter_score(
            text,
            line_num,
            lines,
            custom_pattern,
            usize::MAX,
        );

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

    pub fn detect_volume(
        &self,
        text: &str,
        line_num: usize,
        lines: &[&str],
        custom_pattern: Option<&str>,
    ) -> Option<ChapterMatchResult> {
        let trimmed = text.trim();

        if !VOLUME_RE.is_match(trimmed) {
            return None;
        }

        let score = self.calculator.calculate_chapter_score(
            text,
            line_num,
            lines,
            custom_pattern,
            50,
        );

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

    #[allow(dead_code)]
    pub fn detect_subchapter(
        &self,
        text: &str,
        line_num: usize,
        lines: &[&str],
    ) -> Option<ChapterMatchResult> {
        let trimmed = text.trim();

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
            usize::MAX,
        );

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

    #[allow(dead_code)]
    pub fn detect_part(
        &self,
        text: &str,
        line_num: usize,
        lines: &[&str],
    ) -> Option<ChapterMatchResult> {
        let trimmed = text.trim();

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

    #[allow(dead_code)]
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

            if let Some(result) = self.detect_chapter(trimmed, line_num, lines, custom_pattern) {
                results.push((line_num, trimmed.to_string(), result));
            }
        }

        results
    }

    #[allow(dead_code)]
    pub fn set_scoring_factors(&mut self, factors: ScoringFactors) {
        self.calculator.set_factors(factors);
    }

    #[allow(dead_code)]
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

        assert!(result.is_none() || !result.unwrap().is_match);
    }

    #[test]
    fn test_detect_all_chapters() {
        let detector = ChapterDetector::new();
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

        assert_eq!(results.len(), 3);

        let line_numbers: Vec<usize> = results.iter().map(|r| r.0).collect();
        assert!(line_numbers.contains(&2));
        assert!(line_numbers.contains(&5));
        assert!(line_numbers.contains(&8));
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
        let long_chapter = "第1529章 六爻点龙入门根基，天子望气登峰造极";
        let lines = vec!["", long_chapter, ""];

        let result = detector.detect_chapter(long_chapter, 1, &lines, None);

        assert!(result.is_some());
        assert!(result.unwrap().is_match);
    }

    #[test]
    fn test_detect_wan_chapter() {
        let detector = ChapterDetector::new();
        let lines = vec!["", "第一万章 大结局", ""];

        let result = detector.detect_chapter("第一万章 大结局", 1, &lines, None);

        assert!(result.is_some());
        assert!(result.unwrap().is_match);
    }
}
