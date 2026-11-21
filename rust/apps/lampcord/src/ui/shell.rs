use gpui::{Div, ParentElement, SharedString, Stateful, Styled, div, prelude::*, px};

use crate::backend::Snapshot;
use crate::ui::theme::M3Theme;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AppPage {
    #[default]
    Main,
    Settings,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SettingsPage {
    #[default]
    Account,
    Appearance,
    VoiceAndVideo,
    Devices,
    About,
}

impl SettingsPage {
    pub fn label(self) -> &'static str {
        match self {
            Self::Account => "Account",
            Self::Appearance => "Appearance",
            Self::VoiceAndVideo => "Voice & Video",
            Self::Devices => "Devices",
            Self::About => "About",
        }
    }

    pub fn all() -> &'static [SettingsPage] {
        &[
            Self::Account,
            Self::Appearance,
            Self::VoiceAndVideo,
            Self::Devices,
            Self::About,
        ]
    }
}

pub fn settings_nav_item(
    id: impl Into<SharedString>,
    page: SettingsPage,
    selected: bool,
    theme: &M3Theme,
) -> Stateful<Div> {
    let (bg, fg) = if selected {
        (theme.secondary_container, theme.on_secondary_container)
    } else {
        (theme.surface_container_low, theme.on_surface_variant)
    };
    div()
        .id(id.into())
        .flex()
        .items_center()
        .px(px(14.))
        .py(px(10.))
        .rounded(px(20.))
        .bg(bg)
        .text_color(fg)
        .text_sm()
        .hover(|s| s.bg(theme.surface_container_high).cursor_pointer())
        .child(page.label())
}

pub fn empty_state(title: &str, body: &str, theme: &M3Theme) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .gap(px(8.))
        .p(px(20.))
        .rounded(px(20.))
        .bg(theme.surface_container)
        .child(
            div()
                .text_base()
                .text_color(theme.on_surface)
                .child(SharedString::from(title.to_owned())),
        )
        .child(
            div()
                .text_sm()
                .text_color(theme.on_surface_variant)
                .child(SharedString::from(body.to_owned())),
        )
}

pub fn section_title(text: &str, theme: &M3Theme) -> impl IntoElement {
    div()
        .text_xs()
        .text_color(theme.on_surface_variant)
        .child(SharedString::from(text.to_owned()))
}

pub fn panel_card(theme: &M3Theme) -> gpui::Div {
    div()
        .flex()
        .flex_col()
        .gap(px(12.))
        .p(px(16.))
        .rounded(px(20.))
        .bg(theme.surface_container_low)
        .border_1()
        .border_color(theme.outline_variant)
}

pub fn snapshot_user_label(snapshot: &Snapshot) -> String {
    snapshot
        .current_user
        .clone()
        .unwrap_or_else(|| "Not signed in".to_owned())
}
