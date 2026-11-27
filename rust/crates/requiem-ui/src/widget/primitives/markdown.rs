use std::ops::Range;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SpanAttrs {
    pub bold: bool,
    pub italic: bool,
    pub code: bool,
    pub strikethrough: bool,
    pub underline: bool,
    pub spoiler: bool,

    pub link: bool,

    pub subtext: bool,

    /// The marker characters themselves, not the spanned text.
    pub delimiter: bool,
}

pub struct Span {
    pub range: Range<usize>,
    pub attrs: SpanAttrs,

    pub href: Option<Range<usize>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MarkdownFeatures {
    pub bold: bool,
    pub italic: bool,
    pub code: bool,
    pub strikethrough: bool,
    pub underline: bool,
    pub spoiler: bool,
    pub link: bool,

    pub quotes: bool,

    /// `[label](url)` hides the url and paints `label` as the link.
    pub masked_links: bool,

    pub subtext: bool,

    /// Headings, lists, and quotes break out of the inline run.
    pub blocks: bool,
}

impl MarkdownFeatures {
    pub const NONE: Self = Self {
        bold: false,
        italic: false,
        code: false,
        strikethrough: false,
        underline: false,
        spoiler: false,
        link: false,
        quotes: false,
        masked_links: false,
        subtext: false,
        blocks: false,
    };

    pub const ALL: Self = Self {
        bold: true,
        italic: true,
        code: true,
        strikethrough: true,
        underline: true,
        spoiler: true,
        link: true,
        quotes: true,
        masked_links: false,
        subtext: true,
        blocks: false,
    };

    pub const CHAT: Self = Self {
        subtext: false,
        blocks: true,
        masked_links: true,
        ..Self::ALL
    };

    /// Every inline span with masked links, and no blocks.
    pub const INLINE: Self = Self {
        masked_links: true,
        subtext: false,
        ..Self::ALL
    };

    /// Profile bios.
    pub const BIO: Self = Self::INLINE;

