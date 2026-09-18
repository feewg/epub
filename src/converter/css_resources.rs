//! 自定义 CSS 资源打包器
//!
//! 处理 `custom_css` / `extended_css` / `css_variables` 三个入口中的资源引用：
//!
//! - 本地 `url(...)` 引用（图片/字体）：解析存在则打包进 EPUB 并重写引用路径；
//! - 本地无条件 `@import "x.css"`：读取文件内容递归内联（带循环与深度保护）；
//! - 远程 URL（`http://` 等）、协议相对 `//`、`file:`/盘符等带 scheme 的引用、
//!   片段引用 `#id`、`@import` 的 media/layer/supports 条件、不存在或扩展名
//!   不在白名单的本地文件：一律返回明确错误，而不是生成缺少资源的 EPUB。
//!
//! 程序自身由 `--font` 生成的 `@font-face` 规则不经过本模块，保持原样。
//!
//! 实现为单趟字符扫描（识别注释与字符串），不引入完整 CSS 解析器：
//! 注释和字符串字面量内部的 `url(`/`@import` 不会被误处理。

use crate::error::{KafError, Result};
use epub_builder::{EpubBuilder, ZipLibrary};
use std::collections::HashMap;
use std::io::Cursor;
use std::path::{Path, PathBuf};

/// 允许打包进 EPUB 的 CSS 资源扩展名。
const SUPPORTED_EXTENSIONS: &[&str] = &[
    "png", "jpg", "jpeg", "gif", "svg", "webp", "bmp", "ttf", "otf", "woff", "woff2",
];

/// `@import` 递归内联的最大深度。
const MAX_IMPORT_DEPTH: usize = 8;

/// 自定义 CSS 资源打包器。
pub(crate) struct CssResourcePackager<'a> {
    builder: &'a mut EpubBuilder<ZipLibrary>,
    /// 已打包资源：源文件规范化路径 -> EPUB 内资源路径（去重）。
    assets: HashMap<PathBuf, String>,
    /// 资源命名序号，按首次引用顺序分配，保证输出确定。
    next_index: usize,
}

impl<'a> CssResourcePackager<'a> {
    pub(crate) fn new(builder: &'a mut EpubBuilder<ZipLibrary>) -> Self {
        Self {
            builder,
            assets: HashMap::new(),
            next_index: 0,
        }
    }

    /// 处理一段自定义 CSS 文本，返回重写资源引用后的文本。
    ///
    /// `dirs` 是解析相对引用的基准目录（按顺序尝试）；`label` 仅用于错误信息。
    pub(crate) fn process_css(
        &mut self,
        css: &str,
        dirs: &[PathBuf],
        label: &str,
    ) -> Result<String> {
        self.scan(css, dirs, label, &mut Vec::new())
    }

    fn scan(
        &mut self,
        css: &str,
        dirs: &[PathBuf],
        label: &str,
        import_stack: &mut Vec<PathBuf>,
    ) -> Result<String> {
        let bytes = css.as_bytes();
        let mut output = String::with_capacity(css.len());
        let mut index = 0usize;
        while index < bytes.len() {
            let rest = &bytes[index..];
            // 注释：原样保留，内部不识别任何引用。
            if rest.starts_with(b"/*") {
                let end = css[index + 2..]
                    .find("*/")
                    .map(|offset| (index + 2 + offset + 2).min(bytes.len()))
                    .unwrap_or(bytes.len());
                output.push_str(&css[index..end]);
                index = end;
                continue;
            }
            // 字符串：原样保留。
            let current = rest[0];
            if current == b'"' || current == b'\'' {
                let end = string_end(bytes, index);
                output.push_str(&css[index..end]);
                index = end;
                continue;
            }
            // @import：仅支持本地无条件导入。
            if starts_with_ignore_case(rest, b"@import")
                && rest.len() > 7
                && matches!(rest[7], b' ' | b'\t' | b'\r' | b'\n' | b'"' | b'\'')
            {
                let (inlined, end) = self.handle_import(css, index, dirs, label, import_stack)?;
                output.push_str(&inlined);
                index = end;
                continue;
            }
            // url( ：本地资源打包重写，其余情况显式拒绝或原样保留。
            if starts_with_ignore_case(rest, b"url(") {
                let (rewritten, end) = self.handle_url(css, index, dirs, label)?;
                output.push_str(&rewritten);
                index = end;
                continue;
            }
            let character = css[index..].chars().next().expect("索引处必有字符");
            output.push(character);
            index += character.len_utf8();
        }
        Ok(output)
    }

