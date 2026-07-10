//! Markdown 解析器
//!
//! 使用 CommonMark 事件流将 Markdown 转换为可嵌入 EPUB 的 XHTML 片段。

use crate::error::Result;
use crate::model::Section;
use pulldown_cmark::{html, CowStr, Event, Options, Parser as CommonMarkParser, Tag, TagEnd};
use tracing::debug;

/// Markdown 图片资源信息
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarkdownImage {
    /// 图片路径
    pub path: String,
    /// 图片替代文本
    pub alt: String,
    /// 所属章节索引
    pub chapter_index: usize,
}

/// Markdown 解析器
pub struct MarkdownParser {
    collect_images: bool,
    images: Vec<MarkdownImage>,
}

impl MarkdownParser {
    /// 创建新的 Markdown 解析器
    pub fn new() -> Self {
        Self {
            collect_images: true,
            images: Vec::new(),
        }
    }

    /// 设置是否收集图片资源
    pub fn with_image_collection(mut self, collect: bool) -> Self {
        self.collect_images = collect;
        self
    }

    /// 解析 Markdown 内容为章节列表。
    ///
    /// 每个顶层标题都会开始一个新章节；没有标题的内容会放入标题为空的章节。
    pub fn parse(&mut self, content: &str) -> Result<Vec<Section>> {
        self.images.clear();
        let events = CommonMarkParser::new_ext(content, Self::options())
            .map(Self::sanitize_event)
            .collect::<Vec<_>>();

        let mut sections = Vec::new();
        let mut pending_content = Vec::new();
        let mut current_title = String::new();
        let mut current_content = Vec::new();
        let mut block_depth = 0usize;
        let mut index = 0;

        while index < events.len() {
            if block_depth == 0 && matches!(events[index], Event::Start(Tag::Heading { .. })) {
                if !current_title.is_empty() || !current_content.is_empty() {
                    self.push_section(&mut sections, &current_title, &current_content);
                    current_content.clear();
                } else if !pending_content.is_empty() {
                    self.push_section(&mut sections, "", &pending_content);
                    pending_content.clear();
                }

                let heading_start = index + 1;
                index = heading_start;
                while index < events.len()
                    && !matches!(events[index], Event::End(TagEnd::Heading(_)))
                {
                    index += 1;
                }
                current_title = Self::plain_text(&events[heading_start..index]);
                index += 1;
                continue;
            }

            let event = events[index].clone();
            if current_title.is_empty() && sections.is_empty() {
                pending_content.push(event.clone());
            } else {
                current_content.push(event.clone());
            }
            match event {
                Event::Start(ref tag) if Self::is_block_container(tag) => block_depth += 1,
                Event::End(ref end) if Self::is_block_container_end(end) => {
                    block_depth = block_depth.saturating_sub(1);
                }
                _ => {}
            }
            index += 1;
        }

        if !current_title.is_empty() || !current_content.is_empty() {
            self.push_section(&mut sections, &current_title, &current_content);
        } else if !pending_content.is_empty() {
            self.push_section(&mut sections, "", &pending_content);
        }

        debug!(sections = sections.len(), "Markdown 解析完成");
        Ok(sections)
    }

    fn options() -> Options {
        Options::ENABLE_STRIKETHROUGH
            | Options::ENABLE_TABLES
            | Options::ENABLE_TASKLISTS
            | Options::ENABLE_YAML_STYLE_METADATA_BLOCKS
    }

    fn is_block_container(tag: &Tag<'_>) -> bool {
        !matches!(
            tag,
            Tag::Heading { .. }
                | Tag::Emphasis
                | Tag::Strong
                | Tag::Strikethrough
                | Tag::Superscript
                | Tag::Subscript
                | Tag::Link { .. }
                | Tag::Image { .. }
        )
    }

