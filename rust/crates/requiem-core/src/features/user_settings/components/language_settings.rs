use crate::ui::twemoji;
use crate::ui;
use requiem_ui::components::{MessageTextProps, message_text};
use requiem_ui::control;
use requiem_ui::node::Node;
use requiem_ui::signal::{Signal, signal};

use crate::utils::prefs::{self, Prefs};
use crate::utils::time::{TimeFormat, set_time_format};

pub struct Language {
    pub tag: &'static str,

    pub native: &'static str,

    pub english: &'static str,

    pub flag: &'static str,
}

macro_rules! lang {
    ($tag:literal, $native:literal, $name:expr, $flag:literal) => {
        Language {
            tag: $tag,
            native: $native,
            english: $name,
            flag: $flag,
        }
    };
}

pub const LANGUAGES: [Language; 31] = [
    lang!("da", "Dansk", requiem_i18n::key!("language-name-da"), "🇩🇰"),
    lang!(
        "de",
        "Deutsch",
        requiem_i18n::key!("language-name-de"),
        "🇩🇪"
    ),
    lang!("en-GB", "English, UK", "language-name-en-GB", "🇬🇧"),
    lang!("en-US", "English, US", "language-name-en-US", "🇺🇸"),
    lang!("es-ES", "Español", "language-name-es-ES", "🇪🇸"),
    lang!(
        "es-419",
        "Español, LATAM",
        requiem_i18n::key!("language-name-es-419"),
        "🇲🇽"
    ),
    lang!(
        "fr",
        "Français",
        requiem_i18n::key!("language-name-fr"),
        "🇫🇷"
    ),
    lang!(
        "hr",
        "Hrvatski",
        requiem_i18n::key!("language-name-hr"),
        "🇭🇷"
    ),
    lang!(
        "it",
        "Italiano",
        requiem_i18n::key!("language-name-it"),
        "🇮🇹"
    ),
    lang!(
        "lt",
        "Lietuviškai",
        requiem_i18n::key!("language-name-lt"),
        "🇱🇹"
    ),
    lang!("hu", "Magyar", requiem_i18n::key!("language-name-hu"), "🇭🇺"),
    lang!(
        "nl",
        "Nederlands",
        requiem_i18n::key!("language-name-nl"),
        "🇳🇱"
    ),
    lang!("no", "Norsk", requiem_i18n::key!("language-name-no"), "🇳🇴"),
    lang!("pl", "Polski", requiem_i18n::key!("language-name-pl"), "🇵🇱"),
    lang!("pt-BR", "Português do Brasil", "language-name-pt-BR", "🇧🇷"),
    lang!("ro", "Română", requiem_i18n::key!("language-name-ro"), "🇷🇴"),
    lang!("fi", "Suomi", requiem_i18n::key!("language-name-fi"), "🇫🇮"),
    lang!("sv-SE", "Svenska", "language-name-sv-SE", "🇸🇪"),
    lang!(
        "vi",
        "Tiếng Việt",
        requiem_i18n::key!("language-name-vi"),
        "🇻🇳"
    ),
    lang!("tr", "Türkçe", requiem_i18n::key!("language-name-tr"), "🇹🇷"),
    lang!(
        "cs",
        "Čeština",
        requiem_i18n::key!("language-name-cs"),
        "🇨🇿"
    ),
    lang!(
        "el",
        "Ελληνικά",
        requiem_i18n::key!("language-name-el"),
        "🇬🇷"
    ),
    lang!(
        "bg",
        "български",
        requiem_i18n::key!("language-name-bg"),
        "🇧🇬"
    ),
    lang!(
        "ru",
        "Русский",
        requiem_i18n::key!("language-name-ru"),
        "🇷🇺"
    ),
    lang!(
        "uk",
        "Українська",
        requiem_i18n::key!("language-name-uk"),
        "🇺🇦"
    ),
    lang!("hi", "हिंदी", requiem_i18n::key!("language-name-hi"), "🇮🇳"),
    lang!("th", "ไทย", requiem_i18n::key!("language-name-th"), "🇹🇭"),
    lang!("zh-CN", "中文", "language-name-zh-CN", "🇨🇳"),
    lang!("ja", "日本語", requiem_i18n::key!("language-name-ja"), "🇯🇵"),
    lang!("zh-TW", "繁體中文", "language-name-zh-TW", "🇹🇼"),
    lang!("ko", "한국어", requiem_i18n::key!("language-name-ko"), "🇰🇷"),
];

thread_local! {
    static CHOSEN: Signal<String> = signal("en-US".to_string());
    static OPEN: Signal<bool> = signal(false);
    static TIME: Signal<TimeFormat> = signal(TimeFormat::Auto);
}

pub fn chosen() -> Signal<String> {
    CHOSEN.with(Clone::clone)
}

pub fn open() -> Signal<bool> {
    OPEN.with(Clone::clone)
}

pub fn time_format() -> Signal<TimeFormat> {
    TIME.with(Clone::clone)
}

pub fn init(requested: &[&str], forced: bool) -> &'static str {
    let mut prefs = prefs::load();
    if forced {
        prefs.language = None;
    }
    let time = prefs
        .time_format
        .as_deref()
        .and_then(TimeFormat::from_key)
        .unwrap_or_default();
    set_time_format(time);
    TIME.with(|t| t.set(time));
    let used = match prefs
        .language
        .as_deref()
        .filter(|tag| LANGUAGES.iter().any(|l| l.tag == *tag))
    {
        Some(tag) => {
            CHOSEN.with(|c| c.set(tag.to_string()));
            requiem_i18n::set_locale(&[tag])
        }
        None => {
            let used = requiem_i18n::set_locale(requested);

            let shown = requested
                .iter()
                .find_map(|r| language_for(r))
                .map_or(used, |l| l.tag);
            CHOSEN.with(|c| c.set(shown.to_string()));
            used
        }
    };
    used
}

fn language_for(request: &str) -> Option<&'static Language> {
    let request = request.replace('_', "-");
    let primary = request.split('-').next()?;
    LANGUAGES
        .iter()
        .find(|l| l.tag.eq_ignore_ascii_case(&request))
        .or_else(|| {
            LANGUAGES.iter().find(|l| {
                l.tag
                    .split('-')
                    .next()
                    .is_some_and(|p| p.eq_ignore_ascii_case(primary))
            })
        })
}

pub fn choose(tag: &str) {
    requiem_i18n::set_locale(&[tag]);
    CHOSEN.with(|c| c.set(tag.to_string()));
    let mut saved = prefs::load();
    saved.language = Some(tag.to_string());
    prefs::save(&saved);
    control::locale_changed();
}

pub fn choose_time_format(format: TimeFormat) {
    set_time_format(format);
    TIME.with(|t| t.set(format));
    let saved = Prefs {
        time_format: Some(format.key().to_string()),
        ..prefs::load()
    };
    prefs::save(&saved);
    control::locale_changed();
}

pub fn flag(language: &Language) -> Node {
    message_text(MessageTextProps {
        content: language.flag.to_string(),
        size: 16.0,
        color: ui::TEXT_DEFAULT,
        emoji_provider: Some(twemoji::tile),
        emoji_scale: Some(1.2),
        ..Default::default()
    })
}
