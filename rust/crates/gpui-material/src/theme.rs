//! Material 3 Expressive color tokens and standard light and dark palettes.
//!
//! Values below define baseline Material Design color roles in sRGB.

use gpui::Rgba;

/// Material 3 Expressive color scheme in GPUI rgba.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct M3Palette {
    /// High-emphasis accent color for prominent interactive components and active tracks.
    pub primary: Rgba,
    /// High-contrast content color drawn on top of [`Self::primary`].
    pub on_primary: Rgba,
    /// Prominent container background with lower visual weight than primary.
    pub primary_container: Rgba,
    /// High-contrast content color drawn on top of [`Self::primary_container`].
    pub on_primary_container: Rgba,
    /// Secondary accent color for less prominent components.
    pub secondary: Rgba,
    /// Content color drawn on top of [`Self::secondary`].
    pub on_secondary: Rgba,
    /// Medium-emphasis container background for tonal buttons and inactive tracks.
    pub secondary_container: Rgba,
    /// Contrast content color drawn on top of [`Self::secondary_container`].
    pub on_secondary_container: Rgba,
    /// Tertiary accent color for contrasting accents.
    pub tertiary: Rgba,
    /// Content color drawn on top of [`Self::tertiary`].
    pub on_tertiary: Rgba,
    /// Container background for tertiary elements.
    pub tertiary_container: Rgba,
    /// Content color drawn on top of [`Self::tertiary_container`].
    pub on_tertiary_container: Rgba,
    /// Error color for danger/error states.
    pub error: Rgba,
    /// Content color drawn on top of [`Self::error`].
    pub on_error: Rgba,
    /// Container background for error badges and alerts.
    pub error_container: Rgba,
    /// Content color drawn on top of [`Self::error_container`].
    pub on_error_container: Rgba,
    /// Base canvas background color.
    pub background: Rgba,
    /// Text and icons on base canvas background.
    pub on_background: Rgba,
    /// Base background color for screens, dialogs, and window surfaces.
    pub surface: Rgba,
    /// High-emphasis text and icon foreground on [`Self::surface`].
    pub on_surface: Rgba,
    /// Surface variant for subtle contrast.
    pub surface_variant: Rgba,
    /// Medium-emphasis secondary text, helper labels, and subtle icons on surface.
    pub on_surface_variant: Rgba,
    /// Inverted primary color.
    pub inverse_primary: Rgba,
    /// Inverted background for floating popovers, tooltips, and snackbars.
    pub inverse_surface: Rgba,
    /// High-contrast text on top of [`Self::inverse_surface`].
    pub inverse_on_surface: Rgba,
    /// Distinct border and outline color for medium-low emphasis components.
    pub outline: Rgba,
    /// Subtle decorative divider and separator line color.
    pub outline_variant: Rgba,
    /// Lowest-elevation container background for ambient canvas and subtle surfaces.
    pub surface_container_lowest: Rgba,
    /// Low-elevation container background.
    pub surface_container_low: Rgba,
    /// Standard container background for cards, bars, and panels.
    pub surface_container: Rgba,
    /// High-elevation container background.
    pub surface_container_high: Rgba,
    /// Highest-elevation container background for switches and elevated surfaces.
    pub surface_container_highest: Rgba,
    /// The dimmest surface (tone 87 light, 6 dark), e.g. Android's two-pane app bar.
    pub surface_dim: Rgba,
    /// The brightest surface (tone 98 light, 24 dark), e.g. Android's settings rows.
    pub surface_bright: Rgba,
}

