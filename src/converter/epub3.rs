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

/// 从章节标题中提取章节编号：`第N[章回节卷部幕集]`、`Chapter/Section/Page N`
/// 或以数字开头的标题；N 可以是阿拉伯数字或中文数字。
static CHAPTER_TITLE_NUMBER: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"(?i)(?:第\s*([0-9]+|[零〇一二三四五六七八九十百千万两]+)\s*[章回节卷部幕集]|(?:chapter|section|page)\s*([0-9]+)|^([0-9]+))",
    )
    .expect("固定章节编号提取正则必须有效")
});

/// 页眉图片文件名的编号形式：纯数字、纯中文数字或 `第N章` 形式。
static CHAPTER_STEM_NUMBER: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^第\s*([0-9]+|[零〇一二三四五六七八九十百千万两]+)\s*[章回节卷部幕集]$")
        .expect("固定页眉文件名编号正则必须有效")
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
        let css = self.build_css(&mut builder)?;
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
                    .title(self.nav_label(&section.title))
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
                std::fs::read(resolved).map_err(|error| {
                    KafError::EpubGenerationFailed(format!(
                        "无法读取封面图片 {}: {error}",
                        path.display()
                    ))
                })?
            }
            CoverSource::Data { data, .. } => data.clone(),
        };
        let (optimized, _) = cover::optimize_cover(&raw, &CoverConfig::default())?;
        let (data, mime, extension) = Self::prepare_epub_image(&optimized).map_err(|error| {
            KafError::EpubGenerationFailed(format!(
                "封面图片无法解码或超出资源限制（宽/高 <= {}px，内存 <= {}MB）: {error}",
                cover::MAX_DECODE_WIDTH,
                cover::MAX_DECODE_ALLOC / (1024 * 1024)
            ))
        })?;
        let internal_path = PathBuf::from(format!("cover.{extension}"));
        builder.add_cover_image(internal_path, Cursor::new(data), mime)?;
        Ok(())
    }

    /// 校验并准备写入 EPUB 的图片资源。
    ///
    /// 统一策略：
    /// - JPEG/PNG/GIF：先在资源限制内完整解码校验（防止截断/损坏数据混入），
    ///   校验通过后按原始字节透传，避免重编码损失合法内容；
    /// - 其他可解码格式：解码后统一转码为 PNG。
    fn prepare_epub_image(data: &[u8]) -> Result<(Vec<u8>, &'static str, &'static str)> {
        let format = cover::detect_image_format(data)?;
        match format {
            ImageFormat::Jpeg => {
                cover::validate_decodable(data)?;
                Ok((data.to_vec(), "image/jpeg", "jpg"))
            }
            ImageFormat::Png => {
                cover::validate_decodable(data)?;
                Ok((data.to_vec(), "image/png", "png"))
            }
            ImageFormat::Gif => {
                cover::validate_decodable(data)?;
                Ok((data.to_vec(), "image/gif", "gif"))
            }
            _ => {
                let image = cover::decode_image_with_limits(data)?;
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
            let raw = std::fs::read(&path).map_err(|error| {
                KafError::EpubGenerationFailed(format!(
                    "无法读取页眉图片 {}: {error}",
                    path.display()
                ))
            })?;
            let (data, mime, extension) = Self::prepare_epub_image(&raw).map_err(|error| {
                KafError::EpubGenerationFailed(format!(
                    "页眉图片无法解码或超出资源限制 {}: {error}",
                    path.display()
                ))
            })?;
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
            let key = self.resolve_local_image(path_part, source.as_str())?;
            let resource = if let Some(existing) = embedded.get(&key) {
                existing.clone()
            } else {
                let raw = std::fs::read(&key).map_err(|error| {
                    KafError::EpubGenerationFailed(format!(
                        "无法读取正文图片 {}: {error}",
                        key.display()
                    ))
                })?;
                let (data, mime, extension) = Self::prepare_epub_image(&raw).map_err(|error| {
                    KafError::EpubGenerationFailed(format!(
                        "正文图片无法解码或超出资源限制 {}: {error}",
                        key.display()
                    ))
                })?;
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

    /// 单个实体名称允许的最大长度，超过该长度的 `&...;` 序列按字面量处理。
    const MAX_ENTITY_LENGTH: usize = 32;

    /// 完整解码 XML 属性值中的字符引用。
    ///
    /// 支持 XML 预定义实体（`&amp;` `&lt;` `&gt;` `&apos;` `&quot;`）、
    /// 十进制（`&#39;`）与十六进制（`&#x27;`、`&#X27;`）字符引用。
    /// 单趟从左到右扫描，`&amp;lt;` 正确解码为字面量 `&lt;`，不会二次解码；
    /// 无法识别的实体（如 HTML 的 `&nbsp;`）按原样保留。
    fn decode_xml_attribute(value: &str) -> String {
        if !value.contains('&') {
            return value.to_string();
        }
        let mut result = String::with_capacity(value.len());
        let mut rest = value;
        while let Some(amp) = rest.find('&') {
            result.push_str(&rest[..amp]);
            let after = &rest[amp + 1..];
            let mut consumed = None;
            if let Some(semi) = after
                .find(';')
                .filter(|offset| *offset <= Self::MAX_ENTITY_LENGTH)
            {
                if let Some(decoded) = Self::decode_entity(&after[..semi]) {
                    result.push_str(&decoded);
                    consumed = Some(semi + 1);
                }
            }
            match consumed {
                Some(length) => rest = &after[length..],
                None => {
                    result.push('&');
                    rest = after;
                }
            }
        }
        result.push_str(rest);
        result
    }

    /// 解码单个实体内容（不含 `&` 与 `;`）。
    fn decode_entity(entity: &str) -> Option<String> {
        let decoded = if let Some(hex) = entity
            .strip_prefix("#x")
            .or_else(|| entity.strip_prefix("#X"))
        {
            char::from_u32(u32::from_str_radix(hex, 16).ok()?)
        } else if let Some(decimal) = entity.strip_prefix('#') {
            char::from_u32(decimal.parse::<u32>().ok()?)
        } else {
            match entity {
                "amp" => Some('&'),
                "lt" => Some('<'),
                "gt" => Some('>'),
                "apos" => Some('\''),
                "quot" => Some('"'),
                _ => None,
            }
        }?;
        let mut out = String::with_capacity(decoded.len_utf8());
        out.push(decoded);
        Some(out)
    }

    /// 将（已解码 XML 字符引用的）图片 URI 路径解析为本地文件。
    ///
    /// 依次尝试百分号解码后的路径和原始路径；每个候选都必须再次通过
    /// 远程引用检查，防止 `http%3A%2F%2F...` 之类编码 URI 被误当成本地文件。
    fn resolve_local_image(&self, path_part: &str, original: &str) -> Result<PathBuf> {
        let mut candidates: Vec<String> = Vec::new();
        if let Some(decoded) = Self::percent_decode_path(path_part) {
            if decoded != path_part {
                candidates.push(decoded);
            }
        }
        candidates.push(path_part.to_string());

        for candidate in &candidates {
            if Self::is_external_image(candidate) {
                return Err(KafError::ParseError(format!(
                    "EPUB 不支持未打包的远程或危险图片引用: {original}（解码后: {candidate}）"
                )));
            }
            if let Ok(resolved) =
                Self::resolve_resource_path(Path::new(candidate), self.book.filename.parent())
            {
                return Ok(std::fs::canonicalize(&resolved).unwrap_or(resolved));
            }
        }
        Err(KafError::FileNotFound(format!(
            "正文图片不存在: {path_part}（源引用: {original}）"
        )))
    }

    /// 对图片路径做百分号解码。
    ///
    /// 仅解码合法的 `%XX` 序列；孤立的 `%`（如文件名本身包含 `%`）按字面量保留。
    /// 解码结果必须是有效 UTF-8，否则返回 `None`，由调用方回退到原始路径。
    fn percent_decode_path(value: &str) -> Option<String> {
        if !value.contains('%') {
            return None;
        }
        let bytes = value.as_bytes();
        let mut decoded = Vec::with_capacity(bytes.len());
        let mut index = 0;
        while index < bytes.len() {
            if bytes[index] == b'%' && index + 2 < bytes.len() {
                let hex = &bytes[index + 1..index + 3];
                let is_hex = hex.iter().all(|byte| byte.is_ascii_hexdigit());
                if is_hex {
                    let high = (hex[0] as char).to_digit(16);
                    let low = (hex[1] as char).to_digit(16);
                    if let (Some(high), Some(low)) = (high, low) {
                        decoded.push((high * 16 + low) as u8);
                        index += 3;
                        continue;
                    }
                }
            }
            decoded.push(bytes[index]);
            index += 1;
        }
        String::from_utf8(decoded).ok()
    }

    fn is_external_image(source: &str) -> bool {
        let lower = source.trim().to_ascii_lowercase();
        lower.is_empty()
            || lower.starts_with("http://")
            || lower.starts_with("https://")
            || lower.starts_with("data:")
            || lower.starts_with('#')
            // 协议相对（//host/...）与 UNC（\\server\...）不允许作为本地图片路径。
            || lower.starts_with("//")
            || lower.starts_with("\\\\")
            || lower
                .split(['/', '?', '#'])
                .next()
                .is_some_and(|prefix| prefix.contains(':'))
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
                for (index, section) in sections.iter().enumerate() {
                    let title = section.title.trim();
                    // 空标题不做任何匹配，避免误选任意首图。
                    let matched = if title.is_empty() {
                        None
                    } else {
                        Self::match_header_image(title, &available)
                    };
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

    /// 为章节标题挑选页眉图片。
    ///
    /// 匹配优先级（同一优先级内按文件名长度、字典序确定唯一结果）：
    /// 1. 文件名（去扩展名）与标题完全一致；
    /// 2. 标题中的章节编号与文件名编号数值相等
    ///    （`1`/`01`/`第10章`/中文数字 `十` 互相等价）。
    ///
    /// 不再做子串包含匹配，避免 `第10章` 误选 `1.png`、空标题误选任意首图。
    fn match_header_image<'a>(title: &str, available: &'a [PathBuf]) -> Option<&'a PathBuf> {
        if let Some(image) = available
            .iter()
            .find(|path| Self::file_stem_str(path).is_some_and(|stem| stem == title))
        {
            return Some(image);
        }

        let chapter = Self::chapter_number_of(title)?;
        available
            .iter()
            .filter(|path| {
                Self::file_stem_str(path)
                    .is_some_and(|stem| Self::stem_number(stem) == Some(chapter))
            })
            .min_by_key(|path| {
                Self::file_stem_str(path)
                    .map(|stem| (stem.len(), stem.to_string()))
                    .unwrap_or_default()
            })
    }

    /// 文件名（不含扩展名）的字符串形式。
    fn file_stem_str(path: &Path) -> Option<&str> {
        path.file_stem().and_then(|value| value.to_str())
    }

    /// 从章节标题中提取章节编号（见 `CHAPTER_TITLE_NUMBER`）。
    fn chapter_number_of(title: &str) -> Option<u64> {
        let title = title.trim();
        if title.is_empty() {
            return None;
        }
        let captures = (*CHAPTER_TITLE_NUMBER).captures(title)?;
        let token = captures
            .iter()
            .skip(1)
            .find_map(|group| group.map(|value| value.as_str()))?;
        Self::parse_numeral(token)
    }

    /// 提取页眉图片文件名（不含扩展名）对应的章节编号。
    fn stem_number(stem: &str) -> Option<u64> {
        let stem = stem.trim();
        if stem.is_empty() {
            return None;
        }
        if let Some(value) = Self::parse_numeral(stem) {
            return Some(value);
        }
        let token = (*CHAPTER_STEM_NUMBER).captures(stem)?.get(1)?.as_str();
        Self::parse_numeral(token)
    }

    /// 解析数字字符串：纯阿拉伯数字（含前导零，如 `01`）或中文数字。
    fn parse_numeral(token: &str) -> Option<u64> {
        if token.is_empty() {
            return None;
        }
        if token.bytes().all(|byte| byte.is_ascii_digit()) {
            return token.parse::<u64>().ok();
        }
        Self::chinese_numeral_value(token)
    }

    /// 将中文数字（如 `十`、`二十一`、`一百零三`、`十万`）换算为数值。
    ///
    /// 无法解析时返回 `None`，此时该标题/文件名不参与编号匹配。
    fn chinese_numeral_value(token: &str) -> Option<u64> {
        if token.is_empty() {
            return None;
        }
        let digit_of = |character: char| -> u64 {
            match character {
                '一' => 1,
                '二' | '两' => 2,
                '三' => 3,
                '四' => 4,
                '五' => 5,
                '六' => 6,
                '七' => 7,
                '八' => 8,
                '九' => 9,
                _ => 0,
            }
        };
        let mut total: u64 = 0;
        let mut section: u64 = 0;
        let mut digit: u64 = 0;
        for character in token.chars() {
            match character {
                '零' | '〇' => digit = 0,
                '一' | '二' | '两' | '三' | '四' | '五' | '六' | '七' | '八' | '九' => {
                    digit = digit_of(character);
                }
                '十' | '百' | '千' => {
                    let unit = match character {
                        '十' => 10u64,
                        '百' => 100,
                        _ => 1_000,
                    };
                    section += digit.max(1).checked_mul(unit)?;
                    digit = 0;
                }
                '万' => {
                    total = total
                        .checked_add(section.checked_add(digit)?)?
                        .checked_mul(10_000)?;
                    section = 0;
                    digit = 0;
                }
                _ => return None,
            }
        }
        Some(total + section + digit)
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

    /// 章节的可见标签（nav / toc / guide / XHTML `<title>` 用）。
    ///
    /// 空白标题使用 `unknown_title` 兜底，避免生成空的目录标签和
    /// `<title></title>`（EPUBCheck RSC-005 要求 title 非空）。
    /// 不修改 Section 原文，正文标题仍按原样渲染。
    fn nav_label<'a>(&'a self, title: &'a str) -> &'a str {
        if title.trim().is_empty() {
            &self.book.unknown_title
        } else {
            title
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
        html.push_str(&escape_xml(self.nav_label(&section.title)));
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
                escape_xml(self.nav_label(&section.title))
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

    /// 生成样式表。
    ///
    /// 除程序自身生成的规则外，`custom_css` / `extended_css` / `css_variables`
    /// 三个入口统一经过 `CssResourcePackager`：本地 `url()` 资源被打包并重写，
    /// 本地无条件 `@import` 被递归内联，远程/带条件/缺失/不支持的引用显式报错。
    fn build_css(&self, builder: &mut EpubBuilder<ZipLibrary>) -> Result<String> {
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

        let book_dir: PathBuf = self
            .book
            .filename
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_default();
        let mut packager = crate::converter::css_resources::CssResourcePackager::new(builder);

        if let Some(path) = &self.book.custom_css {
            let resolved = Self::resolve_resource_path(path, self.book.filename.parent())?;
            // 自定义 CSS 内的相对引用以 CSS 文件所在目录优先，其次输入文件目录。
            let mut dirs = Vec::new();
            if let Some(parent) = resolved.parent() {
                dirs.push(parent.to_path_buf());
            }
            dirs.push(book_dir.clone());
            let label = format!("custom_css({})", path.display());
            let content = std::fs::read_to_string(&resolved).map_err(|error| {
                KafError::ParseError(format!(
                    "无法读取自定义 CSS {}: {error}",
                    resolved.display()
                ))
            })?;
            css.push_str("\n/* 用户自定义 CSS */\n");
            css.push_str(&packager.process_css(&content, &dirs, &label)?);
            css.push('\n');
        }
        if let Some(extended) = &self.book.extended_css {
            let dirs = vec![book_dir.clone()];
            css.push_str("\n/* 扩展 CSS */\n");
            css.push_str(&packager.process_css(extended, &dirs, "extended_css")?);
            css.push('\n');
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

            let dirs = vec![book_dir.clone()];
            css.push_str("\n:root {\n");
            for (key, value) in variables {
                let label = format!("css_variables[{key}]");
                let processed = packager.process_css(value, &dirs, &label)?;
                css.push_str(&format!("  --{key}: {processed};\n"));
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
