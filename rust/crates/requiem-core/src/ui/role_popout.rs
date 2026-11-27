use crate::utils::link;
use requiem_ui::components::Anchor;
use requiem_ui::components::PopoverAlign;
use requiem_ui::components::PopoverProps;
use requiem_ui::components::{
    ButtonProps, ButtonSize, ButtonVariant, TextProps, TextVariant, div, popover_backdrop,
};
use requiem_ui::node::Node;
use requiem_ui::signal::{Signal, signal};
use requiem_ui::widget::AbsoluteRect;
use super::classes::role_popout as cls;

use super::TEXT_DEFAULT;
use super::author_style::RoleIcon;
use crate::config::BackendConfig;
use crate::images::ImageCache;

const GAP: f32 = 8.0;

const LEARN_MORE_URL: &str = "https://support.discord.com/hc/en-us/articles/360028038352";

#[derive(Clone)]
struct Open {
    role_name: String,
    icon: RoleIcon,
    anchor: AbsoluteRect,
}

thread_local! {
    static OPEN: Signal<Option<Open>> = signal(None);
}

pub fn open(role_name: String, icon: RoleIcon, anchor: AbsoluteRect) {
    OPEN.with(|open| {
        open.set(Some(Open {
            role_name,
            icon,
            anchor,
        }))
    });
}

pub fn close() {
    OPEN.with(|open| open.set(None));
}

pub fn build(images: &ImageCache, backend: &BackendConfig) -> Node {
    let layer = div().class(cls::LAYER);
    let Some(open) = OPEN.with(Signal::get) else {
        return layer;
    };
    let icon = super::author_style::role_icon_node(&open.icon, cls::ICON, images, backend)
        .unwrap_or_else(|| div().class(cls::ICON));
    let text = div()
        .class(cls::TEXT)
        .child(
            TextProps::styled(open.role_name.clone(), TextVariant::HEADING_MD_SEMIBOLD)
                .line_height(20.0_f32)
                .color(TEXT_DEFAULT)
                .line_clamp(1_u32)
                .build(),
        )
        .child(
            TextProps::styled(
                requiem_i18n::t!("role-popout-description", role = open.role_name.clone())
                    .to_string(),
                TextVariant::TEXT_SM_NORMAL,
            )
            .line_height(18.0_f32)
            .color(TEXT_DEFAULT)
            .build(),
        );
    let learn_more = ButtonProps::default()
        .variant(ButtonVariant::Secondary)
        .size(ButtonSize::Small)
        .full_width(true)
        .label(requiem_i18n::t!("role-popout-learn-more").to_string())
        .on_click(|| {
            close();
            link::open_url(LEARN_MORE_URL);
        })
        .build();
    let anchor = Anchor::new(open.anchor)
        .align(PopoverAlign::Center)
        .spacing(GAP);
    let panel = PopoverProps::default()
        .anchor(anchor)
        .build()
        .class(cls::POPOUT)
        .children([
            div().class(cls::ROW).child(icon).child(text),
            div().class(cls::CTA).child(learn_more),
        ]);
    layer
        .class(cls::LAYER_OPEN)
        .child(popover_backdrop(close))
        .child(panel)
}
