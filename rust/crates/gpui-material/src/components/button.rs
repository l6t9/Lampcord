//! Material 3 Expressive Button component.
//!
//! Supports five sizes (ExtraSmall to ExtraLarge), five emphasis variants
//! (Filled, FilledTonal, Elevated, Outlined, Text), two shapes (Round, Square),
//! and spring corner morphing with state layer animations.

use gpui::{IntoElement, ParentElement, Rgba, SharedString, Styled, Window, div, prelude::*, px};

use crate::components::icon::M3Icon;
use crate::motion::{SpringParams, SpringState, spring_step};
use crate::theme::M3Palette;

/// Available button sizes matching Material 3 Expressive specifications.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ButtonSize {
    /// 32dp height, 16dp horizontal padding.
    ExtraSmall,
    /// 40dp height, 16dp horizontal padding.
    Small,
    /// 56dp height, 24dp horizontal padding.
    #[default]
    Medium,
    /// 96dp height, 48dp horizontal padding.
    Large,
    /// 136dp height, 64dp horizontal padding.
    ExtraLarge,
}

/// Visual emphasis variants matching Material 3 specifications.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ButtonVariant {
    /// High emphasis with primary container.
    #[default]
    Filled,
    /// Medium-high emphasis with secondary container.
    FilledTonal,
    /// Medium emphasis with surface container and shadow.
    Elevated,
    /// Medium-low emphasis with outline border and transparent container.
    Outlined,
    /// Low emphasis with no container or border.
    Text,
}

/// Container shape style.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ButtonShape {
    /// Full round (pill) when idle, morphing to smaller radius on press.
    #[default]
    Round,
    /// Square corners when idle, morphing to smaller radius on press.
    Square,
}

/// Layout metrics matching Material 3 Expressive specifications.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ButtonMetrics {
    /// Container height.
    pub container_height: f32,
    /// Leading edge padding.
    pub leading_space: f32,
    /// Trailing edge padding.
    pub trailing_space: f32,
    /// Icon size if an icon is displayed.
    pub icon_size: f32,
    /// Space between icon and label.
    pub icon_label_space: f32,
    /// Corner radius for round shape when idle (50% of container height).
    pub round_corner: f32,
    /// Corner radius for square shape when idle.
    pub square_corner: f32,
    /// Corner radius when pressed.
    pub pressed_corner: f32,
    /// Outline border width for Outlined variant.
    pub outlined_outline_width: f32,
    /// Label typography font size.
    pub font_size: f32,
}

impl ButtonMetrics {
    /// Returns the exact token metrics for the given button size.
    #[must_use]
    pub fn for_size(size: ButtonSize) -> Self {
        match size {
            ButtonSize::ExtraSmall => Self {
                container_height: 32.0,
                leading_space: 16.0,
                trailing_space: 16.0,
                icon_size: 20.0,
                icon_label_space: 8.0,
                round_corner: 16.0,
                square_corner: 12.0,
                pressed_corner: 8.0,
                outlined_outline_width: 1.0,
                font_size: 12.0,
            },
            ButtonSize::Small => Self {
                container_height: 40.0,
                leading_space: 16.0,
                trailing_space: 16.0,
                icon_size: 20.0,
                icon_label_space: 8.0,
                round_corner: 20.0,
                square_corner: 12.0,
                pressed_corner: 8.0,
                outlined_outline_width: 1.0,
                font_size: 14.0,
            },
            ButtonSize::Medium => Self {
                container_height: 56.0,
                leading_space: 24.0,
                trailing_space: 24.0,
                icon_size: 24.0,
                icon_label_space: 8.0,
                round_corner: 28.0,
                square_corner: 16.0,
                pressed_corner: 12.0,
                outlined_outline_width: 1.0,
                font_size: 16.0,
            },
            ButtonSize::Large => Self {
                container_height: 96.0,
                leading_space: 48.0,
                trailing_space: 48.0,
                icon_size: 32.0,
                icon_label_space: 12.0,
                round_corner: 48.0,
                square_corner: 28.0,
                pressed_corner: 16.0,
                outlined_outline_width: 2.0,
                font_size: 20.0,
            },
            ButtonSize::ExtraLarge => Self {
                container_height: 136.0,
                leading_space: 64.0,
                trailing_space: 64.0,
                icon_size: 40.0,
                icon_label_space: 16.0,
                round_corner: 68.0,
                square_corner: 28.0,
                pressed_corner: 16.0,
                outlined_outline_width: 3.0,
                font_size: 24.0,
            },
        }
    }
}

