use super::markdown::MarkdownFeatures;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Block {
    Text(String),

    Heading { level: u8, content: String },

    Subtext(String),
    List(List),

    /// `> ` lines or everything after `>>> `, markers removed.
    Quote(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct List {
    pub ordered: bool,

    pub start: u64,

    pub items: Vec<Vec<Block>>,
}

const MAX_LIST_LEVEL: usize = 11;

pub fn blocks(text: &str, features: &MarkdownFeatures) -> Option<Vec<Block>> {
    if !features.blocks {
        return None;
    }
    let parsed = parse_blocks(text, true, 0, features.quotes);
    match parsed.as_slice() {
        [] | [Block::Text(_)] => None,
        _ => Some(parsed),
    }
}

fn parse_blocks(text: &str, allow_heading: bool, level: usize, quotes: bool) -> Vec<Block> {
    let mut out = Vec::new();
    let mut para_start = 0usize;
    let mut pos = 0usize;

    while pos < text.len() {
        let line_end = text[pos..].find('\n').map_or(text.len(), |rel| pos + rel);
        let line_start_char = text[pos..].trim_start_matches(' ').chars().next();
        let found = match line_start_char {
            Some('#') if allow_heading => match_heading(text, pos),
            Some('-') => match_subtext(text, pos).or_else(|| match_list(text, pos, level, quotes)),
            Some('*') | Some('0'..='9') => match_list(text, pos, level, quotes),
            Some('>') if quotes => match_quote(text, pos),
            _ => None,
        };
        if let Some((block, end)) = found {
            flush_text(&mut out, &text[para_start..pos]);
            out.push(block);
            pos = end;
            para_start = end;
            continue;
        }
        pos = line_end + 1;
    }
    if para_start < text.len() {
        out.push(Block::Text(text[para_start..].to_string()));
    }
    out
}

fn flush_text(out: &mut Vec<Block>, piece: &str) {
    if piece.is_empty() {
        return;
    }
    match piece.strip_suffix('\n') {
        Some("") => out.push(Block::Text(" ".to_string())),
        Some(rest) => out.push(Block::Text(rest.to_string())),
        None => out.push(Block::Text(piece.to_string())),
    }
}

/// A run of `> ` lines, or `>>> ` and everything after it.
fn match_quote(text: &str, pos: usize) -> Option<(Block, usize)> {
    let first = text[pos..].trim_start_matches(' ');
    if let Some(rest) = first.strip_prefix(">>> ") {
        return Some((Block::Quote(rest.to_string()), text.len()));
    }
    let mut lines: Vec<&str> = Vec::new();
    let mut end = pos;
    while end < text.len() {
        let line_end = text[end..].find('\n').map_or(text.len(), |rel| end + rel);
        let line = text[end..line_end].trim_start_matches(' ');
        match line.strip_prefix("> ") {
            Some(rest) if !line.starts_with(">>> ") => lines.push(rest),
            _ => break,
        }
        end = (line_end + 1).min(text.len());
    }
    (!lines.is_empty()).then(|| (Block::Quote(lines.join("\n")), end))
}

fn match_heading(text: &str, pos: usize) -> Option<(Block, usize)> {
    let bytes = text.as_bytes();
    let mut i = pos;
    while bytes.get(i) == Some(&b' ') {
        i += 1;
    }
    let hashes = bytes[i..].iter().take_while(|b| **b == b'#').count();
    if !(1..=3).contains(&hashes) {
        return None;
    }
    i += hashes;

    let after_hashes = i;
    while text[i..].chars().next().is_some_and(char::is_whitespace) {
        i += text[i..].chars().next().map_or(1, char::len_utf8);
    }
    if i == after_hashes || i >= text.len() {
        return None;
    }

    let more_hashes = bytes[i..].iter().take_while(|b| **b == b'#').count();
    if (1..=3).contains(&more_hashes)
        && text[i + more_hashes..]
            .chars()
            .next()
            .is_some_and(char::is_whitespace)
    {
        return None;
    }
    let line_end = text[i..].find('\n').map_or(text.len(), |rel| i + rel);

    let line = &text[i..line_end];
    let mut content = line.trim_end().trim_end_matches('#');
    if content.is_empty() {
        content = &line[..line.chars().next().map_or(0, char::len_utf8)];
    }

    let end = if line_end >= text.len() {
        text.len()
    } else {
        let mut j = line_end;
        while text[j..].chars().next().is_some_and(char::is_whitespace) {
            j += text[j..].chars().next().map_or(1, char::len_utf8);
        }
        if j >= text.len() {
            text.len()
        } else {
            text[..j]
                .rfind('\n')
                .map_or(line_end + 1, |last| last + 1)
                .max(line_end + 1)
        }
    };
    Some((
        Block::Heading {
            level: hashes as u8,
            content: content.to_string(),
        },
        end,
    ))
}

fn match_subtext(text: &str, pos: usize) -> Option<(Block, usize)> {
    let bytes = text.as_bytes();
    let mut i = pos;
    while bytes.get(i) == Some(&b' ') {
        i += 1;
    }
    if !text[i..].starts_with("-#") {
        return None;
    }
    i += 2;
    let spaces = bytes[i..].iter().take_while(|b| **b == b' ').count();
    if spaces == 0 {
        return None;
    }
    let line_end = text[i..].find('\n').map_or(text.len(), |rel| i + rel);

    let mut content_start = i + spaces;
    if content_start >= line_end {
        if spaces < 2 {
            return None;
        }
        content_start = line_end - 1;
    } else if text[content_start..].starts_with("-#") {
        if spaces < 2 {
            return None;
        }
        content_start -= 1;
    }
    let end = if line_end < text.len() {
        line_end + 1
    } else {
        line_end
    };
    Some((
        Block::Subtext(text[content_start..line_end].trim().to_string()),
        end,
    ))
}

fn marker_len(s: &str) -> Option<usize> {
    let bytes = s.as_bytes();
    match bytes.first()? {
        b'*' | b'-' => Some(1),
        b'0'..=b'9' => {
            let digits = bytes.iter().take_while(|b| b.is_ascii_digit()).count();
            (bytes.get(digits) == Some(&b'.')).then_some(digits + 1)
        }
        _ => None,
    }
}

fn starts_item(line: &str, indent: usize) -> bool {
    let spaces = line.bytes().take_while(|b| *b == b' ').count();
    (indent..=indent + 1).contains(&spaces)
        && marker_len(&line[spaces..])
            .is_some_and(|m| line.as_bytes().get(spaces + m) == Some(&b' '))
}

fn match_list(text: &str, pos: usize, level: usize, quotes: bool) -> Option<(Block, usize)> {
    if level >= MAX_LIST_LEVEL {
        return None;
    }
    let rest = &text[pos..];
    let indent = rest.bytes().take_while(|b| *b == b' ').count();
    let marker = marker_len(&rest[indent..])?;
    if rest.as_bytes().get(indent + marker) != Some(&b' ') {
        return None;
    }
    let first_line_end = rest.find('\n').unwrap_or(rest.len());
    if first_line_end <= indent + marker + 1 {
        return None;
    }

    let indent_str = &rest[..indent];
    let mut line_start = pos;
    let end = loop {
        let line_end = text[line_start..]
            .find('\n')
            .map_or(text.len(), |rel| line_start + rel);
        if line_end >= text.len() {
            break text.len();
        }
        let next = line_end + 1;
        let tail = &text[next..];
        let continues = tail.starts_with(' ')
            || (tail.starts_with(indent_str)
                && starts_item(tail, indent)
                && tail.bytes().take_while(|b| *b == b' ').count() == indent);
        if !continues {
            break next;
        }
        line_start = next;
    };
    let list_text = &text[pos..end];

    let prefix_spaces = rest[indent + marker..]
        .bytes()
        .take_while(|b| *b == b' ')
        .count();
    let prefix_len = indent + marker + prefix_spaces;
    let first_marker = &rest[indent..indent + marker];
    let ordered = marker > 1;
    let start = if ordered {
        first_marker
            .trim_end_matches('.')
            .parse::<u64>()
            .unwrap_or(1)
            .clamp(1, 1_000_000_000)
    } else {
        1
    };

    let mut raw_items: Vec<String> = Vec::new();
    for line in list_text.split_inclusive('\n') {
        if raw_items.is_empty() || starts_item(line.trim_end_matches('\n'), indent) {
            raw_items.push(String::new());
        }
        raw_items.last_mut().expect("just pushed").push_str(line);
    }
    let items = raw_items
        .iter()
        .map(|raw| {
            let spaces = raw.bytes().take_while(|b| *b == b' ').count();
            let after_marker = marker_len(&raw[spaces..]).map_or(spaces, |m| spaces + m);
            let content_start = after_marker
                + raw[after_marker..]
                    .bytes()
                    .take_while(|b| *b == b' ')
                    .count();
            let body: Vec<String> = raw[content_start..]
                .split('\n')
                .enumerate()
                .map(|(i, line)| {
                    if i == 0 {
                        line.to_string()
                    } else {
                        line[line
                            .bytes()
                            .take_while(|b| *b == b' ')
                            .count()
                            .min(prefix_len)..]
                            .to_string()
                    }
                })
                .collect();
            let mut body = body.join("\n");
            if body.ends_with('\n') {
                body.pop();
                body.truncate(body.trim_end_matches(' ').len());
            }
            let mut blocks = parse_blocks(&body, false, level + 1, quotes);
            for block in &mut blocks {
                if let Block::Text(content) = block {
                    trim_trailing_breaks(content);
                }
            }
            blocks.retain(|b| !matches!(b, Block::Text(t) if t.is_empty()));
            blocks
        })
        .collect();
    Some((
        Block::List(List {
            ordered,
            start,
            items,
        }),
        end,
    ))
}

fn trim_trailing_breaks(content: &mut String) {
    let trimmed = content.trim_end();
    let tail = &content[trimmed.len()..];
    if let Some(first_break) = tail.find('\n') {
        content.truncate(trimmed.len() + first_break);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn chat() -> MarkdownFeatures {
        MarkdownFeatures::CHAT
    }

    fn text(s: &str) -> Block {
        Block::Text(s.to_string())
    }

    fn heading(level: u8, s: &str) -> Block {
        Block::Heading {
            level,
            content: s.to_string(),
        }
    }

    fn list(ordered: bool, start: u64, items: Vec<Vec<Block>>) -> Block {
        Block::List(List {
            ordered,
            start,
            items,
        })
    }

    #[test]
    fn plain_text_has_no_blocks() {
        assert_eq!(blocks("just text\nover two lines", &chat()), None);
        assert_eq!(blocks("a # b and - c", &chat()), None);
        assert_eq!(blocks("# h", &MarkdownFeatures::ALL), None);
    }

    #[test]
    fn headings_have_three_levels() {
        assert_eq!(
            blocks("# one\n## two\n### three", &chat()),
            Some(vec![
                heading(1, "one"),
                heading(2, "two"),
                heading(3, "three")
            ])
        );

        assert_eq!(blocks("#### four", &chat()), None);
        assert_eq!(blocks("#nospace", &chat()), None);
        assert_eq!(blocks("\\# escaped", &chat()), None);

        assert_eq!(blocks("# # x", &chat()), None);
    }

    #[test]
    fn heading_drops_trailing_hashes_and_swallows_blank_lines() {
        assert_eq!(
            blocks("# title ##\n\n\nbody", &chat()),
            Some(vec![heading(1, "title "), text("body")])
        );
    }

    #[test]
    fn text_around_a_heading_loses_the_line_break_next_to_it() {
        assert_eq!(
            blocks("before\n# h\nafter", &chat()),
            Some(vec![text("before"), heading(1, "h"), text("after")])
        );

        assert_eq!(
            blocks("before\n\n# h", &chat()),
            Some(vec![text("before\n"), heading(1, "h")])
        );
    }

    #[test]
    fn subtext() {
        assert_eq!(
            blocks("-# small\nnext", &chat()),
            Some(vec![Block::Subtext("small".into()), text("next")])
        );
        assert_eq!(blocks("-#nospace", &chat()), None);

        assert_eq!(
            blocks("-# a\n\nb", &chat()),
            Some(vec![Block::Subtext("a".into()), text("\nb")])
        );
    }

    #[test]
    fn bullet_list() {
        assert_eq!(
            blocks("- a\n- b\n* c", &chat()),
            Some(vec![list(
                false,
                1,
                vec![vec![text("a")], vec![text("b")], vec![text("c")]]
            )])
        );
    }

    #[test]
    fn ordered_list_keeps_its_start() {
        assert_eq!(
            blocks("3. a\n7. b", &chat()),
            Some(vec![list(true, 3, vec![vec![text("a")], vec![text("b")]])])
        );
        assert_eq!(
            blocks("0. a", &chat()),
            Some(vec![list(true, 1, vec![vec![text("a")]])])
        );
    }

    #[test]
    fn list_needs_a_space_after_the_marker() {
        assert_eq!(blocks("-a\n*b*\n1.x", &chat()), None);
    }

    #[test]
    fn list_ends_at_a_plain_line() {
        assert_eq!(
            blocks("- a\ntext\n- b", &chat()),
            Some(vec![
                list(false, 1, vec![vec![text("a")]]),
                text("text"),
                list(false, 1, vec![vec![text("b")]])
            ])
        );

        assert_eq!(
            blocks("- a\n\ntext", &chat()),
            Some(vec![list(false, 1, vec![vec![text("a")]]), text("\ntext")])
        );
    }

    #[test]
    fn text_above_a_list_loses_the_line_break() {
        assert_eq!(
            blocks("intro\n- a", &chat()),
            Some(vec![text("intro"), list(false, 1, vec![vec![text("a")]])])
        );
    }

    #[test]
    fn nested_lists() {
        assert_eq!(
            blocks("- a\n  - b\n  - c\n- d", &chat()),
            Some(vec![list(
                false,
                1,
                vec![
                    vec![
                        text("a"),
                        list(false, 1, vec![vec![text("b")], vec![text("c")]])
                    ],
                    vec![text("d")]
                ]
            )])
        );
        assert_eq!(
            blocks("1. a\n   - b", &chat()),
            Some(vec![list(
                true,
                1,
                vec![vec![text("a"), list(false, 1, vec![vec![text("b")]])]]
            )])
        );
    }

    #[test]
    fn continuation_lines_stay_in_their_item() {
        assert_eq!(
            blocks("- a\n b\n- c", &chat()),
            Some(vec![list(
                false,
                1,
                vec![vec![text("a\nb")], vec![text("c")]]
            )])
        );
    }

    #[test]
    fn headings_are_text_inside_a_list_item() {
        assert_eq!(
            blocks("- # not a heading", &chat()),
            Some(vec![list(false, 1, vec![vec![text("# not a heading")]])])
        );
    }

    #[test]
    fn nothing_after_a_triple_quote_is_a_block() {
        assert_eq!(
            blocks(">>> quoted\n# not a heading\n- nor a list", &chat()),
            Some(vec![Block::Quote(
                "quoted\n# not a heading\n- nor a list".into()
            )])
        );
    }

    #[test]
    fn quote_lines_group_into_one_block() {
        assert_eq!(
            blocks("> one\n> two\n# heading", &chat()),
            Some(vec![Block::Quote("one\ntwo".into()), heading(1, "heading")])
        );
        assert_eq!(
            blocks("before\n> one\nafter", &chat()),
            Some(vec![
                text("before"),
                Block::Quote("one".into()),
                text("after")
            ])
        );
    }

    #[test]
    fn inline_markdown_stays_in_the_content() {
        assert_eq!(
            blocks("## **bold** title", &chat()),
            Some(vec![heading(2, "**bold** title")])
        );
        assert_eq!(
            blocks("* **x**", &chat()),
            Some(vec![list(false, 1, vec![vec![text("**x**")]])])
        );
    }
}
