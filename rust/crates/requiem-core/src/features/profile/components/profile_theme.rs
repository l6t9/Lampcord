use skia_safe::Color;

use crate::models::user::DisplayNameStyles;
use crate::models::user_profile::{ProfileMeta, UserProfile};

#[derive(Clone, Copy, Debug, Default)]
pub struct ProfileColors {
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
        .map(rgb)
}

/// Resolve colors for a real profile: the profile's own theme wins over guild
/// metadata, with the account accent color as the last fallback.
pub fn resolve(profile: &UserProfile) -> ProfileColors {
    let theme = meta_theme(profile.user_profile.as_ref())
        .or_else(|| meta_theme(profile.guild_member_profile.as_ref()))
        .or_else(|| {
            let accent = rgb(profile.user.accent_color?);
            Some((accent, darken(accent)))
        });

    let name_color = name_style_color(
        profile
            .user_profile
            .as_ref()
            .and_then(|meta| meta.display_name_styles.as_ref()),
    )
    .or_else(|| {
        name_style_color(
            profile
                .guild_member
                .as_ref()
                .and_then(|member| member.display_name_styles.as_ref()),
        )
    })
    .or_else(|| {
        name_style_color(
            profile
                .guild_member_profile
                .as_ref()
                .and_then(|meta| meta.display_name_styles.as_ref()),
        )
    })
    .or_else(|| name_style_color(profile.user.display_name_styles.as_ref()));

    ProfileColors { theme, name_color }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn custom_theme_wins_over_guild_metadata_and_accent() {
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
    fn accent_color_is_the_last_theme_fallback() {
        let profile: UserProfile = serde_json::from_value(serde_json::json!({
            "user": { "id": "1", "username": "a", "accent_color": 3368601 },
        }))
        .unwrap();
        let colors = resolve(&profile);
        let (primary, secondary) = colors.theme.unwrap();
        assert_eq!((primary.r(), primary.g(), primary.b()), (0x33, 0x66, 0x99));
        assert_eq!(secondary.r(), 0x14);
        assert!(colors.name_color.is_none());
    }
}
