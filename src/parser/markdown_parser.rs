//! Markdown 解析器
//!
//! 使用 CommonMark 事件流将 Markdown 转换为可嵌入 EPUB 的 XHTML 片段。

use crate::error::Result;
use crate::model::Section;
use pulldown_cmark::{html, CowStr, Event, Options, Parser as CommonMarkParser, Tag, TagEnd};
use std::collections::{HashMap, HashSet, VecDeque};
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
    /// 标题最大长度（按字符计）：超过该长度的标题不产生新章节，而是并入正文
    max_title_length: usize,
}

impl MarkdownParser {
    /// 创建新的 Markdown 解析器
    pub fn new() -> Self {
        Self {
            collect_images: true,
            images: Vec::new(),
            max_title_length: usize::MAX,
        }
    }

    /// 设置标题最大长度（字符数）
    pub fn with_max_title_length(mut self, max_title_length: usize) -> Self {
        self.max_title_length = max_title_length;
        self
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

        // 先按标题切分出原始章节（脚注定义仍留在事件流中，随所属章节提取）
        let mut raw_sections: Vec<(String, Vec<Event<'static>>)> = Vec::new();
        let mut pending_content: Vec<Event<'static>> = Vec::new();
        let mut current_title = String::new();
        let mut current_content = Vec::new();
        let mut block_depth = 0usize;
        let mut index = 0;

        while index < events.len() {
            if block_depth == 0 && matches!(events[index], Event::Start(Tag::Heading { .. })) {
                let heading_start = index + 1;
                let mut heading_end = heading_start;
                while heading_end < events.len()
                    && !matches!(events[heading_end], Event::End(TagEnd::Heading(_)))
                {
                    heading_end += 1;
                }
                let title = Self::plain_text(&events[heading_start..heading_end]);

                if title.chars().count() > self.max_title_length {
                    // 超长标题不产生新章节：标题事件原样并入当前章节内容，不丢失任何内容
                    let block_last = heading_end.min(events.len() - 1);
                    let target: &mut Vec<Event<'static>> =
                        if current_title.is_empty() && raw_sections.is_empty() {
                            &mut pending_content
                        } else {
                            &mut current_content
                        };
                    target.extend(events[index..=block_last].iter().cloned());
                    index = heading_end + 1;
                    continue;
                }

                if !current_title.is_empty() || !current_content.is_empty() {
                    raw_sections.push((
                        std::mem::take(&mut current_title),
                        std::mem::take(&mut current_content),
                    ));
                } else if !pending_content.is_empty() {
                    raw_sections.push((String::new(), std::mem::take(&mut pending_content)));
                }

                current_title = title;
                index = heading_end + 1;
                continue;
            }

            let event = events[index].clone();
            if current_title.is_empty() && raw_sections.is_empty() {
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
            raw_sections.push((current_title, current_content));
        } else if !pending_content.is_empty() {
            raw_sections.push((String::new(), pending_content));
        }

        // 脚注统一处理：
        // 1) 按所属章节提取定义；同章重复 label 的后续定义降级为普通内容（文字不丢、无重复锚点）
        // 2) 引用解析就近优先（本章定义 → 全篇首个定义），并对定义内部的脚注引用做
        //    有界传递闭包（队列 + 去重集合，循环/自引用安全，每章每 label 至多一个锚点）
        // 3) 未被引用的定义渲染到最后一章，同样应用传递闭包
        let mut footnotes = FootnoteState::default();
        for (section_index, (_, section_events)) in raw_sections.iter_mut().enumerate() {
            *section_events = footnotes.extract_definitions(
                std::mem::take(section_events),
                section_index,
                &mut HashSet::new(),
            );
        }
        for (section_index, (_, section_events)) in raw_sections.iter_mut().enumerate() {
            footnotes.resolve_section(section_events, section_index);
        }
        if let Some(last) = raw_sections.len().checked_sub(1) {
            footnotes.append_orphans(&mut raw_sections[last].1, last);
        }

        let mut sections = Vec::new();
        for (title, section_events) in raw_sections {
            self.push_section(&mut sections, &title, section_events);
        }

        debug!(sections = sections.len(), "Markdown 解析完成");
        Ok(sections)
    }

    fn options() -> Options {
        Options::ENABLE_STRIKETHROUGH
            | Options::ENABLE_TABLES
            | Options::ENABLE_TASKLISTS
            | Options::ENABLE_YAML_STYLE_METADATA_BLOCKS
            | Options::ENABLE_FOOTNOTES
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

    /// 定稿单个章节：渲染 XHTML、收集图片，空章节（无标题且无内容）丢弃。
    fn push_section(
        &mut self,
        sections: &mut Vec<Section>,
        title: &str,
        events: Vec<Event<'static>>,
    ) {
        let chapter_index = sections.len();
        if self.collect_images {
            self.collect_images_from_events(&events, chapter_index);
        }

        let mut content = String::new();
        html::push_html(&mut content, events.into_iter());
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

/// 单条脚注定义（含定义所在的原始章节索引，保持文档出现顺序）
struct FootnoteDef {
    section: usize,
    label: String,
    events: Vec<Event<'static>>,
}

/// `parse` 过程中的脚注解析状态。
///
/// 解析策略：
/// - 定义按“所属章节”提取；同章重复 label 的后续定义降级为普通内容（文字不丢、无重复锚点）；
/// - 引用解析就近优先：优先使用本章定义（支持每章编号重置的排版），
///   无本章定义时回退到全篇首个定义（跨章引用/文末定义仍可解析）；
/// - 定义内部的脚注引用做有界传递闭包：队列 + 已处理集合，
///   循环引用/自引用安全，每章每个 label 至多渲染一个锚点定义。
#[derive(Default)]
struct FootnoteState {
    /// 全部定义（文档顺序）
    defs: Vec<FootnoteDef>,
    /// label → `defs` 索引列表（文档顺序）
    by_label: HashMap<String, Vec<usize>>,
    /// 已被某章节选择渲染的定义索引
    used: HashSet<usize>,
    /// 各章节已落地锚点的 label 集合（索引为原始章节号）
    anchored: Vec<HashSet<String>>,
}

impl FootnoteState {
    /// 提取一个章节事件流中的脚注定义（含嵌套在定义内部容器中的定义）。
    ///
    /// 本章首次出现的 label 进入定义表；同章重复 label 的定义剥离外层
    /// `FootnoteDefinition` 标签，内部事件（段落/代码块/列表等完整闭合块）
    /// 作为普通内容保留在原位置。定义内部（如缩进引用块中）再出现的定义
    /// 递归收集为独立定义，从父定义内部移除，避免与父定义一起渲染时
    /// 产生重复锚点；`seen` 在整章（含嵌套层级）共享，保证每章每 label
    /// 至多注册一个定义。
    fn extract_definitions(
        &mut self,
        events: Vec<Event<'static>>,
        section: usize,
        seen: &mut HashSet<String>,
    ) -> Vec<Event<'static>> {
        let mut flow = Vec::with_capacity(events.len());
        let mut index = 0;
        while index < events.len() {
            let Event::Start(Tag::FootnoteDefinition(label)) = &events[index] else {
                flow.push(events[index].clone());
                index += 1;
                continue;
            };
            let mut depth = 1usize;
            let mut end = index + 1;
            while end < events.len() {
                match &events[end] {
                    Event::Start(Tag::FootnoteDefinition(_)) => depth += 1,
                    Event::End(TagEnd::FootnoteDefinition) => depth -= 1,
                    _ => {}
                }
                if depth == 0 {
                    break;
                }
                end += 1;
            }
            // 递归提取嵌套定义：父定义存储的内部事件不再包含子定义
            let inner = self.extract_definitions(events[(index + 1)..end].to_vec(), section, seen);
            if seen.insert(label.to_string()) {
                // 本章首个定义：提取后按引用就近渲染
                let label = label.to_string();
                self.by_label
                    .entry(label.clone())
                    .or_default()
                    .push(self.defs.len());
                self.defs.push(FootnoteDef {
                    section,
                    label,
                    events: inner,
                });
            } else {
                // 同章重复定义：内容按普通块保留，不产生锚点/不重复 id
                flow.extend(inner);
            }
            index = end + 1;
        }
        flow
    }

    /// 解析 label 在指定章节应使用的定义：就近（本章）优先，回退全篇首个定义。
    fn resolve_for(&self, section: usize, label: &str) -> Option<usize> {
        let indices = self.by_label.get(label)?;
        indices
            .iter()
            .copied()
            .find(|&idx| self.defs[idx].section == section)
            .or_else(|| indices.first().copied())
    }

    /// 事件流中出现的脚注引用 label（按出现顺序去重）。
    fn references_in(events: &[Event<'static>]) -> Vec<String> {
        let mut seen = HashSet::new();
        let mut labels = Vec::new();
        for event in events {
            if let Event::FootnoteReference(label) = event {
                if seen.insert(label.to_string()) {
                    labels.push(label.to_string());
                }
            }
        }
        labels
    }

    /// 追加一个定义（带锚点）到章节事件末尾。
    fn push_definition(events: &mut Vec<Event<'static>>, label: &str, inner: &[Event<'static>]) {
        events.push(Event::Start(Tag::FootnoteDefinition(CowStr::Boxed(
            label.to_string().into_boxed_str(),
        ))));
        events.extend(inner.iter().cloned());
        events.push(Event::End(TagEnd::FootnoteDefinition));
    }

    /// 解析一个章节：正文引用到的脚注定义（含定义内部的嵌套引用，有界闭包）
    /// 追加到章节事件末尾，保证引用锚点与定义位于同一 XHTML 文件。
    fn resolve_section(&mut self, events: &mut Vec<Event<'static>>, section: usize) {
        let mut pending: VecDeque<String> = Self::references_in(events).into_iter().collect();
        let mut queued: HashSet<String> = pending.iter().cloned().collect();
        let mut additions: Vec<(String, Vec<Event<'static>>)> = Vec::new();

        while let Some(label) = pending.pop_front() {
            let Some(idx) = self.resolve_for(section, &label) else {
                continue; // 未定义的引用：保持原状，不追加定义
            };
            self.used.insert(idx);
            let inner = self.defs[idx].events.clone();
            // 有界传递闭包：嵌套引用入队，每章每 label 只处理一次
            for next in Self::references_in(&inner) {
                if queued.insert(next.clone()) {
                    pending.push_back(next);
                }
            }
            additions.push((label, inner));
        }

        if additions.is_empty() {
            return;
        }
        if self.anchored.len() <= section {
            self.anchored.resize(section + 1, HashSet::new());
        }
        for (label, inner) in additions {
            Self::push_definition(events, &label, &inner);
            self.anchored[section].insert(label);
        }
    }

    /// 未被任何章节使用的定义渲染到最后一章：同 label 已有锚点时降级为普通内容
    /// （文字不丢、id 不重复），并补齐已渲染内容中引用到的全部依赖定义。
    fn append_orphans(&mut self, events: &mut Vec<Event<'static>>, section: usize) {
        let mut anchored: HashSet<String> = self.anchored.get(section).cloned().unwrap_or_default();
        let mut queued: HashSet<String> = anchored.clone();
        let mut pending: VecDeque<String> = VecDeque::new();

        for idx in 0..self.defs.len() {
            if self.used.contains(&idx) {
                continue;
            }
            let (label, inner) = (self.defs[idx].label.clone(), self.defs[idx].events.clone());
            for next in Self::references_in(&inner) {
                if queued.insert(next.clone()) {
                    pending.push_back(next);
                }
            }
            if anchored.insert(label.clone()) {
                Self::push_definition(events, &label, &inner);
            } else {
                // 同 label 锚点已存在：仅保留内容，不再产生重复锚点
                events.extend(inner);
            }
        }

        // 补齐闭包依赖：已渲染内容中引用到的 label 在本章还没有锚点时补上定义
        while let Some(label) = pending.pop_front() {
            if anchored.contains(&label) {
                continue;
            }
            let Some(idx) = self.resolve_for(section, &label) else {
                continue;
            };
            let inner = self.defs[idx].events.clone();
            for next in Self::references_in(&inner) {
                if queued.insert(next.clone()) {
                    pending.push_back(next);
                }
            }
            anchored.insert(label.clone());
            Self::push_definition(events, &label, &inner);
        }
    }
}

impl Default for MarkdownParser {
    fn default() -> Self {
        Self::new()
    }
}
