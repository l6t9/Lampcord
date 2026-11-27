use crate::classes::button as classes_button;
use crate::font_system::TextFonts;
use crate::font_system;
use crate::icon_handle;
use crate::widget::primitives::PulsingEllipsisWidget;
use crate::widget;
use skia_safe::Color;
use skia_safe::font_style::Weight;

use crate::node::Node;
use crate::widget::primitives::ButtonWidget;

use super::text::{TextProps, text};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ButtonVariant {
    #[default]
    Primary,
    Secondary,
    CriticalPrimary,
    CriticalSecondary,
    Active,

    /// White fill, black label, for use over imagery.
    OverlayPrimary,

    /// Translucent black fill, white label, for use over imagery.
    OverlaySecondary,

    /// White fill, black label, 20px icon. No glow behind the fill.
    Expressive,

    /// No fill at rest. Hover and press fill the button and turn the icon white.
    IconOnly,

    /// No variant classes. Click behavior only, styling comes from `extra_classes`.
    Bare,
}

impl ButtonVariant {
    pub fn parse(name: &str) -> Option<Self> {
        Some(match name {
            "primary" => ButtonVariant::Primary,
            "secondary" => ButtonVariant::Secondary,
            "critical-primary" => ButtonVariant::CriticalPrimary,
            "critical-secondary" => ButtonVariant::CriticalSecondary,
            "active" => ButtonVariant::Active,
            "overlay-primary" => ButtonVariant::OverlayPrimary,
            "overlay-secondary" => ButtonVariant::OverlaySecondary,
            "expressive" => ButtonVariant::Expressive,
            "icon-only" => ButtonVariant::IconOnly,
            "bare" => ButtonVariant::Bare,
            _ => return None,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ButtonSize {
    #[default]
    Medium,
    Small,
}

/// Closure wrapper so a builder setter accepts `|| { ... }`.
#[derive(Default)]
pub struct OnClick(pub Option<Box<dyn FnMut()>>);

impl<F: FnMut() + 'static> From<F> for OnClick {
    fn from(f: F) -> Self {
        OnClick(Some(Box::new(f)))
    }
}

/// Extra click callback. `true` when Shift was held.
#[derive(Default)]
pub struct OnClickShift(pub Option<Box<dyn FnMut(bool)>>);

impl<F: FnMut(bool) + 'static> From<F> for OnClickShift {
    fn from(f: F) -> Self {
        OnClickShift(Some(Box::new(f)))
    }
}

/// Fires once on hover enter and once on leave.
#[derive(Default)]
pub struct OnHoverChange(pub Option<Box<dyn FnMut(bool)>>);

/// Click callback given the button's on-screen rect.
#[derive(Default)]
pub struct OnClickAt(pub Option<Box<dyn FnMut(widget::AbsoluteRect)>>);

impl<F: FnMut(widget::AbsoluteRect) + 'static> From<F> for OnClickAt {
    fn from(f: F) -> Self {
        OnClickAt(Some(Box::new(f)))
    }
}

impl<F: FnMut(bool) + 'static> From<F> for OnHoverChange {
    fn from(f: F) -> Self {
        OnHoverChange(Some(Box::new(f)))
    }
}

#[derive(Default)]
pub struct ButtonProps {
    pub variant: ButtonVariant,
    pub size: ButtonSize,

    /// Centered label. `None` leaves the content to the caller.
    pub label: Option<String>,
    pub extra_classes: Vec<String>,
    pub on_click: OnClick,
    pub on_click_shift: OnClickShift,
    pub on_hover_change: OnHoverChange,

    /// Click callback given the button's on-screen rect, for anchoring a popout.
    pub on_click_at: OnClickAt,

    /// Stretch to the container width.
    pub full_width: bool,

    /// Half opacity, no hover, no clicks.
    pub disabled: bool,

    /// Hides the label in place and shows a pulsing ellipsis. Clicks stop.
    pub loading: bool,

    /// Minimum width in px.
    pub min_width: Option<f32>,

    /// Overrides the ambient fonts. `None` uses those.
    pub fonts: Option<TextFonts>,

    /// Icon before the label, drawn in the label color. No label makes a square icon button.
    pub icon: Option<icon_handle::IconSpec>,
}

crate::builder_fields!(ButtonProps {
    variant: ButtonVariant,
    size: ButtonSize,
    opt label: String,
    extra_classes: Vec<String>,
    on_click: OnClick,
    on_click_shift: OnClickShift,
    on_hover_change: OnHoverChange,
    on_click_at: OnClickAt,
    full_width: bool,
    disabled: bool,
    loading: bool,
    opt min_width: f32,
    opt fonts: TextFonts,
    opt icon: icon_handle::IconSpec,
});

impl ButtonProps {
    /// Labelled button. `variant` is a [`ButtonVariant`] or a parseable name.
    pub fn new(
        variant: impl Into<ButtonVariant>,
        label: impl Into<String>,
        on_click: impl Into<OnClick>,
    ) -> Self {
        Self {
            variant: variant.into(),
            label: Some(label.into()),
            on_click: on_click.into(),
            ..Default::default()
        }
    }

