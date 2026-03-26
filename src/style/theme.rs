//! 主题系统
//!
//! 提供多种预设主题和自定义主题支持
//!
//! ThemePreset 定义在 model.rs 中，避免循环依赖

use serde::{Deserialize, Serialize};

/// 颜色方案
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ColorScheme {
    /// 主背景色
    pub background: String,
    /// 主文字色
    pub text: String,
    /// 次要文字色
    pub text_secondary: String,
    /// 强调色/链接色
    pub accent: String,
    /// 边框色
    pub border: String,
    /// 章节标题背景
    pub chapter_bg: String,
    /// 章节标题文字
    pub chapter_text: String,
    /// 选中/高亮色
    pub highlight: String,
}

/// 字体排版
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Typography {
    /// 正文字体
    pub body_font: String,
    /// 标题字体
    pub heading_font: String,
    /// 基础字号（px）
    pub base_size: u32,
    /// 行高倍数
    pub line_height: f32,
    /// 段落间距
    pub paragraph_spacing: String,
    /// 标题字重
    pub heading_weight: u32,
}

/// 间距设置
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Spacing {
    /// 页面内边距
    pub page_padding: String,
    /// 章节间距
    pub chapter_margin: String,
    /// 段落缩进
    pub paragraph_indent: String,
    /// 行内边距
    pub line_padding: String,
}

/// 主题
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Theme {
    /// 主题名称
    pub name: String,
    /// 颜色方案
    pub colors: ColorScheme,
    /// 字体排版
    pub typography: Typography,
    /// 间距设置
    pub spacing: Spacing,
}

impl Theme {
    /// 创建浅色主题
    pub fn light() -> Self {
        Self {
            name: "浅色主题".to_string(),
            colors: ColorScheme {
                background: "#ffffff".to_string(),
                text: "#333333".to_string(),
                text_secondary: "#666666".to_string(),
                accent: "#0066cc".to_string(),
                border: "#dddddd".to_string(),
                chapter_bg: "#f8f8f8".to_string(),
                chapter_text: "#222222".to_string(),
                highlight: "#fff3cd".to_string(),
            },
            typography: Typography {
                body_font: "system-ui, -apple-system, sans-serif".to_string(),
                heading_font: "system-ui, -apple-system, sans-serif".to_string(),
                base_size: 18,
                line_height: 1.8,
                paragraph_spacing: "1em".to_string(),
                heading_weight: 600,
            },
            spacing: Spacing {
                page_padding: "2rem".to_string(),
                chapter_margin: "3rem".to_string(),
                paragraph_indent: "2em".to_string(),
                line_padding: "0.5em".to_string(),
            },
        }
    }

    /// 创建深色主题
    pub fn dark() -> Self {
        Self {
            name: "深色主题".to_string(),
            colors: ColorScheme {
                background: "#1a1a1a".to_string(),
                text: "#e0e0e0".to_string(),
                text_secondary: "#a0a0a0".to_string(),
                accent: "#66b3ff".to_string(),
                border: "#444444".to_string(),
                chapter_bg: "#2a2a2a".to_string(),
                chapter_text: "#f0f0f0".to_string(),
                highlight: "#4a4a00".to_string(),
            },
            typography: Typography {
                body_font: "system-ui, -apple-system, sans-serif".to_string(),
                heading_font: "system-ui, -apple-system, sans-serif".to_string(),
                base_size: 18,
                line_height: 1.8,
                paragraph_spacing: "1em".to_string(),
                heading_weight: 600,
            },
            spacing: Spacing {
                page_padding: "2rem".to_string(),
                chapter_margin: "3rem".to_string(),
                paragraph_indent: "2em".to_string(),
                line_padding: "0.5em".to_string(),
            },
        }
    }

    /// 创建护眼模式（Sepia）
    pub fn sepia() -> Self {
        Self {
            name: "护眼模式".to_string(),
            colors: ColorScheme {
                background: "#f4ecd8".to_string(),
                text: "#433422".to_string(),
                text_secondary: "#5c4a35".to_string(),
                accent: "#8b4513".to_string(),
                border: "#d4c5a9".to_string(),
                chapter_bg: "#e8dcc0".to_string(),
                chapter_text: "#2d1f0f".to_string(),
                highlight: "#d4c5a9".to_string(),
            },
            typography: Typography {
                body_font: "Georgia, 'Times New Roman', serif".to_string(),
                heading_font: "Georgia, 'Times New Roman', serif".to_string(),
                base_size: 19,
                line_height: 1.9,
                paragraph_spacing: "1.1em".to_string(),
                heading_weight: 600,
            },
            spacing: Spacing {
                page_padding: "2.5rem".to_string(),
                chapter_margin: "3.5rem".to_string(),
                paragraph_indent: "2em".to_string(),
                line_padding: "0.6em".to_string(),
            },
        }
    }

    /// 创建高对比度主题
    pub fn high_contrast() -> Self {
        Self {
            name: "高对比度".to_string(),
            colors: ColorScheme {
                background: "#000000".to_string(),
                text: "#ffffff".to_string(),
                text_secondary: "#cccccc".to_string(),
                accent: "#ffff00".to_string(),
                border: "#ffffff".to_string(),
                chapter_bg: "#1a1a1a".to_string(),
                chapter_text: "#ffffff".to_string(),
                highlight: "#ffff00".to_string(),
            },
            typography: Typography {
                body_font: "Arial, Helvetica, sans-serif".to_string(),
                heading_font: "Arial, Helvetica, sans-serif".to_string(),
                base_size: 20,
                line_height: 2.0,
                paragraph_spacing: "1.2em".to_string(),
                heading_weight: 700,
            },
            spacing: Spacing {
                page_padding: "2rem".to_string(),
                chapter_margin: "4rem".to_string(),
                paragraph_indent: "2.5em".to_string(),
                line_padding: "0.8em".to_string(),
            },
        }
    }

