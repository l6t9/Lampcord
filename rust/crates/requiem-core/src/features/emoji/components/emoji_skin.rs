use requiem_ui::control;
use std::borrow::Cow;
use std::sync::atomic::{AtomicU8, Ordering};
use unicode_segmentation::UnicodeSegmentation;

use emojis::SkinTone;
use requiem_ui::signal::{Signal, signal};

use crate::utils::prefs;

pub const TONES: [SkinTone; 6] = [
    SkinTone::Default,
    SkinTone::Light,
    SkinTone::MediumLight,
    SkinTone::Medium,
    SkinTone::MediumDark,
    SkinTone::Dark,
];

pub const BUTTON_EMOJI: &str = "\u{1F44F}";

static CURRENT: AtomicU8 = AtomicU8::new(0);

thread_local! {

    static PICKED: Signal<u8> = signal(0);
}

fn tone_of(number: u8) -> SkinTone {
    TONES[(number as usize).min(TONES.len() - 1)]
}

pub fn current() -> SkinTone {
    tone_of(CURRENT.load(Ordering::Relaxed))
}

pub fn number(tone: SkinTone) -> u8 {
    TONES.iter().position(|t| *t == tone).unwrap_or(0) as u8
}

pub fn picked() -> Signal<u8> {
    PICKED.with(Clone::clone)
}

pub fn init() {
    let saved = prefs::load().emoji_skin_tone.unwrap_or(0).min(5);
    CURRENT.store(saved, Ordering::Relaxed);
    PICKED.with(|picked| picked.set(saved));
}

pub fn choose(tone: SkinTone) {
    let number = number(tone);
    CURRENT.store(number, Ordering::Relaxed);
    let mut saved = prefs::load();
    saved.emoji_skin_tone = Some(number);
    prefs::save(&saved);
    PICKED.with(|picked| picked.set(number));
    control::locale_changed();
}

fn shown_in(emoji: &str, tone: SkinTone) -> Cow<'_, str> {
    match emojis::get(emoji)
        .filter(|e| e.skin_tone().is_some())
        .and_then(|e| e.with_skin_tone(tone))
    {
        Some(shown) if shown.as_str() != emoji => Cow::Borrowed(shown.as_str()),
        _ => Cow::Borrowed(emoji),
    }
}

pub fn shown(emoji: &str) -> Cow<'_, str> {
    shown_in(emoji, current())
}

pub fn plain(emoji: &str) -> String {
    shown_in(emoji, SkinTone::Default).into_owned()
}

pub fn strip_tones(text: &str) -> Cow<'_, str> {
    restyle_in(text, SkinTone::Default)
}

pub fn restyle(text: &str) -> Cow<'_, str> {
    restyle_in(text, current())
}

fn is_modifier(c: char) -> bool {
    ('\u{1F3FB}'..='\u{1F3FF}').contains(&c)
}

fn restyle_in(text: &str, tone: SkinTone) -> Cow<'_, str> {
    if text.is_ascii() || (tone == SkinTone::Default && !text.chars().any(is_modifier)) {
        return Cow::Borrowed(text);
    }
    let mut out = String::with_capacity(text.len());
    let mut changed = false;
    for grapheme in text.graphemes(true) {
        let shown = shown_in(grapheme, tone);
        changed |= shown != grapheme;
        out.push_str(&shown);
    }
    if changed {
        Cow::Owned(out)
    } else {
        Cow::Borrowed(text)
    }
}

pub fn hand(tone: SkinTone) -> &'static str {
    emojis::get(BUTTON_EMOJI)
        .and_then(|hand| hand.with_skin_tone(tone))
        .map(|e| e.as_str())
        .unwrap_or(BUTTON_EMOJI)
}

pub fn label(tone: SkinTone) -> String {
    match number(tone) {
        1 => requiem_i18n::t!("emoji-skin-tone-light"),
        2 => requiem_i18n::t!("emoji-skin-tone-medium-light"),
        3 => requiem_i18n::t!("emoji-skin-tone-medium"),
        4 => requiem_i18n::t!("emoji-skin-tone-medium-dark"),
        5 => requiem_i18n::t!("emoji-skin-tone-dark"),
        _ => requiem_i18n::t!("emoji-skin-tone-none"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_emoji_is_drawn_in_the_picked_tone_whatever_tone_it_carries() {
        assert_eq!(
            shown_in("\u{1F44D}", SkinTone::Medium),
            "\u{1F44D}\u{1F3FD}"
        );
        assert_eq!(
            shown_in("\u{1F44D}\u{1F3FF}", SkinTone::Light),
            "\u{1F44D}\u{1F3FB}"
        );

        assert_eq!(
            shown_in("\u{1F44D}\u{1F3FD}", SkinTone::Default),
            "\u{1F44D}"
        );

        assert_eq!(shown_in("\u{1F525}", SkinTone::Dark), "\u{1F525}");
        assert_eq!(shown_in("\u{1F60E}", SkinTone::Dark), "\u{1F60E}");
        assert_eq!(shown_in("not an emoji", SkinTone::Dark), "not an emoji");

        let couple = "\u{1F469}\u{200D}\u{2764}\u{FE0F}\u{200D}\u{1F468}";
        assert_eq!(
            emojis::get(&shown_in(couple, SkinTone::Dark)).and_then(|e| e.skin_tone()),
            Some(SkinTone::Dark)
        );
    }

    #[test]
    fn nothing_is_sent_with_a_tone() {
        assert_eq!(
            strip_tones("hi \u{1F44D}\u{1F3FD} and \u{1F44B}\u{1F3FF}! \u{1F525} <:custom:1>"),
            "hi \u{1F44D} and \u{1F44B}! \u{1F525} <:custom:1>"
        );
        let mixed = "\u{1F469}\u{1F3FF}\u{200D}\u{2764}\u{FE0F}\u{200D}\u{1F468}\u{1F3FB}";
        assert_eq!(
            emojis::get(&strip_tones(mixed)).and_then(|e| e.skin_tone()),
            Some(SkinTone::Default)
        );

        assert!(matches!(strip_tones("plain text"), Cow::Borrowed(_)));
        assert!(matches!(
            strip_tones("\u{1F44D} \u{1F525}"),
            Cow::Borrowed(_)
        ));
        assert_eq!(plain("\u{1F44D}\u{1F3FD}"), "\u{1F44D}");
    }

    #[test]
    fn text_for_the_system_font_follows_the_picked_tone() {
        let sent = "hi \u{1F44D}\u{1F3FD} and \u{1F44B} <:custom:123> \u{1F525}";
        assert_eq!(
            restyle_in(sent, SkinTone::Dark),
            "hi \u{1F44D}\u{1F3FF} and \u{1F44B}\u{1F3FF} <:custom:123> \u{1F525}"
        );
        assert_eq!(
            restyle_in(sent, SkinTone::Default),
            "hi \u{1F44D} and \u{1F44B} <:custom:123> \u{1F525}"
        );
        assert!(matches!(
            restyle_in("plain text", SkinTone::Dark),
            Cow::Borrowed(_)
        ));
        assert!(matches!(
            restyle_in("\u{1F44D}\u{1F3FF}", SkinTone::Dark),
            Cow::Borrowed(_)
        ));
    }

    #[test]
    fn the_tones_are_numbered_like_discords() {
        assert_eq!(TONES.map(number), [0, 1, 2, 3, 4, 5]);
        assert_eq!(tone_of(3), SkinTone::Medium);
        assert_eq!(tone_of(200), SkinTone::Dark);
        assert_eq!(hand(SkinTone::MediumDark), "\u{1F44F}\u{1F3FE}");
    }
}
