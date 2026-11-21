//! Material 3 Expressive Split Button component math, tokens, and models.
//!
//! A Split Button consists of a leading button performing a primary action
//! and a trailing button performing a contextual secondary or toggle action (e.g. dropdown menu).
//! Inner corners dynamically morph via damped spring physics on hover, press, and toggle.

use gpui::{Bounds, PathBuilder, Pixels, Rgba, SharedString, Window, point, px};

use crate::motion::{SpringParams, SpringState, spring_step};
use crate::theme::{M3Palette, light_scheme};

/// Split button size categories matching Material 3 Expressive tokens.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum SplitButtonSize {
    /// 32dp container height, compact density.
    ExtraSmall,
    /// 40dp container height, standard default.
    #[default]
    Small,
    /// 56dp container height, prominent actions.
    Medium,
    /// 96dp container height, large expressive hero layout.
    Large,
    /// 136dp container height, extra-large expressive showcase.
    ExtraLarge,
}

/// Material 3 Expressive emphasis styles for SplitButton.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum SplitButtonVariant {
    /// High-emphasis solid fill using `Primary`.
    #[default]
    Filled,
    /// Medium-emphasis container using `SecondaryContainer`.
    Tonal,
    /// Surface container with subtle drop shadow.
    Elevated,
    /// Transparent container with outline border.
    Outlined,
}

/// Leading icon visual types for SplitButton.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum SplitIconKind {
    /// No icon, text only.
    #[default]
    None,
    /// Plus / Add icon (+).
    Add,
    /// Checkmark icon (✓).
    Check,
    /// Share / export node icon.
    Share,
    /// Edit / pencil icon.
    Edit,
    /// Bookmark / ribbon icon.
    Bookmark,
}

/// Geometry and spacing metrics for Material 3 Expressive split buttons.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SplitButtonMetrics {
    /// Container height in dp (32, 40, 56, 96, or 136).
    pub container_height: f32,
    /// Space between leading and trailing buttons, 2dp in all M3E tokens.
    pub between_space: f32,
    /// Inner corner radius when idle (4dp for XS/S/M, 8dp for L, 12dp for XL).
    pub inner_corner: f32,
    /// Inner corner radius when pressed/hovered (8dp for XS, 12dp for S/M, 20dp for L/XL).
    pub inner_corner_pressed: f32,
    /// Inner corner radius when hovered.
    pub inner_corner_hovered: f32,
    /// Outer corner radius (half of container height = CircleShape / 50%).
    pub outer_corner: f32,
    /// Leading button start padding.
    pub leading_padding_start: f32,
    /// Leading button end padding.
    pub leading_padding_end: f32,
    /// Trailing button start padding.
    pub trailing_padding_start: f32,
    /// Trailing button end padding.
    pub trailing_padding_end: f32,
    /// Trailing icon box dimension.
    pub trailing_icon_size: f32,
    /// Suggested font size in px.
    pub font_size: f32,
    /// Leading button minimum width in dp.
    pub leading_min_width: f32,
    /// Suggested dropdown menu min width in px.
    pub menu_min_width: f32,
    /// Dropdown menu item vertical padding in px.
    pub menu_item_padding_y: f32,
    /// Dropdown menu item horizontal padding in px.
    pub menu_item_padding_x: f32,
    /// Dropdown menu item font size in pt/px.
    pub menu_font_size: f32,
    /// Dropdown menu container corner radius in px.
    pub menu_corner_radius: f32,
}

