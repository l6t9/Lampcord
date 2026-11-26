//! Discord Markdown & syntax-highlighted code block renderer producing
//! Qt RichText / StyledText HTML suitable for QML `Text`.

use std::collections::HashMap;

fn escape_html(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            _ => out.push(ch),
        }
    }
    out
}

/// Lightweight keyword/string/comment/number highlighter for fenced code blocks.
fn highlight_code(code: &str, lang: &str) -> String {
    let lang = lang.trim().to_ascii_lowercase();
    let keywords: &[&str] = match lang.as_str() {
        "rs" | "rust" => &[
            "fn", "let", "mut", "pub", "struct", "enum", "impl", "trait", "for", "while", "loop",
            "if", "else", "match", "return", "use", "mod", "crate", "self", "Self", "const",
            "static", "async", "await", "where", "type", "true", "false", "Some", "None", "Ok",
            "Err", "move", "ref", "in", "as",
        ],
        "kt" | "kotlin" | "java" => &[
            "fun",
            "val",
            "var",
            "class",
            "object",
            "interface",
            "data",
            "sealed",
            "enum",
            "if",
            "else",
            "when",
            "for",
            "while",
            "return",
            "import",
            "package",
            "private",
            "public",
            "internal",
            "override",
            "suspend",
            "companion",
            "true",
            "false",
            "null",
            "this",
            "super",
            "in",
            "is",
            "as",
            "try",
            "catch",
            "finally",
        ],
        "js" | "ts" | "javascript" | "typescript" | "qml" => &[
            "function",
            "const",
            "let",
            "var",
            "if",
            "else",
            "for",
            "while",
            "return",
            "import",
            "export",
            "default",
            "class",
            "extends",
            "new",
            "this",
            "true",
            "false",
            "null",
            "undefined",
            "async",
            "await",
            "try",
            "catch",
            "switch",
            "case",
            "break",
            "property",
            "readonly",
            "signal",
            "required",
        ],
        "py" | "python" => &[
            "def", "class", "if", "elif", "else", "for", "while", "return", "import", "from", "as",
            "with", "try", "except", "finally", "yield", "lambda", "pass", "break", "continue",
            "True", "False", "None", "and", "or", "not", "in", "is", "async", "await",
        ],
        "c" | "cpp" | "c++" | "h" | "hpp" => &[
            "int",
            "void",
            "char",
            "bool",
            "float",
            "double",
            "long",
            "short",
            "unsigned",
            "signed",
            "const",
            "static",
            "struct",
            "class",
            "enum",
            "namespace",
            "template",
            "typename",
            "auto",
            "if",
            "else",
            "for",
            "while",
            "switch",
            "case",
            "return",
            "include",
            "true",
            "false",
            "nullptr",
            "public",
            "private",
            "protected",
            "virtual",
            "override",
        ],
        "sh" | "bash" | "zsh" | "shell" => &[
            "if", "then", "else", "elif", "fi", "for", "do", "done", "while", "case", "esac",
            "function", "return", "export", "local", "echo", "cd", "exit", "true", "false",
        ],
        _ => &[
            "fn", "fun", "def", "function", "let", "var", "const", "if", "else", "for", "while",
            "return", "class", "struct", "true", "false", "null", "None",
        ],
    };

    let kw_color = "#C792EA";
    let str_color = "#C3E88D";
    let num_color = "#F78C6C";
    let cmt_color = "#676E95";

    let mut out = String::with_capacity(code.len() * 2);
    for (line_idx, line) in code.lines().enumerate() {
        if line_idx > 0 {
            out.push_str("<br>");
        }
        let trimmed = line.trim_start();
        if trimmed.starts_with("//") || trimmed.starts_with('#') || trimmed.starts_with("--") {
            out.push_str(&format!(
                "<font color=\"{cmt_color}\"><i>{}</i></font>",
                escape_html(line).replace(' ', "&nbsp;")
            ));
            continue;
        }
        let chars: Vec<char> = line.chars().collect();
        let mut i = 0;
        while i < chars.len() {
            let c = chars[i];
            if c == ' ' {
                out.push_str("&nbsp;");
                i += 1;
            } else if c == '\t' {
                out.push_str("&nbsp;&nbsp;&nbsp;&nbsp;");
                i += 1;
            } else if c == '/' && i + 1 < chars.len() && chars[i + 1] == '/' {
                let rest: String = chars[i..].iter().collect();
                out.push_str(&format!(
                    "<font color=\"{cmt_color}\"><i>{}</i></font>",
                    escape_html(&rest).replace(' ', "&nbsp;")
                ));
                break;
            } else if c == '"' || c == '\'' {
                let quote = c;
                let start = i;
                i += 1;
                while i < chars.len() {
                    if chars[i] == '\\' && i + 1 < chars.len() {
                        i += 2;
                        continue;
                    }
                    if chars[i] == quote {
                        i += 1;
                        break;
                    }
                    i += 1;
                }
                let lit: String = chars[start..i].iter().collect();
                out.push_str(&format!(
                    "<font color=\"{str_color}\">{}</font>",
                    escape_html(&lit).replace(' ', "&nbsp;")
                ));
            } else if c.is_ascii_digit() {
                let start = i;
                while i < chars.len()
                    && (chars[i].is_ascii_alphanumeric() || chars[i] == '.' || chars[i] == '_')
                {
                    i += 1;
                }
                let num: String = chars[start..i].iter().collect();
                out.push_str(&format!(
                    "<font color=\"{num_color}\">{}</font>",
                    escape_html(&num)
                ));
            } else if c.is_ascii_alphabetic() || c == '_' {
                let start = i;
                while i < chars.len() && (chars[i].is_ascii_alphanumeric() || chars[i] == '_') {
                    i += 1;
                }
                let word: String = chars[start..i].iter().collect();
                if keywords.contains(&word.as_str()) {
                    out.push_str(&format!(
                        "<font color=\"{kw_color}\"><b>{}</b></font>",
                        escape_html(&word)
                    ));
                } else {
                    out.push_str(&escape_html(&word));
                }
            } else {
                out.push_str(&escape_html(&c.to_string()));
                i += 1;
            }
        }
    }
    out
}