    /// Bot-authored text: chat's spans plus `[label](url)` and `-#`.
    pub const EMBED: Self = Self {
        masked_links: true,
        subtext: true,
        ..Self::CHAT
    };
}

impl Default for MarkdownFeatures {
    fn default() -> Self {
        Self::NONE
    }
}

const DELIMITERS: &[(&str, fn(&mut SpanAttrs), fn(&MarkdownFeatures) -> bool)] = &[
    ("**", |a| a.bold = true, |f| f.bold),
    ("~~", |a| a.strikethrough = true, |f| f.strikethrough),
    ("__", |a| a.underline = true, |f| f.underline),
    ("||", |a| a.spoiler = true, |f| f.spoiler),
    ("`", |a| a.code = true, |f| f.code),
    ("_", |a| a.italic = true, |f| f.italic),
    ("*", |a| a.italic = true, |f| f.italic),
];

/// The text as plain characters: what is shown with every marker removed. For places that
/// can't draw markdown, such as a desktop notification. Block markers (`# `, `-# `, `> `) go
/// too; the words inside spoilers, code and links stay.
pub fn plain_text(text: &str) -> String {
    let unmarked: Vec<String> = text
        .lines()
        .map(|line| {
            let line = line.trim_start_matches(' ');
            if let Some(rest) = line.strip_prefix(">>> ") {
                return rest.to_string();
            }
            if let Some(len) = quote_marker_len(line) {
                return line[len..].to_string();
            }
            if let Some(rest) = line.strip_prefix("-# ") {
                return rest.to_string();
            }
            let hashes = line.bytes().take_while(|b| *b == b'#').count();
            if (1..=3).contains(&hashes) && line[hashes..].starts_with(' ') {
                return line[hashes + 1..].to_string();
            }
            line.to_string()
        })
        .collect();
    let joined = unmarked.join("
");
    let features = MarkdownFeatures {
        masked_links: true,
        ..MarkdownFeatures::ALL
    };
    // Underscores and asterisks inside a bare address belong to it.
    let addresses: Vec<Range<usize>> = scan_urls(&joined).into_iter().map(|u| u.range).collect();
    tokenize(&joined, features)
        .into_iter()
        .filter(|span| {
            !span.attrs.delimiter
                || addresses
                    .iter()
                    .any(|a| a.start <= span.range.start && span.range.end <= a.end)
        })
        .map(|span| &joined[span.range])
        .collect()
}

pub fn tokenize(text: &str, features: MarkdownFeatures) -> Vec<Span> {
    let masked = if features.masked_links && features.link {
        scan_masked_links(text)
    } else {
        Vec::new()
    };
    let skip: Vec<Range<usize>> = masked.iter().map(|m| m.url_part.clone()).collect();
    let mut raw = scan_delimited_spans(text, features, &skip);
    if features.link {
        raw.extend(scan_urls(text).into_iter().filter(|u| {
            !skip
                .iter()
                .any(|r| r.start < u.range.end && u.range.start < r.end)
        }));
    }
    let delim_attrs = SpanAttrs {
        delimiter: true,
        ..Default::default()
    };
    if features.subtext {
        scan_subtext(text, &mut raw);
    }
    for m in &masked {
        raw.push(Span {
            range: m.open.clone(),
            attrs: delim_attrs,
            href: None,
        });
        raw.push(Span {
            range: m.label.clone(),
            attrs: SpanAttrs {
                link: true,
                ..Default::default()
            },
            href: Some(m.url.clone()),
        });
        raw.push(Span {
            range: m.url_part.clone(),
            attrs: delim_attrs,
            href: None,
        });
    }
    merge_spans(text.len(), &raw)
}

fn scan_subtext(text: &str, raw: &mut Vec<Span>) {
    let mut line_start = 0usize;
    for line in text.split('\n') {
        let indent = line.len() - line.trim_start_matches(' ').len();
        let rest = &line[indent..];
        if let Some(after) = rest.strip_prefix("-#") {
            let spaces = after.len() - after.trim_start_matches(' ').len();
            let content = &after[spaces..];
            if spaces > 0 && !content.is_empty() && !content.starts_with("-#") {
                let marker = line_start + indent..line_start + indent + 2 + spaces;
                raw.push(Span {
                    range: marker.clone(),
                    attrs: SpanAttrs {
                        delimiter: true,
                        subtext: true,
                        ..Default::default()
                    },
                    href: None,
                });
                raw.push(Span {
                    range: marker.end..line_start + line.len(),
                    attrs: SpanAttrs {
                        subtext: true,
                        ..Default::default()
                    },
                    href: None,
                });
            }
        }
        line_start += line.len() + 1;
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ComposerCodeBlock {
    pub open: Range<usize>,

    pub content: Range<usize>,

    /// Info string on the opening fence. `None` when the fence named no language.
    pub lang: Option<String>,

    pub lines: Range<usize>,
    pub close: Option<Range<usize>>,
}

impl ComposerCodeBlock {
    pub fn full(&self, text_len: usize) -> Range<usize> {
        self.open.start..self.close.as_ref().map_or(text_len, |c| c.end)
    }
}

fn code_language(rest: &str) -> bool {
    rest.bytes()
        .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'+' | b'-' | b'.' | b'#'))
}

fn fence_starts(text: &str) -> Vec<usize> {
    let bytes = text.as_bytes();
    let mut out = Vec::new();
    let mut i = 0usize;
    while i < bytes.len() {
        if bytes[i] == b'\\' {
            i += 2;
        } else if bytes[i..].starts_with(b"```") {
            out.push(i);
            i += 3;
        } else {
            i += 1;
        }
    }
    out
}

pub fn composer_code_blocks(text: &str) -> Vec<ComposerCodeBlock> {
    let fences = fence_starts(text);
    let line_end = |from: usize| text[from..].find('\n').map_or(text.len(), |rel| from + rel);
    fences
        .chunks(2)
        .map(|pair| {
            let open = pair[0]..pair[0] + 3;
            let close = pair.get(1).map(|&c| c..c + 3);
            let close_start = close.as_ref().map_or(text.len(), |c| c.start);
            let open_line_end = line_end(open.end);
            if close_start < open_line_end || !code_language(&text[open.end..open_line_end]) {
                return ComposerCodeBlock {
                    content: open.end..close_start,
                    lang: None,
                    lines: open.end..open.end,
                    open,
                    close,
                };
            }
            let lang =
                Some(text[open.end..open_line_end].to_ascii_lowercase()).filter(|l| !l.is_empty());
            let content_start = (open_line_end + 1).min(close_start);
            let lines_end = match &close {
                Some(_) => text[content_start..close_start]
                    .rfind('\n')
                    .map_or(content_start, |rel| content_start + rel),
                None => text.len(),
            };
            ComposerCodeBlock {
                open,
                content: content_start..close_start,
                lang,
                lines: content_start..lines_end,
                close,
            }
        })
        .collect()
}

pub fn has_open_code_block_before(text: &str, offset: usize) -> bool {
    fence_starts(&text[..offset.min(text.len())]).len() % 2 == 1
}

struct MaskedLink {
    open: Range<usize>,
    label: Range<usize>,

    url_part: Range<usize>,
    url: Range<usize>,
}

fn scan_masked_links(text: &str) -> Vec<MaskedLink> {
    let mut out = Vec::new();
    let mut i = 0;
    while let Some(rel) = text[i..].find('[') {
        let open = i + rel;
        i = open + 1;
        if is_escaped(text, open) {
            continue;
        }
        let label_start = open + 1;
        let Some(close_rel) = text[label_start..].find([']', '\n']) else {
            continue;
        };
        let close = label_start + close_rel;
        if text.as_bytes()[close] != b']' || close == label_start || is_escaped(text, close) {
            continue;
        }
        if !text[close + 1..].starts_with('(') {
            continue;
        }
        // `(<url>)` is the same link with its embed suppressed.
        let angled = text[close + 2..].starts_with('<');
        let url_start = close + 2 + usize::from(angled);
        if !(text[url_start..].starts_with("http://") || text[url_start..].starts_with("https://"))
        {
            continue;
        }
        let stop = |c: char| c.is_whitespace() || c == if angled { '>' } else { ')' };
        let Some(end_rel) = text[url_start..].find(stop) else {
            continue;
        };
        let url_end = url_start + end_rel;
        let part_end = if angled {
            if !text[url_end..].starts_with(">)") {
                continue;
            }
            url_end + 2
        } else {
            if text.as_bytes()[url_end] != b')' {
                continue;
            }
            url_end + 1
        };
        out.push(MaskedLink {
            open: open..label_start,
            label: label_start..close,
            url_part: close..part_end,
            url: url_start..url_end,
        });
        i = part_end;
    }
    out
}

fn scan_delimited_spans(
    text: &str,
    features: MarkdownFeatures,
    skip: &[Range<usize>],
) -> Vec<Span> {
    let len = text.len();
    let mut spans = Vec::new();
    let mut i = 0usize;

    'outer: while i < len {
        if let Some(range) = skip.iter().find(|r| r.contains(&i)) {
            i = range.end;
            continue;
        }

        if let Some(escaped_len) = escape_len(text, i) {
            spans.push(Span {
                range: i..i + 1,
                attrs: SpanAttrs {
                    delimiter: true,
                    ..Default::default()
                },
                href: None,
            });
            i += 1 + escaped_len;
            continue;
        }
        for &(delim, set_attr, enabled) in DELIMITERS {
            if !enabled(&features) || !text[i..].starts_with(delim) {
                continue;
            }
            let content_start = i + delim.len();

            if let Some(rel) = find_unescaped(&text[content_start..], delim) {
                if rel > 0 {
                    let content_end = content_start + rel;
                    let close_end = content_end + delim.len();

                    let delim_attrs = SpanAttrs {
                        delimiter: true,
                        code: delim == "`",
                        ..Default::default()
                    };
                    spans.push(Span {
                        range: i..content_start,
                        attrs: delim_attrs,
                        href: None,
                    });

                    let mut content_attrs = SpanAttrs::default();
                    set_attr(&mut content_attrs);
                    spans.push(Span {
                        range: content_start..content_end,
                        attrs: content_attrs,
                        href: None,
                    });

                    spans.push(Span {
                        range: content_end..close_end,
                        attrs: delim_attrs,
                        href: None,
                    });

                    let mut j = content_start;
                    while j < content_end {
                        if let Some(escaped_len) = escape_len(text, j) {
                            spans.push(Span {
                                range: j..j + 1,
                                attrs: delim_attrs,
                                href: None,
                            });
                            j += 1 + escaped_len;
                        } else {
                            j += next_char_len(text, j);
                        }
                    }

                    i = close_end;
                    continue 'outer;
                }
            }
        }
        i += next_char_len(text, i);
    }

    spans
}

pub fn escape_len(text: &str, i: usize) -> Option<usize> {
    let bytes = text.as_bytes();
    (bytes.get(i) == Some(&b'\\') && bytes.get(i + 1).is_some_and(u8::is_ascii_punctuation))
        .then_some(1)
}

pub fn is_escaped(text: &str, i: usize) -> bool {
    text.as_bytes()[..i]
        .iter()
        .rev()
        .take_while(|b| **b == b'\\')
        .count()
        % 2
        == 1
}

fn find_unescaped(text: &str, delim: &str) -> Option<usize> {
    let mut i = 0;
    while i < text.len() {
        if let Some(escaped_len) = escape_len(text, i) {
            i += 1 + escaped_len;
        } else if text[i..].starts_with(delim) {
            return Some(i);
        } else {
            i += next_char_len(text, i);
        }
    }
    None
}

const URL_PREFIXES: &[&str] = &["https://", "http://", "www.", "discord.gg/"];

const URL_TRAILING_TRIM: &[char] = &['.', ',', '!', '?', ')', '*', '_', '~', '|', '`'];

fn scan_urls(text: &str) -> Vec<Span> {
    let mut spans = Vec::new();
    let mut i = 0usize;

    while i < text.len() {
        let Some(&prefix) = URL_PREFIXES.iter().find(|p| text[i..].starts_with(*p)) else {
            i += next_char_len(text, i);
            continue;
        };

        if prefix == "discord.gg/"
            && text[..i]
                .chars()
                .next_back()
                .is_some_and(|c| c.is_alphanumeric() || c == '.' || c == '-' || c == '/')
        {
            i += next_char_len(text, i);
            continue;
        }
        let mut end = i + prefix.len();
        while end < text.len() {
            let ch = text[end..].chars().next().expect("end < len");
            if ch.is_whitespace() {
                break;
            }
            end += ch.len_utf8();
        }

        let mut trimmed_end = end;
        if text[i..trimmed_end].ends_with(')') && !text[i..trimmed_end].contains('(') {
            trimmed_end -= 1;
        }
        while trimmed_end > i {
            let Some(last) = text[i..trimmed_end].chars().next_back() else {
                break;
            };
            if URL_TRAILING_TRIM.contains(&last) {
                trimmed_end -= last.len_utf8();
            } else {
                break;
            }
        }

        if trimmed_end > i {
            let attrs = SpanAttrs {
                link: true,
                ..Default::default()
            };
            spans.push(Span {
                range: i..trimmed_end,
                attrs,
                href: Some(i..trimmed_end),
            });
        }

        i = end.max(i + 1);
    }

    spans
}

fn merge_spans(len: usize, raw: &[Span]) -> Vec<Span> {
    let mut bounds: Vec<usize> = raw
        .iter()
        .flat_map(|s| [s.range.start, s.range.end])
        .collect();
    bounds.push(0);
    bounds.push(len);
    bounds.sort_unstable();
    bounds.dedup();

    let mut merged = Vec::new();
    for window in bounds.windows(2) {
        let (start, end) = (window[0], window[1]);
        if start >= end {
            continue;
        }
        let mut attrs = SpanAttrs::default();
        let mut href = None;
        for raw_span in raw {
            if raw_span.range.start <= start && end <= raw_span.range.end {
                href = href.or_else(|| raw_span.href.clone());
                attrs.bold |= raw_span.attrs.bold;
                attrs.italic |= raw_span.attrs.italic;
                attrs.code |= raw_span.attrs.code;
                attrs.strikethrough |= raw_span.attrs.strikethrough;
                attrs.underline |= raw_span.attrs.underline;
                attrs.spoiler |= raw_span.attrs.spoiler;
                attrs.link |= raw_span.attrs.link;
                attrs.subtext |= raw_span.attrs.subtext;
                attrs.delimiter |= raw_span.attrs.delimiter;
            }
        }
        merged.push(Span {
            range: start..end,
            attrs,
            href,
        });
    }
    merged
}

pub fn quote_prefix_ranges(text: &str) -> Vec<Range<usize>> {
    quote_prefix_ranges_excluding(text, &[])
}

pub(crate) fn quote_prefix_ranges_excluding(
    text: &str,
    excluded: &[Range<usize>],
) -> Vec<Range<usize>> {
    let mut ranges = Vec::new();
    let mut line_start = 0usize;
    for line in text.split('\n') {
        if !excluded
            .iter()
            .any(|r| r.start <= line_start && line_start < r.end)
        {
            if let Some(end) = quote_marker_len(line) {
                ranges.push(line_start..line_start + end);
            }
        }
        line_start += line.len() + 1;
    }
    ranges
}

pub fn quoted_lines(text: &str) -> Vec<bool> {
    quoted_lines_excluding(text, &[])
}

pub(crate) fn quoted_lines_excluding(text: &str, excluded: &[Range<usize>]) -> Vec<bool> {
    let mut line_start = 0usize;
    text.split('\n')
        .map(|line| {
            let quoted = !excluded
                .iter()
                .any(|r| r.start <= line_start && line_start < r.end)
                && quote_marker_len(line).is_some();
            line_start += line.len() + 1;
            quoted
        })
        .collect()
}

fn quote_marker_len(line: &str) -> Option<usize> {
    let bytes = line.as_bytes();
    let mut i = 0;
    while i < bytes.len() && (bytes[i] == b' ' || bytes[i] == b'\t') {
        i += 1;
    }
    let marker_start = i;
    while i < bytes.len() && bytes[i] == b'>' && i - marker_start < 3 {
        i += 1;
    }
    if i == marker_start {
        return None;
    }
    if i < bytes.len() && bytes[i] == b' ' {
        i += 1;
    }
    Some(i)
}

fn next_char_len(text: &str, at: usize) -> usize {
    text[at..].chars().next().map_or(1, char::len_utf8)
}

pub(crate) fn visible_subranges(range: Range<usize>, hidden: &[Range<usize>]) -> Vec<Range<usize>> {
    let mut pieces = vec![range];
    for h in hidden {
        let mut next = Vec::with_capacity(pieces.len());
        for r in pieces {
            if h.end <= r.start || h.start >= r.end {
                next.push(r);
            } else {
                if r.start < h.start {
                    next.push(r.start..h.start);
                }
                if h.end < r.end {
                    next.push(h.end..r.end);
                }
            }
        }
        pieces = next;
    }
    pieces
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kinds(text: &str) -> Vec<(SpanAttrs, &str)> {
        tokenize(text, MarkdownFeatures::ALL)
            .into_iter()
            .map(|s| (s.attrs, &text[s.range]))
            .collect()
    }

    fn masked(text: &str) -> Vec<(SpanAttrs, &str, Option<&str>)> {
        let features = MarkdownFeatures {
            masked_links: true,
            ..MarkdownFeatures::ALL
        };
        tokenize(text, features)
            .into_iter()
            .map(|s| (s.attrs, &text[s.range], s.href.map(|h| &text[h])))
            .collect()
    }

    #[test]
    fn angled_masked_links_link_the_label_and_hide_the_target() {
        let link = SpanAttrs {
            link: true,
            ..Default::default()
        };
        let text = "[first](<https://example.com/a/1>) \u{2022} [second](https://example.com/b)";
        let spans = masked(text);
        assert!(spans.contains(&(link, "first", Some("https://example.com/a/1"))));
        assert!(spans.contains(&(delim(), "](<https://example.com/a/1>)", None)));
        assert!(spans.contains(&(link, "second", Some("https://example.com/b"))));
        assert!(spans.contains(&(plain(), " \u{2022} ", None)));
    }

    #[test]
    fn masked_link_hides_wrapper_and_links_the_label() {
        let link = SpanAttrs {
            link: true,
            ..Default::default()
        };
        assert_eq!(
            masked("see [the docs](https://a.b/c_d_e) now"),
            [
                (plain(), "see ", None),
                (delim(), "[", None),
                (link, "the docs", Some("https://a.b/c_d_e")),
                (delim(), "](https://a.b/c_d_e)", None),
                (plain(), " now", None)
            ]
        );

        assert!(
            masked("[x](notaurl)")
                .iter()
                .all(|(a, _, h)| !a.link && h.is_none())
        );
        assert!(
            masked(r"\[x](https://a.b)")
                .iter()
                .all(|(a, t, _)| !(a.link && *t == "x"))
        );
        assert!(
            kinds("[x](https://a.b)")
                .iter()
                .all(|(a, t)| !(a.link && *t == "x"))
        );
    }

    #[test]
    fn bare_url_has_href() {
        let spans = masked("go https://a.b/c.");
        assert!(spans.contains(&(
            SpanAttrs {
                link: true,
                ..Default::default()
            },
            "https://a.b/c",
            Some("https://a.b/c")
        )));
    }

    #[test]
    fn backslash_escapes_a_delimiter() {
        let text = r"\*not italic\* and **bold**";
        let bold = SpanAttrs {
            bold: true,
            ..Default::default()
        };
        assert_eq!(
            kinds(text),
            [
                (delim(), "\\"),
                (plain(), "*not italic"),
                (delim(), "\\"),
                (plain(), "* and "),
                (delim(), "**"),
                (bold, "bold"),
                (delim(), "**")
            ]
        );
    }

    #[test]
    fn escaped_closer_does_not_close() {
        let text = r"*a\*b*";
        let italic = SpanAttrs {
            italic: true,
            ..Default::default()
        };
        let spans = kinds(text);
        assert_eq!(spans.first(), Some(&(delim(), "*")));
        assert!(spans.iter().any(|(a, t)| a.delimiter && *t == "\\"));
        assert!(spans.iter().any(|(a, t)| *a == italic && t.contains('b')));
    }

    #[test]
    fn is_escaped_counts_backslash_parity() {
        assert!(is_escaped(r"\<", 1));
        assert!(!is_escaped(r"\\<", 2));
        assert!(!is_escaped("<", 0));
    }

    fn plain() -> SpanAttrs {
        SpanAttrs::default()
    }
    fn delim() -> SpanAttrs {
        SpanAttrs {
            delimiter: true,
            ..Default::default()
        }
    }
    fn bold() -> SpanAttrs {
        SpanAttrs {
            bold: true,
            ..Default::default()
        }
    }
    fn italic() -> SpanAttrs {
        SpanAttrs {
            italic: true,
            ..Default::default()
        }
    }
    fn code() -> SpanAttrs {
        SpanAttrs {
            code: true,
            ..Default::default()
        }
    }
    fn code_delim() -> SpanAttrs {
        SpanAttrs {
            code: true,
            delimiter: true,
            ..Default::default()
        }
    }
    fn strike() -> SpanAttrs {
        SpanAttrs {
            strikethrough: true,
            ..Default::default()
        }
    }
    fn underline() -> SpanAttrs {
        SpanAttrs {
            underline: true,
            ..Default::default()
        }
    }
    fn spoiler() -> SpanAttrs {
        SpanAttrs {
            spoiler: true,
            ..Default::default()
        }
    }
    fn link() -> SpanAttrs {
        SpanAttrs {
            link: true,
            ..Default::default()
        }
    }

    #[test]
    fn composer_subtext_marks_the_marker_and_the_line() {
        let sub = SpanAttrs {
            subtext: true,
            ..Default::default()
        };
        let sub_delim = SpanAttrs {
            subtext: true,
            delimiter: true,
            ..Default::default()
        };
        assert_eq!(
            kinds(
                "a
-# small **x**
b"
            ),
            vec![
                (
                    plain(),
                    "a
"
                ),
                (sub_delim, "-# "),
                (sub, "small "),
                (
                    SpanAttrs {
                        subtext: true,
                        delimiter: true,
                        ..Default::default()
                    },
                    "**"
                ),
                (
                    SpanAttrs {
                        subtext: true,
                        bold: true,
                        ..Default::default()
                    },
                    "x"
                ),
                (
                    SpanAttrs {
                        subtext: true,
                        delimiter: true,
                        ..Default::default()
                    },
                    "**"
                ),
                (
                    plain(),
                    "
b"
                ),
            ]
        );

        assert_eq!(kinds("-#no"), vec![(plain(), "-#no")]);
        assert_eq!(kinds("a -# b"), vec![(plain(), "a -# b")]);
    }

    #[test]
    fn plain_text_is_one_span() {
        assert_eq!(kinds("just plain text"), vec![(plain(), "just plain text")]);
    }

    #[test]
    fn bold_splits_into_delimiter_content_delimiter() {
        assert_eq!(
            kinds("**bold**"),
            vec![(delim(), "**"), (bold(), "bold"), (delim(), "**")]
        );
    }

    #[test]
    fn underscore_and_asterisk_both_mean_italic() {
        assert_eq!(
            kinds("_a_"),
            vec![(delim(), "_"), (italic(), "a"), (delim(), "_")]
        );
        assert_eq!(
            kinds("*a*"),
            vec![(delim(), "*"), (italic(), "a"), (delim(), "*")]
        );
    }

    #[test]
    fn code_span() {
        assert_eq!(
            kinds("`code`"),
            vec![(code_delim(), "`"), (code(), "code"), (code_delim(), "`")]
        );
    }

    fn blocks(text: &str) -> Vec<(&str, Option<String>, &str, bool)> {
        composer_code_blocks(text)
            .into_iter()
            .map(|b| {
                (
                    &text[b.content.clone()],
                    b.lang.clone(),
                    &text[b.lines.clone()],
                    b.close.is_some(),
                )
            })
            .collect()
    }

    #[test]
    fn fenced_block_with_language() {
        let text = "hello\n```rust\nfn main() {\n}\n```\nafter";
        assert_eq!(
            blocks(text),
            vec![(
                "fn main() {\n}\n",
                Some("rust".into()),
                "fn main() {\n}",
                true
            )]
        );
    }

    #[test]
    fn unclosed_block_runs_to_the_end() {
        assert_eq!(
            blocks("```rs\nlet x = `y`"),
            vec![("let x = `y`", Some("rs".into()), "let x = `y`", false)]
        );

        assert_eq!(
            blocks("```rust"),
            vec![("", Some("rust".into()), "", false)]
        );
        assert_eq!(blocks("```"), vec![("", None, "", false)]);
    }

    #[test]
    fn same_line_block_is_code_without_language() {
        assert_eq!(
            blocks("a ```b **x**``` c"),
            vec![("b **x**", None, "", true)]
        );
    }

    #[test]
    fn non_language_rest_of_line_is_code_and_unhighlighted() {
        assert_eq!(
            blocks("```py extra\nx = 1\n```"),
            vec![("py extra\nx = 1\n", None, "", true)]
        );
    }

    #[test]
    fn mid_line_fence_opens_and_partial_closing_line_is_not_highlighted() {
        assert_eq!(
            blocks("see ```js\nlet y = 2;\nfoo```"),
            vec![("let y = 2;\nfoo", Some("js".into()), "let y = 2;", true)]
        );
    }

    #[test]
    fn escaped_fence_is_literal() {
        assert!(composer_code_blocks("\\```\nz").is_empty());
    }

    #[test]
    fn open_block_before_caret_counts_fences() {
        let text = "```js\nlet a;\n```\nafter";
        assert!(has_open_code_block_before(text, 8));
        assert!(!has_open_code_block_before(text, text.len()));
        assert!(!has_open_code_block_before("plain", 5));
    }

    #[test]
    fn unicode_code_block_is_byte_safe() {
        let text = "😀\n```\nlet café = \"✓\";\n```";
        assert_eq!(blocks(text)[0].0, "let café = \"✓\";\n");
    }

    #[test]
    fn quote_markers_inside_code_stay_literal() {
        let text = "```\n> literal\n```\n> quote";
        let excluded: Vec<Range<usize>> = composer_code_blocks(text)
            .into_iter()
            .map(|b| b.content)
            .collect();
        assert_eq!(
            quoted_lines_excluding(text, &excluded),
            vec![false, false, false, true]
        );
        let ranges = quote_prefix_ranges_excluding(text, &excluded);
        assert_eq!(ranges.len(), 1);
        assert_eq!(&text[ranges[0].clone()], "> ");
    }

    #[test]
    fn strikethrough_span() {
        assert_eq!(
            kinds("~~gone~~"),
            vec![(delim(), "~~"), (strike(), "gone"), (delim(), "~~")]
        );
    }

    #[test]
    fn underline_span_beats_italic_underscore() {
        assert_eq!(
            kinds("__ul__"),
            vec![(delim(), "__"), (underline(), "ul"), (delim(), "__")]
        );
    }

    #[test]
    fn spoiler_span() {
        assert_eq!(
            kinds("||secret||"),
            vec![(delim(), "||"), (spoiler(), "secret"), (delim(), "||")]
        );
    }

    #[test]
    fn bare_https_url_is_linked() {
        assert_eq!(
            kinds("see https://example.com/x for more"),
            vec![
                (plain(), "see "),
                (link(), "https://example.com/x"),
                (plain(), " for more"),
            ]
        );
    }

    #[test]
    fn url_inside_markdown_link_wrapper_only_links_the_url_part() {
        assert_eq!(
            kinds("[a link](https://example.com)"),
            vec![
                (plain(), "[a link]("),
                (link(), "https://example.com"),
                (plain(), ")"),
            ]
        );
    }

    #[test]
    fn url_trailing_punctuation_not_linked() {
        assert_eq!(
            kinds("go to www.example.com."),
            vec![
                (plain(), "go to "),
                (link(), "www.example.com"),
                (plain(), ".")
            ]
        );
    }

    #[test]
    fn mixed_with_surrounding_plain_text() {
        assert_eq!(
            kinds("hey **bold** and `code` ok"),
            vec![
                (plain(), "hey "),
                (delim(), "**"),
                (bold(), "bold"),
                (delim(), "**"),
                (plain(), " and "),
                (code_delim(), "`"),
                (code(), "code"),
                (code_delim(), "`"),
                (plain(), " ok"),
            ]
        );
    }

    #[test]
    fn empty_delimiter_pair_is_left_as_plain() {
        assert_eq!(kinds("****"), vec![(plain(), "****")]);
    }

    #[test]
    fn unmatched_delimiter_is_plain() {
        assert_eq!(kinds("**unterminated"), vec![(plain(), "**unterminated")]);
    }

    #[test]
    fn quote_prefix_is_detected_and_hidden_range_computed() {
        let text = "> quoted line\nplain line\n>>> also quoted";
        assert_eq!(quoted_lines(text), vec![true, false, true]);
        let ranges = quote_prefix_ranges(text);
        assert_eq!(&text[ranges[0].clone()], "> ");
        assert_eq!(&text[ranges[1].clone()], ">>> ");
    }

    #[test]
    fn line_without_quote_marker_has_no_hidden_range() {
        assert_eq!(
            quote_prefix_ranges("just plain text"),
            Vec::<Range<usize>>::new()
        );
        assert_eq!(quoted_lines("just plain text"), vec![false]);
    }

    #[test]
    fn none_features_disables_every_delimiter_and_link() {
        let text = "**bold** _italic_ `code` ~~s~~ __u__ ||spoiler|| https://example.com";
        let spans = tokenize(text, MarkdownFeatures::NONE);
        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].attrs, SpanAttrs::default());
        assert_eq!(&text[spans[0].range.clone()], text);
    }

    #[test]
    fn a_feature_subset_only_enables_its_own_delimiters() {
        let features = MarkdownFeatures {
            bold: true,
            ..MarkdownFeatures::NONE
        };
        assert_eq!(
            tokenize("**bold** _italic_ https://example.com", features)
                .into_iter()
                .map(|s| s.attrs)
                .collect::<Vec<_>>(),
            vec![delim(), bold(), delim(), plain()]
        );
    }
}