impl M3Palette {
    /// Linearly interpolate all color tokens between `self` and `other` by parameter `t` [0.0, 1.0].
    #[must_use]
    pub fn lerp(&self, other: &Self, t: f32) -> Self {
        let t = t.clamp(0.0, 1.0);
        let lerp_rgba = |a: Rgba, b: Rgba| Rgba {
            r: a.r + (b.r - a.r) * t,
            g: a.g + (b.g - a.g) * t,
            b: a.b + (b.b - a.b) * t,
            a: a.a + (b.a - a.a) * t,
        };
        Self {
            primary: lerp_rgba(self.primary, other.primary),
            on_primary: lerp_rgba(self.on_primary, other.on_primary),
            primary_container: lerp_rgba(self.primary_container, other.primary_container),
            on_primary_container: lerp_rgba(self.on_primary_container, other.on_primary_container),
            secondary: lerp_rgba(self.secondary, other.secondary),
            on_secondary: lerp_rgba(self.on_secondary, other.on_secondary),
            secondary_container: lerp_rgba(self.secondary_container, other.secondary_container),
            on_secondary_container: lerp_rgba(
                self.on_secondary_container,
                other.on_secondary_container,
            ),
            tertiary: lerp_rgba(self.tertiary, other.tertiary),
            on_tertiary: lerp_rgba(self.on_tertiary, other.on_tertiary),
            tertiary_container: lerp_rgba(self.tertiary_container, other.tertiary_container),
            on_tertiary_container: lerp_rgba(
                self.on_tertiary_container,
                other.on_tertiary_container,
            ),
            error: lerp_rgba(self.error, other.error),
            on_error: lerp_rgba(self.on_error, other.on_error),
            error_container: lerp_rgba(self.error_container, other.error_container),
            on_error_container: lerp_rgba(self.on_error_container, other.on_error_container),
            background: lerp_rgba(self.background, other.background),
            on_background: lerp_rgba(self.on_background, other.on_background),
            surface: lerp_rgba(self.surface, other.surface),
            on_surface: lerp_rgba(self.on_surface, other.on_surface),
            surface_variant: lerp_rgba(self.surface_variant, other.surface_variant),
            on_surface_variant: lerp_rgba(self.on_surface_variant, other.on_surface_variant),
            inverse_primary: lerp_rgba(self.inverse_primary, other.inverse_primary),
            inverse_surface: lerp_rgba(self.inverse_surface, other.inverse_surface),
            inverse_on_surface: lerp_rgba(self.inverse_on_surface, other.inverse_on_surface),
            outline: lerp_rgba(self.outline, other.outline),
            outline_variant: lerp_rgba(self.outline_variant, other.outline_variant),
            surface_container_lowest: lerp_rgba(
                self.surface_container_lowest,
                other.surface_container_lowest,
            ),
            surface_container_low: lerp_rgba(
                self.surface_container_low,
                other.surface_container_low,
            ),
            surface_container: lerp_rgba(self.surface_container, other.surface_container),
            surface_container_high: lerp_rgba(
                self.surface_container_high,
                other.surface_container_high,
            ),
            surface_container_highest: lerp_rgba(
                self.surface_container_highest,
                other.surface_container_highest,
            ),
            surface_dim: lerp_rgba(self.surface_dim, other.surface_dim),
            surface_bright: lerp_rgba(self.surface_bright, other.surface_bright),
        }
    }
}

#[inline]
fn srgb(r: u8, g: u8, b: u8) -> Rgba {
    Rgba {
        r: r as f32 / 255.0,
        g: g as f32 / 255.0,
        b: b as f32 / 255.0,
        a: 1.0,
    }
}

