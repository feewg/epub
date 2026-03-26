//! 配置预设
//!
//! 提供不同使用场景的配置示例

use std::collections::HashMap;

/// 配置预设类型
#[derive(Debug, Clone, Copy)]
pub enum ConfigPreset {
    /// 基础配置
    Basic,
    /// 网络小说配置
    WebNovel,
    /// 完整配置
    Full,
    /// 简约配置
    Minimal,
    /// 出版物配置
    Publication,
}

impl ConfigPreset {
    /// 获取配置名称
    pub fn name(&self) -> &str {
        match self {
            Self::Basic => "basic",
            Self::WebNovel => "webnovel",
            Self::Full => "full",
            Self::Minimal => "minimal",
            Self::Publication => "publication",
        }
    }

    /// 获取配置描述
    pub fn description(&self) -> &str {
        match self {
            Self::Basic => "基础配置 - 适合一般的小说转换",
            Self::WebNovel => "网络小说配置 - 适合在线小说格式",
            Self::Full => "完整配置 - 包含所有可配置项",
            Self::Minimal => "简约配置 - 只包含最基本的设置",
            Self::Publication => "出版物配置 - 适合正式出版书籍",
        }
    }
}

/// 生成配置示例
pub fn generate_config_examples() -> HashMap<String, String> {
    let mut examples = HashMap::new();

    examples.insert(ConfigPreset::Basic.name().to_string(), generate_basic_config());
    examples.insert(ConfigPreset::WebNovel.name().to_string(), generate_webnovel_config());
    examples.insert(ConfigPreset::Full.name().to_string(), generate_full_config());
    examples.insert(ConfigPreset::Minimal.name().to_string(), generate_minimal_config());
    examples.insert(ConfigPreset::Publication.name().to_string(), generate_publication_config());

    examples
}

/// 生成基础配置
pub fn generate_basic_config() -> String {
    r#"# kaf-cli 基础配置文件
# 适合一般的小说转换

# 书名
bookname: "示例小说"

# 作者
author: "YSTYLE"

# 章节匹配规则
chapter_match: "^第.{1,8}章"

# 标题最大字数
max_title_length: 35

# 段落缩进字数
indent: 2

# 标题对齐方式
align: "center"

# 书籍语言
lang: "zh"

# 输出格式
format: "all"
"#.to_string()
}

/// 生成网络小说配置
pub fn generate_webnovel_config() -> String {
    r#"# kaf-cli 网络小说配置文件
# 适合在线小说格式

# 书名
bookname: "网络小说示例"

# 作者
author: "网络作者"

# 章节匹配规则（更宽松）
chapter_match: "^第.{1,15}章|^[0-9]+\s+\S+"

# 卷匹配规则
volume_match: "^第[0-9一二三四五六七八九十零〇百千两 ]+[卷部]"

# 排除规则
exclusion_pattern: "^第[0-9一二三四五六七八九十零〇百千两 ]+(部门|部队|部属|部分|部件|部落)"

# 标题最大字数（网络小说标题通常较长）
max_title_length: 50

# 段落缩进字数
indent: 2

# 段落间距（网络小说适合更大间距）
paragraph_spacing: "1em"

# 行高（网络小说适合更大的行高）
line_height: "2.0"

# 标题对齐方式
align: "center"

# 书籍语言
lang: "zh"

# 输出格式
format: "all"

# 分离章节序号和标题
separate_chapter_number: false

# CSS 变量（网络小说主题）
css_variables:
  primary-color: "#2c3e50"
  background-color: "#ecf0f1"
  text-color: "#34495e"
  link-color: "#3498db"
"#.to_string()
}