fn is_emoji_cdn_url(url: &str) -> bool {
    url.starts_with("https://cdn.discordapp.com/emojis/")
        || url.starts_with("https://media.discordapp.net/emojis/")
        || url.starts_with("https://cdn.discordapp.com/stickers/")
        || url.starts_with("https://media.discordapp.net/stickers/")
}

fn is_emoji_start(ch: char) -> bool {
    let cp = ch as u32;
    matches!(
        cp,
        0x231A..=0x231B
            | 0x23E9..=0x23FA
            | 0x25AA..=0x25AB
            | 0x25B6
            | 0x25C0
            | 0x25FB..=0x25FE
            | 0x2600..=0x27BF
            | 0x2934..=0x2935
            | 0x2B05..=0x2B07
            | 0x2B1B..=0x2B1C
            | 0x2B50
            | 0x2B55
            | 0x3030
            | 0x303D
            | 0x3297
            | 0x3299
            | 0x1F000..=0x1FAFF
    )
}

fn is_emoji_continuation(ch: char) -> bool {
    let cp = ch as u32;
    cp == 0xFE0F
        || cp == 0x200D
        || cp == 0x20E3
        || (0x1F3FB..=0x1F3FF).contains(&cp)
        || (0xE0020..=0xE007F).contains(&cp)
}

/// Consumes one Unicode emoji cluster starting at the beginning of `s`.
fn consume_unicode_emoji(s: &str) -> Option<&str> {
    let mut chars = s.char_indices();
    let (_, first) = chars.next()?;
    let mut end = first.len_utf8();

    // Regional indicator flag pairs (e.g. 🇺🇸)
    if (0x1F1E6..=0x1F1FF).contains(&(first as u32)) {
        if let Some((idx, second)) = chars.next() {
            if (0x1F1E6..=0x1F1FF).contains(&(second as u32)) {
                return Some(&s[..idx + second.len_utf8()]);
            }
        }
        return None;
    }

    // Keycap sequences (e.g. 1️⃣)
    if first.is_ascii_digit() || first == '#' || first == '*' {
        let rest = &s[end..];
        if let Some(after_fe0f) = rest.strip_prefix('\u{FE0F}') {
            if after_fe0f.starts_with('\u{20E3}') {
                return Some(&s[..end + '\u{FE0F}'.len_utf8() + '\u{20E3}'.len_utf8()]);
            }
        } else if rest.starts_with('\u{20E3}') {
            return Some(&s[..end + '\u{20E3}'.len_utf8()]);
        }
        return None;
    }

    if !is_emoji_start(first) {
        return None;
    }

    let mut expect_zwj_next = false;
    for (idx, ch) in chars {
        if expect_zwj_next {
            if is_emoji_start(ch) || (0x2600..=0x27BF).contains(&(ch as u32)) {
                end = idx + ch.len_utf8();
                expect_zwj_next = false;
                continue;
            }
            break;
        }
        if ch == '\u{200D}' {
            end = idx + ch.len_utf8();
            expect_zwj_next = true;
        } else if is_emoji_continuation(ch) {
            end = idx + ch.len_utf8();
        } else {
            break;
        }
    }

    Some(&s[..end])
}