impl SplitButtonMetrics {
    /// Resolve exact metrics for the given M3E size category.
    #[must_use]
    pub fn for_size(size: SplitButtonSize) -> Self {
        match size {
            SplitButtonSize::ExtraSmall => Self {
                container_height: 32.0,
                between_space: 2.0,
                inner_corner: 4.0,
                inner_corner_pressed: 8.0,
                inner_corner_hovered: 8.0,
                outer_corner: 16.0,
                leading_padding_start: 12.0,
                leading_padding_end: 10.0,
                trailing_padding_start: 13.0,
                trailing_padding_end: 13.0,
                trailing_icon_size: 16.0,
                font_size: 12.0,
                leading_min_width: 36.0,
                menu_min_width: 110.0,
                menu_item_padding_y: 4.0,
                menu_item_padding_x: 8.0,
                menu_font_size: 11.0,
                menu_corner_radius: 8.0,
            },
            SplitButtonSize::Small => Self {
                container_height: 40.0,
                between_space: 2.0,
                inner_corner: 4.0,
                inner_corner_pressed: 12.0,
                inner_corner_hovered: 12.0,
                outer_corner: 20.0,
                leading_padding_start: 16.0,
                leading_padding_end: 12.0,
                trailing_padding_start: 13.0,
                trailing_padding_end: 13.0,
                trailing_icon_size: 18.0,
                font_size: 14.0,
                leading_min_width: 48.0,
                menu_min_width: 140.0,
                menu_item_padding_y: 6.0,
                menu_item_padding_x: 10.0,
                menu_font_size: 13.0,
                menu_corner_radius: 12.0,
            },
            SplitButtonSize::Medium => Self {
                container_height: 56.0,
                between_space: 2.0,
                inner_corner: 4.0,
                inner_corner_pressed: 12.0,
                inner_corner_hovered: 12.0,
                outer_corner: 28.0,
                leading_padding_start: 24.0,
                leading_padding_end: 24.0,
                trailing_padding_start: 15.0,
                trailing_padding_end: 15.0,
                trailing_icon_size: 22.0,
                font_size: 16.0,
                leading_min_width: 64.0,
                menu_min_width: 180.0,
                menu_item_padding_y: 8.0,
                menu_item_padding_x: 14.0,
                menu_font_size: 15.0,
                menu_corner_radius: 16.0,
            },
            SplitButtonSize::Large => Self {
                container_height: 96.0,
                between_space: 2.0,
                inner_corner: 8.0,
                inner_corner_pressed: 20.0,
                inner_corner_hovered: 20.0,
                outer_corner: 48.0,
                leading_padding_start: 48.0,
                leading_padding_end: 48.0,
                trailing_padding_start: 29.0,
                trailing_padding_end: 29.0,
                trailing_icon_size: 32.0,
                font_size: 20.0,
                leading_min_width: 104.0,
                menu_min_width: 220.0,
                menu_item_padding_y: 12.0,
                menu_item_padding_x: 18.0,
                menu_font_size: 18.0,
                menu_corner_radius: 20.0,
            },
            SplitButtonSize::ExtraLarge => Self {
                container_height: 136.0,
                between_space: 2.0,
                inner_corner: 12.0,
                inner_corner_pressed: 20.0,
                inner_corner_hovered: 20.0,
                outer_corner: 68.0,
                leading_padding_start: 64.0,
                leading_padding_end: 64.0,
                trailing_padding_start: 43.0,
                trailing_padding_end: 43.0,
                trailing_icon_size: 40.0,
                font_size: 24.0,
                leading_min_width: 144.0,
                menu_min_width: 260.0,
                menu_item_padding_y: 16.0,
                menu_item_padding_x: 24.0,
                menu_font_size: 22.0,
                menu_corner_radius: 24.0,
            },
        }
    }
}

impl Default for SplitButtonMetrics {
    fn default() -> Self {
        Self::for_size(SplitButtonSize::Small)
    }
}

/// Colors resolved from M3 palette tokens for a SplitButton variant.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SplitButtonColors {
    /// Container background color.
    pub container: Rgba,
    /// Content color for text and icons.
    pub content: Rgba,
    /// State layer overlay color (hover/press).
    pub state_layer: Rgba,
    /// Optional outline border color.
    pub border: Option<Rgba>,
    /// Whether this variant casts a drop shadow.
    pub has_shadow: bool,
    /// Disabled container color.
    pub disabled_container: Rgba,
    /// Disabled content color.
    pub disabled_content: Rgba,
    /// Disabled outline border color.
    pub disabled_border: Option<Rgba>,
}

impl SplitButtonColors {
    /// Filled split button color scheme.
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

