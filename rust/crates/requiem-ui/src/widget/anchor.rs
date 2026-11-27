use crate::app;
use super::AbsoluteRect;
use crate::components::{PopoverAlign, PopoverSide};

pub const VIEWPORT_MARGIN: f32 = 12.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Anchor {
    pub target: AbsoluteRect,
    pub position: PopoverSide,

    pub align: Option<PopoverAlign>,
    /// Gap between the target and the popover, in px.
    pub spacing: f32,
    /// Flip `position` when the preferred side does not fit.
    pub auto_invert: bool,
    /// Slide along the edge so the popover stays inside the window.
    pub nudge: bool,
}

impl Anchor {
    pub fn new(target: AbsoluteRect) -> Self {
        Self {
            target,
            position: PopoverSide::Right,
            align: None,
            spacing: 8.0,
            auto_invert: true,
            nudge: true,
        }
    }

    pub fn position(mut self, position: PopoverSide) -> Self {
        self.position = position;
        self
    }

    pub fn align(mut self, align: PopoverAlign) -> Self {
        self.align = Some(align);
        self
    }

    pub fn spacing(mut self, spacing: f32) -> Self {
        self.spacing = spacing;
        self
    }

    pub fn auto_invert(mut self, on: bool) -> Self {
        self.auto_invert = on;
        self
    }

    pub fn nudge(mut self, on: bool) -> Self {
        self.nudge = on;
        self
    }
}

pub fn viewport() -> AbsoluteRect {
    let (width, height) = app::current_window_size();
    AbsoluteRect {
        x: 0.0,
        y: app::current_top_inset(),
        width,
        height,
    }
}

fn opposite(side: PopoverSide) -> PopoverSide {
    match side {
        PopoverSide::Top => PopoverSide::Bottom,
        PopoverSide::Bottom => PopoverSide::Top,
        PopoverSide::Left => PopoverSide::Right,
        PopoverSide::Right => PopoverSide::Left,
    }
}

pub fn place(anchor: &Anchor, size: (f32, f32), viewport: AbsoluteRect) -> (f32, f32) {
    let target = AbsoluteRect {
        x: anchor.target.x - viewport.x,
        y: anchor.target.y - viewport.y,
        ..anchor.target
    };
    let (x, y) = place_in(anchor, target, size, (viewport.width, viewport.height));
    (x + viewport.x, y + viewport.y)
}

fn place_in(
    anchor: &Anchor,
    target: AbsoluteRect,
    size: (f32, f32),
    layer: (f32, f32),
) -> (f32, f32) {
    let first = position_style(anchor, anchor.position, target, size, layer);
    let mut best = (anchor.position, first);
    let mut room = overflow(anchor.position, first, size, layer);
    if anchor.auto_invert && room < 0.0 {
        let flipped = opposite(anchor.position);
        let second = position_style(anchor, flipped, target, size, layer);
        let flipped_room = overflow(flipped, second, size, layer);
        if flipped_room > room {
            best = (flipped, second);
            room = flipped_room;
        }
    }
    let (side, (x, mut y)) = best;

    if room < 0.0 && room.abs() < size.1 {
        match side {
            PopoverSide::Top => y -= room,
            PopoverSide::Bottom => y += room,
            _ => {}
        }
    }
    (x, y)
}

fn overflow(
    side: PopoverSide,
    (x, y): (f32, f32),
    (w, h): (f32, f32),
    (lw, lh): (f32, f32),
) -> f32 {
    match side {
        PopoverSide::Top => y,
        PopoverSide::Bottom => lh - (y + h),
        PopoverSide::Left => x,
        PopoverSide::Right => lw - (x + w),
    }
}

fn position_style(
    anchor: &Anchor,
    side: PopoverSide,
    t: AbsoluteRect,
    (w, h): (f32, f32),
    layer: (f32, f32),
) -> (f32, f32) {
    let align = anchor.align.unwrap_or(PopoverAlign::Start);
    let s = anchor.spacing;
    match side {
        PopoverSide::Top => (horizontal(anchor, align, t, w, layer.0), t.y - s - h),
        PopoverSide::Bottom => (horizontal(anchor, align, t, w, layer.0), t.y + t.height + s),
        PopoverSide::Left => (t.x - s - w, vertical(anchor, align, t, h, layer.1)),
        PopoverSide::Right => (t.x + t.width + s, vertical(anchor, align, t, h, layer.1)),
    }
}

