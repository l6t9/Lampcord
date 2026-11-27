use crate::ui::classes::emoji_style_menu;
use crate::ui::classes::emoji_style_menu as cls;
use crate::ui::emoji_picker;
use requiem_ui::app;
use std::cell::Cell;
use std::rc::Rc;

use emojis::SkinTone;
use requiem_ui::components::{
    BasePopoverProps, ButtonProps, ButtonVariant, FormItemProps, PopoverAlign, PopoverSide,
    SelectOption, SelectProps, SelectSearch, TextProps, base_popover::fit, div, text,
};
use requiem_ui::node::Node;
use requiem_ui::signal::{Signal, signal};
use requiem_ui::widget::primitives::ImageWidget;
use skia_safe::Color;

use crate::ui::TEXT_SECONDARY;
use crate::ui::emoji_skin;
use crate::ui::requiem_emoji::{self, EmojiPack, PackStatus};

pub const PANEL_DOM_ID: &str = "emoji-style-menu";
pub const BUTTON_DOM_ID: &str = "emoji-style-menu-button";

const PANEL_WIDTH: f32 = 342.0;
const PANEL_HEIGHT: f32 = 210.0;

thread_local! {

    static PLACEMENT: Cell<(PopoverSide, f32)> = const { Cell::new((PopoverSide::Top, 0.0)) };
    static OPEN: Signal<bool> = signal(false);

    static SELECT_OPEN: Signal<bool> = signal(false);
    static SELECT_QUERY: Signal<Option<String>> = signal(None);
    static SELECT_ACTIVE: Signal<usize> = signal(0);
}

pub fn is_open() -> bool {
    OPEN.with(|open| open.peek())
}

pub fn close() {
    OPEN.with(|open| {
        if open.peek() {
            open.set(false);
        }
    });
    close_select();
}

fn close_select() {
    SELECT_OPEN.with(|open| {
        if open.peek() {
            open.set(false);
        }
    });
    SELECT_QUERY.with(|query| {
        if query.peek().is_some() {
            query.set(None);
        }
    });
    SELECT_ACTIVE.with(|active| {
        if active.peek() != 0 {
            active.set(0);
        }
    });
}

fn pack_preview(pack: EmojiPack) -> Node {
    let class = emoji_style_menu::PREVIEW;
    match requiem_emoji::preview(pack) {
        Some(image) => Node::new("div", ImageWidget::new(Some(image))).class(class),
        None if pack == EmojiPack::NATIVE => div().class(class).child(text(TextProps {
            content: "\u{1F600}".to_string(),
            size: 16.0,
            color: Color::WHITE,
            ..Default::default()
        })),
        None => div().class(class),
    }
}

fn status(pack: EmojiPack, status: PackStatus) -> Node {
    let content = match status {
        PackStatus::NotDownloaded => requiem_i18n::t!("requiem-emoji-status-not-downloaded"),
        PackStatus::Downloading { percent } => {
            requiem_i18n::t!("requiem-emoji-status-downloading", percent = percent)
        }
        PackStatus::Failed => requiem_i18n::t!("requiem-emoji-status-failed"),

        PackStatus::Ready if !pack.is_remote() => String::new(),
        PackStatus::Ready => requiem_i18n::t!("requiem-emoji-status-downloaded"),
    };
    text(TextProps {
        content,
        size: 14.0,
        color: TEXT_SECONDARY,
        ..Default::default()
    })
}

fn tone_item(tone: SkinTone, selected: bool) -> Node {
    let item = ButtonProps::default()
        .variant(ButtonVariant::Bare)
        .extra_classes(vec![cls::TONE.to_string()])
        .on_click(move || emoji_skin::choose(tone))
        .build()
        .child(emoji_picker::emoji_glyph_exact(
            emoji_skin::hand(tone),
            cls::HAND,
            20.0,
        ));
    if selected {
        item.class(cls::TONE_SELECTED)
    } else {
        item
    }
}

pub fn build() -> Node {
    Node::reactive(|| {
        let tone = emoji_skin::current();
        let _ = emoji_skin::picked().get();
        let statuses = requiem_emoji::statuses().get();
        let _ = requiem_emoji::preview_tick().get();
        let picked = requiem_emoji::picked();
        let open = OPEN.with(Signal::get);

        let button = ButtonProps::default()
            .variant(ButtonVariant::Bare)
            .extra_classes(vec![cls::BUTTON.to_string()])
            .on_click_at(|button| {
                if is_open() {
                    close();
                } else {
                    PLACEMENT.with(|placement| {
                        placement.set(fit(
                            button,
                            (PANEL_WIDTH, PANEL_HEIGHT),
                            PopoverSide::Top,
                            PopoverAlign::Center,
                            app::current_window_size(),
                        ))
                    });
                    OPEN.with(|open| open.set(true));
                }
            })
            .build()
            .dom_id(BUTTON_DOM_ID)
            .child(emoji_picker::emoji_glyph_exact(
                emoji_skin::hand(tone),
                cls::HAND,
                20.0,
            ));
        let mut wrapper = div().class(cls::WRAPPER).child(button);

        if open {
            let tones = FormItemProps::new(
                requiem_i18n::t!("emoji-style-menu-skin-tone"),
                div()
                    .class(cls::TONES)
                    .children(emoji_skin::TONES.map(|t| tone_item(t, t == tone))),
            )
            .build();

            let options: Vec<SelectOption> = EmojiPack::available()
                .into_iter()
                .map(|pack| {
                    SelectOption::new(pack.key(), pack.label())
                        .with_leading(pack_preview(pack))
                        .with_trailing(status(pack, statuses[pack.index()]))
                })
                .collect();
            let search = SelectSearch {
                query: SELECT_QUERY.with(Signal::get),
                active: SELECT_ACTIVE.with(Signal::get),
                on_query: Rc::new(|typed| {
                    SELECT_QUERY.with(|query| query.set(Some(typed)));
                    SELECT_ACTIVE.with(|active| active.set(0));
                }),
                on_active: Rc::new(|row| SELECT_ACTIVE.with(|active| active.set(row))),
            };
            let select = SelectProps::new(
                || {
                    if SELECT_OPEN.with(|open| open.peek()) {
                        close_select();
                    } else {
                        SELECT_OPEN.with(|open| open.set(true));
                    }
                },
                |key: String| {
                    close_select();
                    if let Some(pack) = EmojiPack::from_key(&key) {
                        requiem_emoji::select(pack);
                    }
                },
            )
            .search(search)
            .options(options)
            .trigger_leading(pack_preview(picked))
            .trigger_trailing(status(picked, statuses[picked.index()]))
            .selected(picked.key().to_string())
            .open(SELECT_OPEN.with(Signal::get))
            .build();
            let style =
                FormItemProps::new(requiem_i18n::t!("emoji-style-menu-style"), select).build();

            let (side, shift) = PLACEMENT.with(Cell::get);
            let popover = BasePopoverProps::new(vec![tones, style])
                .side(side)
                .align(PopoverAlign::Center)
                .width(PANEL_WIDTH)
                .shift(shift)
                .dom_id(PANEL_DOM_ID)
                .build();
            wrapper = wrapper.child(popover);
        }
        wrapper
    })
    .class(cls::HOLDER)
}