/// Color scheme for a button.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ButtonColors {
    /// Button container background color.
    pub container: Rgba,
    /// Label and icon foreground content color.
    pub content: Rgba,
    /// State layer overlay color for hover and press.
    pub state_layer: Rgba,
    /// Optional outline border color.
    pub border: Option<Rgba>,
    /// Whether an elevation drop shadow is enabled.
    pub has_shadow: bool,
    /// Container background when disabled.
    pub disabled_container: Rgba,
    /// Content color when disabled.
    pub disabled_content: Rgba,
    /// Border color when disabled.
    pub disabled_border: Option<Rgba>,
}

impl ButtonColors {
    /// Filled button (high emphasis).
    #[must_use]
    pub fn filled(p: &M3Palette) -> Self {
        Self {
            container: p.primary,
            content: p.on_primary,
            state_layer: p.on_primary,
            border: None,
            has_shadow: false,
            disabled_container: Rgba {
                a: 0.12,
                ..p.on_surface
            },
            disabled_content: Rgba {
                a: 0.38,
                ..p.on_surface
            },
            disabled_border: None,
        }
    }

    /// Filled tonal button (medium-high emphasis).
    #[must_use]
    pub fn filled_tonal(p: &M3Palette) -> Self {
        Self {
            container: p.secondary_container,
            content: p.on_secondary_container,
            state_layer: p.on_secondary_container,
            border: None,
            has_shadow: false,
            disabled_container: Rgba {
                a: 0.12,
                ..p.on_surface
            },
            disabled_content: Rgba {
                a: 0.38,
                ..p.on_surface
            },
            disabled_border: None,
        }
    }

    /// Elevated button (medium emphasis with shadow).
    #[must_use]
    pub fn elevated(p: &M3Palette) -> Self {
        Self {
            container: p.surface_container_highest,
            content: p.primary,
            state_layer: p.primary,
            border: None,
            has_shadow: true,
            disabled_container: Rgba {
                a: 0.12,
                ..p.on_surface
            },
            disabled_content: Rgba {
                a: 0.38,
                ..p.on_surface
            },
            disabled_border: None,
        }
    }

    /// Outlined button (medium-low emphasis with outline border).
    #[must_use]
    pub fn outlined(p: &M3Palette) -> Self {
        Self {
            container: Rgba {
                a: 0.0,
                ..p.surface
            },
            content: p.primary,
            state_layer: p.primary,
            border: Some(p.outline),
            has_shadow: false,
            disabled_container: Rgba {
                a: 0.0,
                ..p.surface
            },
            disabled_content: Rgba {
                a: 0.38,
                ..p.on_surface
            },
            disabled_border: Some(Rgba {
                a: 0.12,
                ..p.on_surface
            }),
        }
    }

    /// Text button (low emphasis without container or border).
    #[must_use]
    pub fn text(p: &M3Palette) -> Self {
        Self {
            container: Rgba {
                a: 0.0,
                ..p.surface
            },
            content: p.primary,
            state_layer: p.primary,
            border: None,
            has_shadow: false,
            disabled_container: Rgba {
                a: 0.0,
                ..p.surface
            },
            disabled_content: Rgba {
                a: 0.38,
                ..p.on_surface
            },
            disabled_border: None,
        }
    }

