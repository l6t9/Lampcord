use crate::classes::avatar as cls;
use crate::widget::primitives;
use std::rc::Rc;

use skia_safe::{Color, Image, Typeface};

use crate::node::{ImageBinding, Node};
use crate::signal::Signal;
use crate::widget::primitives::{AvatarShape, AvatarWidget, FallbackIcon, StatusCutout};

#[derive(Default)]
pub struct AvatarProps {
    pub image: Option<Image>,
    /// Shown when `image` is absent.
    pub fallback_initial: String,
    pub fallback_background: Color,

    pub typeface: Option<Typeface>,
    pub shape: AvatarShape,

    /// Hole punched for a status dot, in the avatar's own coordinates.
    pub status_cutout: Option<StatusCutout>,

    /// Icon drawn instead of `fallback_initial` when there is no image.
    pub fallback_icon: Option<FallbackIcon>,

    /// Replaces `image` when the signal changes, without rebuilding the node.
    pub image_binding: Option<ImageBinding>,
}

crate::builder_fields!(AvatarProps {
    opt image: Image,
    fallback_initial: String,
    fallback_background: Color,
    opt typeface: Typeface,
    shape: AvatarShape,
    opt status_cutout: StatusCutout,
    opt fallback_icon: FallbackIcon,
});

impl AvatarProps {
    pub fn build(self) -> Node {
        avatar(self)
    }

    pub fn bind_image(
        mut self,
        tick: Signal<u64>,
        resolve: impl Fn() -> Option<Image> + 'static,
    ) -> Self {
        self.image_binding = Some(ImageBinding {
            tick,
            resolve: Rc::new(resolve),
        });
        self
    }
}

pub fn avatar(props: AvatarProps) -> Node {
    let typeface = props.typeface.expect("AvatarProps::typeface is required");
    let widget = AvatarWidget::new(
        props.image,
        props.fallback_initial,
        props.fallback_background,
        typeface,
        props.shape,
    )
    .with_status_cutout(props.status_cutout)
    .with_fallback_icon(props.fallback_icon);
    let mut node = Node::new("div", widget);
    node.image_binding = props.image_binding;
    node
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AvatarStatusSize {
    Px20,

    Px32,

    Px80,

    Px40,

    Px120,
}

pub fn avatar_with_status(
    props: AvatarProps,
    size: AvatarStatusSize,
    status: Option<primitives::PresenceStatus>,
) -> Node {
    let (wrap_class, avatar_class, dot_class, cutout) = match size {
        AvatarStatusSize::Px20 => (
            cls::WRAP_20,
            cls::AVATAR_20,
            cls::DOT_20,
            StatusCutout::ROUND_20,
        ),
        AvatarStatusSize::Px32 => (
            cls::WRAP_32,
            cls::AVATAR_32,
            cls::DOT_32,
            StatusCutout::ROUND_32,
        ),
        AvatarStatusSize::Px80 => (
            cls::WRAP_80,
            cls::AVATAR_80,
            cls::DOT_80,
            StatusCutout::ROUND_80,
        ),
        AvatarStatusSize::Px40 => (
            cls::WRAP_40,
            cls::AVATAR_40,
            cls::DOT_40,
            StatusCutout::ROUND_40,
        ),
        AvatarStatusSize::Px120 => (
            cls::WRAP_120,
            cls::AVATAR_120,
            cls::DOT_120,
            StatusCutout::ROUND_120,
        ),
    };
    let props = match status {
        Some(_) => props.status_cutout(cutout),
        None => props,
    };
    let mut wrap = super::div()
        .class(wrap_class)
        .child(props.build().class(avatar_class));
    if let Some(status) = status {
        wrap = wrap.child(super::status_indicator(status).class(dot_class));
    }
    wrap
}
