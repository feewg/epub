//! EPUB 3.0 生成器模块

use crate::error::{KafError, Result};
use crate::model::{Book, CoverSource, HeaderMode, ImagePosition, Section};
use crate::utils::cover::{self, CoverConfig};
use crate::utils::html::{escape_xml, remove_invalid_xml_chars};
use epub_builder::{EpubBuilder, EpubContent, ReferenceType, ZipLibrary};
use image::ImageFormat;
use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::{BTreeMap, HashMap};
use std::io::Cursor;
use std::path::{Path, PathBuf};

static IMAGE_SRC: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(?is)(<img\b[^>]*?\bsrc\s*=\s*")([^"]*)(")"#).expect("固定图片 src 正则必须有效")
});

static CHAPTER_NUMBER: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^(第[0-9一二三四五六七八九十零〇百千万两 ]+[章回])(?:\s*(.+))?$")
        .expect("固定章节编号正则必须有效")
});

/// EPUB 3.0 生成器
pub struct EpubConverter3 {
    book: Book,
}

impl EpubConverter3 {
    pub fn new(book: Book) -> Self {
        Self { book }
    }

    fn resolve_resource_path(path: &Path, input_parent: Option<&Path>) -> Result<PathBuf> {
        crate::utils::file::resolve_resource_path(path, input_parent)
    }

    /// 生成 EPUB 文件
    pub async fn generate(&self, sections: &[Section]) -> Result<Vec<u8>> {
        let mut builder = EpubBuilder::new(ZipLibrary::new()?)?;
        builder.epub_version(epub_builder::EpubVersion::V30);

        let title = self
            .book
            .bookname
            .clone()
            .unwrap_or_else(|| "Unknown".to_string());
        builder.metadata("title", &title)?;
        builder.metadata("author", &self.book.author)?;
        builder.metadata("lang", format!("{:?}", self.book.lang).to_lowercase())?;

        if let Some(font_path) = &self.book.font {
            self.embed_font(font_path, &mut builder)?;
        }
        let css = self.build_css()?;
        builder.stylesheet(css.as_bytes())?;

        if let Some(source) = &self.book.cover {
            self.embed_cover(source, &mut builder)?;
        }

        let header_images = self.load_header_images(sections).await?;
        let header_resources = self.embed_header_images(&header_images, &mut builder)?;
        let rewritten_content = self.embed_content_images(sections, &mut builder)?;

        let toc_content = self.generate_toc_content(sections);
        builder.add_content(
            EpubContent::new("toc.xhtml", toc_content.as_bytes())
                .title("目录")
                .reftype(ReferenceType::Toc),
        )?;

        for (index, section) in sections.iter().enumerate() {
            let header_html = match header_images.get(&index) {
                Some(path) => {
                    let resource = header_resources.get(path).ok_or_else(|| {
                        KafError::EpubGenerationFailed(format!(
                            "页眉资源未写入 EPUB: {}",
                            path.display()
                        ))
                    })?;
                    Some(self.generate_header_html(resource)?)
                }
                None => None,
            };

            let mut rewritten = section.clone();
            rewritten.content = rewritten_content[index].clone();
            let chapter_html = self.generate_chapter_html(&rewritten, index, header_html);
            let file_name = format!("chapter_{index}.xhtml");
            builder.add_content(
                EpubContent::new(&file_name, chapter_html.as_bytes())
                    .title(&section.title)
                    .reftype(ReferenceType::Text),
            )?;
        }

        let mut cursor = Cursor::new(Vec::new());
        builder.generate(&mut cursor)?;
        Ok(cursor.into_inner())
    }

    fn embed_cover(
        &self,
        source: &CoverSource,
        builder: &mut EpubBuilder<ZipLibrary>,
    ) -> Result<()> {
        let raw = match source {
            CoverSource::Local { path } => {
                let resolved = Self::resolve_resource_path(path, self.book.filename.parent())?;
                std::fs::read(resolved)?
            }
            CoverSource::Data { data, .. } => data.clone(),
        };
        let (optimized, _) = cover::optimize_cover(&raw, &CoverConfig::default())?;
        let (data, mime, extension) = Self::prepare_epub_image(&optimized)?;
        let internal_path = PathBuf::from(format!("cover.{extension}"));
        builder.add_cover_image(internal_path, Cursor::new(data), mime)?;
        Ok(())
    }