fn vertical(anchor: &Anchor, align: PopoverAlign, t: AbsoluteRect, h: f32, lh: f32) -> f32 {
    let nudge_top = |top: f32| {
        if !anchor.nudge {
            return top;
        }
        if top < 0.0 {
            return 0.0;
        }
        let over = top + h - lh + VIEWPORT_MARGIN;
        if over > 0.0 {
            VIEWPORT_MARGIN.max(top - over)
        } else {
            top
        }
    };
    match align {
        PopoverAlign::Start => nudge_top(t.y),
        PopoverAlign::Center => nudge_top(t.y + (t.height - h) / 2.0),
        PopoverAlign::End => {
            let bottom = lh - (t.y + t.height);
            let bottom = if anchor.nudge {
                let over = bottom + h - lh + VIEWPORT_MARGIN;
                if over > 0.0 {
                    VIEWPORT_MARGIN.max(bottom - over)
                } else {
                    bottom
                }
            } else {
                bottom
            };
            lh - bottom - h
        }
    }
}

fn horizontal(anchor: &Anchor, align: PopoverAlign, t: AbsoluteRect, w: f32, lw: f32) -> f32 {
    let nudge_left = |left: f32| {
        if !anchor.nudge {
            return left;
        }
        let over = left + w - lw + VIEWPORT_MARGIN;
        if over > 0.0 {
            VIEWPORT_MARGIN.max(left - over)
        } else {
            VIEWPORT_MARGIN.max(left)
        }
    };
    match align {
        PopoverAlign::Start => nudge_left(t.x),
        PopoverAlign::Center => nudge_left(t.x + (t.width - w) / 2.0),
        PopoverAlign::End => {
            let right = lw - (t.x + t.width);
            let shift = if anchor.nudge {
                (lw - right - w - VIEWPORT_MARGIN).min(0.0)
            } else {
                0.0
            };
            lw - (right + shift) - w
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const LAYER: AbsoluteRect = AbsoluteRect {
        x: 0.0,
        y: 0.0,
        width: 1365.0,
        height: 768.0,
    };

    #[test]
    fn a_message_authors_popout_rises_to_stay_12px_inside_the_window() {
        let name = AbsoluteRect {
            x: 447.0,
            y: 585.0,
            width: 48.0,
            height: 21.0,
        };
        assert_eq!(
            place(&Anchor::new(name), (300.0, 294.0), LAYER),
            (503.0, 462.0)
        );
    }

    #[test]
    fn a_member_list_popout_sits_16px_left_of_the_row_top_edges_flush() {
        let row = AbsoluteRect {
            x: 1109.0,
            y: 161.0,
            width: 248.0,
            height: 42.0,
        };
        assert_eq!(
            place(
                &Anchor::new(row).position(PopoverSide::Left).spacing(16.0),
                (300.0, 344.0),
                LAYER
            ),
            (793.0, 161.0)
        );
    }

    #[test]
    fn a_panel_with_no_room_on_its_side_inverts() {
        let target = AbsoluteRect {
            x: 1200.0,
            y: 100.0,
            width: 40.0,
            height: 20.0,
        };
        let (x, _) = place(&Anchor::new(target), (300.0, 200.0), LAYER);
        assert_eq!(x, 1200.0 - 8.0 - 300.0);
    }

    #[test]
    fn a_panel_below_a_target_near_the_bottom_flips_above_it_or_overlaps() {
        let target = AbsoluteRect {
            x: 100.0,
            y: 700.0,
            width: 40.0,
            height: 20.0,
        };
        assert_eq!(
            place(
                &Anchor::new(target).position(PopoverSide::Bottom),
                (200.0, 100.0),
                LAYER
            )
            .1,
            700.0 - 8.0 - 100.0
        );

        let tall = (200.0, 760.0);
        let (_, y) = place(
            &Anchor::new(AbsoluteRect { y: 400.0, ..target }).position(PopoverSide::Bottom),
            tall,
            LAYER,
        );
        assert_eq!(y, 0.0);
    }

    #[test]
    fn coordinates_are_relative_to_the_viewport() {
        let layer = AbsoluteRect { y: 30.0, ..LAYER };
        let name = AbsoluteRect {
            x: 447.0,
            y: 615.0,
            width: 48.0,
            height: 21.0,
        };
        assert_eq!(
            place(&Anchor::new(name), (300.0, 294.0), layer),
            (503.0, 492.0)
        );
    }
}
