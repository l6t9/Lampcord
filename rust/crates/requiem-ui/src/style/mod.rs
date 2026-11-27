pub mod animation;
pub mod calc;
pub mod cascade;
pub mod parser;
pub mod selector_impl;
pub mod transform;
pub mod transition;
pub mod values;

use crate::event;
use skia_safe::Color;
use taffy::style::Style as TaffyStyle;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ScrollbarKind {
    #[default]
    Legacy,
    Thin,
    Auto,
    None,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ScrollbarStyle {
    pub kind: ScrollbarKind,
    pub fade: bool,
}

#[derive(Clone, Debug, Default)]
pub struct ComputedStyle {
    pub scrollbar: ScrollbarStyle,

    pub opacity: Option<f32>,

    /// `mix-blend-mode`: how the element's painted result combines with what is behind it.
    pub blend_mode: Option<BlendMode>,

    pub filter: Option<Filter>,

    pub box_shadows: Vec<BoxShadow>,

    pub pointer_events_none: bool,

    pub cursor: Option<event::CursorIcon>,
    pub layout: TaffyStyle,
    pub background: Option<Color>,
    pub corner_radius: f32,

    /// `border-radius: 50%`: each corner's radius is half the box's own width and height, so
    /// a wide box is an ellipse. The other radii are circular.
    pub corner_ellipse: bool,

    pub squircle: bool,

    pub corner_radius_top_left: Option<f32>,

    pub corner_radius_top_right: Option<f32>,
    pub corner_radius_bottom_right: Option<f32>,
    pub corner_radius_bottom_left: Option<f32>,

    pub border_color: Option<Color>,
    pub border_width: f32,

    pub border_dashed: bool,

    pub border_bottom_color: Option<Color>,
    pub border_bottom_width: f32,

    pub border_top_color: Option<Color>,
    pub border_top_width: f32,
    pub border_left_color: Option<Color>,
    pub border_left_width: f32,

    pub border_right_color: Option<Color>,
    pub border_right_width: f32,

    pub outline_color: Option<Color>,
    pub outline_width: Option<f32>,

    pub text_color: Option<Color>,

    pub text_underline: bool,

    pub text_layout: TextLayoutStyle,

    pub is_hovered: bool,

    pub transitions: Vec<transition::TransitionSpec>,

    pub animations: Vec<animation::AnimationSpec>,

    pub transform: Option<transform::Transform>,

    pub transform_origin: Option<(f32, f32)>,
}

impl ComputedStyle {
    pub fn to_taffy(&self) -> TaffyStyle {
        self.layout.clone()
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BoxShadow {
    pub x: f32,
    pub y: f32,
    pub blur: f32,
    pub spread: f32,
    pub color: Color,

    pub inset: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BlendMode {
    Multiply,
    Screen,
    Overlay,
    Darken,
    Lighten,
    ColorDodge,
    ColorBurn,
    HardLight,
    SoftLight,
    Difference,
    Exclusion,
}

impl BlendMode {
    /// `normal` is the absence of a blend mode, so it isn't one of these.
    pub fn parse(value: &str) -> Option<Self> {
        Some(match value.trim() {
            "multiply" => Self::Multiply,
            "screen" => Self::Screen,
            "overlay" => Self::Overlay,
            "darken" => Self::Darken,
            "lighten" => Self::Lighten,
            "color-dodge" => Self::ColorDodge,
            "color-burn" => Self::ColorBurn,
            "hard-light" => Self::HardLight,
            "soft-light" => Self::SoftLight,
            "difference" => Self::Difference,
            "exclusion" => Self::Exclusion,
            _ => return None,
        })
    }

    pub fn to_skia(self) -> skia_safe::BlendMode {
        match self {
            Self::Multiply => skia_safe::BlendMode::Multiply,
            Self::Screen => skia_safe::BlendMode::Screen,
            Self::Overlay => skia_safe::BlendMode::Overlay,
            Self::Darken => skia_safe::BlendMode::Darken,
            Self::Lighten => skia_safe::BlendMode::Lighten,
            Self::ColorDodge => skia_safe::BlendMode::ColorDodge,
            Self::ColorBurn => skia_safe::BlendMode::ColorBurn,
            Self::HardLight => skia_safe::BlendMode::HardLight,
            Self::SoftLight => skia_safe::BlendMode::SoftLight,
            Self::Difference => skia_safe::BlendMode::Difference,
            Self::Exclusion => skia_safe::BlendMode::Exclusion,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Filter {
    pub blur: f32,

    pub brightness: f32,

    pub drop_shadow: Option<DropShadow>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DropShadow {
    pub x: f32,
    pub y: f32,

    pub blur: f32,
    pub color: Color,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TextLayoutStyle {
    pub nowrap: bool,

    pub ellipsis: bool,

    pub line_clamp: Option<u32>,
}

#[cfg(test)]
mod blend_mode_tests {
    use super::BlendMode;

    #[test]
    fn the_css_names_map_and_normal_is_no_blend_mode() {
        assert_eq!(BlendMode::parse("multiply"), Some(BlendMode::Multiply));
        assert_eq!(BlendMode::parse(" hard-light "), Some(BlendMode::HardLight));
        assert_eq!(BlendMode::parse("normal"), None);
        assert_eq!(BlendMode::parse("plus-lighter"), None);
    }
}