    fn prepare_epub_image(data: &[u8]) -> Result<(Vec<u8>, &'static str, &'static str)> {
        let format = cover::detect_image_format(data)?;
        match format {
            ImageFormat::Jpeg => Ok((data.to_vec(), "image/jpeg", "jpg")),
            ImageFormat::Png => Ok((data.to_vec(), "image/png", "png")),
            ImageFormat::Gif => Ok((data.to_vec(), "image/gif", "gif")),
            _ => {
                let image = image::load_from_memory(data)?;
                let mut output = Vec::new();
                image.write_to(&mut Cursor::new(&mut output), ImageFormat::Png)?;
                Ok((output, "image/png", "png"))
            }
        }
    }

    fn embed_header_images(
        &self,
        header_images: &HashMap<usize, PathBuf>,
        builder: &mut EpubBuilder<ZipLibrary>,
    ) -> Result<HashMap<PathBuf, String>> {
        let mut paths = header_images.values().cloned().collect::<Vec<_>>();
        paths.sort();
        paths.dedup();

        let mut resources = HashMap::new();
        for (index, path) in paths.into_iter().enumerate() {
            let raw = std::fs::read(&path)?;
            let (data, mime, extension) = Self::prepare_epub_image(&raw)?;
            let resource = format!("images/header-{index}.{extension}");
            builder.add_resource(PathBuf::from(&resource), Cursor::new(data), mime)?;
            resources.insert(path, resource);
        }
        Ok(resources)
    }

    fn embed_content_images(
        &self,
        sections: &[Section],
        builder: &mut EpubBuilder<ZipLibrary>,
    ) -> Result<Vec<String>> {
        let mut embedded: HashMap<PathBuf, String> = HashMap::new();
        let mut next_resource = 0usize;
        sections
            .iter()
            .map(|section| {
                self.rewrite_content_images(
                    &section.content,
                    builder,
                    &mut embedded,
                    &mut next_resource,
                )
            })
            .collect()
    }

    fn rewrite_content_images(
        &self,
        content: &str,
        builder: &mut EpubBuilder<ZipLibrary>,
        embedded: &mut HashMap<PathBuf, String>,
        next_resource: &mut usize,
    ) -> Result<String> {
        let mut rewritten = String::with_capacity(content.len());
        let mut cursor = 0;

        for captures in IMAGE_SRC.captures_iter(content) {
            let full = captures.get(0).expect("完整图片匹配");
            let prefix = captures.get(1).expect("图片 src 前缀");
            let source = captures.get(2).expect("图片 src");
            let suffix = captures.get(3).expect("图片 src 后缀");
            rewritten.push_str(&content[cursor..prefix.end()]);

            let decoded = Self::decode_xml_attribute(source.as_str());
            if Self::is_external_image(&decoded) {
                return Err(KafError::ParseError(format!(
                    "EPUB 不支持未打包的远程或危险图片引用: {decoded}"
                )));
            }

            let path_part = decoded.split(['?', '#']).next().unwrap_or(decoded.as_str());
            let path_part = Self::percent_decode_path(path_part)?;
            let resolved =
                Self::resolve_resource_path(Path::new(&path_part), self.book.filename.parent())?;
            let key = std::fs::canonicalize(&resolved).unwrap_or(resolved);
            let resource = if let Some(existing) = embedded.get(&key) {
                existing.clone()
            } else {
                let raw = std::fs::read(&key)?;
                let (data, mime, extension) = Self::prepare_epub_image(&raw)?;
                let resource = format!("images/content-{}.{extension}", *next_resource);
                *next_resource += 1;
                builder.add_resource(PathBuf::from(&resource), Cursor::new(data), mime)?;
                embedded.insert(key, resource.clone());
                resource
            };
            rewritten.push_str(&escape_xml(&resource));
            rewritten.push_str(suffix.as_str());
            cursor = full.end();
        }
        rewritten.push_str(&content[cursor..]);
        Ok(rewritten)
    }