    /// Tonal split button color scheme.
    #[must_use]
    pub fn tonal(p: &M3Palette) -> Self {
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

    /// Elevated split button color scheme.
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

    /// Outlined split button color scheme.
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

    /// Resolve colors for any variant from an M3 palette.
    #[must_use]
    pub fn for_variant(variant: SplitButtonVariant, p: &M3Palette) -> Self {
        match variant {
            SplitButtonVariant::Filled => Self::filled(p),
            SplitButtonVariant::Tonal => Self::tonal(p),
            SplitButtonVariant::Elevated => Self::elevated(p),
            SplitButtonVariant::Outlined => Self::outlined(p),
        }
    }
}

impl Default for SplitButtonColors {
    fn default() -> Self {
        Self::filled(&light_scheme())
    }
}

/// Draw a vector chevron icon that animates from down (0.0) to up (1.0).
pub fn draw_chevron_icon(window: &mut Window, bounds: Bounds<Pixels>, progress: f32, color: Rgba) {
    let w = f32::from(bounds.size.width);
    let h = f32::from(bounds.size.height);
    let stroke_width = (w * 0.14).clamp(1.5, 3.2);

    let ox = bounds.left();
    let oy = bounds.top();

    let left_x = ox + px(w * 0.22);
    let right_x = ox + px(w * 0.78);
    let mid_x = ox + px(w * 0.50);

    // When progress = 0.0 (down): base is higher, apex is lower.
    // When progress = 1.0 (up): base is lower, apex is higher.
    let base_y = oy + px(h * (0.38 + 0.24 * progress));
    let apex_y = oy + px(h * (0.62 - 0.24 * progress));

    let mut builder = PathBuilder::stroke(px(stroke_width));
    builder.move_to(point(left_x, base_y));
    builder.line_to(point(mid_x, apex_y));
    builder.line_to(point(right_x, base_y));

    if let Ok(path) = builder.build() {
        window.paint_path(path, color);
    }
}

/// Draw leading vector icon on GPUI canvas.
pub fn draw_split_icon(
    window: &mut Window,
    bounds: Bounds<Pixels>,
    kind: SplitIconKind,
    color: Rgba,
) {
    let w = f32::from(bounds.size.width);
    let h = f32::from(bounds.size.height);
    let ox = bounds.left();
    let oy = bounds.top();
    let stroke_width = (w * 0.12).clamp(1.6, 3.0);

    let mut builder = PathBuilder::stroke(px(stroke_width));

    match kind {
        SplitIconKind::None => return,
        SplitIconKind::Add => {
            // Horizontal line
            builder.move_to(point(ox + px(w * 0.20), oy + px(h * 0.50)));
            builder.line_to(point(ox + px(w * 0.80), oy + px(h * 0.50)));
            // Vertical line
            builder.move_to(point(ox + px(w * 0.50), oy + px(h * 0.20)));
            builder.line_to(point(ox + px(w * 0.50), oy + px(h * 0.80)));
        }
        SplitIconKind::Check => {
            builder.move_to(point(ox + px(w * 0.18), oy + px(h * 0.52)));
            builder.line_to(point(ox + px(w * 0.42), oy + px(h * 0.76)));
            builder.line_to(point(ox + px(w * 0.82), oy + px(h * 0.26)));
        }
        SplitIconKind::Share => {
            // Node triangle / connection branches
            builder.move_to(point(ox + px(w * 0.72), oy + px(h * 0.28)));
            builder.line_to(point(ox + px(w * 0.32), oy + px(h * 0.50)));
            builder.line_to(point(ox + px(w * 0.72), oy + px(h * 0.72)));
        }
        SplitIconKind::Edit => {
            // Diagonal pencil body & tip
            builder.move_to(point(ox + px(w * 0.22), oy + px(h * 0.78)));
            builder.line_to(point(ox + px(w * 0.74), oy + px(h * 0.26)));
            builder.move_to(point(ox + px(w * 0.22), oy + px(h * 0.78)));
            builder.line_to(point(ox + px(w * 0.44), oy + px(h * 0.78)));
            builder.move_to(point(ox + px(w * 0.22), oy + px(h * 0.78)));
            builder.line_to(point(ox + px(w * 0.22), oy + px(h * 0.56)));
        }
        SplitIconKind::Bookmark => {
            builder.move_to(point(ox + px(w * 0.26), oy + px(h * 0.18)));
            builder.line_to(point(ox + px(w * 0.74), oy + px(h * 0.18)));
            builder.line_to(point(ox + px(w * 0.74), oy + px(h * 0.82)));
            builder.line_to(point(ox + px(w * 0.50), oy + px(h * 0.62)));
            builder.line_to(point(ox + px(w * 0.26), oy + px(h * 0.82)));
            builder.line_to(point(ox + px(w * 0.26), oy + px(h * 0.18)));
        }
    }

    if let Ok(path) = builder.build() {
        window.paint_path(path, color);
    }
}

/// State and interaction model for a Material 3 Expressive Split Button.
///
/// # Examples
///
/// ```rust,no_run
/// use gpui_material::split_button::{SplitButtonModel, SplitButtonSize, SplitButtonVariant};
///
/// let mut btn = SplitButtonModel::new("Save")
///     .size(SplitButtonSize::Medium)
///     .variant(SplitButtonVariant::Filled)
///     .menu_items(["Save & Publish", "Save Draft", "Export"]);
/// ```
#[derive(Clone, Debug)]
pub struct SplitButtonModel {
    /// Action label text for the leading button.
    pub label: SharedString,
    /// Leading icon kind.
    pub icon: SplitIconKind,
    /// Size specification.
    pub size: SplitButtonSize,
    /// Visual style variant.
    pub variant: SplitButtonVariant,
    /// Whether the button responds to interaction.
    pub enabled: bool,
    /// Whether the trailing button is toggled open (checked state for dropdown).
    pub checked: bool,
    /// Whether the leading button is actively pressed.
    pub is_leading_pressed: bool,
    /// Whether the leading button is hovered.
    pub is_leading_hovered: bool,
    /// Whether the trailing button is actively pressed.
    pub is_trailing_pressed: bool,
    /// Whether the trailing button is hovered.
    pub is_trailing_hovered: bool,
    /// Spring state for leading inner corner radius morphing.
    pub leading_corner: SpringState,
    /// Spring state for trailing inner corner radius morphing.
    pub trailing_corner: SpringState,
    /// Spring state for chevron rotation / morph (0.0 to 1.0).
    pub chevron_progress: SpringState,
    /// Spring state for leading state layer opacity.
    pub leading_state_layer: SpringState,
    /// Spring state for trailing state layer opacity.
    pub trailing_state_layer: SpringState,
    /// Spring state for dropdown menu expansion and opacity animation (0.0 to 1.0).
    pub menu_progress: SpringState,
    /// Spring state for leading button width (smooth horizontal expansion / shrinking).
    pub leading_width: SpringState,
    /// Timestamp when dropdown was last closed via outside event (used to prevent immediate reopening).
    pub closed_at: Option<std::time::Instant>,
    /// Optional dropdown menu item options.
    pub menu_items: Vec<SharedString>,
    /// Currently selected option index, if any.
    pub selected_index: Option<usize>,
}

impl SplitButtonModel {
    /// Calculate target natural width of the leading button based on label text, icon, and metrics.
    #[must_use]
    pub fn target_leading_width(&self) -> f32 {
        let metrics = self.metrics();
        let mut w = metrics.leading_padding_start + metrics.leading_padding_end;
        if self.icon != SplitIconKind::None {
            let icon_sz = (metrics.container_height * 0.38).clamp(14.0, 24.0);
            w += icon_sz + 8.0;
        }
        let char_count = self.label.chars().count();
        let glyph_w = metrics.font_size * 0.60;
        w += char_count as f32 * glyph_w + 4.0;
        w.max(metrics.leading_min_width)
    }