    fn is_block_container_end(tag: &TagEnd) -> bool {
        !matches!(
            tag,
            TagEnd::Heading(_)
                | TagEnd::Emphasis
                | TagEnd::Strong
                | TagEnd::Strikethrough
                | TagEnd::Superscript
                | TagEnd::Subscript
                | TagEnd::Link
                | TagEnd::Image
        )
    }

    fn sanitize_event(event: Event<'_>) -> Event<'static> {
        match event {
            // EPUB 章节内容不接受源文档中的任意原始 HTML。
            Event::Html(raw) | Event::InlineHtml(raw) => {
                Event::Text(CowStr::Boxed(raw.into_string().into_boxed_str()))
            }
            Event::Start(Tag::Link {
                link_type,
                dest_url,
                title,
                id,
            }) => Event::Start(Tag::Link {
                link_type,
                dest_url: CowStr::Boxed(Self::safe_uri(&dest_url, false).into_boxed_str()),
                title: title.into_static(),
                id: id.into_static(),
            }),
            Event::Start(Tag::Image {
                link_type,
                dest_url,
                title,
                id,
            }) => Event::Start(Tag::Image {
                link_type,
                dest_url: CowStr::Boxed(Self::safe_uri(&dest_url, true).into_boxed_str()),
                title: title.into_static(),
                id: id.into_static(),
            }),
            other => other.into_static(),
        }
    }

    fn safe_uri(value: &str, image: bool) -> String {
        let trimmed = value.trim();
        let lower = trimmed.to_ascii_lowercase();
        let allowed_scheme = lower.starts_with("http://")
            || lower.starts_with("https://")
            || (!image && lower.starts_with("mailto:"));
        let has_scheme = trimmed
            .split(['/', '?', '#'])
            .next()
            .is_some_and(|prefix| prefix.contains(':'));
        if allowed_scheme || !has_scheme {
            trimmed.to_string()
        } else {
            "#".to_string()
        }
    }

    fn push_section(
        &mut self,
        sections: &mut Vec<Section>,
        title: &str,
        events: &[Event<'static>],
    ) {
        let chapter_index = sections.len();
        if self.collect_images {
            self.collect_images_from_events(events, chapter_index);
        }

        let mut content = String::new();
        html::push_html(&mut content, events.iter().cloned());
        if !title.is_empty() || !content.trim().is_empty() {
            sections.push(Section {
                title: title.to_string(),
                content,
                subsections: Vec::new(),
            });
        }
    }

    fn collect_images_from_events(&mut self, events: &[Event<'static>], chapter_index: usize) {
        let mut image: Option<(String, String)> = None;
        for event in events {
            match event {
                Event::Start(Tag::Image { dest_url, .. }) => {
                    image = Some((dest_url.to_string(), String::new()));
                }
                Event::Text(text) | Event::Code(text) if image.is_some() => {
                    if let Some((_, alt)) = image.as_mut() {
                        alt.push_str(text);
                    }
                }
                Event::End(TagEnd::Image) => {
                    if let Some((path, alt)) = image.take() {
                        self.images.push(MarkdownImage {
                            path,
                            alt,
                            chapter_index,
                        });
                    }
                }
                _ => {}
            }
        }
    }

    fn plain_text(events: &[Event<'static>]) -> String {
        let mut text = String::new();
        for event in events {
            match event {
                Event::Text(value) | Event::Code(value) => text.push_str(value),
                Event::SoftBreak | Event::HardBreak => text.push(' '),
                _ => {}
            }
        }
        text.trim().to_string()
    }

    /// HTML/XML 转义特殊字符
    pub fn escape_html(text: &str) -> String {
        crate::utils::html::escape_xml(text)
    }

    /// 获取收集到的图片资源列表
    pub fn images(&self) -> &[MarkdownImage] {
        &self.images
    }

    /// 清空收集到的图片资源
    pub fn clear_images(&mut self) {
        self.images.clear();
    }
}

impl Default for MarkdownParser {
    fn default() -> Self {
        Self::new()
    }
}