    fn decode_xml_attribute(value: &str) -> String {
        value
            .replace("&quot;", "\"")
            .replace("&apos;", "'")
            .replace("&#39;", "'")
            .replace("&lt;", "<")
            .replace("&gt;", ">")
            .replace("&amp;", "&")
    }

    fn is_external_image(source: &str) -> bool {
        let lower = source.trim().to_ascii_lowercase();
        lower.is_empty()
            || lower.starts_with("http://")
            || lower.starts_with("https://")
            || lower.starts_with("data:")
            || lower.starts_with('#')
            || lower
                .split(['/', '?', '#'])
                .next()
                .is_some_and(|prefix| prefix.contains(':'))
    }

    fn percent_decode_path(value: &str) -> Result<String> {
        let bytes = value.as_bytes();
        let mut decoded = Vec::with_capacity(bytes.len());
        let mut index = 0;
        while index < bytes.len() {
            if bytes[index] == b'%' {
                if index + 2 >= bytes.len() {
                    return Err(KafError::ParseError(format!(
                        "图片路径包含无效的百分号编码: {value}"
                    )));
                }
                let hex = std::str::from_utf8(&bytes[index + 1..index + 3])
                    .map_err(|error| KafError::ParseError(error.to_string()))?;
                let byte = u8::from_str_radix(hex, 16).map_err(|_| {
                    KafError::ParseError(format!("图片路径包含无效的百分号编码: {value}"))
                })?;
                decoded.push(byte);
                index += 3;
            } else {
                decoded.push(bytes[index]);
                index += 1;
            }
        }
        String::from_utf8(decoded)
            .map_err(|_| KafError::ParseError(format!("图片路径不是有效 UTF-8: {value}")))
    }

    async fn load_header_images(&self, sections: &[Section]) -> Result<HashMap<usize, PathBuf>> {
        let mut images = HashMap::new();
        match self.book.chapter_header.mode {
            HeaderMode::Folder => {
                let Some(folder) = &self.book.chapter_header.image_folder else {
                    return Err(KafError::ParseError(
                        "页眉模式为 folder 时必须设置 image_folder".to_string(),
                    ));
                };
                let folder = Self::resolve_resource_path(folder, self.book.filename.parent())?;
                if !folder.is_dir() {
                    return Err(KafError::ParseError(format!(
                        "章节页眉路径不是目录: {}",
                        folder.display()
                    )));
                }

                let mut available = Vec::new();
                let mut entries = tokio::fs::read_dir(&folder).await?;
                while let Some(entry) = entries.next_entry().await? {
                    let path = entry.path();
                    if path.is_file() {
                        let extension = path
                            .extension()
                            .and_then(|value| value.to_str())
                            .unwrap_or_default()
                            .to_ascii_lowercase();
                        if matches!(
                            extension.as_str(),
                            "jpg" | "jpeg" | "png" | "gif" | "webp" | "bmp" | "tif" | "tiff"
                        ) {
                            available.push(path);
                        }
                    }
                }
                available.sort();
                let number = Regex::new(r"\d+")?;
                for (index, section) in sections.iter().enumerate() {
                    let exact = available.iter().find(|image| {
                        let stem = image
                            .file_stem()
                            .and_then(|value| value.to_str())
                            .unwrap_or("");
                        !stem.is_empty()
                            && (section.title.contains(stem) || stem.contains(&section.title))
                    });
                    let matched = exact.or_else(|| {
                        number.find(&section.title).and_then(|chapter_number| {
                            available.iter().find(|image| {
                                image
                                    .file_stem()
                                    .and_then(|value| value.to_str())
                                    .is_some_and(|stem| stem.contains(chapter_number.as_str()))
                            })
                        })
                    });
                    if let Some(image) = matched {
                        images.insert(index, image.clone());
                    }
                }
            }
            HeaderMode::Single => {
                let Some(image) = &self.book.chapter_header.image else {
                    return Ok(images);
                };
                let resolved = Self::resolve_resource_path(image, self.book.filename.parent())?;
                for index in 0..sections.len() {
                    images.insert(index, resolved.clone());
                }
            }
        }
        Ok(images)
    }

