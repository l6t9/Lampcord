use std::collections::HashMap;
use std::sync::OnceLock;

use super::emoji_names_data::DISCORD_EMOJI_NAMES;

fn normalize(emoji: &str) -> String {
    emoji.chars().filter(|c| *c != '\u{FE0F}').collect()
}

fn canonical_index() -> &'static HashMap<String, &'static str> {
    static INDEX: OnceLock<HashMap<String, &'static str>> = OnceLock::new();
    INDEX.get_or_init(|| {
        emojis::iter()
            .map(|e| (normalize(e.as_str()), e.as_str()))
            .collect()
    })
}

fn snake_case(name: &str) -> String {
    let mut out = String::new();
    for ch in name.to_lowercase().chars() {
        match ch {
            'a'..='z' | '0'..='9' => out.push(ch),
            '\u{e0}'..='\u{e5}' => out.push('a'),
            '\u{e8}'..='\u{eb}' => out.push('e'),
            '\u{ec}'..='\u{ef}' => out.push('i'),
            '\u{f2}'..='\u{f6}' => out.push('o'),
            '\u{f9}'..='\u{fc}' => out.push('u'),
            '\u{f1}' => out.push('n'),
            '\u{e7}' => out.push('c'),
            _ => {
                if !out.ends_with('_') && !out.is_empty() {
                    out.push('_');
                }
            }
        }
    }
    out.trim_end_matches('_').to_string()
}

fn fallback_entries() -> &'static [(&'static str, &'static [&'static str])] {
    static ENTRIES: OnceLock<Vec<(&'static str, &'static [&'static str])>> = OnceLock::new();
    ENTRIES.get_or_init(|| {
        let known = names_index();
        emojis::iter()
            .filter(|e| !known.contains_key(&normalize(e.as_str())))
            .filter(|e| e.skin_tone().is_none())
            .map(|e| {
                let mut list: Vec<String> = e.shortcodes().map(str::to_string).collect();
                let cldr = snake_case(e.name());
                if !cldr.is_empty() && !list.contains(&cldr) {
                    list.push(cldr);
                }
                let list: &'static [&'static str] = Box::leak(
                    list.into_iter()
                        .map(|n| &*Box::leak(n.into_boxed_str()))
                        .collect::<Vec<&'static str>>()
                        .into_boxed_slice(),
                );
                (e.as_str(), list)
            })
            .filter(|(_, list)| !list.is_empty())
            .collect()
    })
}

fn fallback_names_index() -> &'static HashMap<String, &'static [&'static str]> {
    static INDEX: OnceLock<HashMap<String, &'static [&'static str]>> = OnceLock::new();
    INDEX.get_or_init(|| {
        fallback_entries()
            .iter()
            .map(|(emoji, names)| (normalize(emoji), *names))
            .collect()
    })
}

fn names_index() -> &'static HashMap<String, &'static [&'static str]> {
    static INDEX: OnceLock<HashMap<String, &'static [&'static str]>> = OnceLock::new();
    INDEX.get_or_init(|| {
        DISCORD_EMOJI_NAMES
            .iter()
            .map(|(emoji, names)| (normalize(emoji), *names))
            .collect()
    })
}

pub fn names_of(emoji: &str) -> Option<&'static [&'static str]> {
    let key = normalize(emoji);
    names_index()
        .get(&key)
        .or_else(|| fallback_names_index().get(&key))
        .copied()
}

pub fn emoji_for_name(name: &str) -> Option<&'static str> {
    static INDEX: OnceLock<HashMap<&'static str, &'static str>> = OnceLock::new();
    let index = INDEX.get_or_init(|| {
        DISCORD_EMOJI_NAMES
            .iter()
            .flat_map(|(emoji, names)| {
                let canonical = canonical_index()
                    .get(&normalize(emoji))
                    .copied()
                    .unwrap_or(emoji);
                names.iter().map(move |name| (*name, canonical))
            })
            .collect()
    });
    index.get(name).copied().or_else(|| {
        static FALLBACK: OnceLock<HashMap<&'static str, &'static str>> = OnceLock::new();
        FALLBACK
            .get_or_init(|| {
                fallback_entries()
                    .iter()
                    .flat_map(|(emoji, names)| names.iter().map(move |name| (*name, *emoji)))
                    .collect()
            })
            .get(name)
            .copied()
    })
}