/// Returns true if the message contains only emojis (Unicode, custom, or
/// fake-Nitro CDN links) and whitespace, with between 1 and 27 emojis.
fn is_jumbo_emoji_only(content: &str) -> bool {
    let mut rest = content.trim();
    if rest.is_empty() {
        return false;
    }
    let mut count = 0usize;
    while !rest.is_empty() {
        rest = rest.trim_start();
        if rest.is_empty() {
            break;
        }
        // Custom emoji <:name:id> or <a:name:id>
        if rest.starts_with('<') {
            if let Some(close) = rest.find('>') {
                let inner = &rest[1..close];
                if let Some(after) = inner.strip_prefix("a:").or_else(|| inner.strip_prefix(':')) {
                    if let Some((name, id)) = after.split_once(':') {
                        if !name.is_empty()
                            && !id.is_empty()
                            && id.chars().all(|c| c.is_ascii_digit())
                        {
                            count += 1;
                            rest = &rest[close + 1..];
                            continue;
                        }
                    }
                }
            }
            return false;
        }
        // Fake Nitro masked link: [name](https://cdn.discordapp.com/emojis/...)
        if rest.starts_with('[') {
            if let Some(mid) = rest[1..].find("](") {
                let after_mid = 1 + mid + 2;
                if let Some(end) = rest[after_mid..].find(')') {
                    let url = &rest[after_mid..after_mid + end];
                    if is_emoji_cdn_url(url) {
                        count += 1;
                        rest = &rest[after_mid + end + 1..];
                        continue;
                    }
                }
            }
            return false;
        }
        // Bare Discord emoji CDN URL
        if rest.starts_with("https://") {
            let end = rest.find(char::is_whitespace).unwrap_or(rest.len());
            if is_emoji_cdn_url(&rest[..end]) {
                count += 1;
                rest = &rest[end..];
                continue;
            }
            return false;
        }
        // Unicode emoji
        if let Some(emoji) = consume_unicode_emoji(rest) {
            count += 1;
            rest = &rest[emoji.len()..];
            continue;
        }
        return false;
    }
    (1..=27).contains(&count)
}

