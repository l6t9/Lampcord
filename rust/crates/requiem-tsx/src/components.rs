use requiem_ui::colors;
use requiem_ui::components::{Align, Direction, Justify};
use requiem_ui::icon_handle::Icon;
use requiem_ui::icons;
use skia_safe::Color;

pub const TEXT_COLORS: &[(&str, Color)] = &[
    ("text-strong", colors::TEXT_STRONG),
    ("text-default", colors::TEXT_DEFAULT),
    ("text-secondary", colors::TEXT_SECONDARY),
    ("text-muted", colors::TEXT_MUTED),
    ("text-subtle", colors::TEXT_SUBTLE),
    ("text-critical", colors::TEXT_CRITICAL),
    ("text-link", colors::TEXT_LINK),
    ("text-anchor", colors::TEXT_ANCHOR),
    ("text-positive", colors::TEXT_POSITIVE),
];

pub const BUTTON_VARIANTS: &[&str] = &[
    "primary",
    "secondary",
    "critical-primary",
    "critical-secondary",
    "active",
    "overlay-primary",
    "overlay-secondary",
    "expressive",
    "icon-only",
    "bare",
];

pub const BUTTON_SIZES: &[&str] = &["small", "medium"];

pub const UI_MODULE: &str = "@requiem/ui";

pub const TYPES_MODULE: &str = "@requiem/types";

pub const APP_MODULE: &str = "@requiem/app";

pub const UI_COMPONENTS: &[&str] = &["Text", "Button", "ScrollView", "Stack", "Chevron"];

pub const STACK_DIRECTIONS: &[(&str, Direction)] = &[
    ("vertical", Direction::Vertical),
    ("horizontal", Direction::Horizontal),
    ("vertical-reverse", Direction::VerticalReverse),
    ("horizontal-reverse", Direction::HorizontalReverse),
];

pub const STACK_ALIGNS: &[(&str, Align)] = &[
    ("start", Align::Start),
    ("center", Align::Center),
    ("end", Align::End),
    ("stretch", Align::Stretch),
    ("baseline", Align::Baseline),
];

pub const STACK_JUSTIFIES: &[(&str, Justify)] = &[
    ("start", Justify::Start),
    ("center", Justify::Center),
    ("end", Justify::End),
    ("space-between", Justify::SpaceBetween),
    ("space-around", Justify::SpaceAround),
];

pub fn named<T: Copy>(table: &[(&str, T)], name: &str) -> Option<T> {
    table
        .iter()
        .find_map(|(known, value)| (*known == name).then_some(*value))
}

pub fn names<'a, T>(table: &[(&'a str, T)]) -> Vec<&'a str> {
    table.iter().map(|(name, _)| *name).collect()
}

pub fn icon_named(name: &str) -> Option<Icon> {
    icons::ALL
        .iter()
        .find_map(|(known, icon)| (*known == name).then_some(*icon))
}

pub fn text_color(name: &str) -> Option<Color> {
    TEXT_COLORS
        .iter()
        .find_map(|(known, color)| (*known == name).then_some(*color))
}

#[cfg(test)]
mod tests {
    use super::*;
    use requiem_ui::components::ButtonVariant;

    #[test]
    fn every_declared_button_variant_parses() {
        for name in BUTTON_VARIANTS {
            assert!(ButtonVariant::parse(name).is_some(), "{name}");
        }
    }
}
