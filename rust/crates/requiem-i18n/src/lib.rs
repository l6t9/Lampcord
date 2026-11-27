use std::borrow::Cow;
use std::sync::{Arc, RwLock};

use fluent_bundle::concurrent::FluentBundle;
use fluent_bundle::{FluentArgs, FluentResource, FluentValue};
use unic_langid::LanguageIdentifier;

mod catalog;

pub use catalog::{Arg, ArgKind, Message, catalog, message};

pub const FALLBACK: &str = "en-US";

include!(concat!(env!("OUT_DIR"), "/locales.rs"));

type Bundle = FluentBundle<FluentResource>;

struct State {
    chain: Vec<Arc<Bundle>>,
    locale: &'static str,
}

static STATE: RwLock<Option<State>> = RwLock::new(None);

fn build_bundle(tag: &str) -> Option<Bundle> {
    let (_, sources) = LOCALES.iter().find(|(t, _)| *t == tag)?;
    let langid: LanguageIdentifier = tag.parse().ok()?;
    let mut bundle = FluentBundle::new_concurrent(vec![langid]);

    bundle.set_use_isolating(false);

    let _ = bundle.add_builtins();
    for source in *sources {
        let resource = match FluentResource::try_new((*source).to_string()) {
            Ok(resource) => resource,
            Err((resource, errors)) => {
                log::error!("i18n: {tag}: syntax errors in a .ftl file: {errors:?}");
                resource
            }
        };
        if let Err(errors) = bundle.add_resource(resource) {
            log::error!("i18n: {tag}: duplicate message ids: {errors:?}");
        }
    }
    Some(bundle)
}

pub fn available_locales() -> impl Iterator<Item = &'static str> {
    LOCALES.iter().map(|(tag, _)| *tag)
}

pub fn set_locale(requested: &[&str]) -> &'static str {
    let wanted: Vec<LanguageIdentifier> = requested
        .iter()
        .filter_map(|tag| tag.replace('_', "-").parse().ok())
        .collect();
    let available: Vec<LanguageIdentifier> = available_locales()
        .filter_map(|tag| tag.parse().ok())
        .collect();
    let default: LanguageIdentifier = FALLBACK.parse().expect("valid fallback tag");
    let negotiated = fluent_langneg::negotiate_languages(
        &wanted,
        &available,
        Some(&default),
        fluent_langneg::NegotiationStrategy::Filtering,
    );
    let chosen = negotiated
        .first()
        .and_then(|langid| {
            available_locales().find(|tag| {
                tag.parse::<LanguageIdentifier>()
                    .is_ok_and(|t| t == **langid)
            })
        })
        .unwrap_or(FALLBACK);

    let tags: &[&str] = if chosen == FALLBACK {
        &[FALLBACK]
    } else {
        &[chosen, FALLBACK]
    };
    let chain = tags
        .iter()
        .filter_map(|tag| build_bundle(tag))
        .map(Arc::new)
        .collect();
    *STATE.write().unwrap() = Some(State {
        chain,
        locale: chosen,
    });
    chosen
}

pub fn locale() -> &'static str {
    ensure_init();
    STATE
        .read()
        .unwrap()
        .as_ref()
        .map_or(FALLBACK, |state| state.locale)
}

fn ensure_init() {
    if STATE.read().unwrap().is_none() {
        set_locale(&[]);
    }
}

pub fn escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for ch in text.chars() {
        if matches!(
            ch,
            '\\' | '*' | '_' | '~' | '`' | '|' | '>' | '<' | '[' | ']' | '#'
        ) {
            out.push('\\');
        }
        out.push(ch);
    }
    out
}

pub enum Value {
    Text(String),

    Markdown(String),
    Number(f64),
}

impl From<&str> for Value {
    fn from(text: &str) -> Self {
        Value::Text(text.to_string())
    }
}
impl From<String> for Value {
    fn from(text: String) -> Self {
        Value::Text(text)
    }
}
impl From<&String> for Value {
    fn from(text: &String) -> Self {
        Value::Text(text.clone())
    }
}
impl From<Cow<'_, str>> for Value {
    fn from(text: Cow<'_, str>) -> Self {
        Value::Text(text.into_owned())
    }
}
macro_rules! number_from {
    ($($t:ty),*) => {$(impl From<$t> for Value { fn from(n: $t) -> Self { Value::Number(n as f64) } })*};
}
number_from!(i8, i16, i32, i64, u8, u16, u32, u64, usize, f32, f64);