pub fn primary_name(emoji: &str) -> Option<&'static str> {
    names_of(emoji)?.first().copied()
}

pub fn name_matches(emoji: &str, query: &str) -> bool {
    names_of(emoji).is_some_and(|names| names.iter().any(|n| n.contains(query)))
}

pub struct NameMatch {
    pub emoji: &'static str,

    pub name: &'static str,
    rank: u8,
}

pub fn lookup(query: &str, limit: usize) -> Vec<NameMatch> {
    let mut found: Vec<NameMatch> = DISCORD_EMOJI_NAMES
        .iter()
        .chain(fallback_entries().iter())
        .filter_map(|(emoji, names)| {
            let canonical = *canonical_index().get(&normalize(emoji))?;
            let (rank, name) = names
                .iter()
                .filter_map(|n| {
                    let rank = if *n == query {
                        0
                    } else if n.starts_with(query) {
                        1
                    } else if n.contains(query) {
                        2
                    } else {
                        return None;
                    };
                    Some((rank, *n))
                })
                .min_by_key(|(rank, n)| (*rank, n.len()))?;
            Some(NameMatch {
                emoji: canonical,
                name,
                rank,
            })
        })
        .collect();
    found.sort_by_key(|m| (m.rank, m.name.len()));
    found.truncate(limit);
    found
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn knows_discords_own_aliases() {
        assert_eq!(
            names_of("🙂"),
            Some(&["slight_smile", "slightly_smiling_face"][..])
        );
        assert_eq!(primary_name("👍"), Some("thumbsup"));
        assert!(name_matches("🙂", "slight_s"));
        assert_eq!(
            primary_name("☺"),
            primary_name("☺\u{FE0F}"),
            "variation selector is ignored"
        );
    }

    #[test]
    fn names_map_back_to_the_emoji_reactions_use() {
        assert_eq!(emoji_for_name("thumbsup"), Some("\u{1F44D}"));
        assert_eq!(emoji_for_name("+1"), Some("\u{1F44D}"));
        assert_eq!(emoji_for_name("100"), Some("\u{1F4AF}"));

        assert_eq!(emoji_for_name("heart"), Some("\u{2764}\u{FE0F}"));
        assert_eq!(emoji_for_name("not_an_emoji"), None);
    }

    #[test]
    fn lookup_orders_exact_then_prefix_then_substring() {
        let names: Vec<&str> = lookup("smile", 50).iter().map(|m| m.name).collect();
        assert_eq!(names[0], "smile", "exact alias first: {names:?}");
        assert!(names.contains(&"slight_smile"), "{names:?}");
        let first_substring = names
            .iter()
            .position(|n| !n.starts_with("smile"))
            .expect("some substring match");
        assert!(
            names[..first_substring]
                .iter()
                .all(|n| n.starts_with("smile")),
            "{names:?}"
        );
    }

    #[test]
    fn emoji_newer_than_discords_table_get_names_from_the_emojis_crate() {
        assert!(!names_index().contains_key("\u{1FAEA}"));
        assert_eq!(primary_name("\u{1FAEA}"), Some("distorted_face"));
        assert_eq!(emoji_for_name("distorted_face"), Some("\u{1FAEA}"));
        assert!(
            lookup("distorted", 10)
                .iter()
                .any(|m| m.emoji == "\u{1FAEA}")
        );

        assert_eq!(primary_name("\u{1F604}"), Some("smile"));
        assert_eq!(snake_case("face with peeking eye"), "face_with_peeking_eye");
        assert_eq!(snake_case("piñata"), "pinata");
    }

    #[test]
    fn lookup_only_offers_emoji_the_atlas_can_draw() {
        for m in lookup("a", 2000) {
            assert!(
                super::super::twemoji::tile(m.emoji).is_some(),
                "{} has no Twemoji tile",
                m.emoji
            );
        }
    }
}
