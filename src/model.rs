//! 数据模型模块
//!
//! 定义了项目中所有的核心数据结构

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// 书籍配置
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Book {
    /// 输入文件路径
    pub filename: PathBuf,

    /// 书名（默认从文件名提取）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bookname: Option<String>,

    /// 作者
    #[serde(default = "default_author")]
    pub author: String,

    /// 章节列表
    #[serde(skip)]
    #[allow(dead_code)]
    pub sections: Vec<Section>,

    /// 章节标题匹配正则
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chapter_match: Option<String>,

    /// 卷标题匹配正则
    #[serde(skip_serializing_if = "Option::is_none")]
    pub volume_match: Option<String>,

    /// 排除规则正则
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclusion_pattern: Option<String>,

    /// 标题最大字数
    #[serde(default = "default_max_title_length")]
    pub max_title_length: usize,

    /// 段落缩进字数
    #[serde(default = "default_indent")]
    pub indent: usize,

    /// 标题对齐方式
    #[serde(default)]
    pub align: TextAlignment,

    /// 未知章节默认名称
    #[serde(default = "default_unknown_title")]
    pub unknown_title: String,

    /// 封面图片路径
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cover: Option<CoverSource>,

    /// 嵌入字体路径
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font: Option<PathBuf>,

    /// 段落间距
    #[serde(default = "default_paragraph_spacing")]
    pub paragraph_spacing: String,

    /// 行高
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_height: Option<String>,

    /// 添加教程
    #[serde(default)]
    pub add_tips: bool,

    /// 书籍语言
    #[serde(default)]
    pub lang: Language,

    /// 输出格式
    #[serde(default)]
    pub format: OutputFormat,

    /// 输出文件名（不含扩展名）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_name: Option<String>,

    /// 分离章节序号和标题
    #[serde(default)]
    pub separate_chapter_number: bool,

    /// 自定义 CSS 文件
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_css: Option<PathBuf>,

    /// 扩展 CSS（内联）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extended_css: Option<String>,

    /// CSS 变量
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    #[serde(default)]
    pub css_variables: HashMap<String, String>,

    /// 章节页眉图片
    #[serde(default)]
    pub chapter_header: ChapterHeader,

    /// 主题预设
    #[serde(default)]
    pub theme: ThemePreset,

    /// 输入格式
    #[serde(default)]
    pub input_format: InputFormat,
}

/// 封面来源
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CoverSource {
    /// 本地图片
    Local {
        path: PathBuf,
    },
    /// 内存中的图片数据（用于程序化生成封面等场景）
    Data {
        /// 图片二进制数据
        data: Vec<u8>,
        /// MIME 类型，如 "image/jpeg"、"image/png"
        format: String,
    },
}

/// 章节页眉图片配置
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct ChapterHeader {
    /// 页眉图片路径
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<PathBuf>,

    /// 页眉图片文件夹
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_folder: Option<PathBuf>,

    /// 图片位置
    #[serde(default)]
    pub position: ImagePosition,

    /// 图片高度
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<String>,

    /// 图片宽度
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<String>,

    /// 匹配模式
    #[serde(default)]
    pub mode: HeaderMode,
}

/// 章节结构
#[derive(Debug, Clone, Serialize, Default)]
pub struct Section {
    /// 章节标题
    pub title: String,

    /// 章节内容（HTML）
    pub content: String,

    /// 子章节（用于卷结构）
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub subsections: Vec<Section>,
}

/// 文本对齐方式
#[derive(Debug, Clone, Copy, Deserialize, Serialize, Default, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TextAlignment {
    Left,
    #[default]
    Center,
    Right,
}

/// 主题预设
#[derive(Debug, Clone, Copy, Deserialize, Serialize, Default, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ThemePreset {
    /// 浅色主题（默认）
    #[default]
    Light,
    /// 深色主题
    Dark,
    /// 护眼模式（ sepia 色调）
    Sepia,
    /// 高对比度
    HighContrast,
    /// 现代简约
    Modern,
    /// 传统文学
    Traditional,
}

impl ThemePreset {
    /// 获取所有预设主题列表
    #[allow(dead_code)]
    pub fn all() -> Vec<Self> {
        vec![
            ThemePreset::Light,
            ThemePreset::Dark,
            ThemePreset::Sepia,
            ThemePreset::HighContrast,
            ThemePreset::Modern,
            ThemePreset::Traditional,
        ]
    }

