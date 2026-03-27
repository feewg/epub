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
    #[allow(dead_code)]
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
    "# kaf-cli 基础配置文件\n\
# 适合一般的小说转换\n\
\n\
# 书名\n\
bookname: \"示例小说\"\n\
\n\
# 作者\n\
author: \"YSTYLE\"\n\
\n\
# 章节匹配规则\n\
chapter_match: \"^第.{1,8}章\"\n\
\n\
# 标题最大字数\n\
max_title_length: 35\n\
\n\
# 段落缩进字数\n\
indent: 2\n\
\n\
# 标题对齐方式\n\
align: \"center\"\n\
\n\
# 书籍语言\n\
lang: \"zh\"\n\
\n\
# 输出格式\n\
format: \"all\"\n".to_string()
}

/// 生成网络小说配置
pub fn generate_webnovel_config() -> String {
    "# kaf-cli 网络小说配置文件\n\
# 适合在线小说格式\n\
\n\
# 书名\n\
bookname: \"网络小说示例\"\n\
\n\
# 作者\n\
author: \"网络作者\"\n\
\n\
# 章节匹配规则（更宽松）\n\
chapter_match: \"^第.{1,15}章|^[0-9]+\\s+\\S+\"\n\
\n\
# 卷匹配规则\n\
volume_match: \"^第[0-9一二三四五六七八九十零〇百千两 ]+[卷部]\"\n\
\n\
# 排除规则\n\
exclusion_pattern: \"^第[0-9一二三四五六七八九十零〇百千两 ]+(部门|部队|部属|部分|部件|部落)\"\n\
\n\
# 标题最大字数（网络小说标题通常较长）\n\
max_title_length: 50\n\
\n\
# 段落缩进字数\n\
indent: 2\n\
\n\
# 段落间距（网络小说适合更大间距）\n\
paragraph_spacing: \"1em\"\n\
\n\
# 行高（网络小说适合更大的行高）\n\
line_height: \"2.0\"\n\
\n\
# 标题对齐方式\n\
align: \"center\"\n\
\n\
# 书籍语言\n\
lang: \"zh\"\n\
\n\
# 输出格式\n\
format: \"all\"\n\
\n\
# 分离章节序号和标题\n\
separate_chapter_number: false\n\
\n\
# CSS 变量（网络小说主题）\n\
css_variables:\n\
  primary-color: \"#2c3e50\"\n\
  background-color: \"#ecf0f1\"\n\
  text-color: \"#34495e\"\n\
  link-color: \"#3498db\"\n".to_string()
}

/// 生成完整配置
pub fn generate_full_config() -> String {
    "# kaf-cli 完整配置文件\n\
# 包含所有可配置项\n\
\n\
# ========== 书籍信息 ==========\n\
# 书名\n\
bookname: \"完整示例\"\n\
\n\
# 作者\n\
author: \"示例作者\"\n\
\n\
# 输出文件名（不含扩展名）\n\
# output_name: \"custom_output\"\n\
\n\
# ========== 章节识别 ==========\n\
# 章节匹配规则（正则表达式）\n\
chapter_match: \"^第.{1,8}章\"\n\
\n\
# 卷匹配规则（正则表达式）\n\
volume_match: \"^第[0-9一二三四五六七八九十零〇百千两 ]+[卷部]\"\n\
\n\
# 排除规则（正则表达式）\n\
exclusion_pattern: \"^第[0-9一二三四五六七八九十零〇百千两 ]+(部门|部队|部属|部分|部件|部落)\"\n\
\n\
# 标题最大字数\n\
max_title_length: 35\n\
\n\
# 未知章节默认名称\n\
unknown_title: \"未知章节\"\n\
\n\
# ========== 段落处理 ==========\n\
# 段落缩进字数\n\
indent: 2\n\
\n\
# 段落间距\n\
paragraph_spacing: \"0.5em\"\n\
\n\
# 行高\n\
line_height: \"1.8\"\n\
\n\
# ========== 样式设置 ==========\n\
# 标题对齐方式 (left, center, right)\n\
align: \"center\"\n\
\n\
# 封面图片路径\n\
# cover: \"cover.jpg\"\n\
\n\
# 自定义 CSS 文件路径\n\
# custom_css: \"custom.css\"\n\
\n\
# 扩展 CSS（内联）\n\
# extended_css: |\n\
#   .content {\n\
#     font-size: 18px;\n\
#     line-height: 1.8;\n\
#   }\n\
\n\
# 嵌入字体文件路径（支持 TTF/OTF 格式）\n\
# font: \"fonts/custom.ttf\"\n\
\n\
# CSS 变量\n\
css_variables:\n\
  primary-color: \"#333333\"\n\
  background-color: \"#ffffff\"\n\
  text-color: \"#333333\"\n\
  link-color: \"#0000ff\"\n\
\n\
# ========== 输出设置 ==========\n\
# 书籍语言 (zh, en, de, fr, it, es, ja, pt, ru, nl)\n\
lang: \"zh\"\n\
\n\
# 输出格式 (epub, all)\n\
format: \"all\"\n\
\n\
# ========== 高级设置 ==========\n\
# 是否添加教程\n\
add_tips: false\n\
\n\
# 是否分离章节序号和标题\n\
separate_chapter_number: false\n\
\n\
# 章节页眉图片配置\n\
# chapter_header:\n\
#   image: \"header.jpg\"           # 页眉图片路径\n\
#   image_folder: \"headers/\"     # 页眉图片文件夹\n\
#   position: \"center\"           # 图片位置 (left, center, right)\n\
#   height: \"100px\"              # 图片高度\n\
#   width: \"auto\"                # 图片宽度\n\
#   mode: \"single\"               # 匹配模式 (single, folder)\n".to_string()
}

/// 生成简约配置
pub fn generate_minimal_config() -> String {
    "# kaf-cli 简约配置文件\n\
# 只包含最基本的设置\n\
\n\
# 书名\n\
bookname: \"简约示例\"\n\
\n\
# 作者\n\
author: \"作者名\"\n\
\n\
# 输出格式\n\
format: \"epub\"\n".to_string()
}

/// 生成出版物配置
pub fn generate_publication_config() -> String {
    "# kaf-cli 出版物配置文件\n\
# 适合正式出版书籍\n\
\n\
# 书名\n\
bookname: \"出版物示例\"\n\
\n\
# 作者\n\
author: \"正式作者\"\n\
\n\
# 章节匹配规则（出版物格式更严格）\n\
chapter_match: \"^第[0-9一二三四五六七八九十零〇]+章\"\n\
\n\
# 标题最大字数（出版物标题通常较短）\n\
max_title_length: 25\n\
\n\
# 段落缩进字数（出版物通常使用更大的缩进）\n\
indent: 2\n\
\n\
# 段落间距（出版物适合标准的段落间距）\n\
paragraph_spacing: \"0.5em\"\n\
\n\
# 行高（出版物适合标准的行高）\n\
line_height: \"1.6\"\n\
\n\
# 标题对齐方式（出版物通常居中对齐）\n\
align: \"center\"\n\
\n\
# 书籍语言\n\
lang: \"zh\"\n\
\n\
# 输出格式\n\
format: \"epub\"\n\
\n\
# CSS 变量（出版物主题 - 传统风格）\n\
css_variables:\n\
  primary-color: \"#000000\"\n\
  background-color: \"#ffffff\"\n\
  text-color: \"#000000\"\n\
  link-color: \"#0000ff\"\n\
\n\
# 出版物通常不添加教程\n\
add_tips: false\n\
\n\
# 出版物通常不分离章节序号\n\
separate_chapter_number: false\n".to_string()
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
