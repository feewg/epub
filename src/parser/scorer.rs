//! 章节评分机制
//!
//! 为章节识别提供评分系统，提高识别准确率

use std::collections::HashSet;

/// 评分因素配置
#[derive(Debug, Clone)]
pub struct ScoringFactors {
    /// 正则匹配得分权重
    pub regex_weight: f32,
    /// 独立成行得分权重
    pub line_position_weight: f32,
    /// 长度得分权重
    pub length_weight: f32,
    /// 上下文得分权重
    pub context_weight: f32,
    /// 格式得分权重
    pub format_weight: f32,
    /// 最小识别阈值
    pub min_threshold: f32,
}

impl Default for ScoringFactors {
    fn default() -> Self {
        Self {
            regex_weight: 0.30,      // 正则匹配重要
            line_position_weight: 0.40, // 独立成行最重要（大幅提高）
            length_weight: 0.10,
            context_weight: 0.10,
            format_weight: 0.10,
            min_threshold: 0.50,     // 降低阈值（因为前面有空行的要求更严格）
        }
    }
}

/// 章节评分结果
#[derive(Debug, Clone, Default)]
pub struct ChapterScore {
    /// 正则匹配得分
    pub regex_score: f32,
    /// 独立成行得分
    pub line_position_score: f32,
    /// 长度得分
    pub length_score: f32,
    /// 上下文得分
    pub context_score: f32,
    /// 格式得分
    pub format_score: f32,
    /// 总得分
    pub total_score: f32,
}

impl ChapterScore {
    /// 创建新评分
    pub fn new() -> Self {
        Self::default()
    }

    /// 计算总分
    pub fn calculate_total(&mut self, factors: &ScoringFactors) {
        self.total_score = self.regex_score * factors.regex_weight
            + self.line_position_score * factors.line_position_weight
            + self.length_score * factors.length_weight
            + self.context_score * factors.context_weight
            + self.format_score * factors.format_weight;
    }

    /// 是否达到阈值
    pub fn passes_threshold(&self, factors: &ScoringFactors) -> bool {
        self.total_score >= factors.min_threshold
    }
}

/// 评分计算器
pub struct ScoreCalculator {
    factors: ScoringFactors,
    /// 常见章节开头模式
    chapter_prefixes: HashSet<String>,
    /// 常见卷开头模式
    volume_prefixes: HashSet<String>,
    /// 常见标点符号
    punctuation: HashSet<char>,
    /// 章节数字模式
    number_patterns: Vec<String>,
}

impl ScoreCalculator {
    /// 创建新的评分计算器
    pub fn new() -> Self {
        let mut chapter_prefixes = HashSet::new();
        for i in 0..=1000 {
            chapter_prefixes.insert(format!("第{}章", i));
            chapter_prefixes.insert(format!("第{}节", i));
            chapter_prefixes.insert(format!("第{}节", i));
        }

        let cn_numbers = ["零", "一", "二", "三", "四", "五", "六", "七", "八", "九", "十"];
        for num in cn_numbers.iter() {
            chapter_prefixes.insert(format!("第{}章", num));
        }

        let mut volume_prefixes = HashSet::new();
        for i in 0..=100 {
            volume_prefixes.insert(format!("第{}卷", i));
            volume_prefixes.insert(format!("第{}部", i));
        }

        let mut punctuation = HashSet::new();
        for c in ['，', '。', '！', '？', '；', '：', ',', '.', '!', '?', ';', ':', ' '].iter() {
            punctuation.insert(*c);
        }

        let number_patterns = vec![
            r"第\s*\d+\s*章".to_string(),
            r"第\s*[一二三四五六七八九十零〇百千两]+\s*章".to_string(),
            r"Chapter\s*\d+".to_string(),
            r"\d+\.\s*\S+".to_string(),
            r"\[\d+\]\s*\S+".to_string(),
        ];

        Self {
            factors: ScoringFactors::default(),
            chapter_prefixes,
            volume_prefixes,
            punctuation,
            number_patterns,
        }
    }

