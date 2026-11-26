//! The standard emoji set for the picker, from the shared `emojis.json`
//! the Compose client uses, with Twemoji image URLs built the same way
//! (`String.toTwemojiUrl` in the shared model).

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
    }

    #[auto_cxx_name]
    unsafe extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qml_singleton]
        type Emojis = super::EmojisRust;

        /// `[{key, title, emojis: [{n, s, u}]}]`: name, surrogates, image URL.
        #[qinvokable]
        fn groups(self: &Emojis) -> QString;

        /// The Twemoji URL for any emoji string, for frequently used ones.
        #[qinvokable]
        fn image(self: &Emojis, surrogates: QString) -> QString;
    }
}

use std::sync::LazyLock;

use cxx_qt_lib::QString;
use serde_json::{Value, json};

const TWEMOJI: &str = "https://cdn.jsdelivr.net/gh/jdecked/twemoji@v17.0.3/assets/72x72";
const SOURCE: &str = include_str!("../../../../shared/src/commonMain/resources/files/emojis.json");

/// Category keys in emojis.json and their Compose display names, in order.
const CATEGORIES: [(&str, &str); 8] = [
    ("people", "Smileys & People"),
    ("nature", "Animals & Nature"),
    ("food", "Food & Drink"),
    ("activity", "Activities"),
    ("travel", "Travel & Places"),
    ("objects", "Objects"),
    ("symbols", "Symbols"),
    ("flags", "Flags"),
];

static GROUPS: LazyLock<String> = LazyLock::new(|| {
    let source: Value = serde_json::from_str(SOURCE).expect("emojis.json is valid JSON");
    let groups: Vec<Value> = CATEGORIES
        .iter()
        .map(|(key, title)| {
            let emojis: Vec<Value> = source[key]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(|emoji| {
                    let surrogates = emoji["surrogates"].as_str()?;
                    let name = emoji["names"].get(0).and_then(Value::as_str)?;
                    Some(json!({ "n": name, "s": surrogates, "u": twemoji_url(surrogates) }))
                })
                .collect();
            json!({ "key": key, "title": title, "emojis": emojis })
        })
        .collect();
    Value::Array(groups).to_string()
});

/// Codepoints joined by '-', dropping FE0F unless the emoji has a ZWJ.
pub fn twemoji_url(surrogates: &str) -> String {
    let joined = surrogates.contains('\u{200D}');
    let code = surrogates
        .chars()
        .filter(|ch| joined || *ch != '\u{FE0F}')
        .map(|ch| format!("{:x}", ch as u32))
        .collect::<Vec<_>>()
        .join("-")
        .replace("1f441-fe0f-200d-1f5e8-fe0f", "1f441-200d-1f5e8");
    if code.is_empty() {
        String::new()
    } else {
        format!("{TWEMOJI}/{code}.png")
    }
}

#[derive(Default)]
pub struct EmojisRust;

impl qobject::Emojis {
    pub fn groups(&self) -> QString {
        QString::from(GROUPS.as_str())
    }

    pub fn image(&self, surrogates: QString) -> QString {
        QString::from(twemoji_url(&surrogates.to_string()).as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn twemoji_urls_match_the_compose_rules() {
        assert_eq!(twemoji_url("😀"), format!("{TWEMOJI}/1f600.png"));
        // Lone FE0F is dropped…
        assert_eq!(twemoji_url("☺️"), format!("{TWEMOJI}/263a.png"));
        // …but kept in ZWJ sequences.
        assert_eq!(
            twemoji_url("🏳️‍🌈"),
            format!("{TWEMOJI}/1f3f3-fe0f-200d-1f308.png")
        );
        let groups: Value = serde_json::from_str(&GROUPS).unwrap();
        assert_eq!(groups.as_array().unwrap().len(), 8);
        assert!(groups[0]["emojis"].as_array().unwrap().len() > 100);
    }
}
