use skia_safe::Color;

pub fn token(name: &str, fallback: Color) -> Color {
    crate::theme_color(name, fallback)
}

const TEXT_STRONG_DARK: Color = Color::from_argb(255, 0xfb, 0xfb, 0xfb);
const TEXT_DEFAULT_DARK: Color = Color::from_argb(255, 0xef, 0xef, 0xf1);
const TEXT_SECONDARY_DARK: Color = Color::from_argb(255, 0xab, 0xac, 0xb2);
const TEXT_MUTED_DARK: Color = Color::from_argb(255, 0x96, 0x97, 0x9e);
const TEXT_SUBTLE_DARK: Color = Color::from_argb(255, 0x81, 0x82, 0x8a);
const TEXT_CRITICAL_DARK: Color = Color::from_argb(255, 0xf8, 0x7e, 0x7a);
const TEXT_LINK_DARK: Color = Color::from_argb(255, 0x79, 0x8d, 0xf9);
const TEXT_ANCHOR_DARK: Color = Color::from_argb(255, 0x4d, 0x96, 0xee);
const TEXT_POSITIVE_DARK: Color = Color::from_argb(255, 0x5e, 0xb4, 0x79);
const WARNING_DARK: Color = Color::from_argb(255, 0xea, 0x98, 0x00);
const SURFACE_HIGH_DARK: Color = Color::from_argb(255, 0x24, 0x24, 0x29);
const SURFACE_LOWEST_DARK: Color = Color::from_argb(255, 0x12, 0x12, 0x14);
const ICON_MUTED_DARK: Color = Color::from_argb(255, 0x96, 0x97, 0x9e);
const SPINE_DEFAULT_DARK: Color = Color::from_argb(255, 0x47, 0x48, 0x51);

pub fn text_strong() -> Color {
    token("text-strong", TEXT_STRONG_DARK)
}

pub fn text_default() -> Color {
    token("text-default", TEXT_DEFAULT_DARK)
}

pub fn text_secondary() -> Color {
    token("text-subtle", TEXT_SECONDARY_DARK)
}

pub fn text_muted() -> Color {
    token("text-muted", TEXT_MUTED_DARK)
}

pub fn text_subtle() -> Color {
    token("channels-default", TEXT_SUBTLE_DARK)
}

pub fn text_critical() -> Color {
    token("text-feedback-critical", TEXT_CRITICAL_DARK)
}

pub fn text_link() -> Color {
    token("text-brand", TEXT_LINK_DARK)
}

pub fn text_anchor() -> Color {
    token("text-link", TEXT_ANCHOR_DARK)
}

pub fn text_positive() -> Color {
    token("text-feedback-positive", TEXT_POSITIVE_DARK)
}

pub fn warning() -> Color {
    token("text-feedback-warning", WARNING_DARK)
}

pub fn surface_high() -> Color {
    token("background-surface-high", SURFACE_HIGH_DARK)
}

pub fn surface_lowest() -> Color {
    token("background-base-lowest", SURFACE_LOWEST_DARK)
}

pub fn icon_muted() -> Color {
    token("icon-muted", ICON_MUTED_DARK)
}
fn code_palette() -> bool {
    crate::theme_name() == "light"
}

fn code(dark: Color, light: Color) -> Color {
    if code_palette() { light } else { dark }
}

pub fn text_code() -> Color {
    code(TEXT_CODE, Color::from_rgb(0x2e, 0x2e, 0x34))
}

pub fn text_code_keyword() -> Color {
    code(TEXT_CODE_KEYWORD, Color::from_rgb(0xc4, 0x1a, 0x16))
}

pub fn text_code_string() -> Color {
    code(TEXT_CODE_STRING, Color::from_rgb(0x1a, 0x7f, 0x37))
}

pub fn text_code_comment() -> Color {
    code(TEXT_CODE_COMMENT, Color::from_rgb(0x6e, 0x77, 0x81))
}

pub fn text_code_number() -> Color {
    code(TEXT_CODE_NUMBER, Color::from_rgb(0x95, 0x3c, 0x00))
}

pub fn text_code_type() -> Color {
    code(TEXT_CODE_TYPE, Color::from_rgb(0x82, 0x50, 0xdf))
}

pub fn text_code_title() -> Color {
    code(TEXT_CODE_TITLE, Color::from_rgb(0x05, 0x50, 0xae))
}

pub fn text_code_property() -> Color {
    code(TEXT_CODE_PROPERTY, Color::from_rgb(0x05, 0x5d, 0x67))
}

pub fn text_code_builtin() -> Color {
    code(TEXT_CODE_BUILTIN, Color::from_rgb(0x95, 0x53, 0x00))
}

pub fn text_code_addition() -> Color {
    code(TEXT_CODE_ADDITION, Color::from_rgb(0x1a, 0x7f, 0x37))
}

pub fn text_code_deletion() -> Color {
    code(TEXT_CODE_DELETION, Color::from_rgb(0xc4, 0x1a, 0x16))
}