    /// Resolves colors for the given variant.
    #[must_use]
    pub fn for_variant(variant: ButtonVariant, p: &M3Palette) -> Self {
        match variant {
            ButtonVariant::Filled => Self::filled(p),
            ButtonVariant::FilledTonal => Self::filled_tonal(p),
            ButtonVariant::Elevated => Self::elevated(p),
            ButtonVariant::Outlined => Self::outlined(p),
            ButtonVariant::Text => Self::text(p),
        }
    }
}

/// Interactive model for an M3 Expressive Button.
///
/// # Examples
///
/// ```rust,no_run
/// use gpui_material::button::{ButtonModel, ButtonSize, ButtonVariant};
/// use gpui_material::theme::light_scheme;
///
/// let mut btn = ButtonModel::new(0, "Save")
///     .size(ButtonSize::Medium)
///     .variant(ButtonVariant::Filled);
///
/// // Advance spring physics on animation frames:
/// // btn.advance_springs(window, dt);
///
/// // Render with current palette:
/// let palette = light_scheme();
/// let element = btn.render(&palette);
/// ```
#[derive(Clone, Debug)]
pub struct ButtonModel {
    /// Button label text.
    pub label: SharedString,
    /// Button size token.
    pub size: ButtonSize,
    /// Button emphasis variant.
    pub variant: ButtonVariant,
    /// Button shape style (Round or Square).
    pub shape: ButtonShape,
    /// Whether the button is enabled for interaction.
    pub enabled: bool,
    /// Whether the pointer is currently hovering over the button.
    pub is_hovered: bool,
    /// Whether the pointer is currently pressing down on the button.
    pub is_pressed: bool,
    /// Optional unique element tag for stable spring identity.
    pub id: usize,
    /// Spring state for corner morphing.
    pub corner: SpringState,
    /// Spring state for state layer alpha (hover and press overlay).
    pub state_layer: SpringState,
    /// Optional custom color scheme overriding standard variant tokens.
    pub custom_colors: Option<ButtonColors>,
    /// Optional custom layout metrics overriding standard size tokens.
    pub custom_metrics: Option<ButtonMetrics>,
    /// Optional custom spring physics overriding default expressive spring.
    pub custom_spring: Option<SpringParams>,
    /// Optional leading M3 vector icon.
    pub icon: Option<M3Icon>,
    /// Optional trailing M3 vector icon.
    pub trailing_icon: Option<M3Icon>,
}

impl ButtonModel {
    /// Creates a new button model.
    #[must_use]
    pub fn new(id: usize, label: impl Into<SharedString>) -> Self {
        let metrics = ButtonMetrics::for_size(ButtonSize::Medium);
        Self {
            label: label.into(),
            size: ButtonSize::Medium,
            variant: ButtonVariant::Filled,
            shape: ButtonShape::Round,
            enabled: true,
            is_hovered: false,
            is_pressed: false,
            id,
            corner: SpringState {
                pos: metrics.round_corner,
                vel: 0.0,
            },
            state_layer: SpringState::default(),
            custom_colors: None,
            custom_metrics: None,
            custom_spring: None,
            icon: None,
            trailing_icon: None,
        }
    }

    /// Sets an optional leading M3 vector icon for this button.
    #[must_use]
    pub fn icon(mut self, icon: M3Icon) -> Self {
        self.icon = Some(icon);
        self
    }

    /// Sets an optional trailing M3 vector icon for this button.
    #[must_use]
    pub fn trailing_icon(mut self, icon: M3Icon) -> Self {
        self.trailing_icon = Some(icon);
        self
    }

    /// Sets the button size.
    #[must_use]
    pub fn size(mut self, size: ButtonSize) -> Self {
        self.size = size;
        let metrics = self
            .custom_metrics
            .unwrap_or_else(|| ButtonMetrics::for_size(size));
        self.corner.pos = self.target_corner(&metrics);
        self
    }