    /// Calculate minimum content width required by the menu items (text + padding + checkmark).
    #[must_use]
    pub fn menu_content_width(&self) -> f32 {
        let metrics = self.metrics();
        let mut max_w: f32 = 0.0;
        for item in &self.menu_items {
            let char_count = item.chars().count();
            let text_w = char_count as f32 * (metrics.menu_font_size * 0.60);
            let item_w = text_w + metrics.menu_item_padding_x * 2.0 + 28.0;
            if item_w > max_w {
                max_w = item_w;
            }
        }
        max_w
    }

    /// Calculate the total current width of the entire split button (leading + space + trailing).
    #[must_use]
    pub fn total_width(&self) -> f32 {
        let metrics = self.metrics();
        let trailing_w = (metrics.trailing_padding_start
            + metrics.trailing_icon_size
            + metrics.trailing_padding_end)
            .max(metrics.container_height * 0.75);
        self.leading_width.pos + metrics.between_space + trailing_w
    }

    /// Create a new Split Button model with default small size and filled variant.
    #[must_use]
    pub fn new(label: impl Into<SharedString>) -> Self {
        let label_str: SharedString = label.into();
        let metrics = SplitButtonMetrics::for_size(SplitButtonSize::Small);
        let mut model = Self {
            label: label_str,
            icon: SplitIconKind::None,
            size: SplitButtonSize::Small,
            variant: SplitButtonVariant::Filled,
            enabled: true,
            checked: false,
            is_leading_pressed: false,
            is_leading_hovered: false,
            is_trailing_pressed: false,
            is_trailing_hovered: false,
            leading_corner: SpringState {
                pos: metrics.inner_corner,
                vel: 0.0,
            },
            trailing_corner: SpringState {
                pos: metrics.inner_corner,
                vel: 0.0,
            },
            chevron_progress: SpringState::default(),
            leading_state_layer: SpringState::default(),
            trailing_state_layer: SpringState::default(),
            menu_progress: SpringState::default(),
            leading_width: SpringState::default(),
            closed_at: None,
            menu_items: Vec::new(),
            selected_index: None,
        };
        model.leading_width.pos = model.target_leading_width();
        model
    }