pub fn background_code() -> Color {
    code(BACKGROUND_CODE, Color::from_argb(20, 0x58, 0x65, 0xf2))
}
pub fn live(color: Color) -> Color {
    match (color.a(), color.r(), color.g(), color.b()) {
        (255, 0xfb, 0xfb, 0xfb) => text_strong(),
        (255, 0xef, 0xef, 0xf1) => text_default(),
        (255, 0xab, 0xac, 0xb2) => text_secondary(),
        (255, 0x96, 0x97, 0x9e) => text_muted(),
        (255, 0x81, 0x82, 0x8a) => text_subtle(),
        (255, 0xf8, 0x7e, 0x7a) => text_critical(),
        (255, 0x79, 0x8d, 0xf9) => text_link(),
        (255, 0x4d, 0x96, 0xee) => text_anchor(),
        (255, 0x5e, 0xb4, 0x79) => text_positive(),
        (255, 0xea, 0x98, 0x00) => warning(),
        (255, 0x24, 0x24, 0x29) => surface_high(),
        (255, 0x12, 0x12, 0x14) => surface_lowest(),
        (255, 0x1a, 0x1a, 0x1e) => token("background-base-lower", color),
        (255, 0x20, 0x20, 0x24) => token("background-base-low", color),
        (255, 0x4f, 0x56, 0x60) => icon_muted(),
        (255, 0x47, 0x48, 0x51) => token("spine-default", color),
        (0x33, 0x97, 0x97, 0x9f) => token("border-normal", color),
        (0x14, 0x97, 0x97, 0x9f) => token("background-mod-muted", color),
        _ => color,
    }
}

pub const TEXT_STRONG: Color = TEXT_STRONG_DARK;
pub const TEXT_DEFAULT: Color = TEXT_DEFAULT_DARK;
pub const TEXT_SECONDARY: Color = TEXT_SECONDARY_DARK;
pub const TEXT_MUTED: Color = TEXT_MUTED_DARK;
pub const TEXT_SUBTLE: Color = TEXT_SUBTLE_DARK;
pub const TEXT_CRITICAL: Color = TEXT_CRITICAL_DARK;
pub const TEXT_LINK: Color = TEXT_LINK_DARK;
pub const TEXT_ANCHOR: Color = TEXT_ANCHOR_DARK;
pub const TEXT_POSITIVE: Color = TEXT_POSITIVE_DARK;
pub const WARNING: Color = WARNING_DARK;
pub const SURFACE_HIGH: Color = SURFACE_HIGH_DARK;
pub const SPINE_DEFAULT: Color = SPINE_DEFAULT_DARK;

pub const TEXT_PLACEHOLDER: Color = Color::from_argb(255, 0x6c, 0x6d, 0x76);

pub const INPUT_PLACEHOLDER: Color = Color::from_argb(255, 0x8f, 0x90, 0x97);

pub const RED_400: Color = Color::from_argb(255, 0xda, 0x3e, 0x44);

pub const TEXT_MENTION: Color = Color::from_argb(255, 0xa9, 0xbb, 0xff);

pub const BADGE_NOTIFICATION: Color = Color::from_argb(255, 0xd2, 0x2d, 0x39);

pub const GREEN_360: Color = Color::from_argb(255, 0x3d, 0x9e, 0x60);

pub const BRAND: Color = Color::from_argb(255, 0x58, 0x65, 0xf2);

pub const DEFAULT_ROLE: Color = Color::from_argb(255, 0x99, 0xaa, 0xb5);

pub const BACKGROUND_MOD_SUBTLE: Color = Color::from_argb(0x1f, 0x97, 0x97, 0x9f);

pub const BACKGROUND_MOD_MUTED: Color = Color::from_argb(0x14, 0x97, 0x97, 0x9f);

pub const BACKGROUND_MOD_NORMAL: Color = Color::from_argb(0x29, 0x97, 0x97, 0x9f);

pub const SCROLLBAR_THIN_THUMB: Color = Color::from_argb(255, 0x5f, 0x60, 0x6a);

pub const SCROLLBAR_AUTO_THUMB: Color = Color::from_argb(255, 0x66, 0x67, 0x70);

pub const MENTION_BACKGROUND: Color = Color::from_argb(0x3d, 0x58, 0x64, 0xf2);

pub const TEXT_CODE: Color = Color::from_argb(255, 0xe4, 0xe4, 0xe6);

pub const TEXT_CODE_KEYWORD: Color = Color::from_argb(255, 0xff, 0x96, 0x91);

pub const TEXT_CODE_STRING: Color = Color::from_argb(255, 0x73, 0xc4, 0x8b);

pub const TEXT_CODE_COMMENT: Color = Color::from_argb(255, 0x99, 0x9a, 0xa1);

pub const TEXT_CODE_NUMBER: Color = Color::from_argb(255, 0xef, 0xa2, 0x75);

pub const TEXT_CODE_TYPE: Color = Color::from_argb(255, 0xff, 0x8b, 0xe2);

pub const TEXT_CODE_TITLE: Color = Color::from_argb(255, 0x9d, 0xb0, 0xff);

pub const TEXT_CODE_PROPERTY: Color = Color::from_argb(255, 0x63, 0xc1, 0xca);

pub const TEXT_CODE_BUILTIN: Color = Color::from_argb(255, 0xf4, 0x9f, 0x00);

pub const TEXT_CODE_ADDITION: Color = Color::from_argb(255, 0x7a, 0xc9, 0x91);

pub const TEXT_CODE_DELETION: Color = Color::from_argb(255, 0xff, 0x96, 0x91);

pub const BACKGROUND_CODE: Color = Color::from_argb(20, 0x58, 0x65, 0xf2);

pub const BORDER_NORMAL: Color = Color::from_argb(0x33, 0x97, 0x97, 0x9f);

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use super::*;

    #[test]
    fn a_known_dark_color_follows_the_theme_and_white_does_not() {
        crate::set_theme_name("light");
        crate::install_theme_tokens(HashMap::from([(
            "text-strong".into(),
            "#28282d".into(),
        )]));
        assert_eq!(live(TEXT_STRONG), Color::from_rgb(0x28, 0x28, 0x2d));
        assert_eq!(live(Color::WHITE), Color::WHITE);
        crate::install_theme_tokens(HashMap::new());
        crate::set_theme_name("dark");
    }
}