    /// 处理 `@import`（`at` 指向 `@`）。
    ///
    /// 仅接受 `@import "x.css";` / `@import 'x.css';` / `@import url(x.css);`
    /// 且不带任何 media/layer/supports 条件的形式；本地文件递归内联。
    fn handle_import(
        &mut self,
        css: &str,
        at: usize,
        dirs: &[PathBuf],
        label: &str,
        import_stack: &mut Vec<PathBuf>,
    ) -> Result<(String, usize)> {
        let bytes = css.as_bytes();
        let mut cursor = at + "@import".len();
        cursor += count_leading_whitespace(&bytes[cursor..]);

        // 解析导入目标：url(...) 或带引号的字符串。
        let (target, mut cursor_after) = if starts_with_ignore_case(&bytes[cursor..], b"url(") {
            let (value, end) = url_token_value(css, cursor);
            (value, end)
        } else if bytes.get(cursor) == Some(&b'"') || bytes.get(cursor) == Some(&b'\'') {
            let end = string_end(bytes, cursor);
            let inner = &css[cursor + 1..end.saturating_sub(1)];
            (unquote_css_string(inner), end)
        } else {
            return Err(KafError::ParseError(format!(
                "不支持的 @import 语法（{label}）：目标必须是本地 CSS 文件的字符串或 url() 引用"
            )));
        };

        cursor_after += count_leading_whitespace(&bytes[cursor_after..]);
        match bytes.get(cursor_after) {
            None => {}
            Some(b';') => cursor_after += 1,
            Some(_) => {
                return Err(KafError::ParseError(format!(
                    "不支持的 @import 语法（{label}）：EPUB 生成不支持带 media/layer/supports 条件的 @import，请直接合并 CSS 文件"
                )));
            }
        }

        if import_stack.len() >= MAX_IMPORT_DEPTH {
            return Err(KafError::ParseError(format!(
                "@import 嵌套超过 {MAX_IMPORT_DEPTH} 层（{label}）：存在过深的导入链"
            )));
        }

        let resolved = self.resolve_asset(&target, dirs, label)?;
        if resolved
            .extension()
            .and_then(|v| v.to_str())
            .map(|v| v.eq_ignore_ascii_case("css"))
            != Some(true)
        {
            return Err(KafError::ParseError(format!(
                "@import 目标不是 .css 文件（{label}）: {target}"
            )));
        }
        let canonical = std::fs::canonicalize(&resolved).unwrap_or_else(|_| resolved.clone());
        if import_stack.contains(&canonical) {
            return Err(KafError::ParseError(format!(
                "检测到循环 @import（{label}）: {}",
                canonical.display()
            )));
        }

        let content = std::fs::read_to_string(&resolved).map_err(|error| {
            KafError::ParseError(format!(
                "无法读取 @import 的 CSS 文件 {}: {error}",
                resolved.display()
            ))
        })?;
        import_stack.push(canonical);
        // 导入文件中的相对引用以其自身目录优先，其次继承原基准目录。
        let mut nested_dirs = Vec::new();
        if let Some(parent) = resolved.parent() {
            nested_dirs.push(parent.to_path_buf());
        }
        nested_dirs.extend(dirs.iter().cloned());
        let inlined = self.scan(&content, &nested_dirs, label, import_stack)?;
        import_stack.pop();

        let header = format!("/* 内联 @import: {} */\n", resolved.display());
        Ok((format!("{header}{inlined}\n"), cursor_after))
    }