    /// 创建现代简约主题
    pub fn modern() -> Self {
        Self {
            name: "现代简约".to_string(),
            colors: ColorScheme {
                background: "#fafafa".to_string(),
                text: "#2c3e50".to_string(),
                text_secondary: "#7f8c8d".to_string(),
                accent: "#3498db".to_string(),
                border: "#ecf0f1".to_string(),
                chapter_bg: "#ffffff".to_string(),
                chapter_text: "#2c3e50".to_string(),
                highlight: "#e8f4f8".to_string(),
            },
            typography: Typography {
                body_font: "'Segoe UI', Roboto, 'Helvetica Neue', sans-serif".to_string(),
                heading_font: "'Segoe UI', Roboto, 'Helvetica Neue', sans-serif".to_string(),
                base_size: 17,
                line_height: 1.7,
                paragraph_spacing: "0.9em".to_string(),
                heading_weight: 500,
            },
            spacing: Spacing {
                page_padding: "1.5rem".to_string(),
                chapter_margin: "2.5rem".to_string(),
                paragraph_indent: "1.5em".to_string(),
                line_padding: "0.4em".to_string(),
            },
        }
    }

    /// 创建传统文学主题
    pub fn traditional() -> Self {
        Self {
            name: "传统文学".to_string(),
            colors: ColorScheme {
                background: "#fdfbf7".to_string(),
                text: "#2b2b2b".to_string(),
                text_secondary: "#5a5a5a".to_string(),
                accent: "#8b0000".to_string(),
                border: "#d4d4d4".to_string(),
                chapter_bg: "#f5f5dc".to_string(),
                chapter_text: "#1a1a1a".to_string(),
                highlight: "#f0e68c".to_string(),
            },
            typography: Typography {
                body_font: "'Noto Serif CJK SC', 'Source Han Serif SC', 'SimSun', serif".to_string(),
                heading_font: "'Noto Serif CJK SC', 'Source Han Serif SC', 'SimSun', serif".to_string(),
                base_size: 20,
                line_height: 2.0,
                paragraph_spacing: "1.2em".to_string(),
                heading_weight: 700,
            },
            spacing: Spacing {
                page_padding: "3rem".to_string(),
                chapter_margin: "4rem".to_string(),
                paragraph_indent: "2.5em".to_string(),
                line_padding: "0.8em".to_string(),
            },
        }
    }

    /// 生成 CSS 变量定义
    pub fn to_css_variables(&self) -> String {
        format!(
            r#":root {{
    /* 颜色 */
    --bg-color: {bg};
    --text-color: {text};
    --text-secondary: {text_sec};
    --accent-color: {accent};
    --border-color: {border};
    --chapter-bg: {chap_bg};
    --chapter-text: {chap_text};
    --highlight-color: {highlight};
    
    /* 字体 */
    --body-font: {body_font};
    --heading-font: {heading_font};
    --base-size: {base_size}px;
    --line-height: {line_height};
    --paragraph-spacing: {para_space};
    --heading-weight: {heading_weight};
    
    /* 间距 */
    --page-padding: {page_pad};
    --chapter-margin: {chap_margin};
    --paragraph-indent: {para_indent};
    --line-padding: {line_pad};
}}"#,
            bg = self.colors.background,
            text = self.colors.text,
            text_sec = self.colors.text_secondary,
            accent = self.colors.accent,
            border = self.colors.border,
            chap_bg = self.colors.chapter_bg,
            chap_text = self.colors.chapter_text,
            highlight = self.colors.highlight,
            body_font = self.typography.body_font,
            heading_font = self.typography.heading_font,
            base_size = self.typography.base_size,
            line_height = self.typography.line_height,
            para_space = self.typography.paragraph_spacing,
            heading_weight = self.typography.heading_weight,
            page_pad = self.spacing.page_padding,
            chap_margin = self.spacing.chapter_margin,
            para_indent = self.spacing.paragraph_indent,
            line_pad = self.spacing.line_padding,
        )
    }
}

impl Default for Theme {
    fn default() -> Self {
        Theme::light()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_creation() {
        let light = Theme::light();
        assert_eq!(light.name, "浅色主题");
        assert!(light.typography.base_size > 0);

        let dark = Theme::dark();
        assert_eq!(dark.name, "深色主题");

        let sepia = Theme::sepia();
        assert_eq!(sepia.name, "护眼模式");
    }

    #[test]
    fn test_css_variables() {
        let theme = Theme::light();
        let css = theme.to_css_variables();
        assert!(css.contains("--bg-color:"));
        assert!(css.contains("--text-color:"));
        assert!(css.contains("--body-font:"));
    }

    #[test]
    fn test_theme_serialization() {
        let theme = Theme::light();
        let json = serde_json::to_string(&theme).unwrap();
        let deserialized: Theme = serde_json::from_str(&json).unwrap();
        assert_eq!(theme.name, deserialized.name);
    }
}