    fn generate_header_html(&self, resource: &str) -> Result<String> {
        let position = match self.book.chapter_header.position {
            ImagePosition::Left => "left",
            ImagePosition::Center => "center",
            ImagePosition::Right => "right",
        };
        let margins = match self.book.chapter_header.position {
            ImagePosition::Left => "margin-left: 0; margin-right: auto",
            ImagePosition::Center => "margin-left: auto; margin-right: auto",
            ImagePosition::Right => "margin-left: auto; margin-right: 0",
        };
        let mut image_styles = vec![margins.to_string()];
        if let Some(height) = &self.book.chapter_header.height {
            Self::validate_css_dimension(height, false)?;
            image_styles.push(format!("height: {height}"));
        }
        if let Some(width) = &self.book.chapter_header.width {
            Self::validate_css_dimension(width, true)?;
            image_styles.push(format!("width: {width}"));
        }
        Ok(format!(
            "<div class=\"chapter-header {position}\"><img src=\"{}\" alt=\"chapter header\" style=\"{};\"/></div>",
            escape_xml(resource),
            escape_xml(image_styles.join("; "))
        ))
    }

    fn validate_css_dimension(value: &str, allow_auto: bool) -> Result<()> {
        let value = value.trim();
        let valid = (allow_auto && value.eq_ignore_ascii_case("auto"))
            || value == "0"
            || Regex::new(r"^\d+(?:\.\d+)?(?:px|em|rem|%|vh|vw)$")?.is_match(value);
        if valid {
            Ok(())
        } else {
            Err(KafError::ParseError(format!("无效的页眉图片尺寸: {value}")))
        }
    }

    fn generate_chapter_html(
        &self,
        section: &Section,
        _index: usize,
        header_image: Option<String>,
    ) -> String {
        let mut html = String::from(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<!DOCTYPE html>\n<html xmlns=\"http://www.w3.org/1999/xhtml\">\n<head>\n  <meta charset=\"utf-8\"/>\n  <title>",
        );
        html.push_str(&escape_xml(&section.title));
        html.push_str("</title>\n  <link rel=\"stylesheet\" type=\"text/css\" href=\"stylesheet.css\"/>\n</head>\n<body>\n");
        if let Some(image) = header_image {
            html.push_str(&image);
            html.push('\n');
        }
        html.push_str("<h3 class=\"chapter-title\">");
        if self.book.separate_chapter_number {
            if let Some((number, title)) = self.split_chapter_number(&section.title) {
                html.push_str("<span class=\"chapter-number\">");
                html.push_str(&escape_xml(number));
                html.push_str("</span><br/>");
                html.push_str(&escape_xml(title));
            } else {
                html.push_str(&escape_xml(&section.title));
            }
        } else {
            html.push_str(&escape_xml(&section.title));
        }
        html.push_str("</h3><div class=\"chapter-content\">");
        html.push_str(&remove_invalid_xml_chars(&section.content));
        html.push_str("</div>\n</body>\n</html>");
        html
    }

