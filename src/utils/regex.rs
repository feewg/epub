//! 正则表达式工具模块
//!
//! 提供正则表达式相关工具函数

use crate::error::Result;
use crate::model::{DEFAULT_CHAPTER_MATCH, DEFAULT_EXCLUSION, DEFAULT_VOLUME_MATCH};
use regex::Regex;
use std::collections::HashMap;

/// 预编译的正则表达式缓存
pub struct RegexCache {
    cache: HashMap<String, Regex>,
}

impl RegexCache {
    /// 创建新的正则表达式缓存
    pub fn new() -> Self {
        let mut cache = HashMap::new();

        // 预编译默认正则
        if let Ok(regex) = Regex::new(DEFAULT_CHAPTER_MATCH) {
            cache.insert(DEFAULT_CHAPTER_MATCH.to_string(), regex);
        }
        if let Ok(regex) = Regex::new(DEFAULT_VOLUME_MATCH) {
            cache.insert(DEFAULT_VOLUME_MATCH.to_string(), regex);
        }
        if let Ok(regex) = Regex::new(DEFAULT_EXCLUSION) {
            cache.insert(DEFAULT_EXCLUSION.to_string(), regex);
        }

        Self { cache }
    }

    /// 获取或编译正则表达式
    pub fn get_or_compile(&mut self, pattern: &str) -> Result<&Regex> {
        if self.cache.contains_key(pattern) {
            return Ok(self.cache.get(pattern).unwrap());
        }

        let regex = Regex::new(pattern)?;
        self.cache.insert(pattern.to_string(), regex);
        Ok(self.cache.get(pattern).unwrap())
    }

    /// 检查文本是否匹配章节标题
    #[allow(dead_code)]
    pub fn is_chapter(&mut self, text: &str, custom_pattern: Option<&str>) -> Result<bool> {
        let pattern = custom_pattern.unwrap_or(DEFAULT_CHAPTER_MATCH);
        let regex = self.get_or_compile(pattern)?;
        Ok(regex.is_match(text))
    }

    /// 检查文本是否匹配卷标题
    #[allow(dead_code)]
    pub fn is_volume(&mut self, text: &str, custom_pattern: Option<&str>) -> Result<bool> {
        let pattern = custom_pattern.unwrap_or(DEFAULT_VOLUME_MATCH);
        let regex = self.get_or_compile(pattern)?;
        Ok(regex.is_match(text))
    }

    /// 检查文本是否匹配排除规则
    pub fn is_excluded(&mut self, text: &str, custom_pattern: Option<&str>) -> Result<bool> {
        let pattern = custom_pattern.unwrap_or(DEFAULT_EXCLUSION);
        let regex = self.get_or_compile(pattern)?;
        Ok(regex.is_match(text))
    }
}

impl Default for RegexCache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_regex_cache() {
        let mut cache = RegexCache::new();
        let regex = cache.get_or_compile(r"^第.{1,8}章$").unwrap();
        assert!(regex.is_match("第一章"));
        assert!(regex.is_match("第十章"));
        assert!(!regex.is_match("第一章内容"));
    }

    #[test]
    fn test_is_chapter() {
        let mut cache = RegexCache::new();
        assert!(cache.is_chapter("第一章", None).unwrap());
        assert!(cache.is_chapter("序章", None).unwrap());
        assert!(!cache.is_chapter("这是一个段落", None).unwrap());
    }

    #[test]
    fn test_is_volume() {
        let mut cache = RegexCache::new();
        assert!(cache.is_volume("第一卷", None).unwrap());
        assert!(cache.is_volume("第十卷", None).unwrap());
        assert!(!cache.is_volume("第一章", None).unwrap());
    }

    #[test]
    fn test_is_excluded() {
        let mut cache = RegexCache::new();
        // 测试 "部门" 结尾
        assert!(cache.is_excluded("第一部门", None).unwrap());
        // 测试 "部队" 结尾  
        assert!(cache.is_excluded("第一部队", None).unwrap());
        // 测试非排除项
        assert!(!cache.is_excluded("第一章", None).unwrap());
        assert!(!cache.is_excluded("第一卷", None).unwrap());
    }
}