/// Material 3 Expressive light scheme tailored for Rust.
///
/// Inspired by the official Rust geometric banner: vibrant flaming rust
/// primary (`#E85B2D`), warm golden apricot container (`#FECA91`), soft apricot
/// secondary (`#FFE2C7`), and warm sunlight cream surface (`#FFFAF5`).
#[must_use]
pub fn light_scheme() -> M3Palette {
    M3Palette {
        primary: srgb(232, 91, 45),
        on_primary: srgb(255, 255, 255),
        primary_container: srgb(254, 202, 145),
        on_primary_container: srgb(59, 14, 0),
        secondary: srgb(119, 87, 78),
        on_secondary: srgb(255, 255, 255),
        secondary_container: srgb(255, 226, 199),
        on_secondary_container: srgb(46, 21, 10),
        tertiary: srgb(107, 93, 47),
        on_tertiary: srgb(255, 255, 255),
        tertiary_container: srgb(245, 226, 168),
        on_tertiary_container: srgb(35, 27, 0),
        error: srgb(186, 26, 26),
        on_error: srgb(255, 255, 255),
        error_container: srgb(255, 218, 214),
        on_error_container: srgb(65, 0, 2),
        background: srgb(255, 250, 245),
        on_background: srgb(35, 26, 22),
        surface: srgb(255, 250, 245),
        on_surface: srgb(35, 26, 22),
        surface_variant: srgb(245, 222, 214),
        on_surface_variant: srgb(87, 70, 63),
        inverse_primary: srgb(255, 181, 155),
        inverse_surface: srgb(52, 42, 38),
        inverse_on_surface: srgb(254, 243, 237),
        outline: srgb(154, 126, 114),
        outline_variant: srgb(226, 203, 192),
        surface_container_lowest: srgb(255, 255, 255),
        surface_container_low: srgb(254, 246, 240),
        surface_container: srgb(249, 239, 232),
        surface_container_high: srgb(244, 233, 226),
        surface_container_highest: srgb(245, 225, 212),
        surface_dim: srgb(236, 224, 216),
        surface_bright: srgb(255, 248, 245),
    }
}

/// Material 3 Expressive dark scheme tailored for Rust.
///
/// Features radiant molten coral-rust primary (`#FA8559`), deep dark roast
/// mahogany on-primary (`#541702`), burning copper container (`#99310D`),
/// sleek dark plum-charcoal secondary container (`#3A3036`), and deep neutral
/// obsidian surface (`#121016`).
#[must_use]
pub fn dark_scheme() -> M3Palette {
    M3Palette {
        primary: srgb(250, 133, 89),
        on_primary: srgb(84, 23, 2),
        primary_container: srgb(153, 49, 13),
        on_primary_container: srgb(254, 219, 208),
        secondary: srgb(231, 189, 178),
        on_secondary: srgb(68, 42, 34),
        secondary_container: srgb(58, 48, 54),
        on_secondary_container: srgb(254, 219, 208),
        tertiary: srgb(216, 197, 141),
        on_tertiary: srgb(58, 47, 6),
        tertiary_container: srgb(82, 69, 26),
        on_tertiary_container: srgb(245, 226, 168),
        error: srgb(255, 180, 171),
        on_error: srgb(105, 0, 5),
        error_container: srgb(147, 0, 10),
        on_error_container: srgb(255, 218, 214),
        background: srgb(18, 16, 22),
        on_background: srgb(235, 230, 238),
        surface: srgb(18, 16, 22),
        on_surface: srgb(235, 230, 238),
        surface_variant: srgb(83, 67, 62),
        on_surface_variant: srgb(200, 194, 204),
        inverse_primary: srgb(153, 49, 13),
        inverse_surface: srgb(235, 230, 238),
        inverse_on_surface: srgb(48, 45, 54),
        outline: srgb(145, 140, 150),
        outline_variant: srgb(72, 68, 78),
        surface_container_lowest: srgb(13, 11, 16),
        surface_container_low: srgb(26, 24, 30),
        surface_container: srgb(32, 29, 36),
        surface_container_high: srgb(36, 33, 41),
        surface_container_highest: srgb(40, 37, 46),
        surface_dim: srgb(18, 16, 22),
        surface_bright: srgb(56, 52, 62),
    }
}

/// Converts an Argb color from material-colors into a GPUI Rgba.
#[inline]
pub fn argb_to_rgba(c: material_colors::color::Argb) -> Rgba {
    Rgba {
        r: c.red as f32 / 255.0,
        g: c.green as f32 / 255.0,
        b: c.blue as f32 / 255.0,
        a: c.alpha as f32 / 255.0,
    }
}