    /// 处理 `url(...)`（`at` 指向 `u`/`U`）。
    ///
    /// 返回替换后的文本片段与结束索引。`data:` URI 原样保留；
    /// 本地文件打包为 `css-assets/N.ext` 并重写引用；其余显式报错。
    fn handle_url(
        &mut self,
        css: &str,
        at: usize,
        dirs: &[PathBuf],
        label: &str,
    ) -> Result<(String, usize)> {
        let (value, quoted, end) = url_token(css, at);
        if value.is_empty() {
            return Err(KafError::ParseError(format!(
                "自定义 CSS 中出现空的 url() 引用（{label}）"
            )));
        }
        if value.trim_start().len() >= 5 && value[..5].eq_ignore_ascii_case("data:") {
            // 内嵌 data URI 自包含，无需打包，原样保留。
            return Ok((css[at..end].to_string(), end));
        }
        if value.starts_with('#') {
            return Err(KafError::ParseError(format!(
                "自定义 CSS 不支持片段引用 url({value})（{label}）：EPUB 样式表中没有可引用的文档内元素"
            )));
        }
        if has_uri_scheme(&value) || value.starts_with("//") || value.starts_with("\\\\") {
            return Err(KafError::ParseError(format!(
                "自定义 CSS 不支持远程或带协议的 url({value})（{label}）：请使用本地资源，由转换器打包进 EPUB"
            )));
        }

        // 分离片段（SVG sprite 等形式），路径部分参与打包。
        let (path_part, fragment) = match value.split_once('#') {
            Some((path, fragment)) => (path.to_string(), Some(fragment.to_string())),
            None => (value.clone(), None),
        };

        let resolved = self.resolve_asset(&path_part, dirs, label)?;
        let resource = self.package_asset(&resolved)?;
        let reference = match fragment {
            Some(fragment) => format!("{resource}#{fragment}"),
            None => resource,
        };
        if quoted {
            Ok((format!("url(\"{reference}\")"), end))
        } else {
            Ok((format!("url({reference})"), end))
        }
    }

    /// 解析 CSS 本地资源引用（尝试原始路径与百分号解码路径）。
    fn resolve_asset(&self, reference: &str, dirs: &[PathBuf], label: &str) -> Result<PathBuf> {
        let mut candidates = vec![reference.to_string()];
        if let Some(decoded) = percent_decode_uri(reference) {
            if decoded != reference {
                candidates.push(decoded);
            }
        }
        for candidate in &candidates {
            for dir in dirs {
                let joined = dir.join(candidate);
                if joined.is_file() {
                    return Ok(std::fs::canonicalize(&joined).unwrap_or(joined));
                }
            }
            if let Ok(cwd) = std::env::current_dir() {
                let joined = cwd.join(candidate);
                if joined.is_file() {
                    return Ok(std::fs::canonicalize(&joined).unwrap_or(joined));
                }
            }
        }
        Err(KafError::FileNotFound(format!(
            "自定义 CSS 引用的资源不存在（{label}）: {reference}"
        )))
    }

    /// 打包资源文件并返回 EPUB 内相对 stylesheet.css 的引用路径。
    fn package_asset(&mut self, resolved: &Path) -> Result<String> {
        if let Some(existing) = self.assets.get(resolved) {
            return Ok(existing.clone());
        }
        let extension = resolved
            .extension()
            .and_then(|value| value.to_str())
            .unwrap_or_default()
            .to_ascii_lowercase();
        let mime = css_asset_mime(&extension).ok_or_else(|| {
            KafError::ParseError(format!(
                "自定义 CSS 引用了不支持的资源类型 .{extension}（支持: {SUPPORTED_EXTENSIONS:?}）: {}",
                resolved.display()
            ))
        })?;
        let data = std::fs::read(resolved).map_err(|error| {
            KafError::ParseError(format!(
                "无法读取自定义 CSS 资源 {}: {error}",
                resolved.display()
            ))
        })?;
        let resource = format!("css-assets/{}.{}", self.next_index, extension);
        self.next_index += 1;
        self.builder
            .add_resource(PathBuf::from(&resource), Cursor::new(data), mime)?;
        self.assets.insert(resolved.to_path_buf(), resource.clone());
        Ok(resource)
    }
}