    pub fn build(self) -> Node {
        button(self)
    }
}

pub fn button(props: ButtonProps) -> Node {
    let mut widget = ButtonWidget::new();
    let inert = props.disabled || props.loading;
    if let Some(on_click) = props.on_click.0.filter(|_| !inert) {
        widget = widget.with_on_click(on_click);
    }
    if let Some(on_click_shift) = props.on_click_shift.0 {
        widget = widget.with_on_click_shift(on_click_shift);
    }
    if let Some(on_hover_change) = props.on_hover_change.0 {
        widget = widget.with_on_hover_change(on_hover_change);
    }
    if let Some(on_click_at) = props.on_click_at.0.filter(|_| !inert) {
        widget = widget.with_on_click_at(on_click_at);
    }

    let mut node = Node::new("button", widget);
    if props.variant != ButtonVariant::Bare {
        node = node
            .class(classes_button::ROOT)
            .class(match props.size {
                ButtonSize::Small => classes_button::SM,
                ButtonSize::Medium => classes_button::MD,
            })
            .class(variant_class(props.variant));
    }
    if props.full_width {
        node = node.class(classes_button::FULL_WIDTH);
    }
    if props.disabled {
        node = node.class(classes_button::DISABLED);
    }
    if let Some(min_width) = props.min_width {
        node = node.style(format!("min-width: {min_width}px"));
    }
    node = node.classes(props.extra_classes);

    if let Some(path) = &props.icon {
        if props.label.is_none() {
            node = node.class(classes_button::ICON_ONLY);
        }
        let icon_class = match props.variant {
            ButtonVariant::Expressive => classes_button::EXPRESSIVE_ICON,
            ButtonVariant::IconOnly if props.size == ButtonSize::Small => {
                classes_button::ICON_ONLY_ICON_SM
            }
            ButtonVariant::IconOnly => classes_button::ICON_ONLY_ICON_MD,
            _ => classes_button::ICON,
        };
        node = node.child(super::icon::icon(path, label_color(props.variant)).class(icon_class));
    }

    if let Some(label) = props.label {
        let fonts = font_system::resolve(props.fonts);
        let (size, weight) = label_type(props.size);
        let mut label_node = text(TextProps {
            content: label,
            size,
            color: label_color(props.variant),
            weight: weight.into(),
            fonts: Some(fonts),
            line_clamp: Some(1),
            ..Default::default()
        });
        if props.loading {
            label_node = label_node.class(classes_button::LABEL_HIDDEN);
        }
        node = node.child(label_node);
    }
    if props.loading {
        let scale = if props.size == ButtonSize::Small {
            0.75
        } else {
            0.9
        };
        node = node.class(classes_button::LOADING).child(
            Node::new(
                "span",
                PulsingEllipsisWidget::new(
                    label_color(props.variant),
                    scale,
                ),
            )
            .class(classes_button::SPINNER),
        );
    }
    node
}

fn label_type(size: ButtonSize) -> (f32, Weight) {
    match size {
        ButtonSize::Medium => (16.0, Weight::MEDIUM),
        ButtonSize::Small => (14.0, Weight::MEDIUM),
    }
}

fn variant_class(variant: ButtonVariant) -> &'static str {
    match variant {
        ButtonVariant::Primary => classes_button::PRIMARY,
        ButtonVariant::Secondary => classes_button::SECONDARY,
        ButtonVariant::CriticalPrimary => classes_button::CRITICAL_PRIMARY,
        ButtonVariant::CriticalSecondary => classes_button::CRITICAL_SECONDARY,
        ButtonVariant::Active => classes_button::ACTIVE,
        ButtonVariant::OverlayPrimary => classes_button::OVERLAY_PRIMARY,
        ButtonVariant::OverlaySecondary => classes_button::OVERLAY_SECONDARY,
        ButtonVariant::Expressive => classes_button::EXPRESSIVE,
        ButtonVariant::IconOnly => classes_button::ICON_ONLY_VARIANT,
        ButtonVariant::Bare => unreachable!("Bare is filtered out before calling variant_class"),
    }
}

fn label_color(variant: ButtonVariant) -> Color {
    match variant {
        ButtonVariant::Primary
        | ButtonVariant::CriticalPrimary
        | ButtonVariant::Active
        | ButtonVariant::OverlaySecondary => Color::WHITE,
        ButtonVariant::OverlayPrimary | ButtonVariant::Expressive => Color::BLACK,
        ButtonVariant::Secondary | ButtonVariant::Bare => Color::from_argb(255, 0xfb, 0xfb, 0xfb),
        ButtonVariant::CriticalSecondary => Color::from_argb(255, 0xf8, 0x7e, 0x7a),
        ButtonVariant::IconOnly => Color::TRANSPARENT,
    }
}