    /// 计算正则匹配得分
    ///
    /// 修改：降低部分匹配的得分，要求更严格的格式
    pub fn score_regex_match(&self, text: &str, pattern: Option<&str>) -> f32 {
        if let Some(pattern) = pattern {
            // 使用正则匹配
            if let Ok(re) = regex::Regex::new(pattern) {
                if re.is_match(text) {
                    return 1.0;
                }
            }
            return 0.0;
        }

        // 前缀匹配（完整前缀，给较高分）
        for prefix in &self.chapter_prefixes {
            if text.starts_with(prefix) {
                // 完整匹配前缀，给较高分
                if text == prefix {
                    return 0.9; // 完全匹配，高分
                } else {
                    // 部分匹配前缀，给较低分
                    return 0.3; // 降低得分，要求其他条件
                }
            }
        }

        // 内置模式匹配（改为部分匹配，降低得分）
        for pattern in &self.number_patterns {
            if let Ok(re) = regex::Regex::new(pattern) {
                if re.is_match(text) {
                    // 检查是否是完整的章节格式
                    // 例如："第一章" 或 "第一章 开始" 给高分
                    // "这是第一章的内容。" 给低分
                    let trimmed = text.trim();

                    // 检查是否以"第"开头
                    if trimmed.starts_with("第") {
                        // 检查"章"字之后的内容
                        if let Some(idx) = trimmed.find("章") {
                            let after_chapter = &trimmed[idx + 3..].trim();
                            // "章"后面是空或少量字符，给较高分
                            if after_chapter.is_empty() || after_chapter.len() <= 20 {
                                return 0.7;
                            }
                        }
                    }

                    // 检查是否以"Chapter"开头
                    if trimmed.starts_with("Chapter") || trimmed.starts_with("chapter") {
                        // 检查数字之后的内容
                        if let Some(idx) = trimmed.find(|c: char| c.is_ascii_digit()) {
                            // 找到数字的结束位置
                            let after_number = &trimmed[idx..];
                            let digit_end = after_number.find(|c: char| !c.is_ascii_digit())
                                .unwrap_or(after_number.len());
                            let after_num = &after_number[digit_end..].trim();
                            // 数字后面是空或少量字符，给较高分
                            if after_num.is_empty() || after_num.len() <= 20 {
                                return 0.7;
                            }
                        }
                    }

                    // 部分匹配，给低分
                    return 0.2;
                }
            }
        }

        0.0
    }

    /// 计算独立成行得分
    ///
    /// 关键：章节标题必须独立成行，即前面必须是换行（或换行加空行）
    pub fn score_line_position(
        &self,
        _current_line: &str,
        line_num: usize,
        lines: &[&str],
    ) -> f32 {
        let mut score: f32 = 0.0;

        // 检查是否独立成行（前面必须有空行或换行）
        if line_num > 0 {
            let prev_line = lines[line_num - 1].trim();

            // 前一行必须为空（空行），否则得分极低
            if prev_line.is_empty() {
                score += 0.6; // 前面有空行，得分高
            } else {
                // 前一行不为空，直接返回低分
                // 章节标题不能在段落中间
                return 0.0;
            }
        } else {
            // 第一行，可以作为章节标题
            score += 0.4;
        }

        // 检查后面是否有空行（加分项，不是必需的）
        if line_num + 1 < lines.len() {
            let next_line = lines[line_num + 1].trim();
            if next_line.is_empty() {
                score += 0.4; // 后面有空行，加分
            }
        }

        score.min(1.0)
    }

    /// 计算长度得分
    pub fn score_length(&self, text: &str, max_title_length: usize) -> f32 {
        // 使用字符数而不是字节数
        let len = text.trim().chars().count();

        // 太短不可能是章节标题
        if len < 2 {
            return 0.0;
        }

        // 太长也不太可能是章节标题
        if len > max_title_length {
            return 0.0;
        }

        // 最佳长度区间
        if len >= 3 && len <= 20 {
            return 1.0;
        } else if len >= 2 && len <= 30 {
            return 0.8;
        } else if len >= 2 && len <= 50 {
            return 0.6;
        }

        0.3
    }

    /// 计算上下文得分
    pub fn score_context(
        &self,
        _current_line: &str,
        line_num: usize,
        lines: &[&str],
    ) -> f32 {
        let mut score: f32 = 0.0;

        // 检查前后空行数量
        let mut empty_before = 0;
        let mut empty_after = 0;

        // 向前检查空行
        if line_num > 0 {
            for i in (0..line_num).rev() {
                if lines[i].trim().is_empty() {
                    empty_before += 1;
                } else {
                    break;
                }
            }
        }

        // 向后检查空行
        for i in (line_num + 1).min(lines.len())..lines.len() {
            if lines[i].trim().is_empty() {
                empty_after += 1;
            } else {
                break;
            }
        }

        // 前后都有空行，得分高
        if empty_before >= 1 && empty_after >= 1 {
            score += 0.7;
        } else if empty_before >= 1 || empty_after >= 1 {
            score += 0.4;
        }

        // 检查相邻行的长度特征
        if line_num > 0 {
            let prev_len = lines[line_num - 1].trim().len();
            if prev_len > 50 {
                // 上一行很长，可能是段落结束
                score += 0.3;
            }
        }

        if line_num + 1 < lines.len() {
            let next_len = lines[line_num + 1].trim().len();
            if next_len > 50 {
                // 下一行很长，可能是段落开始
                score += 0.3;
            }
        }

        score.min(1.0)
    }