    /// 获取主题名称
    #[allow(dead_code)]
    pub fn name(&self) -> &'static str {
        match self {
            ThemePreset::Light => "浅色主题",
            ThemePreset::Dark => "深色主题",
            ThemePreset::Sepia => "护眼模式",
            ThemePreset::HighContrast => "高对比度",
            ThemePreset::Modern => "现代简约",
            ThemePreset::Traditional => "传统文学",
        }
    }

}

/// 书籍语言
#[derive(Debug, Clone, Copy, Deserialize, Serialize, Default, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    #[default]
    Zh,
    En,
    De,
    Fr,
    It,
    Es,
    Ja,
    Pt,
    Ru,
    Nl,
}

/// 输出格式
#[derive(Debug, Clone, Copy, Deserialize, Serialize, Default, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum OutputFormat {
    #[default]
    All,
    Epub,
}

/// 输入格式
#[derive(Debug, Clone, Copy, Deserialize, Serialize, Default, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum InputFormat {
    /// 自动检测（默认）
    #[default]
    Auto,
    /// 纯文本格式
    Txt,
    /// Markdown 格式
    Markdown,
}

/// 图片位置
#[derive(Debug, Clone, Copy, Deserialize, Serialize, Default, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ImagePosition {
    #[default]
    Center,
    Left,
    Right,
}

/// 页眉模式
#[derive(Debug, Clone, Copy, Deserialize, Serialize, Default, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum HeaderMode {
    /// 所有章节使用同一张图片
    #[default]
    Single,
    /// 按章节名从文件夹匹配图片
    Folder,
}

/// 默认章节匹配规则（包含万位数字）
pub const DEFAULT_CHAPTER_MATCH: &str = r"^第[0-9一二三四五六七八九十零〇百千万两 ]+[章回节集幕卷部]|^[Ss]ection.{1,20}$|^[Cc]hapter.{1,20}$|^[Pp]age.{1,20}$|^\d{1,4}$|^\d+、$|^引子$|^楔子$|^章节目录|^章节|^序章|^最终章 \w{1,20}$|^番外\d?\w{0,20}|^完本感言.{0,4}$";

/// 默认卷匹配规则（包含万位数字）
pub const DEFAULT_VOLUME_MATCH: &str = r"^第[0-9一二三四五六七八九十零〇百千万两 ]+[卷部]";

/// 默认排除规则
pub const DEFAULT_EXCLUSION: &str = r"^第[0-9一二三四五六七八九十零〇百千两 ]+(部门|部队|部属|部分|部件|部落|部.*：$)";

// 默认值函数
fn default_author() -> String {
    "YSTYLE".to_string()
}

fn default_max_title_length() -> usize {
    35
}

fn default_indent() -> usize {
    2
}

fn default_unknown_title() -> String {
    "未知章节".to_string()
}

fn default_paragraph_spacing() -> String {
    "0.5em".to_string()
}

impl Default for Book {
    fn default() -> Self {
        Self {
            filename: PathBuf::new(),
            bookname: None,
            author: default_author(),
            sections: Vec::new(),
            chapter_match: None,
            volume_match: None,
            exclusion_pattern: None,
            max_title_length: default_max_title_length(),
            indent: default_indent(),
            align: TextAlignment::default(),
            unknown_title: default_unknown_title(),
            cover: None,
            font: None,
            paragraph_spacing: default_paragraph_spacing(),
            line_height: None,
            add_tips: false,
            lang: Language::default(),
            format: OutputFormat::default(),
            output_name: None,
            separate_chapter_number: false,
            custom_css: None,
            extended_css: None,
            css_variables: HashMap::new(),
            chapter_header: ChapterHeader::default(),
            theme: ThemePreset::default(),
            input_format: InputFormat::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_book_default() {
        let book = Book::default();
        assert_eq!(book.author, "YSTYLE");
        assert_eq!(book.max_title_length, 35);
        assert_eq!(book.indent, 2);
    }

    #[test]
    fn test_book_serialization() {
        let book = Book {
            filename: PathBuf::from("test.txt"),
            bookname: Some("Test Book".to_string()),
            author: "Test Author".to_string(),
            ..Default::default()
        };

        let yaml = serde_yaml::to_string(&book).unwrap();
        let deserialized: Book = serde_yaml::from_str(&yaml).unwrap();

        assert_eq!(deserialized.bookname, book.bookname);
        assert_eq!(deserialized.author, book.author);
    }

    #[test]
    fn test_section_default() {
        let section = Section::default();
        assert_eq!(section.title, "");
        assert_eq!(section.content, "");
        assert!(section.subsections.is_empty());
    }
}
