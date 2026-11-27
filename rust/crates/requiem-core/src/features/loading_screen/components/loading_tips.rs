#[derive(Clone, Debug, PartialEq)]
pub enum Piece {
    Text(String),

    Bold(String),

    Strike(String),

    Keys(String),
}

pub fn tips() -> Vec<Vec<Piece>> {
    (1..)
        .map(|n| format!("loading-tip-{n}"))
        .take_while(|key| requiem_i18n::has(key))
        .map(|key| parse(&requiem_i18n::text(&key)))
        .collect()
}

pub fn parse(text: &str) -> Vec<Piece> {
    const TAGS: [(&str, &str, fn(String) -> Piece); 3] = [
        ("<b>", "</b>", Piece::Bold),
        ("<del>", "</del>", Piece::Strike),
        ("<key>", "</key>", Piece::Keys),
    ];
    let mut pieces = Vec::new();
    let mut rest = text;
    while !rest.is_empty() {
        let next = TAGS
            .iter()
            .filter_map(|(open, close, make)| {
                let start = rest.find(open)?;
                let end = rest[start + open.len()..].find(close)? + start + open.len();
                Some((start, end, open.len(), close.len(), make))
            })
            .min_by_key(|t| t.0);
        match next {
            Some((start, end, open_len, close_len, make)) => {
                if start > 0 {
                    pieces.push(Piece::Text(rest[..start].to_string()));
                }
                pieces.push(make(rest[start + open_len..end].to_string()));
                rest = &rest[end + close_len..];
            }
            None => {
                pieces.push(Piece::Text(rest.to_string()));
                break;
            }
        }
    }
    pieces
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn has_the_clients_tips() {
        let tips = tips();
        assert_eq!(tips.len(), 59);
        assert_eq!(
            parse("a <b>b</b> <key>mod+k</key> c <del>d</del>"),
            [
                Piece::Text("a ".into()),
                Piece::Bold("b".into()),
                Piece::Text(" ".into()),
                Piece::Keys("mod+k".into()),
                Piece::Text(" c ".into()),
                Piece::Strike("d".into())
            ]
        );
        assert_eq!(parse("1 < 2 <b>x"), [Piece::Text("1 < 2 <b>x".into())]);
        assert!(tips.iter().all(|t| !t.is_empty()));
        assert!(
            tips.iter()
                .any(|t| t.contains(&Piece::Keys("shift+return".into())))
        );
        assert!(tips.iter().any(|t| t.contains(&Piece::Bold("bold".into()))));
        assert!(
            tips.iter()
                .any(|t| t.contains(&Piece::Strike("secret".into())))
        );
    }
}