    /// 计算格式得分
    ///
    /// 关键：区分章节标题和普通句子
    pub fn score_format(&self, text: &str) -> f32 {
        let mut score: f32 = 0.0;
        let trimmed = text.trim();

        // 检查是否是常见章节格式（开头）
        let starts_with_chapter = trimmed.starts_with("第") ||
                                 trimmed.starts_with("卷") ||
                                 trimmed.starts_with("部") ||
                                 trimmed.starts_with("Part") ||
                                 trimmed.starts_with("Chapter") ||
                                 (trimmed.len() > 2 && trimmed.chars().next().unwrap().is_ascii_digit());

        if starts_with_chapter {
            score += 0.5; // 以章节标记开头，加分
        }

        // 检查句子结构：章节标题通常是短语，不是完整句子
        // 如果以"这"、"那"、"我"、"你"、"他"、"她"等代词开头，很可能是普通句子
        let starts_with_pronoun = trimmed.starts_with("这") ||
                                  trimmed.starts_with("那") ||
                                  trimmed.starts_with("我") ||
                                  trimmed.starts_with("你") ||
                                  trimmed.starts_with("他") ||
                                  trimmed.starts_with("她") ||
                                  trimmed.starts_with("它");

        if starts_with_pronoun {
            score -= 0.5; // 以代词开头，很可能是普通句子，扣分
        }

        // 如果以"是"开头（如"这是第一章的内容"），也很可能是普通句子
        if trimmed.starts_with("是") {
            score -= 0.5;
        }

        // 不包含过多标点
        let punct_count = trimmed.chars().filter(|c| self.punctuation.contains(c)).count();
        let punct_ratio = punct_count as f32 / trimmed.chars().count() as f32;
        if punct_ratio < 0.3 {
            score += 0.3;
        } else {
            score -= 0.3; // 标点太多，不太像章节标题
        }

        // 检查长度：章节标题通常较短
        let len = trimmed.len();
        if len >= 3 && len <= 20 {
            score += 0.2;
        } else if len > 30 {
            score -= 0.2; // 太长，不太像章节标题
        }

        // 包含"第"和"章"等关键词（降低权重）
        if trimmed.contains("第") && (trimmed.contains("章") || trimmed.contains("节") || trimmed.contains("卷") || trimmed.contains("部")) {
            score += 0.1; // 降低到 0.1
        }

        // 包含问号或感叹号
        if trimmed.contains('?') || trimmed.contains('！') || trimmed.contains('？') {
            score += 0.2;
        }

        // 包含冒号（可能是副标题）
        if trimmed.contains('：') || trimmed.contains(':') {
            score += 0.1;
        }

        score.min(1.0).max(0.0) // 确保不小于0
    }

    /// 计算总分
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

    /// 设置评分因素
    pub fn set_factors(&mut self, factors: ScoringFactors) {
        self.factors = factors;
    }

    /// 获取评分因素
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

        // 测试常见章节格式（完整匹配前缀）
        assert_eq!(calc.score_regex_match("第1章", None), 0.9); // 完全匹配
        assert_eq!(calc.score_regex_match("第一章", None), 0.9); // 完全匹配
        assert_eq!(calc.score_regex_match("Chapter 1", None), 0.7); // 正则匹配，部分得分

        // 测试部分匹配前缀（后面有额外内容）
        assert_eq!(calc.score_regex_match("第1章 开始", None), 0.3); // 部分匹配
        assert_eq!(calc.score_regex_match("第一章 开始", None), 0.3); // 部分匹配

        // 测试普通文本
        assert_eq!(calc.score_regex_match("普通段落", None), 0.0);
    }

    #[test]
    fn test_score_length() {
        let calc = ScoreCalculator::new();

        // 最佳长度（3-20字符）
        assert_eq!(calc.score_length("第1章", 35), 1.0);
        assert_eq!(calc.score_length("第一章的内容", 35), 1.0); // 7个字符

        // 太短
        assert_eq!(calc.score_length("章", 35), 0.0);

        // 太长（超过 max_title_length）
        let long_text = "这个标题非常长非常长非常长非常长非常长非常长非常长非常长非常长非常长非常长";
        assert_eq!(calc.score_length(long_text, 35), 0.0); // 40+ 字符 > 35

        // 边界情况（2-30字符）
        assert_eq!(calc.score_length("第一", 35), 0.8); // 2个字符
        assert_eq!(calc.score_length("第一章标题比较长一些", 35), 1.0); // 10个字符（最佳长度区间 3-20）
    }

    #[test]
    fn test_score_format() {
        let calc = ScoreCalculator::new();

        // 章节格式
        assert!(calc.score_format("第1章 开始？") > 0.5);

        // 段落格式
        assert!(calc.score_format("这是一个完整的句子。") < calc.score_format("第1章"));
    }

    #[test]
    fn test_calculate_chapter_score() {
        let calc = ScoreCalculator::new();
        let lines = vec!["", "第1章 开始", ""];

        let score = calc.calculate_chapter_score("第1章 开始", 1, &lines, None, 35);

        // 章节标题应该有较高分数
        assert!(score.total_score > 0.6);
        assert!(score.regex_score >= 0.3); // 部分匹配前缀
        assert!(score.line_position_score > 0.5); // 前面有空行
        assert!(score.format_score > 0.0); // 格式得分
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
