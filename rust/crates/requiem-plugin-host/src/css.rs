use sha2::{Digest, Sha256};
use std::mem;

pub fn prefix(plugin_id: &str) -> String {
    let hash = Sha256::digest(plugin_id.as_bytes());
    format!(
        "p{:02x}{:02x}{:02x}{:02x}",
        hash[0], hash[1], hash[2], hash[3]
    )
}

pub fn view_class(plugin_id: &str) -> String {
    format!("{}--view", prefix(plugin_id))
}

pub fn scope_class(plugin_id: &str, class: &str) -> String {
    format!("{}-{}", prefix(plugin_id), sanitize_ident(class))
}

fn sanitize_ident(name: &str) -> String {
    name.chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
        .collect()
}

pub fn scope_inline_style(declarations: &str) -> String {
    let mut warnings = Vec::new();
    scope_declarations(declarations, "", &[], &mut warnings)
        .trim()
        .to_string()
}

pub struct Scoped {
    pub css: String,
    pub warnings: Vec<String>,
}

pub fn scope_stylesheet(plugin_id: &str, css: &str) -> Scoped {
    let prefix = prefix(plugin_id);
    let root = format!(".{prefix}--view");
    let css = strip_comments(css);
    let mut warnings = Vec::new();
    let mut out = String::new();

    let blocks = split_blocks(&css);
    let keyframe_names: Vec<String> = blocks
        .iter()
        .filter_map(|(prelude, _)| {
            prelude
                .trim()
                .strip_prefix("@keyframes")
                .map(|name| name.trim().to_string())
        })
        .collect();

    for (prelude, body) in blocks {
        let prelude = prelude.trim();
        if let Some(name) = prelude.strip_prefix("@keyframes") {
            out.push_str(&format!(
                "@keyframes {prefix}-{} {{{}}}\n",
                sanitize_ident(name.trim()),
                body
            ));
            continue;
        }
        if prelude.starts_with('@') {
            warnings.push(format!("dropped unsupported at-rule `{prelude}`"));
            continue;
        }
        let mut selectors = Vec::new();
        for selector in split_top_level(prelude, ',') {
            let selector = selector.trim();
            if selector.is_empty() {
                continue;
            }
            if selector.contains(":root") || names_html_or_body(selector) {
                warnings.push(format!(
                    "dropped selector `{selector}`: plugins can only style their own views"
                ));
                continue;
            }
            selectors.push(format!("{root} {}", prefix_classes(selector, &prefix)));
        }
        if selectors.is_empty() {
            continue;
        }
        let declarations = scope_declarations(&body, &prefix, &keyframe_names, &mut warnings);
        out.push_str(&format!("{} {{{declarations}}}\n", selectors.join(", ")));
    }
    Scoped { css: out, warnings }
}

fn scope_declarations(
    body: &str,
    prefix: &str,
    keyframes: &[String],
    warnings: &mut Vec<String>,
) -> String {
    let mut out = String::new();
    for declaration in body.split(';') {
        let Some((name, value)) = declaration.split_once(':') else {
            continue;
        };
        let name = name.trim();
        let mut value = value.trim().to_string();
        if name.starts_with("--") {
            warnings.push(format!("dropped custom property `{name}`: they're global in the engine, use classes instead"));
            continue;
        }
        if name == "position" && value == "fixed" {
            value = "absolute".to_string();
        }
        if name == "animation" || name == "animation-name" {
            value = value
                .split_whitespace()
                .map(|word| {
                    if keyframes.iter().any(|k| k == word.trim_end_matches(',')) {
                        format!("{prefix}-{word}")
                    } else {
                        word.to_string()
                    }
                })
                .collect::<Vec<_>>()
                .join(" ");
        }
        out.push_str(&format!(" {name}: {value};"));
    }
    out.push(' ');
    out
}

fn prefix_classes(selector: &str, prefix: &str) -> String {
    let mut out = String::with_capacity(selector.len() + 16);
    let mut chars = selector.chars().peekable();
    while let Some(c) = chars.next() {
        out.push(c);
        if c == '.'
            && chars
                .peek()
                .is_some_and(|n| n.is_ascii_alphabetic() || *n == '-' || *n == '_')
        {
            out.push_str(prefix);
            out.push('-');
        }
    }
    out
}

fn strip_comments(css: &str) -> String {
    let mut out = String::with_capacity(css.len());
    let mut rest = css;
    while let Some(start) = rest.find("/*") {
        out.push_str(&rest[..start]);
        match rest[start + 2..].find("*/") {
            Some(end) => rest = &rest[start + 2 + end + 2..],
            None => return out,
        }
    }
    out.push_str(rest);
    out
}

