mod animated_icon;
mod avatar;
mod box_paint;
mod button;
mod checkbox;
mod chevron;
mod click_area;
mod container;
mod context_menu;
mod dismiss_backdrop;
mod gradient_ring;
mod hover_roll;
mod hscroll_view;
mod icon;
mod image;
pub(crate) mod markdown;

/// Markdown stripped to the text it shows.
pub use markdown::plain_text as markdown_plain_text;
pub(crate) mod markdown_blocks;
mod message_text;
mod modal_panel;
mod popover_caret;
mod popover_panel;
mod progress_fill;
mod radio;
mod scroll_view;
mod spinner;
mod status;
mod switch;
mod text;
mod text_input;
mod tooltip;

pub use animated_icon::AnimatedIconWidget;
pub use avatar::{
    AvatarShape, AvatarWidget, FallbackIcon, StatusCutout, draw_fallback_icon,
    render_fallback_avatar,
};
pub use button::ButtonWidget;
pub use checkbox::CheckboxWidget;
pub use chevron::ChevronWidget;
pub use click_area::ClickAreaWidget;
pub use container::{
    BoxWidget, HoverBoxWidget, corner_radii, has_any_corner_radius, squircle_path,
};
pub use context_menu::{
    ContextBackdropWidget, ContextPanelWidget, SubmenuHostWidget, SubmenuPanelWidget, place_menu,
    place_submenu,
};
pub use dismiss_backdrop::DismissBackdropWidget;
pub use gradient_ring::GradientRingWidget;
pub use hover_roll::{HoverFlag, HoverRollWidget};
pub use hscroll_view::HScrollWidget;
pub use icon::IconWidget;
pub use image::{
    ImageWidget, draw_atlas_tile, draw_image_filtered, sampling_for, scaled_cache_stats,
};
pub use markdown::MarkdownFeatures;
pub use message_text::{
    CustomEmojiResolver, EmojiClick, EmojiClickHandler, EmojiFrame, EmojiKind, EmojiProvider,
    MentionTarget, MessageTextWidget, set_link_handler,
    set_channel_name_resolver,
};
pub use modal_panel::{ModalBackdropWidget, ModalPanelWidget};
pub use popover_caret::PopoverCaretWidget;
pub use popover_panel::PopoverPanelWidget;
pub use progress_fill::{ProgressFillWidget, forget_progress};
pub use radio::RadioWidget;
pub use scroll_view::{ScrollAnchor, ScrollPosition, ScrollViewWidget, WheelCarry, WheelGlide};
pub use spinner::PulsingEllipsisWidget;
pub use status::{PresenceStatus, StatusIndicatorWidget};
pub use switch::SwitchWidget;
pub use text::TextWidget;
#[cfg(any(test, feature = "test-hooks"))]
pub use text::painted_lines_of;
pub use text_input::{
    AutocompleteCandidate, AutocompleteTrigger, CodeHighlighter, DecorationStyle, Decorator,
    DynamicAutocomplete, DynamicMatch, TIMESTAMP_TRIGGER_CHAR, TextDecoration, TextInputWidget,
};
pub use tooltip::{PointerEdge, TooltipAnchorWidget, TooltipPointerWidget, TopLayerWidget};