impl From<&material_colors::scheme::Scheme> for M3Palette {
    fn from(s: &material_colors::scheme::Scheme) -> Self {
        Self {
            primary: argb_to_rgba(s.primary),
            on_primary: argb_to_rgba(s.on_primary),
            primary_container: argb_to_rgba(s.primary_container),
            on_primary_container: argb_to_rgba(s.on_primary_container),
            secondary: argb_to_rgba(s.secondary),
            on_secondary: argb_to_rgba(s.on_secondary),
            secondary_container: argb_to_rgba(s.secondary_container),
            on_secondary_container: argb_to_rgba(s.on_secondary_container),
            tertiary: argb_to_rgba(s.tertiary),
            on_tertiary: argb_to_rgba(s.on_tertiary),
            tertiary_container: argb_to_rgba(s.tertiary_container),
            on_tertiary_container: argb_to_rgba(s.on_tertiary_container),
            error: argb_to_rgba(s.error),
            on_error: argb_to_rgba(s.on_error),
            error_container: argb_to_rgba(s.error_container),
            on_error_container: argb_to_rgba(s.on_error_container),
            background: argb_to_rgba(s.background),
            on_background: argb_to_rgba(s.on_background),
            surface: argb_to_rgba(s.surface),
            on_surface: argb_to_rgba(s.on_surface),
            surface_variant: argb_to_rgba(s.surface_variant),
            on_surface_variant: argb_to_rgba(s.on_surface_variant),
            inverse_primary: argb_to_rgba(s.inverse_primary),
            inverse_surface: argb_to_rgba(s.inverse_surface),
            inverse_on_surface: argb_to_rgba(s.inverse_on_surface),
            outline: argb_to_rgba(s.outline),
            outline_variant: argb_to_rgba(s.outline_variant),
            surface_container_lowest: argb_to_rgba(s.surface_container_lowest),
            surface_container_low: argb_to_rgba(s.surface_container_low),
            surface_container: argb_to_rgba(s.surface_container),
            surface_container_high: argb_to_rgba(s.surface_container_high),
            surface_container_highest: argb_to_rgba(s.surface_container_highest),
            surface_dim: argb_to_rgba(s.surface_dim),
            surface_bright: argb_to_rgba(s.surface_bright),
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
struct SerializablePalette {
    primary: [f32; 4],
    on_primary: [f32; 4],
    primary_container: [f32; 4],
    on_primary_container: [f32; 4],
    secondary: [f32; 4],
    on_secondary: [f32; 4],
    secondary_container: [f32; 4],
    on_secondary_container: [f32; 4],
    tertiary: [f32; 4],
    on_tertiary: [f32; 4],
    tertiary_container: [f32; 4],
    on_tertiary_container: [f32; 4],
    error: [f32; 4],
    on_error: [f32; 4],
    error_container: [f32; 4],
    on_error_container: [f32; 4],
    background: [f32; 4],
    on_background: [f32; 4],
    surface: [f32; 4],
    on_surface: [f32; 4],
    surface_variant: [f32; 4],
    on_surface_variant: [f32; 4],
    inverse_primary: [f32; 4],
    inverse_surface: [f32; 4],
    inverse_on_surface: [f32; 4],
    outline: [f32; 4],
    outline_variant: [f32; 4],
    surface_container_lowest: [f32; 4],
    surface_container_low: [f32; 4],
    surface_container: [f32; 4],
    surface_container_high: [f32; 4],
    surface_container_highest: [f32; 4],
    // absent from palettes saved before these roles existed
    #[serde(default)]
    surface_dim: Option<[f32; 4]>,
    #[serde(default)]
    surface_bright: Option<[f32; 4]>,
}

impl serde::Serialize for M3Palette {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let to_arr = |c: Rgba| [c.r, c.g, c.b, c.a];
        let s = SerializablePalette {
            primary: to_arr(self.primary),
            on_primary: to_arr(self.on_primary),
            primary_container: to_arr(self.primary_container),
            on_primary_container: to_arr(self.on_primary_container),
            secondary: to_arr(self.secondary),
            on_secondary: to_arr(self.on_secondary),
            secondary_container: to_arr(self.secondary_container),
            on_secondary_container: to_arr(self.on_secondary_container),
            tertiary: to_arr(self.tertiary),
            on_tertiary: to_arr(self.on_tertiary),
            tertiary_container: to_arr(self.tertiary_container),
            on_tertiary_container: to_arr(self.on_tertiary_container),
            error: to_arr(self.error),
            on_error: to_arr(self.on_error),
            error_container: to_arr(self.error_container),
            on_error_container: to_arr(self.on_error_container),
            background: to_arr(self.background),
            on_background: to_arr(self.on_background),
            surface: to_arr(self.surface),
            on_surface: to_arr(self.on_surface),
            surface_variant: to_arr(self.surface_variant),
            on_surface_variant: to_arr(self.on_surface_variant),
            inverse_primary: to_arr(self.inverse_primary),
            inverse_surface: to_arr(self.inverse_surface),
            inverse_on_surface: to_arr(self.inverse_on_surface),
            outline: to_arr(self.outline),
            outline_variant: to_arr(self.outline_variant),
            surface_container_lowest: to_arr(self.surface_container_lowest),
            surface_container_low: to_arr(self.surface_container_low),
            surface_container: to_arr(self.surface_container),
            surface_container_high: to_arr(self.surface_container_high),
            surface_container_highest: to_arr(self.surface_container_highest),
            surface_dim: Some(to_arr(self.surface_dim)),
            surface_bright: Some(to_arr(self.surface_bright)),
        };
        s.serialize(serializer)
    }
}

impl<'de> serde::Deserialize<'de> for M3Palette {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = SerializablePalette::deserialize(deserializer)?;
        let from_arr = |a: [f32; 4]| Rgba {
            r: a[0],
            g: a[1],
            b: a[2],
            a: a[3],
        };
        Ok(Self {
            primary: from_arr(s.primary),
            on_primary: from_arr(s.on_primary),
            primary_container: from_arr(s.primary_container),
            on_primary_container: from_arr(s.on_primary_container),
            secondary: from_arr(s.secondary),
            on_secondary: from_arr(s.on_secondary),
            secondary_container: from_arr(s.secondary_container),
            on_secondary_container: from_arr(s.on_secondary_container),
            tertiary: from_arr(s.tertiary),
            on_tertiary: from_arr(s.on_tertiary),
            tertiary_container: from_arr(s.tertiary_container),
            on_tertiary_container: from_arr(s.on_tertiary_container),
            error: from_arr(s.error),
            on_error: from_arr(s.on_error),
            error_container: from_arr(s.error_container),
            on_error_container: from_arr(s.on_error_container),
            background: from_arr(s.background),
            on_background: from_arr(s.on_background),
            surface: from_arr(s.surface),
            on_surface: from_arr(s.on_surface),
            surface_variant: from_arr(s.surface_variant),
            on_surface_variant: from_arr(s.on_surface_variant),
            inverse_primary: from_arr(s.inverse_primary),
            inverse_surface: from_arr(s.inverse_surface),
            inverse_on_surface: from_arr(s.inverse_on_surface),
            outline: from_arr(s.outline),
            outline_variant: from_arr(s.outline_variant),
            surface_container_lowest: from_arr(s.surface_container_lowest),
            surface_container_low: from_arr(s.surface_container_low),
            surface_container: from_arr(s.surface_container),
            surface_container_high: from_arr(s.surface_container_high),
            surface_container_highest: from_arr(s.surface_container_highest),
            // older files: the closest roles they do carry
            surface_dim: from_arr(s.surface_dim.unwrap_or(s.surface_container_highest)),
            surface_bright: from_arr(s.surface_bright.unwrap_or(
                if s.surface[0] + s.surface[1] + s.surface[2] > 1.5 {
                    s.surface_container_lowest
                } else {
                    s.surface_container_highest
                },
            )),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn light_primary_matches_rust_m3_primary40() {
        let p = light_scheme();
        let expected = srgb(232, 91, 45);
        assert_eq!(p.primary, expected);
    }

    #[test]
    fn dark_primary_matches_rust_m3_primary80() {
        let p = dark_scheme();
        let expected = srgb(250, 133, 89);
        assert_eq!(p.primary, expected);
    }
}