    /// Sets the emphasis variant.
    #[must_use]
    pub fn variant(mut self, variant: ButtonVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Sets the shape style.
    #[must_use]
    pub fn shape(mut self, shape: ButtonShape) -> Self {
        self.shape = shape;
        let metrics = self
            .custom_metrics
            .unwrap_or_else(|| ButtonMetrics::for_size(self.size));
        self.corner.pos = self.target_corner(&metrics);
        self
    }

    /// Sets the enabled state.
    #[must_use]
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Sets custom button colors, overriding theme variant defaults.
    #[must_use]
    pub fn colors(mut self, colors: ButtonColors) -> Self {
        self.custom_colors = Some(colors);
        self
    }

    /// Sets custom layout metrics, overriding size token defaults.
    #[must_use]
    pub fn metrics(mut self, metrics: ButtonMetrics) -> Self {
        self.corner.pos = self.target_corner(&metrics);
        self.custom_metrics = Some(metrics);
        self
    }

    /// Sets custom spring physics, overriding the default expressive spring.
    #[must_use]
    pub fn spring(mut self, spring: SpringParams) -> Self {
        self.custom_spring = Some(spring);
        self
    }

    /// Calculates the target corner radius based on shape and interaction state.
    #[must_use]
    pub fn target_corner(&self, metrics: &ButtonMetrics) -> f32 {
        if !self.enabled || !self.is_pressed {
            match self.shape {
                ButtonShape::Round => metrics.round_corner,
                ButtonShape::Square => metrics.square_corner,
            }
        } else {
            metrics.pressed_corner
        }
    }

    /// Calculates the target state layer alpha.
    #[must_use]
    pub fn target_state_layer_alpha(&self) -> f32 {
        if !self.enabled {
            0.0
        } else if self.is_pressed {
            0.10
        } else if self.is_hovered {
            0.08
        } else {
            0.0
        }
    }

    /// Advances spring physics by dt seconds.
    /// Returns `true` if any animation is ongoing and another frame is needed.
    pub fn step(&mut self, dt: f32) -> bool {
        let metrics = self
            .custom_metrics
            .unwrap_or_else(|| ButtonMetrics::for_size(self.size));
        let corner_target = self.target_corner(&metrics);
        let layer_target = self.target_state_layer_alpha();

        let spring = self.custom_spring.unwrap_or_else(SpringParams::expressive);

        self.corner = spring_step(spring, self.corner, corner_target, dt);
        self.state_layer = spring_step(spring, self.state_layer, layer_target, dt);

        let settled = self.corner.is_settled(corner_target, 0.05, 0.1)
            && self.state_layer.is_settled(layer_target, 0.002, 0.005);

        !settled
    }

    /// Advance spring animations by dt seconds, requesting an animation frame if unsettled.
    pub fn advance_springs(&mut self, window: &mut Window, dt: f32) {
        if self.step(dt) {
            window.request_animation_frame();
        }
    }

    /// Renders the button as a GPUI element with animated state layer and corner morphing.
    pub fn render(&self, pal: &M3Palette) -> impl IntoElement {
        let metrics = self
            .custom_metrics
            .unwrap_or_else(|| ButtonMetrics::for_size(self.size));
        let colors = self
            .custom_colors
            .unwrap_or_else(|| ButtonColors::for_variant(self.variant, pal));

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
            metrics.outlined_outline_width
        } else {
            0.0
        };

        let mut root = div()
            .id(("m3-btn", self.id))
            .relative()
            .h(px(metrics.container_height))
            .px(px(metrics.leading_space))
            .flex()
            .items_center()
            .justify_center()
            .rounded(px(active_corner))
            .cursor_pointer()
            .bg(bg)
            .text_color(fg)
            .text_size(px(metrics.font_size))
            .font_weight(gpui::FontWeight::MEDIUM)
            .when(colors.has_shadow && self.enabled, |d| d.shadow_sm())
            .when(border_w > 0.0, |d| {
                if let Some(bc) = border {
                    d.border(px(border_w)).border_color(bc)
                } else {
                    d
                }
            });

        // State layer overlay: smoothly fades in on hover/press and fades out on hover-off/release.
        let layer_alpha = self.state_layer.pos.clamp(0.0, 1.0);
        if layer_alpha > 0.001 {
            let state_layer = div()
                .id(("m3-btn-layer", self.id))
                .absolute()
                .inset_0()
                .rounded(px(active_corner))
                .bg(colors.state_layer)
                .opacity(layer_alpha);

            root = root.child(state_layer);
        }

        // Label and optional leading/trailing icon content
        let has_leading = self.icon.is_some();
        let has_trailing = self.trailing_icon.is_some();

        if has_leading || has_trailing {
            let mut content = div()
                .relative()
                .flex()
                .items_center()
                .gap(px(metrics.icon_label_space));

            if let Some(leading) = self.icon {
                content = content.child(leading.element(metrics.icon_size, fg));
            }
            if !self.label.is_empty() {
                content = content.child(div().child(self.label.clone()));
            }
            if let Some(trailing) = self.trailing_icon {
                content = content.child(trailing.element(metrics.icon_size, fg));
            }

            root.child(content)
        } else {
            root.child(div().relative().child(self.label.clone()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn metrics_match_m3e_tokens() {
        let xs = ButtonMetrics::for_size(ButtonSize::ExtraSmall);
        assert_eq!(xs.container_height, 32.0);
        assert_eq!(xs.round_corner, 16.0);
        assert_eq!(xs.square_corner, 12.0);
        assert_eq!(xs.pressed_corner, 8.0);

        let s = ButtonMetrics::for_size(ButtonSize::Small);
        assert_eq!(s.container_height, 40.0);
        assert_eq!(s.round_corner, 20.0);
        assert_eq!(s.square_corner, 12.0);
        assert_eq!(s.pressed_corner, 8.0);

        let m = ButtonMetrics::for_size(ButtonSize::Medium);
        assert_eq!(m.container_height, 56.0);
        assert_eq!(m.round_corner, 28.0);
        assert_eq!(m.square_corner, 16.0);
        assert_eq!(m.pressed_corner, 12.0);

        let l = ButtonMetrics::for_size(ButtonSize::Large);
        assert_eq!(l.container_height, 96.0);
        assert_eq!(l.round_corner, 48.0);
        assert_eq!(l.square_corner, 28.0);
        assert_eq!(l.pressed_corner, 16.0);

        let xl = ButtonMetrics::for_size(ButtonSize::ExtraLarge);
        assert_eq!(xl.container_height, 136.0);
        assert_eq!(xl.round_corner, 68.0);
        assert_eq!(xl.square_corner, 28.0);
        assert_eq!(xl.pressed_corner, 16.0);
    }

    #[test]
    fn hover_state_layer_spring_advances_and_decays() {
        let mut btn = ButtonModel::new(0, "Test");
        assert_eq!(btn.state_layer.pos, 0.0);

        // Hover enter
        btn.is_hovered = true;
        assert_eq!(btn.target_state_layer_alpha(), 0.08);

        // Step physics forward
        for _ in 0..60 {
            btn.step(1.0 / 60.0);
        }
        assert!((btn.state_layer.pos - 0.08).abs() < 0.005);

        // Hover exit
        btn.is_hovered = false;
        assert_eq!(btn.target_state_layer_alpha(), 0.0);

        // Step once: should be decaying, not instantly 0.0
        btn.step(1.0 / 60.0);
        assert!(btn.state_layer.pos > 0.01);

        // Step until settled
        for _ in 0..120 {
            btn.step(1.0 / 60.0);
        }
        assert!(btn.state_layer.pos.abs() < 0.002);
    }

    #[test]
    fn press_corner_morph_advances_and_restores() {
        let mut btn = ButtonModel::new(0, "Test").size(ButtonSize::Medium);
        assert_eq!(btn.corner.pos, 28.0);

        btn.is_pressed = true;
        for _ in 0..60 {
            btn.step(1.0 / 60.0);
        }
        assert!((btn.corner.pos - 12.0).abs() < 0.1);

        btn.is_pressed = false;
        for _ in 0..60 {
            btn.step(1.0 / 60.0);
        }
        assert!((btn.corner.pos - 28.0).abs() < 0.1);
    }
}