pub struct Markdown(pub String);
impl From<Markdown> for Value {
    fn from(markdown: Markdown) -> Self {
        Value::Markdown(markdown.0)
    }
}

pub struct Url;
impl Url {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(url: &str) -> Markdown {
        Markdown(
            url.replace(' ', "%20")
                .replace('(', "%28")
                .replace(')', "%29"),
        )
    }
}

pub fn masked_link(label: &str, url: &str) -> Markdown {
    Markdown(format!("[{}]({})", escape(label), Url::new(url).0))
}

pub struct Mention;
impl Mention {
    pub fn user(id: u64) -> Markdown {
        Markdown(format!("<@{id}>"))
    }
    pub fn channel(id: u64) -> Markdown {
        Markdown(format!("<#{id}>"))
    }
}

pub struct Emoji;
impl Emoji {
    pub fn custom(name: &str, id: u64, animated: bool) -> Markdown {
        Markdown(format!("<{}:{name}:{id}>", if animated { "a" } else { "" }))
    }
}

#[derive(Default)]
pub struct Args {
    values: Vec<(&'static str, Value)>,

    markdown: bool,
}

impl Args {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn markdown(mut self) -> Self {
        self.markdown = true;
        self
    }
    pub fn with(mut self, name: &'static str, value: impl Into<Value>) -> Self {
        self.values.push((name, value.into()));
        self
    }
}

pub fn tr(key: &str, args: &Args) -> String {
    ensure_init();
    let mut fluent_args = FluentArgs::new();
    for (name, value) in &args.values {
        fluent_args.set(
            *name,
            match value {
                Value::Text(text) if args.markdown => FluentValue::String(Cow::Owned(escape(text))),
                Value::Text(text) => FluentValue::String(Cow::Borrowed(text.as_str())),
                Value::Markdown(markdown) => FluentValue::String(Cow::Borrowed(markdown.as_str())),
                Value::Number(n) => FluentValue::from(*n),
            },
        );
    }
    let (id, attribute) = match key.split_once('.') {
        Some((id, attribute)) => (id, Some(attribute)),
        None => (key, None),
    };
    let guard = STATE.read().unwrap();
    let chain = guard
        .as_ref()
        .map_or(&[][..], |state| state.chain.as_slice());
    for bundle in chain {
        let Some(message) = bundle.get_message(id) else {
            continue;
        };
        let pattern = match attribute {
            Some(name) => message.get_attribute(name).map(|a| a.value()),
            None => message.value(),
        };
        let Some(pattern) = pattern else { continue };
        let mut errors = Vec::new();
        let text = bundle
            .format_pattern(pattern, Some(&fluent_args), &mut errors)
            .into_owned();
        if !errors.is_empty() {
            log::warn!("i18n: formatting `{key}`: {errors:?}");
        }
        return text;
    }
    log::warn!("i18n: missing message `{key}`");
    key.to_string()
}

pub fn text(key: &str) -> String {
    tr(key, &Args::new())
}

#[macro_export]
macro_rules! key {
    ($key:literal) => {
        $key
    };
}

pub fn has(key: &str) -> bool {
    ensure_init();
    let id = key.split('.').next().unwrap_or(key);
    STATE
        .read()
        .unwrap()
        .as_ref()
        .is_some_and(|state| state.chain.iter().any(|bundle| bundle.has_message(id)))
}

#[macro_export]
macro_rules! t {
    ($key:expr) => { $crate::tr($key, &$crate::Args::new()) };
    ($key:expr, $($name:ident = $value:expr),+ $(,)?) => {
        $crate::tr($key, &$crate::Args::new()$(.with(stringify!($name), $value))+)
    };
}

#[macro_export]
macro_rules! md {
    ($key:expr) => { $crate::tr($key, &$crate::Args::new().markdown()) };
    ($key:expr, $($name:ident = $value:expr),+ $(,)?) => {
        $crate::tr($key, &$crate::Args::new().markdown()$(.with(stringify!($name), $value))+)
    };
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::fs;
    use std::path::Path;
    use std::path;
    use super::*;
    use std::sync::Mutex;

    static LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn plain_and_markdown_pass_through() {
        let _g = LOCK.lock().unwrap();
        set_locale(&["en-US"]);
        assert_eq!(t!("welcome-back", name = "Ana"), "Welcome back, **Ana**!");
        assert!(
            t!("upload-failed").starts_with("Your message could not be delivered.\n\nYou can see")
        );
    }

    #[test]
    fn text_args_are_escaped_but_tokens_are_not() {
        let _g = LOCK.lock().unwrap();
        set_locale(&["en-US"]);
        assert_eq!(
            md!("welcome-back", name = "**x** <@1>"),
            r"Welcome back, **\*\*x\*\* \<@1\>**!"
        );
        assert_eq!(
            masked_link("a]b", "https://x.y/(z)").0,
            r"[a\]b](https://x.y/%28z%29)"
        );
        assert_eq!(
            md!(
                "mentioned-you",
                who = Mention::user(42),
                channel = Mention::channel(7)
            ),
            "<@42> mentioned you in <#7>."
        );
        assert!(
            md!("invited", who = Mention::user(1), server = "A_B")
                .contains(r"**A\_B** <:wave:1234567890123456789>")
        );
        assert_eq!(Emoji::custom("hi", 5, true).0, "<a:hi:5>");
    }

    #[test]
    fn plain_text_args_are_not_escaped() {
        let _g = LOCK.lock().unwrap();
        set_locale(&["en-US"]);
        assert_eq!(
            t!("welcome-back", name = "my_chan *x*"),
            "Welcome back, **my_chan *x***!"
        );
    }

    #[test]
    fn plurals_follow_the_locale() {
        let _g = LOCK.lock().unwrap();
        set_locale(&["en-US"]);
        assert_eq!(t!("unread-messages", count = 1), "**1** new message");
        assert_eq!(t!("unread-messages", count = 3), "**3** new messages");
        assert_eq!(t!("member-count", count = 1), "1 member");
    }

    #[test]
    fn negotiation_and_fallback() {
        let _g = LOCK.lock().unwrap();
        assert_eq!(set_locale(&["de-AT", "en"]), "de");
        assert_eq!(t!("dismiss-message"), "Nachricht ausblenden");

        assert_eq!(t!("i18n-test-english-only"), "English only");
        assert_eq!(set_locale(&["xx-YY"]), FALLBACK);
        assert_eq!(t!("no-such-message"), "no-such-message");
    }

    #[test]
    fn every_locale_file_parses_without_duplicates() {
        for (tag, sources) in LOCALES {
            let mut bundle = FluentBundle::new_concurrent(vec![tag.parse().unwrap()]);
            for source in *sources {
                let resource = FluentResource::try_new((*source).to_string())
                    .unwrap_or_else(|(_, e)| panic!("{tag}: {e:?}"));
                bundle
                    .add_resource(resource)
                    .unwrap_or_else(|e| panic!("{tag}: duplicate ids {e:?}"));
            }
        }
    }

    #[test]
    fn every_key_used_in_the_source_exists() {
        fn rust_files(dir: &Path, out: &mut Vec<path::PathBuf>) {
            let Ok(entries) = fs::read_dir(dir) else {
                return;
            };
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    rust_files(&path, out);
                } else if path.extension().is_some_and(|e| e == "rs") {
                    out.push(path);
                }
            }
        }

