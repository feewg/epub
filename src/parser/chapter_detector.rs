//! 章节检测器

use super::scorer::{ChapterScore, ScoreCalculator, ScoringFactors, SENTENCE_ENDINGS};
use crate::error::Result;
use crate::model::{DEFAULT_CHAPTER_MATCH, DEFAULT_VOLUME_MATCH};
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Mutex;

/// 默认章节标题正则：以 `model::DEFAULT_CHAPTER_MATCH` 为准，
/// 另保留旧版检测器大小写不敏感的 Chapter 匹配能力，避免既有输入回退。
static CHAPTER_RE: Lazy<regex::Regex> = Lazy::new(|| {
    regex::Regex::new(&format!("{DEFAULT_CHAPTER_MATCH}|(?i:^Chapter\\s*\\d+)"))
        .expect("章节标题正则编译失败")
});

/// 默认卷标题正则：与 `model::DEFAULT_VOLUME_MATCH` 保持一致
static VOLUME_RE: Lazy<regex::Regex> =
    Lazy::new(|| regex::Regex::new(DEFAULT_VOLUME_MATCH).expect("卷标题正则编译失败"));

/// 自定义正则缓存上限：防止异常输入导致缓存无界增长
const CUSTOM_PATTERN_CACHE_LIMIT: usize = 64;

/// 正则缓存：避免在章节检测循环中反复编译同一正则
static CUSTOM_PATTERN_CACHE: Lazy<Mutex<HashMap<String, regex::Regex>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

/// 编译（或复用缓存的）自定义正则表达式。
///
/// 非法正则返回 `Err`，由调用方决定如何上报；不再被静默吞掉。
pub(super) fn compile_custom_pattern(pattern: &str) -> Result<regex::Regex> {
    {
        let cache = CUSTOM_PATTERN_CACHE.lock().unwrap();
        if let Some(re) = cache.get(pattern) {
            // 克隆一个 Regex（cheap operation）
            return Ok(re.clone());
        }
    }
    let re = regex::Regex::new(pattern)?;
    let mut cache = CUSTOM_PATTERN_CACHE.lock().unwrap();
    if cache.len() >= CUSTOM_PATTERN_CACHE_LIMIT {
        cache.clear();
    }
    cache.insert(pattern.to_string(), re.clone());
    Ok(re)
}

/// 分隔线字符集合（全角/半角横线、下划线、等号、星号、波浪线等）
const SEPARATOR_CHARS: &[char] = &[
    '-', '－', '−', '–', '‒', '―', '—', '_', '＝', '=', '＊', '*', '～', '~',
];

/// 分隔线最少字符数：过短的行（如 "***"）可能是普通正文，不作分隔线处理
const MIN_SEPARATOR_LEN: usize = 4;

/// 判断某行是否是纯分隔线（如 `----——`、`====`、`*****`、`____`）
pub(super) fn is_separator_line(trimmed: &str) -> bool {
    let count = trimmed.chars().count();
    count >= MIN_SEPARATOR_LEN && trimmed.chars().all(|c| SEPARATOR_CHARS.contains(&c))
}

/// 判断分隔线是否“生效”：
/// - 与上一段正文之间存在空行（含文档开头）时生效；
/// - 紧贴未完段（不以句末标点收尾）时视为段内伪分隔，保留为正文；
/// - 连续分隔线向前追溯时彼此跳过。
fn separator_is_active(lines: &[&str], idx: usize) -> bool {
    let mut j = idx;
    while j > 0 {
        j -= 1;
        let t = lines[j].trim();
        if t.is_empty() {
            return true;
        }
        if is_separator_line(t) {
            continue;
        }
        return t.ends_with(SENTENCE_ENDINGS);
    }
    true
}

/// 将生效的分隔线替换为空行：
/// - 分隔线自身不进入正文；
/// - 其后紧邻的章节标题在前行守卫与评分中获得与空行一致的上下文。
pub(super) fn blank_separator_lines(lines: &[&str]) -> Vec<String> {
    lines
        .iter()
        .enumerate()
        .map(|(idx, line)| {
            let trimmed = line.trim();
            if !trimmed.is_empty() && is_separator_line(trimmed) && separator_is_active(lines, idx)
            {
                String::new()
            } else {
                (*line).to_string()
            }
        })
        .collect()
}

/// “第…”标记中的数字部分（与 model 默认正则保持一致，含空格与万位）
const MARKER_NUMBER_CHARS: &str = "0123456789一二三四五六七八九十零〇百千万两 ";

/// 章节/卷标记字符
const MARKER_CHARS: &str = "章回节集幕卷部";

/// 若行首是“第…[章回节集幕卷部]”标记，返回 (标记字符, 标记之后的字符下标)。
pub(super) fn leading_chapter_marker(chars: &[char]) -> Option<(char, usize)> {
    if chars.first() != Some(&'第') {
        return None;
    }
    let mut i = 1;
    while i < chars.len() && MARKER_NUMBER_CHARS.contains(chars[i]) {
        i += 1;
    }
    let marker = *chars.get(i)?;
    if MARKER_CHARS.contains(marker) {
        Some((marker, i + 1))
    } else {
        None
    }
}

fn is_cjk_ideograph(c: char) -> bool {
    matches!(c as u32, 0x3400..=0x4DBF | 0x4E00..=0x9FFF)
}

