use crate::classes::tooltip as cls;
use crate::font_system;
use std::cell::Cell;
use std::time::Duration;

use super::container::div;
use super::popover::{Anchor, PopoverProps};
use super::tooltip::{TooltipColor, TooltipPosition, TooltipState, bubble};
use crate::components::{PopoverAlign, PopoverSide};
use crate::node::Node;
use crate::signal::{Signal, signal};
use crate::timer::set_timeout;
use crate::widget::AbsoluteRect;

thread_local! {

    static TARGET: Signal<Option<AbsoluteRect>> = signal(None);
    static STATE: TooltipState = TooltipState::new();

    static GENERATION: Cell<u64> = const { Cell::new(0) };
}

pub fn show(label: String, anchor: AbsoluteRect) {
    let generation = GENERATION.with(|g| {
        g.set(g.get() + 1);
        g.get()
    });
    STATE.with(|state| state.set_label(label));
    if TARGET.with(|t| t.peek()) != Some(anchor) {
        TARGET.with(|t| t.set(Some(anchor)));
    }

    set_timeout(Duration::from_millis(1), move || {
        if GENERATION.with(Cell::get) == generation {
            STATE.with(|state| state.force_open(true));
        }
    });
}

pub fn hide() {
    GENERATION.with(|g| g.set(g.get() + 1));
    STATE.with(|state| state.force_open(false));
}

pub fn hover_tooltip_layer() -> Node {
    let layer = Node::reactive(|| {
        let Some(target) = TARGET.with(Signal::get) else {
            return div();
        };
        let state = STATE.with(Clone::clone);
        let bubble = bubble(
            &state,
            TooltipPosition::Top,
            TooltipColor::Primary,
            false,
            None,
            font_system::resolve(None),
        );

        let placement = Anchor::new(target)
            .position(PopoverSide::Top)
            .align(PopoverAlign::Center)
            .spacing(8.0)
            .auto_invert(true);
        PopoverProps::default()
            .anchor(placement)
            .build()
            .class(cls::HOVER_LAYER)
            .bind_class(state.visible_signal(), cls::SHOWN, cls::HIDDEN)
            .child(bubble)
    });
    div().class(cls::HOVER_HOLDER).child(layer)
}