        let (_, sources) = LOCALES.iter().find(|(tag, _)| *tag == FALLBACK).unwrap();
        let mut defined = BTreeSet::new();
        for source in *sources {
            for line in source.lines() {
                if let Some((id, _)) = line.split_once(" =") {
                    if !id.starts_with(['#', ' ', '-']) && !id.is_empty() {
                        defined.insert(id.trim().to_string());
                    }
                }
            }
        }

        let crates = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
        let mut files = Vec::new();
        for krate in ["requiem-core", "requiem"] {
            rust_files(&crates.join(krate).join("src"), &mut files);
        }
        let mut missing = Vec::new();
        for file in files {
            let text = fs::read_to_string(&file).unwrap();
            for opener in ["t!(\"", "md!(\"", "key!(\"", "i18n::text(\"", "i18n::tr(\""] {
                let mut rest = text.as_str();
                while let Some(at) = rest.find(opener) {
                    let boundary = rest[..at]
                        .bytes()
                        .next_back()
                        .is_none_or(|b| !(b.is_ascii_alphanumeric() || b == b'_'));
                    let after = &rest[at + opener.len()..];
                    if !boundary {
                        rest = after;
                        continue;
                    }
                    let end = after.find('"').unwrap_or(0);
                    let id = after[..end].split('.').next().unwrap();
                    if !defined.contains(id) {
                        missing.push(format!(
                            "{}: {id}",
                            file.file_name().unwrap().to_string_lossy()
                        ));
                    }
                    rest = after;
                }
            }
        }
        assert!(
            missing.is_empty(),
            "keys used in code but not defined in en-US:\n{}",
            missing.join("\n")
        );
    }
}