/// 判断“第N章/卷…”标记后是否直接粘连汉字正文（如“第一章鱼很好吃”“第一卷是开头”）。
///
/// 这类行按默认规则不视为标题；用户显式提供的 chapter_match/volume_match 不受此限制。
fn has_glued_cjk_after_marker(trimmed: &str) -> bool {
    let chars: Vec<char> = trimmed.chars().collect();
    match leading_chapter_marker(&chars) {
        Some((_, after)) => chars.get(after).is_some_and(|&c| is_cjk_ideograph(c)),
        None => false,
    }
}

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
    /// 标题最大长度（按字符计）：超过该长度的行不视为标题（保留为正文）
    max_title_length: usize,
}

impl ChapterDetector {
    pub fn new() -> Self {
        Self {
            calculator: ScoreCalculator::new(),
            volume_threshold: 0.50,
            // 与 model::Book 的默认 max_title_length 保持一致
            max_title_length: 35,
        }
    }

    /// 设置标题最大长度（字符数）
    pub fn with_max_title_length(mut self, max_title_length: usize) -> Self {
        self.max_title_length = max_title_length;
        self
    }

    /// 检测章节标题（可失败版本）：非法的自定义正则返回 `Err` 而非静默忽略。
    pub fn try_detect_chapter(
        &self,
        text: &str,
        line_num: usize,
        lines: &[&str],
        custom_pattern: Option<&str>,
    ) -> Result<Option<ChapterMatchResult>> {
        let trimmed = text.trim();

        let has_chapter_format = if let Some(pattern) = custom_pattern {
            compile_custom_pattern(pattern)?.is_match(trimmed)
        } else {
            CHAPTER_RE.is_match(trimmed) && !has_glued_cjk_after_marker(trimmed)
        };

        if !has_chapter_format {
            return Ok(None);
        }

        // 超长标题按配置直接拒绝（内容保留为正文）
        if trimmed.chars().count() > self.max_title_length {
            return Ok(None);
        }

        if line_num > 0 && line_num < lines.len() {
            let prev_line = lines[line_num - 1].trim();
            if !prev_line.is_empty() && !prev_line.ends_with(SENTENCE_ENDINGS) {
                return Ok(None);
            }
        }

        let score = self.calculator.calculate_chapter_score(
            text,
            line_num,
            lines,
            custom_pattern,
            self.max_title_length,
        );

        let is_match = score.passes_threshold(self.calculator.factors());

        Ok(if is_match {
            Some(ChapterMatchResult {
                is_match: true,
                score,
                match_type: MatchType::Chapter,
            })
        } else {
            None
        })
    }

    /// 检测章节标题（兼容旧 API：非法正则按未匹配处理）
    pub fn detect_chapter(
        &self,
        text: &str,
        line_num: usize,
        lines: &[&str],
        custom_pattern: Option<&str>,
    ) -> Option<ChapterMatchResult> {
        self.try_detect_chapter(text, line_num, lines, custom_pattern)
            .ok()
            .flatten()
    }

    /// 检测卷标题（可失败版本）：非法的自定义正则返回 `Err` 而非静默忽略。
    pub fn try_detect_volume(
        &self,
        text: &str,
        line_num: usize,
        lines: &[&str],
        custom_pattern: Option<&str>,
    ) -> Result<Option<ChapterMatchResult>> {
        let trimmed = text.trim();

        // 使用自定义正则或内置正则
        let has_volume_format = if let Some(pattern) = custom_pattern {
            compile_custom_pattern(pattern)?.is_match(trimmed)
        } else {
            VOLUME_RE.is_match(trimmed) && !has_glued_cjk_after_marker(trimmed)
        };

        if !has_volume_format {
            return Ok(None);
        }

        // 卷标题同样受最大长度约束
        if trimmed.chars().count() > self.max_title_length {
            return Ok(None);
        }

        let score = self.calculator.calculate_chapter_score(
            text,
            line_num,
            lines,
            custom_pattern,
            self.max_title_length,
        );

        let is_match = score.total_score >= self.volume_threshold;

        Ok(if is_match {
            Some(ChapterMatchResult {
                is_match: true,
                score,
                match_type: MatchType::Volume,
            })
        } else {
            None
        })
    }

    /// 检测卷标题（兼容旧 API：非法正则按未匹配处理）
    pub fn detect_volume(
        &self,
        text: &str,
        line_num: usize,
        lines: &[&str],
        custom_pattern: Option<&str>,
    ) -> Option<ChapterMatchResult> {
        self.try_detect_volume(text, line_num, lines, custom_pattern)
            .ok()
            .flatten()
    }

    #[allow(dead_code)]
    pub fn detect_subchapter(
        &self,
        text: &str,
        line_num: usize,
        lines: &[&str],
    ) -> Option<ChapterMatchResult> {
        static SUB_RE: Lazy<regex::Regex> = Lazy::new(|| regex::Regex::new(r"^\d+\.\d+").unwrap());

        let trimmed = text.trim();

        let has_prefix =
            trimmed.starts_with("  ") || trimmed.starts_with('\t') || SUB_RE.is_match(trimmed);

        if !has_prefix {
            return None;
        }

        if trimmed.chars().count() > self.max_title_length {
            return None;
        }

        let score = self.calculator.calculate_chapter_score(
            text,
            line_num,
            lines,
            None,
            self.max_title_length,
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

        if trimmed.chars().count() > self.max_title_length {
            return None;
        }

        let score = self.calculator.calculate_chapter_score(
            text,
            line_num,
            lines,
            None,
            self.max_title_length,
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