    fn generate_toc_content(&self, sections: &[Section]) -> String {
        let mut items = String::new();
        for (index, section) in sections.iter().enumerate() {
            items.push_str(&format!(
                "      <li><a href=\"chapter_{index}.xhtml\">{}</a></li>\n",
                escape_xml(&section.title)
            ));
        }
        format!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<!DOCTYPE html>\n<html xmlns=\"http://www.w3.org/1999/xhtml\" xmlns:epub=\"http://www.idpf.org/2007/ops\">\n<head><meta charset=\"utf-8\"/><title>目录</title><link rel=\"stylesheet\" type=\"text/css\" href=\"stylesheet.css\"/></head>\n<body><nav epub:type=\"toc\" id=\"toc\"><h1>目录</h1><ol>\n{items}    </ol></nav></body>\n</html>"
        )
    }

    fn split_chapter_number<'a>(&self, title: &'a str) -> Option<(&'a str, &'a str)> {
        let captures = CHAPTER_NUMBER.captures(title)?;
        Some((
            captures.get(1)?.as_str(),
            captures.get(2).map(|value| value.as_str()).unwrap_or(""),
        ))
    }

    fn font_resource(
        &self,
        font_path: &Path,
    ) -> Result<(&'static str, &'static str, &'static str)> {
        let extension = font_path
            .extension()
            .and_then(|value| value.to_str())
            .unwrap_or_default()
            .to_ascii_lowercase();
        match extension.as_str() {
            "ttf" => Ok(("fonts/custom-font.ttf", "font/ttf", "truetype")),
            "otf" => Ok(("fonts/custom-font.otf", "font/otf", "opentype")),
            "woff" => Ok(("fonts/custom-font.woff", "font/woff", "woff")),
            "woff2" => Ok(("fonts/custom-font.woff2", "font/woff2", "woff2")),
            "ttc" => Ok(("fonts/custom-font.ttc", "font/collection", "opentype")),
            _ => Err(KafError::ParseError(format!(
                "不支持的字体格式: {}",
                font_path.display()
            ))),
        }
    }

    fn embed_font(&self, path: &Path, builder: &mut EpubBuilder<ZipLibrary>) -> Result<()> {
        let resolved = Self::resolve_resource_path(path, self.book.filename.parent())?;
        let (resource, mime, _) = self.font_resource(&resolved)?;
        builder.add_resource(
            PathBuf::from(resource),
            Cursor::new(std::fs::read(resolved)?),
            mime,
        )?;
        Ok(())
    }

    fn build_css(&self) -> Result<String> {
        let theme = match self.book.theme {
            crate::model::ThemePreset::Light => crate::style::Theme::light(),
            crate::model::ThemePreset::Dark => crate::style::Theme::dark(),
            crate::model::ThemePreset::Sepia => crate::style::Theme::sepia(),
            crate::model::ThemePreset::HighContrast => crate::style::Theme::high_contrast(),
            crate::model::ThemePreset::Modern => crate::style::Theme::modern(),
            crate::model::ThemePreset::Traditional => crate::style::Theme::traditional(),
        };
        let mut css = crate::style::CssGenerator::new().generate(&self.book, &theme);

        if let Some(path) = &self.book.font {
            let (resource, _, font_format) = self.font_resource(path)?;
            css.push_str(&format!(
                "\n@font-face {{ font-family: 'CustomFont'; src: url('{resource}') format('{font_format}'); }}\nbody {{ font-family: 'CustomFont', serif; }}\n"
            ));
        }
        if let Some(path) = &self.book.custom_css {
            let resolved = Self::resolve_resource_path(path, self.book.filename.parent())?;
            css.push_str("\n/* 用户自定义 CSS */\n");
            css.push_str(&std::fs::read_to_string(resolved)?);
        }
        if let Some(extended) = &self.book.extended_css {
            css.push_str("\n/* 扩展 CSS */\n");
            css.push_str(extended);
        }
        if !self.book.css_variables.is_empty() {
            let mut variables = BTreeMap::new();

            // Canonical names win when an old alias is also supplied.
            for (key, value) in &self.book.css_variables {
                let stripped = key.strip_prefix("--").unwrap_or(key);
                let canonical = Self::canonical_css_variable(stripped);
                if canonical == stripped {
                    variables.insert(canonical.to_string(), value);
                }
            }
            for (key, value) in &self.book.css_variables {
                let stripped = key.strip_prefix("--").unwrap_or(key);
                let canonical = Self::canonical_css_variable(stripped);
                variables.entry(canonical.to_string()).or_insert(value);
            }

            css.push_str("\n:root {\n");
            for (key, value) in variables {
                css.push_str(&format!("  --{key}: {value};\n"));
            }
            css.push_str("}\n");
        }
        Ok(css)
    }

    fn canonical_css_variable(name: &str) -> &str {
        match name {
            "background-color" => "bg-color",
            "link-color" | "primary-color" => "accent-color",
            "font-size" => "base-size",
            other => other,
        }
    }
}