/// Renders Discord markdown into Qt StyledText/RichText HTML.
pub fn render_markdown(
    content: &str,
    code_bg: &str,
    mention_bg: &str,
    mention_fg: &str,
    muted_fg: &str,
    mono_font: &str,
    member_names: &HashMap<u64, String>,
    channel_names: &HashMap<u64, String>,
    role_info: &HashMap<u64, (String, u32)>,
) -> String {
    if content.is_empty() {
        return String::new();
    }

    let emoji_px: u32 = if is_jumbo_emoji_only(content) { 48 } else { 22 };
    let mut out = String::with_capacity(content.len() * 2);
    let mut rest = content;
    let mut first_block = true;

    while !rest.is_empty() {
        if let Some(fence_start) = rest.find("```") {
            let before = &rest[..fence_start];
            if !before.is_empty() {
                if !first_block {
                    out.push_str("<br>");
                }
                out.push_str(&render_blocks(
                    before,
                    code_bg,
                    mention_bg,
                    mention_fg,
                    muted_fg,
                    mono_font,
                    emoji_px,
                    member_names,
                    channel_names,
                    role_info,
                ));
                first_block = false;
            }
            let after_open = &rest[fence_start + 3..];
            if let Some(fence_end) = after_open.find("```") {
                let block = &after_open[..fence_end];
                let (lang, code) = match block.find('\n') {
                    Some(nl) => {
                        let first = block[..nl].trim();
                        if first.chars().all(|c| {
                            c.is_ascii_alphanumeric() || matches!(c, '+' | '#' | '-' | '_')
                        }) {
                            (first, &block[nl + 1..])
                        } else {
                            ("", block)
                        }
                    }
                    None => ("", block),
                };
                if !first_block {
                    out.push_str("<br>");
                }
                let code_trimmed = code.trim_end_matches('\n');
                let header = if lang.is_empty() {
                    String::new()
                } else {
                    format!(
                        "<font size=\"2\" color=\"{muted_fg}\"><b>{}</b></font><br>",
                        escape_html(lang)
                    )
                };
                out.push_str(&format!(
                    "<table width=\"100%\" cellspacing=\"0\" cellpadding=\"8\" style=\"background-color:{code_bg};\"><tr><td>{header}<span style=\"font-family:'{mono_font}',monospace;font-size:13px;\">{}</span></td></tr></table>",
                    highlight_code(code_trimmed, lang)
                ));
                first_block = false;
                rest = after_open[fence_end + 3..]
                    .strip_prefix('\n')
                    .unwrap_or(&after_open[fence_end + 3..]);
            } else {
                // Unclosed code fence, render as plain inline blocks.
                if !first_block {
                    out.push_str("<br>");
                }
                out.push_str(&render_blocks(
                    &rest[fence_start..],
                    code_bg,
                    mention_bg,
                    mention_fg,
                    muted_fg,
                    mono_font,
                    emoji_px,
                    member_names,
                    channel_names,
                    role_info,
                ));
                break;
            }
        } else {
            if !first_block {
                out.push_str("<br>");
            }
            out.push_str(&render_blocks(
                rest,
                code_bg,
                mention_bg,
                mention_fg,
                muted_fg,
                mono_font,
                emoji_px,
                member_names,
                channel_names,
                role_info,
            ));
            break;
        }
    }

    out
}

#[allow(clippy::too_many_arguments)]
fn render_blocks(
    text: &str,
    code_bg: &str,
    mention_bg: &str,
    mention_fg: &str,
    muted_fg: &str,
    mono_font: &str,
    emoji_px: u32,
    member_names: &HashMap<u64, String>,
    channel_names: &HashMap<u64, String>,
    role_info: &HashMap<u64, (String, u32)>,
) -> String {
    let lines: Vec<&str> = text.split('\n').collect();
    let mut out = String::with_capacity(text.len() * 2);

    for (idx, raw_line) in lines.iter().enumerate() {
        if idx > 0 {
            out.push_str("<br>");
        }
        let line = *raw_line;
        if let Some(h1) = line.strip_prefix("# ") {
            out.push_str(&format!(
                "<span style=\"font-size:22px;font-weight:700;\">{}</span>",
                render_inline(
                    h1,
                    code_bg,
                    mention_bg,
                    mention_fg,
                    muted_fg,
                    mono_font,
                    emoji_px,
                    member_names,
                    channel_names,
                    role_info
                )
            ));
        } else if let Some(h2) = line.strip_prefix("## ") {
            out.push_str(&format!(
                "<span style=\"font-size:19px;font-weight:700;\">{}</span>",
                render_inline(
                    h2,
                    code_bg,
                    mention_bg,
                    mention_fg,
                    muted_fg,
                    mono_font,
                    emoji_px,
                    member_names,
                    channel_names,
                    role_info
                )
            ));
        } else if let Some(h3) = line.strip_prefix("### ") {
            out.push_str(&format!(
                "<span style=\"font-size:17px;font-weight:700;\">{}</span>",
                render_inline(
                    h3,
                    code_bg,
                    mention_bg,
                    mention_fg,
                    muted_fg,
                    mono_font,
                    emoji_px,
                    member_names,
                    channel_names,
                    role_info
                )
            ));
        } else if let Some(sub) = line.strip_prefix("-# ") {
            out.push_str(&format!(
                "<span style=\"font-size:12px;color:{muted_fg};\">{}</span>",
                render_inline(
                    sub,
                    code_bg,
                    mention_bg,
                    mention_fg,
                    muted_fg,
                    mono_font,
                    emoji_px,
                    member_names,
                    channel_names,
                    role_info
                )
            ));
        } else if let Some(quote) = line
            .strip_prefix("> ")
            .or_else(|| line.strip_prefix(">>> "))
        {
            out.push_str(&format!(
                "<span style=\"color:{muted_fg};\">│&nbsp;</span>{}",
                render_inline(
                    quote,
                    code_bg,
                    mention_bg,
                    mention_fg,
                    muted_fg,
                    mono_font,
                    emoji_px,
                    member_names,
                    channel_names,
                    role_info
                )
            ));
        } else if let Some(bullet) = line.strip_prefix("- ").or_else(|| line.strip_prefix("* ")) {
            out.push_str(&format!(
                "&nbsp;&nbsp;•&nbsp;{}",
                render_inline(
                    bullet,
                    code_bg,
                    mention_bg,
                    mention_fg,
                    muted_fg,
                    mono_font,
                    emoji_px,
                    member_names,
                    channel_names,
                    role_info
                )
            ));
        } else {
            out.push_str(&render_inline(
                line,
                code_bg,
                mention_bg,
                mention_fg,
                muted_fg,
                mono_font,
                emoji_px,
                member_names,
                channel_names,
                role_info,
            ));
        }
    }
    out
}

