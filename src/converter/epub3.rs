//! EPUB 3.0 生成器模块
//!
//! 使用 epub-builder 库生成符合标准的 EPUB 文件

use crate::error::{KafError, Result};
use crate::model::{Book, Section};
use crate::utils::html::escape_xml;
use epub_builder::{EpubBuilder, EpubContent, ReferenceType, ZipLibrary};
use std::collections::HashMap;
use std::io::Cursor;
use std::path::{Path, PathBuf};

/// EPUB 3.0 生成器
pub struct EpubConverter3 {
    book: Book,
}

impl EpubConverter3 {
    /// 创建新的 EPUB 3.0 生成器
    pub fn new(book: Book) -> Self {
        Self { book }
    }

    /// 生成 EPUB 文件
    pub async fn generate(&self, sections: &[Section]) -> Result<Vec<u8>> {
        // 创建 ZIP 库
        let zip_library = ZipLibrary::new()?;
        
        // 创建 EPUB 构建器
        let mut builder = EpubBuilder::new(zip_library)?;

        // 设置 EPUB 版本为 3.0
        builder.epub_version(epub_builder::EpubVersion::V30);

        // 设置元数据
        let title = self.book.bookname.clone().unwrap_or_else(|| "Unknown".to_string());
        builder.metadata("title", &title)?;
        builder.metadata("author", &self.book.author)?;
        builder.metadata("lang", &format!("{:?}", self.book.lang).to_lowercase())?;

        // 添加字体（如果有）
        if let Some(ref font_path) = self.book.font {
            self.embed_font(font_path, &mut builder)?;
        }

        // 添加 CSS 样式
        let css = self.build_css()?;
        builder.stylesheet(css.as_bytes())?;

        // 添加封面（如果有）
        if let Some(ref cover_source) = self.book.cover {
            match cover_source {
                crate::model::CoverSource::Local { path } => {
                    // 读取封面图片
                    let cover_data = std::fs::read(path)?;
                    let mime_type = if cover_data.starts_with(&[0x89, 0x50, 0x4E, 0x47]) {
                        "image/png"
                    } else {
                        "image/jpeg"
                    };
                    // 使用 Cursor 包装 Vec<u8> 以提供 Read trait
                    let mut cursor = std::io::Cursor::new(cover_data);
                    builder.add_cover_image(path, &mut cursor, mime_type)?;
                }
            }
        }

        // 加载章节页眉图片
        let header_images = self.load_header_images(sections).await?;

        // 添加目录页
        let toc_content = self.generate_toc_content(sections);
        builder.add_content(
            EpubContent::new("toc.xhtml", toc_content.as_bytes())
                .title("目录")
                .reftype(ReferenceType::Toc),
        )?;

        // 添加章节内容
        for (index, section) in sections.iter().enumerate() {
            // 检查是否有页眉图片
            let header_image_html = if let Some(ref img_path) = header_images.get(&index) {
                // 添加图片到 EPUB
                let img_data = std::fs::read(img_path)?;
                let mime_type = if img_data.starts_with(&[0x89, 0x50, 0x4E, 0x47]) {
                    "image/png"
                } else if img_data.starts_with(&[0xFF, 0xD8, 0xFF]) {
                    "image/jpeg"
                } else {
                    "image/webp"
                };
                
                let file_name = img_path.file_name()
                    .and_then(|f| f.to_str())
                    .unwrap_or("header.jpg");
                let img_path_in_epub = PathBuf::from("images").join(file_name);
                
                // 添加资源
                let mut cursor = std::io::Cursor::new(img_data);
                builder.add_resource(&img_path_in_epub, &mut cursor, mime_type)?;
                
                // 生成 HTML
                let position = match self.book.chapter_header.position {
                    crate::model::ImagePosition::Left => "left",
                    crate::model::ImagePosition::Center => "center",
                    crate::model::ImagePosition::Right => "right",
                };
                
                let height_attr = self.book.chapter_header.height.as_ref()
                    .map(|h| format!(r#" height="{}""#, h))
                    .unwrap_or_default();
                let width_attr = self.book.chapter_header.width.as_ref()
                    .map(|w| format!(r#" width="{}""#, w))
                    .unwrap_or_default();
                
                Some(format!(
                    r#"<div class="chapter-header {}" style="text-align: {};">
                        <img src="{}" alt="chapter header"{}{}/>
                    </div>"#,
                    position, position, img_path_in_epub.display(), height_attr, width_attr
                ))
            } else {
                None
            };

            let chapter_html = self.generate_chapter_html(section, index, header_image_html);
            let file_name = format!("chapter_{}.xhtml", index);
            
            builder.add_content(
                EpubContent::new(&file_name, chapter_html.as_bytes())
                    .title(&section.title)
                    .reftype(ReferenceType::Text),
            )?;
        }

        // 生成 EPUB 文件
        let mut cursor = Cursor::new(Vec::new());
        builder.generate(&mut cursor)?;

        Ok(cursor.into_inner())
    }

    /// 加载章节页眉图片
    async fn load_header_images(&self, sections: &[Section]) -> Result<HashMap<usize, PathBuf>> {
        let mut images = HashMap::new();
        
        match self.book.chapter_header.mode {
            crate::model::HeaderMode::Folder => {
                if let Some(ref folder) = self.book.chapter_header.image_folder {
                    if folder.exists() && folder.is_dir() {
                        // 读取文件夹中的所有图片
                        let mut available_images: Vec<PathBuf> = Vec::new();
                        let mut entries = tokio::fs::read_dir(folder).await?;
                        
                        while let Some(entry) = entries.next_entry().await? {
                            let path = entry.path();
                            let ext = path.extension().and_then(|e| e.to_str())
                                .map(|e| e.to_lowercase());
                            if matches!(ext.as_deref(), Some("jpg") | Some("jpeg") | Some("png") | Some("webp") | Some("gif")) {
                                available_images.push(path);
                            }
                        }
                        
                        // 为每个章节匹配图片
                        for (index, section) in sections.iter().enumerate() {
                            // 尝试完整匹配章节名
                            if let Some(img) = available_images.iter()
                                .find(|img: &&PathBuf| {
                                    let name = img.file_stem().and_then(|s| s.to_str()).unwrap_or("");
                                    section.title.contains(name) || name.contains(&section.title)
                                }) {
                                images.insert(index, img.clone());
                                continue;
                            }
                            
                            // 尝试数字匹配
                            let re = regex::Regex::new(r"\d+").unwrap();
                            if let Some(nums) = re.find(&section.title) {
                                if let Some(img) = available_images.iter()
                                    .find(|img: &&PathBuf| {
                                        let name = img.file_stem().and_then(|s| s.to_str()).unwrap_or("");
                                        name.contains(nums.as_str())
                                    }) {
                                    images.insert(index, img.clone());
                                }
                            }
                        }
                    }
                }
            }
            crate::model::HeaderMode::Single => {
                // 所有章节使用同一张图片
                if let Some(ref img) = self.book.chapter_header.image {
                    if img.exists() {
                        for (index, _) in sections.iter().enumerate() {
                            images.insert(index, img.clone());
                        }
                    }
                }
            }
        }
        
        Ok(images)
    }

    /// 生成章节 HTML
    fn generate_chapter_html(&self, section: &Section, _index: usize, header_image: Option<String>) -> String {
        let mut html = String::new();

        // XHTML 5 DOCTYPE
        html.push_str(r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE html>
<html xmlns="http://www.w3.org/1999/xhtml">
<head>
  <meta charset="utf-8"/>
  <title>"#);
        html.push_str(&escape_xml(&section.title));
        html.push_str(r#"</title>
  <link rel="stylesheet" type="text/css" href="stylesheet.css"/>
</head>
<body>
"#);

        // 添加页眉图片（如果有）
        if let Some(img_html) = header_image {
            html.push_str(&img_html);
            html.push('\n');
        }

        // 章节标题
        html.push_str(r#"<h3 class="chapter-title">"#);
        if self.book.separate_chapter_number {
            if let Some((number, title)) = self.split_chapter_number(&section.title) {
                html.push_str(r#"<span class="chapter-number">"#);
                html.push_str(&escape_xml(&number));
                html.push_str(r#"</span><br/>"#);
                html.push_str(&escape_xml(&title));
            } else {
                html.push_str(&escape_xml(&section.title));
            }
        } else {
            html.push_str(&escape_xml(&section.title));
        }
        html.push_str(r#"</h3>"#);

        // 正文内容
        html.push_str(r#"<div class="chapter-content">"#);
        html.push_str(&section.content);
        html.push_str(r#"</div>"#);

        // HTML 尾部
        html.push_str(r#"
</body>
</html>"#);

        html
    }

    /// 生成目录内容
    fn generate_toc_content(&self, sections: &[Section]) -> String {
        let mut nav_items = String::new();
        
        for (index, section) in sections.iter().enumerate() {
            nav_items.push_str(&format!(
                r#"      <li><a href="chapter_{}.xhtml">{}</a></li>"#,
                index,
                escape_xml(&section.title)
            ));
            nav_items.push('\n');
        }

        format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE html>
<html xmlns="http://www.w3.org/1999/xhtml" xmlns:epub="http://www.idpf.org/2007/ops">
<head>
  <meta charset="utf-8"/>
  <title>目录</title>
  <link rel="stylesheet" type="text/css" href="stylesheet.css"/>
</head>
<body>
  <nav epub:type="toc" id="toc">
    <h1>目录</h1>
    <ol>
{}
    </ol>
  </nav>
</body>
</html>"#,
            nav_items
        )
    }

    /// 分离章节序号和标题
    fn split_chapter_number(&self, title: &str) -> Option<(String, String)> {
        // 匹配 "第一章 标题" 或 "第一章"
        let re = regex::Regex::new(r"^(第[0-9一二三四五六七八九十零〇百千两 ]+[章回])(?:\s*(.+))?$").unwrap();
        if let Some(caps) = re.captures(title) {
            let number = caps.get(1)?.as_str().to_string();
            let text = caps.get(2).map(|m| m.as_str().to_string()).unwrap_or_default();
            Some((number, text))
        } else {
            None
        }
    }

    /// 嵌入字体文件
    fn embed_font(&self, font_path: &Path, builder: &mut EpubBuilder<ZipLibrary>) -> Result<()> {
        if !font_path.exists() {
            return Err(KafError::FileNotFound(font_path.to_string_lossy().to_string()));
        }

        // 读取字体文件
        let font_data = std::fs::read(font_path)?;
        
        // 获取文件名
        let file_name = font_path.file_name()
            .and_then(|f: &std::ffi::OsStr| f.to_str())
            .ok_or_else(|| KafError::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid font filename"
            )))?;

        // 获取 MIME 类型
        let mime_type = if font_path.extension().and_then(|e: &std::ffi::OsStr| e.to_str()) == Some("ttf") {
            "font/ttf"
        } else {
            "font/otf"
        };

        // 创建字体在 EPUB 中的路径
        let font_path_in_epub = PathBuf::from("fonts").join(file_name);
        
        // 添加字体资源
        let mut cursor = std::io::Cursor::new(font_data);
        builder.add_resource(&font_path_in_epub, &mut cursor, mime_type)?;

        Ok(())
    }

    /// 构建 CSS 样式
    fn build_css(&self) -> Result<String> {
        let mut css = String::new();

        // 添加字体定义（如果有自定义字体）
        let font_family = if let Some(ref font_path) = self.book.font {
            if let Some(file_name) = font_path.file_name().and_then(|f| f.to_str()) {
                css.push_str("/* 自定义字体 */\n");
                css.push_str("@font-face {\n");
                css.push_str(&format!("  font-family: 'CustomFont';\n"));
                css.push_str(&format!("  src: url('fonts/{}');\n", file_name));
                css.push_str("  font-display: swap;\n");
                css.push_str("}\n\n");
            }
            "'CustomFont', serif"
        } else {
            "serif"
        };

        // 添加 CSS 变量 :root
        if !self.book.css_variables.is_empty() {
            css.push_str(":root {\n");
            for (key, value) in &self.book.css_variables {
                css.push_str(&format!("  {}: {};\n", key, value));
            }
            css.push_str("}\n\n");
        }

        // 基础样式
        css.push_str(&format!(r#"
body {{
    font-family: {};
    margin: 0;
    padding: 0;
    line-height: 1.5;
}}

h1 {{
    text-align: center;
    font-size: 1.8em;
    margin: 1em 0;
}}

h3.chapter-title {{
    text-align: center;
    margin-top: 1.5em;
    margin-bottom: 1em;
    font-size: 1.3em;
    font-weight: bold;
}}

.chapter-number {{
    display: block;
    font-size: 1.2em;
}}

.chapter-content {{
    margin: 1em;
    text-indent: {}em;
"#, font_family, self.book.indent));

        // 添加行高设置
        if let Some(ref line_height) = self.book.line_height {
            css.push_str(&format!("    line-height: {};\n", line_height));
        }

        css.push_str(&format!(r#"}}

.chapter-content p {{
    margin: {} 0;
    text-align: justify;
}}

/* 章节页眉图片样式 */
.chapter-header {{
    margin: 1em 0;
}}

.chapter-header img {{
    max-width: 100%;
    height: auto;
    display: block;
}}

.chapter-header.left {{
    text-align: left;
}}

.chapter-header.center {{
    text-align: center;
}}

.chapter-header.right {{
    text-align: right;
}}

nav#toc ol {{
    list-style-type: decimal;
}}

nav#toc li {{
    margin: 0.5em 0;
}}

nav#toc a {{
    text-decoration: none;
    color: inherit;
}}
"#, self.book.paragraph_spacing));

        // 加载自定义 CSS 文件
        if let Some(ref custom_css_path) = self.book.custom_css {
            if custom_css_path.exists() {
                match std::fs::read_to_string(custom_css_path) {
                    Ok(custom_css) => {
                        css.push('\n');
                        css.push_str("/* 自定义 CSS 文件 */\n");
                        css.push_str(&custom_css);
                    }
                    Err(e) => {
                        return Err(KafError::Io(e));
                    }
                }
            }
        }

        // 添加内联扩展 CSS
        if let Some(ref extended) = self.book.extended_css {
            css.push('\n');
            css.push_str("/* 扩展 CSS */\n");
            css.push_str(extended);
        }

        Ok(css)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::path::PathBuf;

    #[test]
    fn test_epub_converter_creation() {
        let book = Book {
            filename: PathBuf::from("test.txt"),
            ..Default::default()
        };
        let converter = EpubConverter3::new(book);
        assert_eq!(converter.book.filename, PathBuf::from("test.txt"));
    }

    #[test]
    fn test_split_chapter_number() {
        let book = Book::default();
        let converter = EpubConverter3::new(book);

        assert_eq!(
            converter.split_chapter_number("第一章 序言"),
            Some(("第一章".to_string(), "序言".to_string()))
        );
        assert_eq!(
            converter.split_chapter_number("第一章"),
            Some(("第一章".to_string(), "".to_string()))
        );
        assert!(converter.split_chapter_number("序言").is_none());
    }

    #[test]
    fn test_build_css_basic() {
        let book = Book::default();
        let converter = EpubConverter3::new(book);
        let css = converter.build_css().unwrap();

        assert!(css.contains("body {"));
        assert!(css.contains("h3.chapter-title {"));
        assert!(css.contains("text-indent: 2em;"));
    }

    #[test]
    fn test_build_css_with_variables() {
        let mut css_vars = HashMap::new();
        css_vars.insert("--main-color".to_string(), "#333".to_string());
        css_vars.insert("--bg-color".to_string(), "#fff".to_string());

        let book = Book {
            css_variables: css_vars,
            ..Default::default()
        };
        let converter = EpubConverter3::new(book);
        let css = converter.build_css().unwrap();

        assert!(css.contains(":root {"));
        assert!(css.contains("--main-color: #333;"));
        assert!(css.contains("--bg-color: #fff;"));
    }

    #[test]
    fn test_build_css_with_extended() {
        let book = Book {
            extended_css: Some(".custom { color: red; }".to_string()),
            ..Default::default()
        };
        let converter = EpubConverter3::new(book);
        let css = converter.build_css().unwrap();

        assert!(css.contains(".custom { color: red; }"));
    }

    #[test]
    fn test_build_css_with_line_height() {
        let book = Book {
            line_height: Some("1.8".to_string()),
            ..Default::default()
        };
        let converter = EpubConverter3::new(book);
        let css = converter.build_css().unwrap();

        assert!(css.contains("line-height: 1.8;"));
    }

    #[test]
    fn test_generate_toc_content() {
        let book = Book::default();
        let converter = EpubConverter3::new(book);

        let sections = vec![
            Section {
                title: "第一章".to_string(),
                content: "内容1".to_string(),
                subsections: vec![],
            },
            Section {
                title: "第二章".to_string(),
                content: "内容2".to_string(),
                subsections: vec![],
            },
        ];

        let toc = converter.generate_toc_content(&sections);

        assert!(toc.contains("xmlns:epub=\"http://www.idpf.org/2007/ops\""));
        assert!(toc.contains("epub:type=\"toc\""));
        assert!(toc.contains("chapter_0.xhtml"));
        assert!(toc.contains("chapter_1.xhtml"));
        assert!(toc.contains("第一章"));
        assert!(toc.contains("第二章"));
    }

    #[test]
    fn test_generate_chapter_html() {
        let book = Book::default();
        let converter = EpubConverter3::new(book);

        let section = Section {
            title: "第一章 测试".to_string(),
            content: "<p>这是内容</p>".to_string(),
            subsections: vec![],
        };

        let html = converter.generate_chapter_html(&section, 0, None);

        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("第一章 测试"));
        assert!(html.contains("<p>这是内容</p>"));
        assert!(html.contains("stylesheet.css"));
    }

    #[test]
    fn test_generate_chapter_html_with_header_image() {
        let book = Book::default();
        let converter = EpubConverter3::new(book);

        let section = Section {
            title: "第一章".to_string(),
            content: "内容".to_string(),
            subsections: vec![],
        };

        let header_html = Some(r#"<div class="chapter-header"><img src="header.png"/></div>"#.to_string());
        let html = converter.generate_chapter_html(&section, 0, header_html);

        assert!(html.contains("chapter-header"));
        assert!(html.contains("header.png"));
    }
}
