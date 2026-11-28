use skia_safe::Color;

use crate::models::user::DisplayNameStyles;
use crate::models::user_profile::{ProfileMeta, UserProfile};
use crate::ui::author_style::{NameFill, display_name_fill};

#[derive(Clone, Copy, Debug, Default)]
pub struct ProfileColors {
    /// The two colors bought with Nitro. Accent colors and avatar-derived
    /// colors are deliberately excluded: only an explicit profile theme should
    /// tint a profile card.
    pub theme: Option<(Color, Color)>,
    pub name_color: Option<Color>,
}

fn rgb(value: i64) -> Color {
    let value = value.clamp(0, 0xff_ffff) as u32;
    Color::from_argb(
        255,
        ((value >> 16) & 0xff) as u8,
        ((value >> 8) & 0xff) as u8,
        (value & 0xff) as u8,
    )
}

fn darken(color: Color) -> Color {
    Color::from_argb(
        255,
        (color.r() as u16 * 40 / 100) as u8,
        (color.g() as u16 * 40 / 100) as u8,
        (color.b() as u16 * 40 / 100) as u8,
    )
}

fn meta_theme(meta: Option<&ProfileMeta>) -> Option<(Color, Color)> {
    let colors = meta?.theme_colors.as_ref()?;
    let primary = colors.first().copied()?;
    let secondary = colors.get(1).copied().unwrap_or_else(|| {
        let value = primary.clamp(0, 0xff_ffff) as u32;
        ((((value >> 16) & 0xff) as i64 / 2) << 16)
            | ((((value >> 8) & 0xff) as i64 / 2) << 8)
            | ((value & 0xff) as i64 / 2)
    });
    Some((rgb(primary), rgb(secondary)))
}

fn name_style_color(styles: Option<&DisplayNameStyles>) -> Option<Color> {
    styles
        .and_then(|styles| styles.colors.as_ref())
        .and_then(|colors| colors.first().copied())
        .filter(|value| *value != 0)
        .map(rgb)
}

fn profile_name_styles(profile: &UserProfile) -> Option<&DisplayNameStyles> {
    profile
        .user_profile
        .as_ref()
        .and_then(|meta| meta.display_name_styles.as_ref())
        .or_else(|| {
            profile
                .guild_member
                .as_ref()
                .and_then(|member| member.display_name_styles.as_ref())
        })
        .or_else(|| {
            profile
                .guild_member_profile
                .as_ref()
                .and_then(|meta| meta.display_name_styles.as_ref())
        })
        .or(profile.user.display_name_styles.as_ref())
}

/// The default profile grey, matching the modal's background token.
pub fn default_background() -> Color {
    requiem_ui::colors::token(
        "background-base-lower",
        Color::from_argb(255, 0x1a, 0x1a, 0x1e),
    )
}

/// Every color stop of a Nitro display-name style, for gradient text.
pub fn name_fill(profile: &UserProfile) -> Option<NameFill> {
    display_name_fill(profile_name_styles(profile))
}

/// Resolve colors for a real profile. Only explicit Nitro profile colors become
/// a theme; the account accent color is not a substitute for them.
pub fn resolve(profile: &UserProfile) -> ProfileColors {
    let theme = meta_theme(profile.user_profile.as_ref())
        .or_else(|| meta_theme(profile.guild_member_profile.as_ref()));
    let name_color = name_style_color(profile_name_styles(profile));

    ProfileColors { theme, name_color }
}

/// Keep the single-color accent path for callers that deliberately render
/// accent-derived surfaces, such as the legacy profile preview.
pub fn accent_theme(profile: &UserProfile) -> Option<(Color, Color)> {
    let accent = rgb(profile.user.accent_color?);
    Some((accent, darken(accent)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn custom_theme_wins_over_guild_metadata() {
        let profile: UserProfile = serde_json::from_value(serde_json::json!({
            "user": { "id": "1", "username": "a", "accent_color": 255 },
            "user_profile": { "theme_colors": [16711680, 65280], "display_name_styles": { "colors": [65280] } },
            "guild_member_profile": { "theme_colors": [16776960], "display_name_styles": { "colors": [16776960] } },
        }))
        .unwrap();
        let colors = resolve(&profile);
        let (primary, secondary) = colors.theme.unwrap();
        assert_eq!((primary.r(), primary.g(), primary.b()), (255, 0, 0));
        assert_eq!((secondary.r(), secondary.g(), secondary.b()), (0, 255, 0));
        let name = colors.name_color.unwrap();
        assert_eq!((name.r(), name.g(), name.b()), (0, 255, 0));
    }

    #[test]
    fn accent_color_is_not_a_nitro_profile_theme() {
        let profile: UserProfile = serde_json::from_value(serde_json::json!({
            "user": { "id": "1", "username": "a", "accent_color": 3368601 },
        }))
        .unwrap();
        let colors = resolve(&profile);
        assert!(colors.theme.is_none());
        assert!(colors.name_color.is_none());
        let accent = accent_theme(&profile).unwrap();
        assert_eq!((accent.0.r(), accent.0.g(), accent.0.b()), (0x33, 0x66, 0x99));
    }

    #[test]
    fn display_name_styles_keep_every_gradient_stop() {
        let profile: UserProfile = serde_json::from_value(serde_json::json!({
            "user": { "id": "1", "username": "a", "display_name_styles": { "colors": [16711680, 65280, 255] } },
        }))
        .unwrap();
        assert_eq!(
            name_fill(&profile),
            Some(NameFill::Gradient(vec![
                Color::from_argb(255, 255, 0, 0),
                Color::from_argb(255, 0, 255, 0),
                Color::from_argb(255, 0, 0, 255),
                Color::from_argb(255, 255, 0, 0),
            ]))
        );
    }
}
