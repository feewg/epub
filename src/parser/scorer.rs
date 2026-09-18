use std::collections::HashSet;

/// 句末标点集合：前行守卫与评分共用，保证两者判定一致。
pub(crate) const SENTENCE_ENDINGS: [char; 13] = [
    '。', '！', '？', '.', '!', '?', '"', '”', '…', '」', '』', '）', ')',
];

#[derive(Debug, Clone)]
pub struct ScoringFactors {
    pub regex_weight: f32,
    pub line_position_weight: f32,
    pub length_weight: f32,
    pub context_weight: f32,
    pub format_weight: f32,
    pub min_threshold: f32,
}

impl Default for ScoringFactors {
    fn default() -> Self {
        Self {
            regex_weight: 0.30,
            line_position_weight: 0.40,
            length_weight: 0.10,
            context_weight: 0.10,
            format_weight: 0.10,
            min_threshold: 0.50,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ChapterScore {
    pub regex_score: f32,
    pub line_position_score: f32,
    pub length_score: f32,
    pub context_score: f32,
    pub format_score: f32,
    pub total_score: f32,
}

impl ChapterScore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn calculate_total(&mut self, factors: &ScoringFactors) {
        self.total_score = self.regex_score * factors.regex_weight
            + self.line_position_score * factors.line_position_weight
            + self.length_score * factors.length_weight
            + self.context_score * factors.context_weight
            + self.format_score * factors.format_weight;
    }

    pub fn passes_threshold(&self, factors: &ScoringFactors) -> bool {
        self.total_score >= factors.min_threshold
    }
}

pub struct ScoreCalculator {
    factors: ScoringFactors,
    chapter_prefixes: HashSet<String>,
    #[allow(dead_code)]
    volume_prefixes: HashSet<String>,
    punctuation: HashSet<char>,
    number_patterns: Vec<regex::Regex>,
}

impl ScoreCalculator {
    pub fn new() -> Self {
        let mut chapter_prefixes = HashSet::new();
        for i in 0..=1000 {
            chapter_prefixes.insert(format!("第{}章", i));
            chapter_prefixes.insert(format!("第{}节", i));
        }

        let cn_numbers = [
            "零", "一", "二", "三", "四", "五", "六", "七", "八", "九", "十",
        ];
        for num in cn_numbers.iter() {
            chapter_prefixes.insert(format!("第{}章", num));
        }

        let mut volume_prefixes = HashSet::new();
        for i in 0..=100 {
            volume_prefixes.insert(format!("第{}卷", i));
            volume_prefixes.insert(format!("第{}部", i));
        }

        let mut punctuation = HashSet::new();
        for c in [
            '，', '。', '！', '？', '；', '：', ',', '.', '!', '?', ';', ':', ' ',
        ]
        .iter()
        {
            punctuation.insert(*c);
        }

        let number_patterns: Vec<regex::Regex> = vec![
            r"第\s*\d+\s*章",
            r"第\s*[一二三四五六七八九十零〇百千两]+\s*章",
            r"Chapter\s*\d+",
            r"\d+\.\s*\S+",
            r"\[\d+\]\s*\S+",
        ]
        .into_iter()
        .filter_map(|p| regex::Regex::new(p).ok())
        .collect();

        Self {
            factors: ScoringFactors::default(),
            chapter_prefixes,
            volume_prefixes,
            punctuation,
            number_patterns,
        }
    }

    pub fn score_regex_match(&self, text: &str, pattern: Option<&str>) -> f32 {
        if let Some(pattern) = pattern {
            // 复用章节检测器的正则缓存，避免评分时反复编译同一模式
            if let Ok(re) = super::chapter_detector::compile_custom_pattern(pattern) {
                if re.is_match(text) {
                    return 1.0;
                }
            }
            return 0.0;
        }

        for prefix in &self.chapter_prefixes {
            if text.starts_with(prefix) {
                if text == prefix {
                    return 0.9;
                } else {
                    return 0.3;
                }
            }
        }

        for re in &self.number_patterns {
            if re.is_match(text) {
                let trimmed = text.trim();
                if trimmed.starts_with("第") && trimmed.contains("章") {
                    return 0.7;
                }
                if trimmed.starts_with("Chapter") || trimmed.starts_with("chapter") {
                    return 0.7;
                }
                return 0.2;
            }
        }

        self.score_default_rule(text.trim())
    }

    /// 与 `model::DEFAULT_CHAPTER_MATCH` 对齐的默认形态评分：
    /// 覆盖 序章/楔子/引子/番外/章节目录/最终章/完本感言/Section/Page/Chapter（含大小写混合）/数字章 等。
    ///
    /// 数字行刻意给低分（0.2）：独立成章的数字标题可依靠位置/上下文得分通过阈值，
    /// 而夹在正文流中的数字噪声（如论坛残留的编号）会因缺少空行分隔被拒绝。
    fn score_default_rule(&self, text: &str) -> f32 {
        let count = text.chars().count();

        // 独立数字行：^\d{1,4}$ 或 ^\d+、$
        let digits = text.strip_suffix('、').unwrap_or(text);
        if !digits.is_empty() && digits.bytes().all(|b| b.is_ascii_digit()) {
            return if digits.len() <= 4 || text.ends_with('、') {
                0.2
            } else {
                0.0
            };
        }

        if text == "引子" || text == "楔子" || text == "序章" || text == "番外" {
            return 0.9;
        }

        // Chapter/Section/Page：大小写不敏感（保留旧检测器行为），整行长度与默认正则一致
        for marker in ["Chapter", "Section", "Page"] {
            let prefix = text.get(..marker.len());
            if let Some(prefix) = prefix {
                if prefix.eq_ignore_ascii_case(marker) && count <= marker.len() + 21 {
                    return 0.7;
                }
            }
        }

        if text.starts_with("最终章") && count <= 24 {
            return 0.7;
        }

        if text.starts_with("完本感言") && count <= 8 {
            return 0.9;
        }

        if text.starts_with("序章") || text.starts_with("番外") || text.starts_with("章节") {
            return 0.3;
        }

        // 第N回/集/幕/卷/部（第N章/节 已由前缀扫描覆盖）
        let chars: Vec<char> = text.chars().collect();
        if let Some((marker, _)) = super::chapter_detector::leading_chapter_marker(&chars) {
            if "回集幕卷部".contains(marker) {
                return 0.3;
            }
        }

        0.0
    }

    pub fn score_line_position(&self, _current_line: &str, line_num: usize, lines: &[&str]) -> f32 {
        let mut score: f32 = 0.0;

        if line_num > 0 && line_num < lines.len() {
            let prev_line = lines[line_num - 1].trim();
            if prev_line.is_empty() {
                score += 0.6;
            } else {
                if prev_line.ends_with(SENTENCE_ENDINGS) {
                    score += 0.4;
                } else {
                    return 0.0;
                }
            }
        } else {
            score += 0.4;
        }

        if line_num + 1 < lines.len() {
            let next_line = lines[line_num + 1].trim();
            if next_line.is_empty() {
                score += 0.4;
            }
        }

        score.min(1.0)
    }

    /// 标题长度评分：超过 `max_title_length` 的行给诊断性低分（检测层会直接拒绝该行）。
    pub fn score_length(&self, text: &str, max_title_length: usize) -> f32 {
        let len = text.trim().chars().count();

        if len < 2 {
            return 0.0;
        }

        if len > max_title_length {
            return 0.2;
        }

        if (3..=20).contains(&len) {
            1.0
        } else {
            0.8
        }
    }

    pub fn score_context(&self, _current_line: &str, line_num: usize, lines: &[&str]) -> f32 {
        let mut score: f32 = 0.0;
        let mut empty_before = 0;
        let mut empty_after = 0;

        if line_num > 0 && line_num < lines.len() {
            for i in (0..line_num).rev() {
                if lines[i].trim().is_empty() {
                    empty_before += 1;
                } else {
                    break;
                }
            }
        }

        if line_num + 1 < lines.len() {
            for line in &lines[(line_num + 1)..] {
                if line.trim().is_empty() {
                    empty_after += 1;
                } else {
                    break;
                }
            }
        }

        if empty_before >= 1 && empty_after >= 1 {
            score += 0.7;
        } else if empty_before >= 1 || empty_after >= 1 {
            score += 0.4;
        }

        if line_num > 0 && line_num < lines.len() {
            let prev_len = lines[line_num - 1].trim().len();
            if prev_len > 50 {
                score += 0.3;
            }
        }

        if line_num + 1 < lines.len() {
            let next_len = lines[line_num + 1].trim().len();
            if next_len > 50 {
                score += 0.3;
            }
        }

        score.min(1.0)
    }

    pub fn score_format(&self, text: &str) -> f32 {
        let mut score: f32 = 0.0;
        let trimmed = text.trim();

        let starts_with_chapter = trimmed.starts_with("第")
            || trimmed.starts_with("卷")
            || trimmed.starts_with("部")
            || trimmed.starts_with("Part")
            || trimmed.starts_with("Chapter")
            || (trimmed.len() > 2 && trimmed.chars().next().unwrap().is_ascii_digit());

        if starts_with_chapter {
            score += 0.5;
        }

        let starts_with_pronoun = trimmed.starts_with("这")
            || trimmed.starts_with("那")
            || trimmed.starts_with("我")
            || trimmed.starts_with("你")
            || trimmed.starts_with("他")
            || trimmed.starts_with("她")
            || trimmed.starts_with("它");

        if starts_with_pronoun {
            score -= 0.5;
        }

        if trimmed.starts_with("是") {
            score -= 0.5;
        }

        let punct_count = trimmed
            .chars()
            .filter(|c| self.punctuation.contains(c))
            .count();
        let punct_ratio = punct_count as f32 / trimmed.chars().count() as f32;
        if punct_ratio < 0.3 {
            score += 0.3;
        } else {
            score -= 0.3;
        }

        let len = trimmed.len();
        if (3..=20).contains(&len) {
            score += 0.2;
        } else if len > 30 {
            score -= 0.2;
        }

        if trimmed.contains("第")
            && (trimmed.contains("章")
                || trimmed.contains("节")
                || trimmed.contains("卷")
                || trimmed.contains("部"))
        {
            score += 0.1;
        }

        if trimmed.contains('?') || trimmed.contains('！') || trimmed.contains('？') {
            score -= 0.2; // 疑问句/感叹句不应该是章节标题，扣分
        }

        if trimmed.contains('：') || trimmed.contains(':') {
            score += 0.1;
        }

        score.clamp(0.0, 1.0)
    }

    pub fn calculate_chapter_score(
        &self,
        text: &str,
        line_num: usize,
        lines: &[&str],
        pattern: Option<&str>,
        max_title_length: usize,
    ) -> ChapterScore {
        let mut score = ChapterScore::new();

        score.regex_score = self.score_regex_match(text, pattern);
        score.line_position_score = self.score_line_position(text, line_num, lines);
        score.length_score = self.score_length(text, max_title_length);
        score.context_score = self.score_context(text, line_num, lines);
        score.format_score = self.score_format(text);

        score.calculate_total(&self.factors);

        score
    }

    #[allow(dead_code)]
    pub fn set_factors(&mut self, factors: ScoringFactors) {
        self.factors = factors;
    }

    pub fn factors(&self) -> &ScoringFactors {
        &self.factors
    }
}

impl Default for ScoreCalculator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_score_calculator_creation() {
        let calc = ScoreCalculator::new();
        assert_eq!(calc.factors().min_threshold, 0.50);
    }

    #[test]
    fn test_score_regex_match() {
        let calc = ScoreCalculator::new();

        assert_eq!(calc.score_regex_match("第1章", None), 0.9);
        assert_eq!(calc.score_regex_match("第一章", None), 0.9);
        assert_eq!(calc.score_regex_match("Chapter 1", None), 0.7);
        assert_eq!(calc.score_regex_match("第1章 开始", None), 0.3);
        assert_eq!(calc.score_regex_match("普通段落", None), 0.0);
    }

    #[test]
    fn test_score_length() {
        let calc = ScoreCalculator::new();

        assert_eq!(calc.score_length("第1章", 35), 1.0);
        assert_eq!(calc.score_length("章", 35), 0.0);

        let long_text =
            "这个标题非常长非常长非常长非常长非常长非常长非常长非常长非常长非常长非常长";
        assert!(calc.score_length(long_text, 35) > 0.0);

        assert_eq!(calc.score_length("第一", 35), 0.8);
    }

    #[test]
    fn test_score_format() {
        let calc = ScoreCalculator::new();

        assert!(calc.score_format("第1章 开始？") > 0.5);
        assert!(calc.score_format("这是一个完整的句子。") < calc.score_format("第1章"));
    }

    #[test]
    fn test_calculate_chapter_score() {
        let calc = ScoreCalculator::new();
        let lines = vec!["", "第1章 开始", ""];

        let score = calc.calculate_chapter_score("第1章 开始", 1, &lines, None, 35);

        assert!(score.total_score > 0.6);
        assert!(score.regex_score >= 0.3);
        assert!(score.line_position_score > 0.5);
        assert!(score.format_score > 0.0);
    }

    #[test]
    fn test_chapter_score_passes_threshold() {
        let factors = ScoringFactors::default();
        let mut score = ChapterScore::new();

        score.total_score = 0.4;
        assert!(!score.passes_threshold(&factors));

        score.total_score = 0.6;
        assert!(score.passes_threshold(&factors));
    }
}