#[allow(clippy::too_many_arguments)]
fn render_inline(
    line: &str,
    code_bg: &str,
    mention_bg: &str,
    mention_fg: &str,
    muted_fg: &str,
    mono_font: &str,
    emoji_px: u32,
    member_names: &HashMap<u64, String>,
    channel_names: &HashMap<u64, String>,
    role_info: &HashMap<u64, (String, u32)>,
) -> String {
    let mut out = String::with_capacity(line.len() * 2);
    let bytes = line.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        if bytes[i] == b'`' {
            if let Some(end) = line[i + 1..].find('`') {
                let code = &line[i + 1..i + 1 + end];
                out.push_str(&format!(
                    "<span style=\"background-color:{code_bg};font-family:'{mono_font}',monospace;font-size:14px;\">&nbsp;{}&nbsp;</span>",
                    escape_html(code)
                ));
                i += end + 2;
                continue;
            }
        }

        // Discord tokens: <@id>, <@!id>, <#id>, <@&id>, <:name:id>, <a:name:id>, <t:unix:fmt>
        if bytes[i] == b'<' {
            if let Some(close_rel) = line[i..].find('>') {
                let token = &line[i + 1..i + close_rel];
                if let Some(html) = render_discord_token(
                    token,
                    mention_bg,
                    mention_fg,
                    muted_fg,
                    emoji_px,
                    member_names,
                    channel_names,
                    role_info,
                ) {
                    out.push_str(&html);
                    i += close_rel + 1;
                    continue;
                }
            }
        }

        // URLs: https://... or http://... (Discord CDN emojis render inline as images)
        if line[i..].starts_with("https://") || line[i..].starts_with("http://") {
            let rest = &line[i..];
            let end = rest
                .find(|c: char| c.is_whitespace() || matches!(c, '<' | '>' | '"' | '\''))
                .unwrap_or(rest.len());
            let mut url = &rest[..end];
            while url.ends_with(|c: char| matches!(c, '.' | ',' | ':' | ';' | ')' | ']')) {
                url = &url[..url.len() - 1];
            }
            let escaped = escape_html(url);
            if is_emoji_cdn_url(url) {
                out.push_str(&format!(
                    "<img src=\"{escaped}\" width=\"{emoji_px}\" height=\"{emoji_px}\" align=\"middle\">"
                ));
            } else {
                out.push_str(&format!("<a href=\"{escaped}\">{escaped}</a>"));
            }
            i += url.len();
            continue;
        }

        // Masked link: [label](https://...) — fake Nitro emojis render as images!
        if bytes[i] == b'[' {
            if let Some(mid) = line[i + 1..].find("](") {
                let after_mid = i + 1 + mid + 2;
                if let Some(end) = line[after_mid..].find(')') {
                    let label = &line[i + 1..i + 1 + mid];
                    let url = &line[after_mid..after_mid + end];
                    if url.starts_with("http://") || url.starts_with("https://") {
                        if is_emoji_cdn_url(url) {
                            out.push_str(&format!(
                                "<img src=\"{}\" width=\"{emoji_px}\" height=\"{emoji_px}\" align=\"middle\" alt=\"{}\">",
                                escape_html(url),
                                escape_html(label)
                            ));
                        } else {
                            out.push_str(&format!(
                                "<a href=\"{}\">{}</a>",
                                escape_html(url),
                                escape_html(label)
                            ));
                        }
                        i = after_mid + end + 1;
                        continue;
                    }
                }
            }
        }

        if line[i..].starts_with("***") {
            if let Some(end) = line[i + 3..].find("***") {
                let inner = &line[i + 3..i + 3 + end];
                out.push_str(&format!(
                    "<b><i>{}</i></b>",
                    render_inline(
                        inner,
                        code_bg,
                        mention_bg,
                        mention_fg,
                        muted_fg,
                        mono_font,
                        emoji_px,
                        member_names,
                        channel_names,
                        role_info
                    )
                ));
                i += end + 6;
                continue;
            }
        }

        if line[i..].starts_with("**") {
            if let Some(end) = line[i + 2..].find("**") {
                let inner = &line[i + 2..i + 2 + end];
                out.push_str(&format!(
                    "<b>{}</b>",
                    render_inline(
                        inner,
                        code_bg,
                        mention_bg,
                        mention_fg,
                        muted_fg,
                        mono_font,
                        emoji_px,
                        member_names,
                        channel_names,
                        role_info
                    )
                ));
                i += end + 4;
                continue;
            }
        }

        if line[i..].starts_with("__") {
            if let Some(end) = line[i + 2..].find("__") {
                let inner = &line[i + 2..i + 2 + end];
                out.push_str(&format!(
                    "<u>{}</u>",
                    render_inline(
                        inner,
                        code_bg,
                        mention_bg,
                        mention_fg,
                        muted_fg,
                        mono_font,
                        emoji_px,
                        member_names,
                        channel_names,
                        role_info
                    )
                ));
                i += end + 4;
                continue;
            }
        }

        if line[i..].starts_with("~~") {
            if let Some(end) = line[i + 2..].find("~~") {
                let inner = &line[i + 2..i + 2 + end];
                out.push_str(&format!(
                    "<s>{}</s>",
                    render_inline(
                        inner,
                        code_bg,
                        mention_bg,
                        mention_fg,
                        muted_fg,
                        mono_font,
                        emoji_px,
                        member_names,
                        channel_names,
                        role_info
                    )
                ));
                i += end + 4;
                continue;
            }
        }

        if line[i..].starts_with("||") {
            if let Some(end) = line[i + 2..].find("||") {
                let inner = &line[i + 2..i + 2 + end];
                out.push_str(&format!(
                    "<span style=\"background-color:{code_bg};color:{muted_fg};\">&nbsp;{}&nbsp;</span>",
                    escape_html(inner)
                ));
                i += end + 4;
                continue;
            }
        }

        if bytes[i] == b'*' {
            if let Some(end) = line[i + 1..].find('*') {
                if end > 0 {
                    let inner = &line[i + 1..i + 1 + end];
                    out.push_str(&format!(
                        "<i>{}</i>",
                        render_inline(
                            inner,
                            code_bg,
                            mention_bg,
                            mention_fg,
                            muted_fg,
                            mono_font,
                            emoji_px,
                            member_names,
                            channel_names,
                            role_info
                        )
                    ));
                    i += end + 2;
                    continue;
                }
            }
        }

        // Standard Unicode emoji -> Twemoji PNG with jumbo/inline sizing
        if let Some(emoji) = consume_unicode_emoji(&line[i..]) {
            let url = crate::emoji::twemoji_url(emoji);
            if !url.is_empty() {
                out.push_str(&format!(
                    "<img src=\"{url}\" width=\"{emoji_px}\" height=\"{emoji_px}\" align=\"middle\" alt=\"{}\">",
                    escape_html(emoji)
                ));
                i += emoji.len();
                continue;
            }
        }

        let ch = line[i..].chars().next().unwrap();
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            _ => out.push(ch),
        }
        i += ch.len_utf8();
    }

    out
}

