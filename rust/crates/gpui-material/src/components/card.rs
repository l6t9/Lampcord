//! Material 3 Expressive Card component.
//!
//! Provides three emphasis variants ([`CardVariant::Filled`], [`CardVariant::Elevated`],
//! [`CardVariant::Outlined`]), interactive spring-driven corner morphing, state layer overlays,
//! flexible content slots, and arbitrary child container rendering.
//!
//! Directly transcribed from the upstream AndroidX Compose Material 3 Card specifications
//! (`FilledCardTokens`, `ElevatedCardTokens`, `OutlinedCardTokens`).

use gpui::{
    Div, FontWeight, IntoElement, ParentElement, Rgba, SharedString, Stateful, Styled, Window, div,
    prelude::*, px,
};

use crate::motion::{SpringParams, SpringState, spring_step};
use crate::theme::M3Palette;

/// Visual emphasis variants matching Material 3 specifications.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CardVariant {
    /// Medium-low emphasis with a subtle filled container.
    ///
    /// Uses `SurfaceContainerHighest` with zero resting elevation.
    #[default]
    Filled,
    /// Medium emphasis with a drop shadow.
    ///
    /// Uses `Surface` with Level 1 (1dp) resting shadow elevation.
    Elevated,
    /// High-emphasis boundary outline without a shadow.
    ///
    /// Uses `Surface` container with a 1.0dp `OutlineVariant` border.
    Outlined,
}

/// Layout metrics for a Material 3 card container.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CardMetrics {
    /// Resting corner radius (default: 12.0dp matching `CornerMedium`).
    pub round_corner: f32,
    /// Tactile corner radius when pressed (default: 8.0dp matching `CornerSmall`).
    pub pressed_corner: f32,
    /// Border width for [`CardVariant::Outlined`] (default: 1.0dp).
    pub outline_width: f32,
    /// Horizontal padding inside the card container.
    pub padding_horizontal: f32,
    /// Vertical padding inside the card container.
    pub padding_vertical: f32,
    /// Vertical spacing between header elements (title and subhead).
    pub header_spacing: f32,
    /// Vertical spacing between content blocks (media, body, actions).
    pub content_spacing: f32,
    /// Title typography font size.
    pub title_size: f32,
    /// Subhead typography font size.
    pub subhead_size: f32,
    /// Body typography font size.
    pub body_size: f32,
}

impl Default for CardMetrics {
    fn default() -> Self {
        Self {
            round_corner: 12.0,
            pressed_corner: 8.0,
            outline_width: 1.0,
            padding_horizontal: 16.0,
            padding_vertical: 16.0,
            header_spacing: 4.0,
            content_spacing: 12.0,
            title_size: 16.0,
            subhead_size: 13.0,
            body_size: 14.0,
        }
    }
}

/// Resolved color scheme for a Card variant in current theme.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CardColors {
    /// Background container color.
    pub container: Rgba,
    /// Primary content color (text, titles, icons).
    pub content: Rgba,
    /// Secondary content color (subheads, supporting body text).
    pub secondary_content: Rgba,
    /// State layer overlay color for hover/press interactions.
    pub state_layer: Rgba,
    /// Optional outline border color.
    pub border: Option<Rgba>,
    /// Whether this card variant casts a drop shadow.
    pub has_shadow: bool,
    /// Disabled container background color.
    pub disabled_container: Rgba,
    /// Disabled primary content color.
    pub disabled_content: Rgba,
    /// Disabled secondary content color.
    pub disabled_secondary_content: Rgba,
    /// Optional disabled outline border color.
    pub disabled_border: Option<Rgba>,
}

impl CardColors {
    /// Resolves colors for a [`CardVariant`] from an [`M3Palette`].
    #[must_use]
    pub fn for_variant(variant: CardVariant, p: &M3Palette) -> Self {
        match variant {
            CardVariant::Filled => Self::filled(p),
            CardVariant::Elevated => Self::elevated(p),
            CardVariant::Outlined => Self::outlined(p),
        }
    }

    /// Filled card colors: `SurfaceContainerHighest` container, no outline, no shadow.
    #[must_use]
    pub fn filled(p: &M3Palette) -> Self {
        Self {
            container: p.surface_container_highest,
            content: p.on_surface,
            secondary_content: p.on_surface_variant,
            state_layer: p.on_surface,
            border: None,
            has_shadow: false,
            disabled_container: Rgba {
                a: 0.38,
                ..p.surface_container_highest
            },
            disabled_content: Rgba {
                a: 0.38,
                ..p.on_surface
            },
            disabled_secondary_content: Rgba {
                a: 0.38,
                ..p.on_surface_variant
            },
            disabled_border: None,
        }
    }