/// 返回从 `start` 开始的字符串字面量的结束索引（含引号）。
fn string_end(bytes: &[u8], start: usize) -> usize {
    let quote = bytes[start];
    let mut index = start + 1;
    while index < bytes.len() {
        if bytes[index] == b'\\' {
            index += 2;
            continue;
        }
        if bytes[index] == quote {
            return (index + 1).min(bytes.len());
        }
        index += 1;
    }
    bytes.len()
}

fn count_leading_whitespace(bytes: &[u8]) -> usize {
    bytes
        .iter()
        .take_while(|byte| byte.is_ascii_whitespace())
        .count()
}

fn starts_with_ignore_case(haystack: &[u8], needle: &[u8]) -> bool {
    haystack.len() >= needle.len() && haystack[..needle.len()].eq_ignore_ascii_case(needle)
}

/// 解析 `url(...)` 标记，返回（去引号后的值, 是否带引号, 结束索引）。
fn url_token(css: &str, at: usize) -> (String, bool, usize) {
    let bytes = css.as_bytes();
    let mut cursor = at + "url(".len();
    cursor += count_leading_whitespace(&bytes[cursor..]);
    if bytes.get(cursor) == Some(&b'"') || bytes.get(cursor) == Some(&b'\'') {
        let end = string_end(bytes, cursor);
        let inner = &css[(cursor + 1).min(end)..end.saturating_sub(1)];
        let mut after = end.min(bytes.len());
        after += count_leading_whitespace(&bytes[after..]);
        if after < bytes.len() && bytes[after] == b')' {
            after += 1;
        } else {
            after = bytes.len();
        }
        return (unquote_css_string(inner), true, after);
    }
    let close = bytes[cursor..]
        .iter()
        .position(|byte| *byte == b')')
        .map(|offset| cursor + offset)
        .unwrap_or(bytes.len());
    (
        css[cursor..close].trim().to_string(),
        false,
        (close + 1).min(bytes.len()),
    )
}

/// 仅取 url() 内层值（供 @import 复用），忽略重写结果。
fn url_token_value(css: &str, at: usize) -> (String, usize) {
    let (value, _quoted, end) = url_token(css, at);
    (value, end)
}

/// 还原 CSS 字符串中的简单转义（`\X` -> `X`）。
fn unquote_css_string(inner: &str) -> String {
    let mut result = String::with_capacity(inner.len());
    let mut chars = inner.chars();
    while let Some(character) = chars.next() {
        if character == '\\' {
            if let Some(next) = chars.next() {
                result.push(next);
            }
        } else {
            result.push(character);
        }
    }
    result
}

/// 判断引用是否带有 URI scheme（如 `http:`、`file:`、`C:`）。
///
/// 冒号出现在任何 `/`、`?`、`#` 之前即视为 scheme 前缀。
fn has_uri_scheme(value: &str) -> bool {
    for character in value.chars() {
        match character {
            ':' => return true,
            '/' | '?' | '#' | '\\' => return false,
            _ => {}
        }
    }
    false
}

/// 对 URI 做百分号解码；仅解码合法 `%XX`，孤立 `%` 保留，非 UTF-8 返回 `None`。
fn percent_decode_uri(value: &str) -> Option<String> {
    if !value.contains('%') {
        return None;
    }
    let bytes = value.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%' && index + 2 < bytes.len() {
            let hex = &bytes[index + 1..index + 3];
            if hex.iter().all(|byte| byte.is_ascii_hexdigit()) {
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

fn css_asset_mime(extension: &str) -> Option<&'static str> {
    match extension {
        "png" => Some("image/png"),
        "jpg" | "jpeg" => Some("image/jpeg"),
        "gif" => Some("image/gif"),
        "svg" => Some("image/svg+xml"),
        "webp" => Some("image/webp"),
        "bmp" => Some("image/bmp"),
        // 与 `EpubConverter3::font_resource` 保持一致的字体 MIME 写法。
        "ttf" => Some("font/ttf"),
        "otf" => Some("font/otf"),
        "woff" => Some("font/woff"),
        "woff2" => Some("font/woff2"),
        _ => None,
    }
}