#[allow(clippy::too_many_arguments)]
fn render_discord_token(
    token: &str,
    mention_bg: &str,
    mention_fg: &str,
    muted_fg: &str,
    emoji_px: u32,
    member_names: &HashMap<u64, String>,
    channel_names: &HashMap<u64, String>,
    role_info: &HashMap<u64, (String, u32)>,
) -> Option<String> {
    if let Some(rest) = token.strip_prefix("a:").or_else(|| token.strip_prefix(':')) {
        let animated = token.starts_with("a:");
        let (name, id) = rest.split_once(':')?;
        if !name.is_empty() && id.chars().all(|c| c.is_ascii_digit()) && !id.is_empty() {
            let ext = if animated { "gif" } else { "webp" };
            let req_size = if emoji_px >= 48 { 64 } else { 44 };
            return Some(format!(
                "<img src=\"https://cdn.discordapp.com/emojis/{id}.{ext}?size={req_size}\" width=\"{emoji_px}\" height=\"{emoji_px}\" align=\"middle\" alt=\":{name}:\">"
            ));
        }
    }

    if let Some(raw_id) = token.strip_prefix("@!").or_else(|| token.strip_prefix('@')) {
        if !raw_id.starts_with('&') {
            if let Ok(uid) = raw_id.parse::<u64>() {
                let label = member_names
                    .get(&uid)
                    .map(|n| format!("@{n}"))
                    .unwrap_or_else(|| format!("@{uid}"));
                return Some(format!(
                    "<span style=\"background-color:{mention_bg};color:{mention_fg};\"><b>&nbsp;{}&nbsp;</b></span>",
                    escape_html(&label)
                ));
            }
        }
    }

    if let Some(raw_id) = token.strip_prefix("@&") {
        if let Ok(rid) = raw_id.parse::<u64>() {
            if let Some((name, color)) = role_info.get(&rid) {
                let label = format!("@{name}");
                if *color != 0 {
                    let r = (*color >> 16) & 0xFF;
                    let g = (*color >> 8) & 0xFF;
                    let b = *color & 0xFF;
                    return Some(format!(
                        "<span style=\"background-color:rgba({r},{g},{b},0.16);color:#{color:06x};\"><b>&nbsp;{}&nbsp;</b></span>",
                        escape_html(&label)
                    ));
                }
                return Some(format!(
                    "<span style=\"background-color:{mention_bg};color:{mention_fg};\"><b>&nbsp;{}&nbsp;</b></span>",
                    escape_html(&label)
                ));
            }
            return Some(format!(
                "<span style=\"background-color:{mention_bg};color:{mention_fg};\"><b>&nbsp;@role&nbsp;</b></span>"
            ));
        }
    }

    if let Some(raw_id) = token.strip_prefix('#') {
        if let Ok(cid) = raw_id.parse::<u64>() {
            let label = channel_names
                .get(&cid)
                .map(|n| format!("#{n}"))
                .unwrap_or_else(|| format!("#{cid}"));
            return Some(format!(
                "<span style=\"background-color:{mention_bg};color:{mention_fg};\"><b>&nbsp;{}&nbsp;</b></span>",
                escape_html(&label)
            ));
        }
    }

    if let Some(rest) = token.strip_prefix("t:") {
        let ts_str = rest.split(':').next()?;
        if let Ok(secs) = ts_str.parse::<i64>() {
            let days = secs.div_euclid(86_400);
            let rem = secs.rem_euclid(86_400);
            let h = rem / 3600;
            let m = (rem % 3600) / 60;
            let (year, month, day) = civil_from_days(days);
            return Some(format!(
                "<span style=\"background-color:{mention_bg};color:{muted_fg};\">&nbsp;{year:04}-{month:02}-{day:02} {h:02}:{m:02} UTC&nbsp;</span>"
            ));
        }
    }

    None
}

/// Howard Hinnant's civil-from-days algorithm for UTC dates from Unix days.
pub fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = if m <= 2 { y + 1 } else { y };
    (year, m as u32, d as u32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_code_blocks_and_inline_markdown() {
        let html = render_markdown(
            "Hello **world** `code`\n```rust\nfn main() {}\n```",
            "#222",
            "#333",
            "#aaf",
            "#999",
            "monospace",
            &HashMap::new(),
            &HashMap::new(),
            &HashMap::new(),
        );
        assert!(html.contains("<b>world</b>"));
        assert!(html.contains("code"));
        assert!(html.contains("<b>fn</b>"));
    }

    #[test]
    fn renders_custom_emoji_and_mentions() {
        let mut members = HashMap::new();
        members.insert(42, "Alice".to_owned());
        let html = render_markdown(
            "Hi <@42> <:wave:123456>",
            "#222",
            "#333",
            "#aaf",
            "#999",
            "monospace",
            &members,
            &HashMap::new(),
            &HashMap::new(),
        );
        assert!(html.contains("@Alice"));
        assert!(html.contains("https://cdn.discordapp.com/emojis/123456.webp"));
    }
}