fn split_blocks(css: &str) -> Vec<(String, String)> {
    let mut blocks = Vec::new();
    let mut depth = 0usize;
    let mut prelude = String::new();
    let mut body = String::new();
    for c in css.chars() {
        match c {
            '{' => {
                if depth > 0 {
                    body.push(c);
                }
                depth += 1;
            }
            '}' if depth > 0 => {
                depth -= 1;
                if depth == 0 {
                    blocks.push((mem::take(&mut prelude), mem::take(&mut body)));
                } else {
                    body.push(c);
                }
            }
            _ if depth == 0 => prelude.push(c),
            _ => body.push(c),
        }
    }
    blocks
}

fn split_top_level(text: &str, separator: char) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut depth = 0i32;
    let mut start = 0;
    for (i, c) in text.char_indices() {
        match c {
            '(' => depth += 1,
            ')' => depth -= 1,
            _ if c == separator && depth == 0 => {
                parts.push(&text[start..i]);
                start = i + c.len_utf8();
            }
            _ => {}
        }
    }
    parts.push(&text[start..]);
    parts
}

fn names_html_or_body(selector: &str) -> bool {
    let chars: Vec<char> = selector.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i].is_ascii_alphanumeric() || chars[i] == '-' || chars[i] == '_' {
            let start = i;
            while i < chars.len()
                && (chars[i].is_ascii_alphanumeric() || chars[i] == '-' || chars[i] == '_')
            {
                i += 1;
            }
            let word: String = chars[start..i].iter().collect();
            let after_name_start =
                start > 0 && matches!(chars[start - 1], '.' | '#' | ':' | '[' | '=' | '"' | '\'');
            if !after_name_start && (word == "html" || word == "body") {
                return true;
            }
        } else {
            i += 1;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    const ID: &str = "dev.example.thing";

    #[test]
    fn classes_are_prefixed_and_rules_confined() {
        let p = prefix(ID);
        let scoped = scope_stylesheet(
            ID,
            ".page { gap: 20px; }\n.a .b:hover, div > .c { color: red; }",
        );
        assert!(
            scoped
                .css
                .contains(&format!(".{p}--view .{p}-page {{ gap: 20px; }}")),
            "{}",
            scoped.css
        );
        assert!(
            scoped.css.contains(&format!(
                ".{p}--view .{p}-a .{p}-b:hover, .{p}--view div > .{p}-c"
            )),
            "{}",
            scoped.css
        );
        assert_eq!(scope_class(ID, "page"), format!("{p}-page"));
    }

    #[test]
    fn custom_properties_global_selectors_and_fixed_are_neutralized() {
        let scoped = scope_stylesheet(
            ID,
            ":root { --text-default: red; }\n.x { --mine: 1; position: fixed; color: var(--text-muted); }\nbody { margin: 0 }",
        );
        assert!(
            !scoped.css.contains("--text-default") && !scoped.css.contains("--mine"),
            "{}",
            scoped.css
        );
        assert!(scoped.css.contains("position: absolute"), "{}", scoped.css);
        assert!(scoped.css.contains("var(--text-muted)"), "{}", scoped.css);
        assert!(!scoped.css.contains("margin"), "{}", scoped.css);
        assert_eq!(scoped.warnings.len(), 3, "{:?}", scoped.warnings);
    }

    #[test]
    fn keyframes_are_renamed_with_their_references() {
        let p = prefix(ID);
        let scoped = scope_stylesheet(
            ID,
            "@keyframes spin { from { opacity: 0; } to { opacity: 1; } }\n.s { animation: spin 1s linear infinite; }\n@media (x) { .y { color: red; } }",
        );
        assert!(
            scoped.css.contains(&format!("@keyframes {p}-spin {{")),
            "{}",
            scoped.css
        );
        assert!(
            scoped
                .css
                .contains(&format!("animation: {p}-spin 1s linear infinite")),
            "{}",
            scoped.css
        );
        assert!(!scoped.css.contains("@media"), "{}", scoped.css);
    }

    #[test]
    fn only_type_selectors_named_html_or_body_are_rejected() {
        assert!(names_html_or_body("body"));
        assert!(names_html_or_body("html body .x"));
        assert!(names_html_or_body(".a > body"));
        assert!(!names_html_or_body(".card-body_irbdkf"));
        assert!(!names_html_or_body("#body .html-view"));
        assert!(!names_html_or_body(".header:hover .body"));
    }

    #[test]
    fn comments_are_stripped() {
        let scoped = scope_stylesheet(ID, "/* .evil { } */ .ok { color: red; }");
        assert!(!scoped.css.contains("evil"));
        assert!(scoped.css.contains("-ok"));
    }
}
