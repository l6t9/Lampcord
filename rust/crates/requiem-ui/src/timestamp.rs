use crate::widget::primitives::markdown;
use std::ops::Range;
use std::sync::OnceLock;

pub type Formatter = fn(i64, Option<char>) -> String;

static FORMATTER: OnceLock<Formatter> = OnceLock::new();

pub fn install(formatter: Formatter) {
    let _ = FORMATTER.set(formatter);
}

pub fn format(unix: i64, format: Option<char>) -> Option<String> {
    FORMATTER.get().map(|f| f(unix, format))
}

pub const FORMATS: [char; 8] = ['t', 'T', 'd', 'D', 'f', 'F', 's', 'S'];

fn is_format(c: char) -> bool {
    FORMATS.contains(&c) || c == 'R'
}

pub fn tokens(text: &str) -> Vec<(Range<usize>, i64, Option<char>)> {
    let mut found = Vec::new();
    let mut from = 0;
    while let Some(rel) = text[from..].find("<t:") {
        let start = from + rel;
        from = start + 3;
        if markdown::is_escaped(text, start) {
            continue;
        }
        let rest = &text[start + 3..];
        let sign = usize::from(rest.starts_with('-'));
        let digits = rest[sign..].bytes().take_while(u8::is_ascii_digit).count();
        if digits == 0 || digits > 17 {
            continue;
        }
        let number_end = sign + digits;
        let Ok(unix) = rest[..number_end].parse::<i64>() else {
            continue;
        };
        let tail = &rest[number_end..];
        let (letter, close) = match tail.strip_prefix(':') {
            Some(after) => {
                let mut chars = after.chars();
                match (chars.next(), chars.next()) {
                    (Some(c), Some('>')) if is_format(c) => (Some(c), number_end + 3),
                    _ => continue,
                }
            }
            None if tail.starts_with('>') => (None, number_end + 1),
            None => continue,
        };
        found.push((start..start + 3 + close, unix, letter));
        from = start + 3 + close;
    }
    found
}

pub fn token(unix: i64, format: Option<char>) -> String {
    match format {
        Some(f) => format!("<t:{unix}:{f}>"),
        None => format!("<t:{unix}>"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokens_are_found_with_their_format() {
        let text = "at <t:1760000000:F> or <t:5> not <t:5:x> nor <t:abc> but <t:-3:R>";
        let found: Vec<(&str, i64, Option<char>)> = tokens(text)
            .into_iter()
            .map(|(r, t, f)| (&text[r], t, f))
            .collect();
        assert_eq!(
            found,
            [
                ("<t:1760000000:F>", 1760000000, Some('F')),
                ("<t:5>", 5, None),
                ("<t:-3:R>", -3, Some('R'))
            ]
        );
    }

    #[test]
    fn a_token_round_trips() {
        assert_eq!(token(7, Some('R')), "<t:7:R>");
        assert_eq!(token(7, None), "<t:7>");
    }
}
