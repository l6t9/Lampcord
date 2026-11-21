use gpui::{IntoElement, ObjectFit, ParentElement, SharedString, Styled, div, img, prelude::*, px};

use crate::backend;
use crate::ui::debug_stats::{self, ImageKind};
use crate::ui::icons;
use crate::ui::theme::M3Theme;

pub(crate) fn message_date_key(ts: &str) -> Option<String> {
    if ts.len() >= 10 {
        let date = &ts[0..10];
        if date.as_bytes().get(4) == Some(&b'-') && date.as_bytes().get(7) == Some(&b'-') {
            return Some(date.to_owned());
        }
    }
    None
}

pub(crate) fn format_time_hhmm(ts: &str) -> String {
    if let Some(t_idx) = ts.find('T') {
        let time_part = &ts[t_idx + 1..];
        let hhmm: String = time_part.chars().take(5).collect();
        if hhmm.len() == 5 && hhmm.chars().nth(2) == Some(':') {
            return hhmm;
        }
    }
    ts.chars().take(16).collect()
}

pub(crate) fn format_message_timestamp(ts: &str) -> String {
    if let Some(date) = message_date_key(ts) {
        format!("{date} {}", format_time_hhmm(ts))
    } else {
        ts.chars().take(16).collect()
    }
}

pub(crate) fn should_group_messages(
    prev: &backend::MessageView,
    curr: &backend::MessageView,
) -> bool {
    if prev.author_id != curr.author_id {
        return false;
    }
    // Replies should start a new visual group.
    if curr.reply || curr.reply_preview.is_some() {
        return false;
    }
    message_date_key(&prev.timestamp).is_some()
        && message_date_key(&prev.timestamp) == message_date_key(&curr.timestamp)
}

pub(crate) fn channel_glyph(kind: u8) -> &'static str {
    match kind {
        2 => "🔊",
        4 => "▾",
        5 => "📢",
        13 => "🎙",
        15 => "💬",
        10 | 11 | 12 => "🧵",
        _ => "#",
    }
}

pub(crate) fn channel_icon(kind: u8) -> &'static icons::IconDef {
    match kind {
        2 => &icons::rounded::VolumeUp,         // voice
        4 => &icons::rounded::ExpandMore,       // category (visual)
        5 => &icons::rounded::Campaign,         // announcement
        13 => &icons::rounded::Mic,             // stage
        15 => &icons::rounded::Forum,           // forum
        10 | 11 | 12 => &icons::rounded::Forum, // threads
        _ => &icons::rounded::Tag,              // text / default hash
    }
}

pub(crate) fn channel_prefix_icon(kind: u8) -> &'static str {
    match kind {
        2 | 13 => "🔊 ",
        4 => "",
        5 => "📢 ",
        10 | 11 | 12 => "🧵 ",
        15 => "💬 ",
        _ => "# ",
    }
}

pub(crate) fn avatar_circle(
    id: impl Into<SharedString>,
    url: Option<&str>,
    fallback: &str,
    size_px: f32,
    theme: &M3Theme,
) -> impl IntoElement {
    let fallback_text: String = fallback
        .chars()
        .find(|c| c.is_alphanumeric())
        .map(|c| c.to_uppercase().to_string())
        .unwrap_or_else(|| "?".to_owned());
    let container = div()
        .id(id.into())
        .flex()
        .items_center()
        .justify_center()
        .w(px(size_px))
        .h(px(size_px))
        .rounded_full()
        .overflow_hidden()
        .bg(theme.surface_container_high)
        .text_color(theme.on_surface_variant)
        .flex_shrink_0();
    if let Some(url) = url.filter(|u| !u.is_empty()) {
        debug_stats::record_render_image_with_kind(Some(url), ImageKind::Avatar);
        container.child(
            img(SharedString::from(url.to_owned()))
                .size_full()
                .rounded_full()
                .overflow_hidden()
                .object_fit(ObjectFit::Cover),
        )
    } else {
        let font_size = if size_px >= 36.0 { 14.0 } else { 11.0 };
        container
            .text_size(px(font_size))
            .child(SharedString::from(fallback_text))
    }
}

pub(crate) fn guild_icon_element(
    guild: &backend::GuildView,
    selected: bool,
    theme: &M3Theme,
) -> impl IntoElement {
    let fallback = guild
        .name
        .chars()
        .find(|c| c.is_alphanumeric())
        .map(|c| c.to_uppercase().to_string())
        .unwrap_or_else(|| "#".to_owned());
    let radius = if selected { px(16.) } else { px(24.) };
    let bg = if selected {
        theme.primary_container
    } else {
        theme.surface_container_high
    };
    let fg = if selected {
        theme.on_primary_container
    } else {
        theme.on_surface_variant
    };

    let mut inner = div()
        .flex()
        .items_center()
        .justify_center()
        .w(px(48.))
        .h(px(48.))
        .rounded(radius)
        .overflow_hidden()
        .bg(bg)
        .text_color(fg);

    if let Some(url) = guild.icon_url.as_deref().filter(|u| !u.is_empty()) {
        debug_stats::record_render_image_with_kind(Some(url), ImageKind::GuildIcon);
        inner = inner.child(
            img(SharedString::from(url.to_owned()))
                .size_full()
                .rounded(radius)
                .overflow_hidden()
                .object_fit(ObjectFit::Cover),
        );
    } else {
        inner = inner.text_sm().child(SharedString::from(fallback));
    }
    inner
}