    /// Filled card colors with `SurfaceContainer` container.
    #[must_use]
    pub fn surface_container(p: &M3Palette) -> Self {
        Self {
            container: p.surface_container,
            ..Self::filled(p)
        }
    }

    /// Elevated card colors: `Surface` container with Level 1 elevation shadow.
    #[must_use]
    pub fn elevated(p: &M3Palette) -> Self {
        Self {
            container: p.surface,
            content: p.on_surface,
            secondary_content: p.on_surface_variant,
            state_layer: p.primary,
            border: None,
            has_shadow: true,
            disabled_container: Rgba {
                a: 0.38,
                ..p.surface
            },
            disabled_content: Rgba {
                a: 0.38,
                ..p.on_surface
            },
            disabled_secondary_content: Rgba {
                a: 0.38,
                ..p.on_surface_variant
            },
            disabled_border: None,
        }
    }

    /// Outlined card colors: `Surface` container with 1.0dp `OutlineVariant` border.
    #[must_use]
    pub fn outlined(p: &M3Palette) -> Self {
        Self {
            container: p.surface,
            content: p.on_surface,
            secondary_content: p.on_surface_variant,
            state_layer: p.on_surface,
            border: Some(p.outline_variant),
            has_shadow: false,
            disabled_container: p.surface,
            disabled_content: Rgba {
                a: 0.38,
                ..p.on_surface
            },
            disabled_secondary_content: Rgba {
                a: 0.38,
                ..p.on_surface_variant
            },
            disabled_border: Some(Rgba {
                a: 0.12,
                ..p.outline
            }),
        }
    }
}

/// Material 3 Expressive Card model with spring corner morphing and state layer dynamics.
///
/// Supports three emphasis styles ([`CardVariant::Filled`], [`CardVariant::Elevated`],
/// [`CardVariant::Outlined`]), static container mode, interactive clickable mode with tactile
/// corner morphing, built-in slots (title, subhead, media, body), and custom child rendering.
///
/// # Examples
///
/// ```rust
/// use gpui_material::card::{CardModel, CardVariant};
/// use gpui_material::theme::light_scheme;
///
/// let card = CardModel::new(0)
///     .variant(CardVariant::Elevated)
///     .title("System Status")
///     .subhead("Hyprland Desktop")
///     .body("All compositor services operating normally.");
///
/// let palette = light_scheme();
/// let _element = card.render(&palette);
/// ```
#[derive(Clone, Debug)]
pub struct CardModel {
    /// Unique identifier for GPUI view element caching.
    pub id: usize,
    /// Visual emphasis variant.
    pub variant: CardVariant,
    /// Whether the card is enabled.
    pub enabled: bool,
    /// Whether the card handles click/hover interactions.
    pub interactive: bool,
    /// Whether the card is currently hovered.
    pub is_hovered: bool,
    /// Whether the card is currently pressed.
    pub is_pressed: bool,
    /// Animated corner radius spring state.
    pub corner: SpringState,
    /// Animated state layer opacity spring state.
    pub state_layer: SpringState,
    /// Optional card title text.
    pub title: Option<SharedString>,
    /// Optional secondary subhead text.
    pub subhead: Option<SharedString>,
    /// Optional supporting body text.
    pub body: Option<SharedString>,
    /// Optional decorative media accent height in pixels.
    pub media_height: Option<f32>,
    /// Optional media placeholder badge label.
    pub media_label: Option<SharedString>,
    /// Optional custom metrics override.
    pub custom_metrics: Option<CardMetrics>,
    /// Optional custom colors override.
    pub custom_colors: Option<CardColors>,
    /// Optional container background color override.
    pub container_color: Option<Rgba>,
    /// Optional spring physics override.
    pub custom_spring: Option<SpringParams>,
}

impl CardModel {
    /// Creates a new card model with default filled styling.
    #[must_use]
    pub fn new(id: usize) -> Self {
        let metrics = CardMetrics::default();
        Self {
            id,
            variant: CardVariant::default(),
            enabled: true,
            interactive: false,
            is_hovered: false,
            is_pressed: false,
            corner: SpringState {
                pos: metrics.round_corner,
                vel: 0.0,
            },
            state_layer: SpringState::default(),
            title: None,
            subhead: None,
            body: None,
            media_height: None,
            media_label: None,
            custom_metrics: None,
            custom_colors: None,
            container_color: None,
            custom_spring: None,
        }
    }