    /// Set the button size.
    #[must_use]
    pub fn size(mut self, size: SplitButtonSize) -> Self {
        self.size = size;
        let metrics = SplitButtonMetrics::for_size(size);
        self.leading_corner.pos = metrics.inner_corner;
        self.trailing_corner.pos = metrics.inner_corner;
        self.leading_width.pos = self.target_leading_width();
        self
    }

    /// Set the visual style variant.
    #[must_use]
    pub fn variant(mut self, variant: SplitButtonVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Set the leading icon.
    #[must_use]
    pub fn icon(mut self, icon: SplitIconKind) -> Self {
        self.icon = icon;
        self.leading_width.pos = self.target_leading_width();
        self
    }

    /// Set enabled / disabled state.
    #[must_use]
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Set dropdown menu options.
    #[must_use]
    pub fn menu_items(mut self, items: impl IntoIterator<Item = impl Into<SharedString>>) -> Self {
        self.menu_items = items.into_iter().map(Into::into).collect();
        self
    }

    /// Set the initially selected option index.
    #[must_use]
    pub fn selected_index(mut self, index: Option<usize>) -> Self {
        self.selected_index = index;
        self
    }

    /// Get current metrics for this button's size.
    #[inline]
    #[must_use]
    pub fn metrics(&self) -> SplitButtonMetrics {
        SplitButtonMetrics::for_size(self.size)
    }

    /// Advance spring physics for corner morphs, chevron, and state layers.
    /// Returns `true` if animation is ongoing and another frame is needed.
    pub fn step(&mut self, dt: f32) -> bool {
        let metrics = self.metrics();
        let spring = SpringParams::expressive();

        // 1. Target for leading inner corner:
        // When trailing button is checked (dropdown open), the leading inner corners morph
        // to `inner_corner_hovered` (e.g. 12px) so it doesn't stay flat/sharp while the trailing is circular.
        let leading_target = if self.is_leading_pressed {
            metrics.inner_corner_pressed
        } else if self.is_leading_hovered || self.checked {
            metrics.inner_corner_hovered
        } else {
            metrics.inner_corner
        };
        self.leading_corner = spring_step(spring, self.leading_corner, leading_target, dt);

        // 2. Target for trailing inner corner:
        // When checked, it morphs all the way to `outer_corner` (CircleShape / full round)!
        let trailing_target = if self.checked {
            metrics.outer_corner
        } else if self.is_trailing_pressed {
            metrics.inner_corner_pressed
        } else if self.is_trailing_hovered {
            metrics.inner_corner_hovered
        } else {
            metrics.inner_corner
        };
        self.trailing_corner = spring_step(spring, self.trailing_corner, trailing_target, dt);

        // 3. Target for chevron rotation: 1.0 when checked, 0.0 when idle.
        let chevron_target = if self.checked { 1.0 } else { 0.0 };
        self.chevron_progress = spring_step(spring, self.chevron_progress, chevron_target, dt);

        // 4. Target for leading state layer:
        let leading_layer_target = if !self.enabled {
            0.0
        } else if self.is_leading_pressed {
            0.10
        } else if self.is_leading_hovered {
            0.08
        } else {
            0.0
        };
        self.leading_state_layer =
            spring_step(spring, self.leading_state_layer, leading_layer_target, dt);

        // 5. Target for trailing state layer:
        let trailing_layer_target = if !self.enabled {
            0.0
        } else if self.checked || self.is_trailing_pressed {
            0.10 // TrailingButtonStateLayerAlpha
        } else if self.is_trailing_hovered {
            0.08
        } else {
            0.0
        };
        self.trailing_state_layer =
            spring_step(spring, self.trailing_state_layer, trailing_layer_target, dt);

        // 6. Target for dropdown menu expansion & fast slide:
        let menu_target = if self.checked { 1.0 } else { 0.0 };
        let menu_spring = SpringParams {
            stiffness: 700.0,
            damping: 38.0,
            mass: 1.0,
        };
        self.menu_progress = spring_step(menu_spring, self.menu_progress, menu_target, dt);

        // 7. Target for leading button width (horizontal expansion / shrinking when text changes):
        let target_width = self.target_leading_width();
        let width_spring = SpringParams {
            stiffness: 450.0,
            damping: 30.0,
            mass: 1.0,
        };
        self.leading_width = spring_step(width_spring, self.leading_width, target_width, dt);

        // Check if any spring is still significantly moving:
        (self.leading_corner.pos - leading_target).abs() > 0.05
            || self.leading_corner.vel.abs() > 0.1
            || (self.trailing_corner.pos - trailing_target).abs() > 0.05
            || self.trailing_corner.vel.abs() > 0.1
            || (self.chevron_progress.pos - chevron_target).abs() > 0.01
            || self.chevron_progress.vel.abs() > 0.05
            || (self.leading_state_layer.pos - leading_layer_target).abs() > 0.005
            || self.leading_state_layer.vel.abs() > 0.01
            || (self.trailing_state_layer.pos - trailing_layer_target).abs() > 0.005
            || self.trailing_state_layer.vel.abs() > 0.01
            || (self.menu_progress.pos - menu_target).abs() > 0.005
            || self.menu_progress.vel.abs() > 0.01
            || (self.leading_width.pos - target_width).abs() > 0.1
            || self.leading_width.vel.abs() > 0.2
    }

    /// Advance spring animations by dt seconds, requesting an animation frame if unsettled.
    pub fn advance_springs(&mut self, window: &mut Window, dt: f32) {
        if self.step(dt) {
            window.request_animation_frame();
        }
    }

    /// Toggle dropdown open/closed state if enabled.
    pub fn toggle_dropdown(&mut self) {
        if !self.enabled {
            return;
        }
        // If the menu was closed within the last 300ms by an outside click
        // (which includes clicking the trailing toggle button during Capture phase),
        // do not immediately re-open it.
        if let Some(closed_at) = self.closed_at.take()
            && closed_at.elapsed().as_millis() < 300
        {
            return;
        }
        self.checked = !self.checked;
    }

    /// Close the dropdown menu.
    pub fn close_dropdown(&mut self) {
        if self.checked {
            self.checked = false;
            self.closed_at = Some(std::time::Instant::now());
        }
    }

    /// Select a menu item option by index and close the dropdown.
    pub fn select_menu_item(&mut self, index: usize) {
        if index < self.menu_items.len() {
            self.selected_index = Some(index);
            self.label = self.menu_items[index].clone();
            self.checked = false;
            self.closed_at = Some(std::time::Instant::now());
        }
    }

    /// Computed corner radii (top_left, top_right, bottom_right, bottom_left) for leading button.
    #[must_use]
    pub fn leading_corners(&self) -> (f32, f32, f32, f32) {
        let metrics = self.metrics();
        let outer = metrics.outer_corner;
        let inner = self.leading_corner.pos.clamp(metrics.inner_corner, outer);
        (outer, inner, inner, outer)
    }

    /// Computed corner radii (top_left, top_right, bottom_right, bottom_left) for trailing button.
    #[must_use]
    pub fn trailing_corners(&self) -> (f32, f32, f32, f32) {
        let metrics = self.metrics();
        let outer = metrics.outer_corner;
        let inner = self.trailing_corner.pos.clamp(metrics.inner_corner, outer);
        (inner, outer, outer, inner)
    }

    /// Resolve active colors based on current theme palette.
    #[must_use]
    pub fn resolve_colors(&self, p: &M3Palette) -> SplitButtonColors {
        SplitButtonColors::for_variant(self.variant, p)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn metrics_match_m3e_tokens() {
        let xs = SplitButtonMetrics::for_size(SplitButtonSize::ExtraSmall);
        assert_eq!(xs.container_height, 32.0);
        assert_eq!(xs.between_space, 2.0);
        assert_eq!(xs.inner_corner, 4.0);
        assert_eq!(xs.inner_corner_pressed, 8.0);
        assert_eq!(xs.outer_corner, 16.0);

        let s = SplitButtonMetrics::for_size(SplitButtonSize::Small);
        assert_eq!(s.container_height, 40.0);
        assert_eq!(s.between_space, 2.0);
        assert_eq!(s.inner_corner, 4.0);
        assert_eq!(s.inner_corner_pressed, 12.0);
        assert_eq!(s.outer_corner, 20.0);

        let m = SplitButtonMetrics::for_size(SplitButtonSize::Medium);
        assert_eq!(m.container_height, 56.0);
        assert_eq!(m.between_space, 2.0);
        assert_eq!(m.inner_corner, 4.0);
        assert_eq!(m.inner_corner_pressed, 12.0);
        assert_eq!(m.outer_corner, 28.0);

        let l = SplitButtonMetrics::for_size(SplitButtonSize::Large);
        assert_eq!(l.container_height, 96.0);
        assert_eq!(l.inner_corner, 8.0);
        assert_eq!(l.inner_corner_pressed, 20.0);
        assert_eq!(l.outer_corner, 48.0);

        let xl = SplitButtonMetrics::for_size(SplitButtonSize::ExtraLarge);
        assert_eq!(xl.container_height, 136.0);
        assert_eq!(xl.inner_corner, 12.0);
        assert_eq!(xl.inner_corner_pressed, 20.0);
        assert_eq!(xl.outer_corner, 68.0);
    }

    #[test]
    fn checked_trailing_button_morphs_to_outer_corner() {
        let mut model = SplitButtonModel::new("Save");
        model.checked = true;
        let metrics = model.metrics();

        // Step physics several times
        for _ in 0..120 {
            model.step(1.0 / 60.0);
        }

        // Corner should have morphed from 4.0 to outer_corner (20.0 for small)
        assert!((model.trailing_corner.pos - metrics.outer_corner).abs() < 0.1);
        assert!((model.chevron_progress.pos - 1.0).abs() < 0.05);
    }

    #[test]
    fn pressed_leading_button_morphs_to_inner_corner_pressed() {
        let mut model = SplitButtonModel::new("Save");
        model.is_leading_pressed = true;
        let metrics = model.metrics();

        for _ in 0..120 {
            model.step(1.0 / 60.0);
        }

        assert!((model.leading_corner.pos - metrics.inner_corner_pressed).abs() < 0.1);
    }
}