/// 生成完整配置
pub fn generate_full_config() -> String {
    r#"# kaf-cli 完整配置文件
# 包含所有可配置项

# ========== 书籍信息 ==========
# 书名
bookname: "完整示例"

# 作者
author: "示例作者"

# 输出文件名（不含扩展名）
# output_name: "custom_output"

# ========== 章节识别 ==========
# 章节匹配规则（正则表达式）
chapter_match: "^第.{1,8}章"

# 卷匹配规则（正则表达式）
volume_match: "^第[0-9一二三四五六七八九十零〇百千两 ]+[卷部]"

# 排除规则（正则表达式）
exclusion_pattern: "^第[0-9一二三四五六七八九十零〇百千两 ]+(部门|部队|部属|部分|部件|部落)"

# 标题最大字数
max_title_length: 35

# 未知章节默认名称
unknown_title: "未知章节"

# ========== 段落处理 ==========
# 段落缩进字数
indent: 2

# 段落间距
paragraph_spacing: "0.5em"

# 行高
line_height: "1.8"

# ========== 样式设置 ==========
# 标题对齐方式 (left, center, right)
align: "center"

# 封面图片路径
# cover: "cover.jpg"

# 自定义 CSS 文件路径
# custom_css: "custom.css"

# 扩展 CSS（内联）
# extended_css: |
#   .content {
#     font-size: 18px;
#     line-height: 1.8;
#   }

# 嵌入字体文件路径（支持 TTF/OTF 格式）
# font: "fonts/custom.ttf"

# CSS 变量
css_variables:
  primary-color: "#333333"
  background-color: "#ffffff"
  text-color: "#333333"
  link-color: "#0000ff"

# ========== 输出设置 ==========
# 书籍语言 (zh, en, de, fr, it, es, ja, pt, ru, nl)
lang: "zh"

# 输出格式 (epub, all)
format: "all"

# ========== 高级设置 ==========
# 是否添加教程
add_tips: false

# 是否分离章节序号和标题
separate_chapter_number: false

# 章节页眉图片配置
# chapter_header:
#   image: "header.jpg"           # 页眉图片路径
#   image_folder: "headers/"     # 页眉图片文件夹
#   position: "center"           # 图片位置 (left, center, right)
#   height: "100px"              # 图片高度
#   width: "auto"                # 图片宽度
#   mode: "single"               # 匹配模式 (single, folder)
"#.to_string()
}

/// 生成简约配置
pub fn generate_minimal_config() -> String {
    r#"# kaf-cli 简约配置文件
# 只包含最基本的设置

# 书名
bookname: "简约示例"

# 作者
author: "作者名"

# 输出格式
format: "epub"
"#.to_string()
}

/// 生成出版物配置
pub fn generate_publication_config() -> String {
    r#"# kaf-cli 出版物配置文件
# 适合正式出版书籍

# 书名
bookname: "出版物示例"

# 作者
author: "正式作者"

# 章节匹配规则（出版物格式更严格）
chapter_match: "^第[0-9一二三四五六七八九十零〇]+章"

# 标题最大字数（出版物标题通常较短）
max_title_length: 25

# 段落缩进字数（出版物通常使用更大的缩进）
indent: 2

# 段落间距（出版物适合标准的段落间距）
paragraph_spacing: "0.5em"

# 行高（出版物适合标准的行高）
line_height: "1.6"

# 标题对齐方式（出版物通常居中对齐）
align: "center"

# 书籍语言
lang: "zh"

# 输出格式
format: "epub"

# CSS 变量（出版物主题 - 传统风格）
css_variables:
  primary-color: "#000000"
  background-color: "#ffffff"
  text-color: "#000000"
  link-color: "#0000ff"

# 出版物通常不添加教程
add_tips: false

# 出版物通常不分离章节序号
separate_chapter_number: false
"#.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_preset_names() {
        assert_eq!(ConfigPreset::Basic.name(), "basic");
        assert_eq!(ConfigPreset::WebNovel.name(), "webnovel");
        assert_eq!(ConfigPreset::Full.name(), "full");
        assert_eq!(ConfigPreset::Minimal.name(), "minimal");
        assert_eq!(ConfigPreset::Publication.name(), "publication");
    }

    #[test]
    fn test_generate_config_examples() {
        let examples = generate_config_examples();
        assert_eq!(examples.len(), 5);
        assert!(examples.contains_key("basic"));
        assert!(examples.contains_key("webnovel"));
        assert!(examples.contains_key("full"));
        assert!(examples.contains_key("minimal"));
        assert!(examples.contains_key("publication"));
    }

    #[test]
    fn test_generate_basic_config() {
        let config = generate_basic_config();
        assert!(config.contains("bookname"));
        assert!(config.contains("author"));
        assert!(config.contains("chapter_match"));
    }

    #[test]
    fn test_generate_webnovel_config() {
        let config = generate_webnovel_config();
        assert!(config.contains("网络小说配置"));
        assert!(config.contains("max_title_length: 50"));
        assert!(config.contains("line_height: \"2.0\""));
    }

    #[test]
    fn test_generate_minimal_config() {
        let config = generate_minimal_config();
        assert!(config.contains("简约配置"));
        assert!(config.len() < generate_full_config().len());
    }
}