    /// Sets a custom container background color override.
    #[must_use]
    pub fn container_color(mut self, color: Rgba) -> Self {
        self.container_color = Some(color);
        self
    }

    /// Sets the visual emphasis variant.
    #[must_use]
    pub fn variant(mut self, variant: CardVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Sets whether the card is interactive (hoverable/clickable).
    #[must_use]
    pub fn interactive(mut self, interactive: bool) -> Self {
        self.interactive = interactive;
        self
    }

    /// Sets whether the card is enabled.
    #[must_use]
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Sets the card title.
    #[must_use]
    pub fn title(mut self, title: impl Into<SharedString>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Sets the secondary subhead.
    #[must_use]
    pub fn subhead(mut self, subhead: impl Into<SharedString>) -> Self {
        self.subhead = Some(subhead.into());
        self
    }

    /// Sets the supporting body text.
    #[must_use]
    pub fn body(mut self, body: impl Into<SharedString>) -> Self {
        self.body = Some(body.into());
        self
    }

    /// Sets a decorative media banner block with height and an optional label.
    #[must_use]
    pub fn media(mut self, height: f32, label: Option<impl Into<SharedString>>) -> Self {
        self.media_height = Some(height);
        self.media_label = label.map(Into::into);
        self
    }

    /// Sets custom layout metrics.
    #[must_use]
    pub fn with_metrics(mut self, metrics: CardMetrics) -> Self {
        self.custom_metrics = Some(metrics);
        self
    }

    /// Sets custom color overrides.
    #[must_use]
    pub fn with_colors(mut self, colors: CardColors) -> Self {
        self.custom_colors = Some(colors);
        self
    }

    /// Sets custom spring animation parameters.
    #[must_use]
    pub fn with_spring_params(mut self, spring: SpringParams) -> Self {
        self.custom_spring = Some(spring);
        self
    }

    /// Returns the effective layout metrics.
    #[must_use]
    pub fn metrics(&self) -> CardMetrics {
        self.custom_metrics.unwrap_or_default()
    }

    /// Resolves the effective colors for the given palette.
    #[must_use]
    pub fn resolve_colors(&self, pal: &M3Palette) -> CardColors {
        let mut colors = self
            .custom_colors
            .unwrap_or_else(|| CardColors::for_variant(self.variant, pal));
        if let Some(cc) = self.container_color {
            colors.container = cc;
        }
        colors
    }

    /// Returns the target corner radius based on press state.
    #[must_use]
    pub fn target_corner(&self, metrics: &CardMetrics) -> f32 {
        if self.interactive && self.is_pressed && self.enabled {
            metrics.pressed_corner
        } else {
            metrics.round_corner
        }
    }

    /// Returns the target state layer alpha for hover/press states.
    #[must_use]
    pub fn target_state_layer_alpha(&self) -> f32 {
        if !self.interactive || !self.enabled {
            0.0
        } else if self.is_pressed {
            0.12
        } else if self.is_hovered {
            0.08
        } else {
            0.0
        }
    }

    /// Advances spring physics by `dt` seconds.
    ///
    /// Returns `true` if springs are still in motion and require another frame,
    /// or `false` if all springs have fully settled to their target states.
    pub fn step(&mut self, dt: f32) -> bool {
        let metrics = self.metrics();
        let corner_target = self.target_corner(&metrics);
        let layer_target = self.target_state_layer_alpha();

        let spring = self.custom_spring.unwrap_or_else(SpringParams::expressive);

        self.corner = spring_step(spring, self.corner, corner_target, dt);
        self.state_layer = spring_step(spring, self.state_layer, layer_target, dt);

        let settled = self.corner.is_settled(corner_target, 0.05, 0.1)
            && self.state_layer.is_settled(layer_target, 0.002, 0.005);

        !settled
    }

    /// Advance spring animations by `dt` seconds, requesting an animation frame if unsettled.
    pub fn advance_springs(&mut self, window: &mut Window, dt: f32) {
        if self.step(dt) {
            window.request_animation_frame();
        }
    }

    /// Renders the card container wrapping arbitrary custom child content.
    pub fn render_with<C>(&self, pal: &M3Palette, content: C) -> Stateful<Div>
    where
        C: IntoElement,
    {
        let metrics = self.metrics();
        let colors = self.resolve_colors(pal);

        let (bg, fg, border) = if self.enabled {
            (colors.container, colors.content, colors.border)
        } else {
            (
                colors.disabled_container,
                colors.disabled_content,
                colors.disabled_border,
            )
        };

        let active_corner = if self.corner.pos > 0.0 {
            self.corner.pos
        } else {
            self.target_corner(&metrics)
        };

        let border_w = if border.is_some() {
            metrics.outline_width
        } else {
            0.0
        };

        let mut root = div()
            .id(("m3-card", self.id))
            .relative()
            .overflow_hidden()
            .rounded(px(active_corner))
            .bg(bg)
            .text_color(fg)
            .when(self.interactive && self.enabled, |d| d.cursor_pointer())
            .when(colors.has_shadow && self.enabled, |d| {
                if self.is_hovered && self.interactive {
                    d.shadow_md()
                } else {
                    d.shadow_sm()
                }
            })
            .when(border_w > 0.0, |d| {
                if let Some(bc) = border {
                    d.border(px(border_w)).border_color(bc)
                } else {
                    d
                }
            });

        // Interactive state layer overlay
        if self.interactive && self.enabled && self.state_layer.pos > 0.001 {
            let overlay_color = Rgba {
                a: self.state_layer.pos.clamp(0.0, 1.0),
                ..colors.state_layer
            };
            root = root.child(
                div()
                    .absolute()
                    .inset_0()
                    .rounded(px(active_corner))
                    .bg(overlay_color),
            );
        }

        // Inner content container
        root.child(div().relative().w_full().child(content))
    }

    /// Builds the internal content block (media banner and text slots).
    ///
    /// This method is useful when composing additional elements (such as real
    /// [`ButtonModel`](crate::button::ButtonModel) action buttons) inside the card
    /// before calling [`Self::render_with`].
    pub fn render_content(&self, pal: &M3Palette) -> gpui::Div {
        let metrics = self.metrics();
        let colors = self.resolve_colors(pal);

        let border_w = if colors.border.is_some() {
            metrics.outline_width
        } else {
            0.0
        };

        let active_corner = if self.corner.pos > 0.0 {
            self.corner.pos
        } else {
            self.target_corner(&metrics)
        };

        let inner_corner = (active_corner - border_w).max(0.0);

        let mut content = div().flex().flex_col().w_full();

        // Optional Media accent block
        if let Some(media_h) = self.media_height {
            let media_bg = if self.enabled {
                match self.variant {
                    CardVariant::Filled => pal.secondary_container,
                    CardVariant::Elevated | CardVariant::Outlined => pal.surface_container_highest,
                }
            } else {
                Rgba {
                    a: 0.24,
                    ..pal.surface_container_highest
                }
            };
            let media_fg = if self.enabled {
                pal.on_surface_variant
            } else {
                Rgba {
                    a: 0.38,
                    ..pal.on_surface_variant
                }
            };

            let mut media_block = div()
                .w_full()
                .h(px(media_h))
                .rounded_tl(px(inner_corner))
                .rounded_tr(px(inner_corner))
                .bg(media_bg)
                .flex()
                .items_center()
                .justify_center()
                .text_color(media_fg);

            if let Some(ref lbl) = self.media_label {
                media_block = media_block.child(
                    div()
                        .text_size(px(12.0))
                        .font_weight(FontWeight::MEDIUM)
                        .child(lbl.clone()),
                );
            }

            content = content.child(media_block);
        }

        // Text content block with standard padding
        let has_text = self.title.is_some() || self.subhead.is_some() || self.body.is_some();
        if has_text {
            let mut text_block = div()
                .flex()
                .flex_col()
                .px(px(metrics.padding_horizontal))
                .py(px(metrics.padding_vertical))
                .gap(px(metrics.content_spacing));

            // Header: Title & Subhead
            if self.title.is_some() || self.subhead.is_some() {
                let mut header = div().flex().flex_col().gap(px(metrics.header_spacing));

                if let Some(ref t) = self.title {
                    header = header.child(
                        div()
                            .text_size(px(metrics.title_size))
                            .font_weight(FontWeight::BOLD)
                            .line_height(px(metrics.title_size * 1.25))
                            .child(t.clone()),
                    );
                }

                if let Some(ref s) = self.subhead {
                    header = header.child(
                        div()
                            .text_size(px(metrics.subhead_size))
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(if self.enabled {
                                colors.secondary_content
                            } else {
                                colors.disabled_secondary_content
                            })
                            .line_height(px(metrics.subhead_size * 1.25))
                            .child(s.clone()),
                    );
                }

                text_block = text_block.child(header);
            }

            // Body
            if let Some(ref b) = self.body {
                text_block = text_block.child(
                    div()
                        .text_size(px(metrics.body_size))
                        .text_color(if self.enabled {
                            colors.secondary_content
                        } else {
                            colors.disabled_secondary_content
                        })
                        .line_height(px(metrics.body_size * 1.4))
                        .child(b.clone()),
                );
            }

            content = content.child(text_block);
        }

        content
    }

    /// Renders the card using its built-in slots (media, title, subhead, body).
    pub fn render(&self, pal: &M3Palette) -> Stateful<Div> {
        let content = self.render_content(pal);
        self.render_with(pal, content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::light_scheme;

    #[test]
    fn default_card_creation() {
        let card = CardModel::new(0);
        assert_eq!(card.variant, CardVariant::Filled);
        assert!(card.enabled);
        assert!(!card.interactive);
        assert_eq!(card.metrics().round_corner, 12.0);
        assert_eq!(card.metrics().pressed_corner, 8.0);
    }

    #[test]
    fn variant_colors_resolution() {
        let pal = light_scheme();

        let filled = CardColors::for_variant(CardVariant::Filled, &pal);
        assert_eq!(filled.container, pal.surface_container_highest);
        assert!(!filled.has_shadow);
        assert!(filled.border.is_none());

        let elevated = CardColors::for_variant(CardVariant::Elevated, &pal);
        assert_eq!(elevated.container, pal.surface);
        assert!(elevated.has_shadow);
        assert!(elevated.border.is_none());

        let outlined = CardColors::for_variant(CardVariant::Outlined, &pal);
        assert_eq!(outlined.container, pal.surface);
        assert!(!outlined.has_shadow);
        assert_eq!(outlined.border, Some(pal.outline_variant));
    }

    #[test]
    fn interactive_corner_morph_and_settle() {
        let mut card = CardModel::new(1)
            .interactive(true)
            .variant(CardVariant::Elevated);

        assert_eq!(card.target_corner(&card.metrics()), 12.0);

        // Press the card
        card.is_pressed = true;
        assert_eq!(card.target_corner(&card.metrics()), 8.0);
        assert_eq!(card.target_state_layer_alpha(), 0.12);

        // Advance physics until settled
        let mut steps = 0;
        while card.step(1.0 / 120.0) && steps < 500 {
            steps += 1;
        }

        assert!(steps > 0);
        assert!(steps < 300);
        assert!((card.corner.pos - 8.0).abs() < 0.1);
        assert!((card.state_layer.pos - 0.12).abs() < 0.01);
        assert!(!card.step(1.0 / 120.0), "Settled card must return false");

        // Release the card
        card.is_pressed = false;
        assert_eq!(card.target_corner(&card.metrics()), 12.0);
        assert_eq!(card.target_state_layer_alpha(), 0.0);

        while card.step(1.0 / 120.0) {}
        assert!((card.corner.pos - 12.0).abs() < 0.1);
        assert!(card.state_layer.pos < 0.01);
    }

    #[test]
    fn hover_state_layer() {
        let mut card = CardModel::new(2).interactive(true);
        card.is_hovered = true;
        assert_eq!(card.target_state_layer_alpha(), 0.08);

        card.is_hovered = false;
        assert_eq!(card.target_state_layer_alpha(), 0.0);
    }

    #[test]
    fn disabled_card_inactive() {
        let mut card = CardModel::new(3).interactive(true).enabled(false);
        card.is_pressed = true;
        card.is_hovered = true;
        // Disabled card should not morph corner or activate state layer
        assert_eq!(card.target_corner(&card.metrics()), 12.0);
        assert_eq!(card.target_state_layer_alpha(), 0.0);
    }

    #[test]
    fn card_container_color_override() {
        let pal = light_scheme();
        let card = CardModel::new(4)
            .variant(CardVariant::Filled)
            .container_color(pal.surface_container);
        let resolved = card.resolve_colors(&pal);
        assert_eq!(resolved.container, pal.surface_container);

        let sc_colors = CardColors::surface_container(&pal);
        assert_eq!(sc_colors.container, pal.surface_container);
    }
}
