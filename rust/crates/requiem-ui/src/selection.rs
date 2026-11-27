use crate::clipboard;
use crate::paint;
use std::cell::RefCell;
use std::collections::HashMap;
use std::collections::HashSet;
use std::time::{Duration, Instant};

use crate::event::{MouseButton, UiEvent};
use crate::widget::{AbsoluteRect, WidgetId, WidgetTree};

const MULTI_CLICK: Duration = Duration::from_millis(500);
const MULTI_CLICK_SLOP: f32 = 4.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Point {
    key: u64,
    offset: u64,
}

#[derive(Default)]
struct State {
    anchor: Option<Point>,

    ranges: HashMap<u64, (u64, u64)>,
    dragging: bool,
    last_press: Option<(Instant, f32, f32, u32)>,
}

thread_local! {
    static STATE: RefCell<State> = RefCell::new(State::default());
}

pub fn range_for(key: u64) -> Option<(u64, u64)> {
    STATE.with(|s| s.borrow().ranges.get(&key).copied())
}

pub fn is_active() -> bool {
    STATE.with(|s| s.borrow().ranges.values().any(|(a, b)| a != b))
}

pub fn clear() -> bool {
    STATE.with(|s| {
        let mut s = s.borrow_mut();
        let had = !s.ranges.is_empty();
        s.ranges.clear();
        s.anchor = None;
        s.dragging = false;
        had
    })
}

struct Target {
    id: WidgetId,
    key: u64,
    rect: AbsoluteRect,
}

fn targets(tree: &WidgetTree) -> Vec<Target> {
    let mut found: Vec<Target> = tree
        .nodes
        .iter()
        .filter_map(|(id, node)| {
            let key = node.widget.selection_key()?;
            Some(Target {
                id,
                key,
                rect: paint::absolute_rect(tree, id),
            })
        })
        .filter(|t| t.rect.width > 0.0 && t.rect.height > 0.0)
        .collect();
    found.sort_by(|a, b| {
        a.rect
            .y
            .total_cmp(&b.rect.y)
            .then(a.rect.x.total_cmp(&b.rect.x))
    });
    let mut seen = HashSet::new();
    found.retain(|t| seen.insert(t.key));
    found
}

fn press_count(x: f32, y: f32) -> u32 {
    STATE.with(|s| {
        let mut s = s.borrow_mut();
        let now = Instant::now();
        let count = match s.last_press {
            Some((at, lx, ly, count))
                if now.duration_since(at) < MULTI_CLICK
                    && (x - lx).abs() <= MULTI_CLICK_SLOP
                    && (y - ly).abs() <= MULTI_CLICK_SLOP =>
            {
                (count + 1).min(3)
            }
            _ => 1,
        };
        s.last_press = Some((now, x, y, count));
        count
    })
}

pub fn after_dispatch(tree: &WidgetTree, event: &UiEvent, handler: Option<WidgetId>) -> bool {
    match *event {
        UiEvent::MouseDown {
            x,
            y,
            button: MouseButton::Left,
            ..
        } => {
            if handler.is_some() {
                return clear();
            }
            let all = targets(tree);
            let Some(target) = all.iter().find(|t| t.rect.contains(x, y)) else {
                return clear();
            };
            let widget = &tree.nodes[target.id].widget;
            let offset = widget.selection_offset_at(target.rect, x, y);
            let count = press_count(x, y);
            STATE.with(|s| {
                let mut s = s.borrow_mut();
                s.ranges.clear();
                s.anchor = Some(Point {
                    key: target.key,
                    offset,
                });
                s.dragging = true;
                let range = match count {
                    1 => (offset, offset),
                    2 => widget.selection_word(target.rect, offset),
                    _ => widget.selection_paragraph(target.rect, offset),
                };
                s.ranges.insert(target.key, range);
            });
            true
        }
        UiEvent::MouseMoved { x, y } if STATE.with(|s| s.borrow().dragging) => drag_to(tree, x, y),
        UiEvent::MouseUp {
            button: MouseButton::Left,
            ..
        } => STATE.with(|s| {
            let mut s = s.borrow_mut();
            let was = s.dragging;
            s.dragging = false;

            if s.ranges.values().all(|(a, b)| a == b) {
                let had = !s.ranges.is_empty();
                s.ranges.clear();
                s.anchor = None;
                return had;
            }
            let _ = was;
            false
        }),
        _ => false,
    }
}

fn drag_to(tree: &WidgetTree, x: f32, y: f32) -> bool {
    let Some(anchor) = STATE.with(|s| s.borrow().anchor) else {
        return false;
    };
    let all = targets(tree);
    if all.is_empty() {
        return false;
    }

    let head_index = all
        .iter()
        .position(|t| t.rect.contains(x, y))
        .unwrap_or_else(|| {
            let distance = |t: &Target| {
                if y < t.rect.y {
                    t.rect.y - y
                } else if y > t.rect.y + t.rect.height {
                    y - (t.rect.y + t.rect.height)
                } else {
                    0.0
                }
            };
            (0..all.len())
                .min_by(|&a, &b| distance(&all[a]).total_cmp(&distance(&all[b])))
                .unwrap_or(0)
        });
    let Some(anchor_index) = all.iter().position(|t| t.key == anchor.key) else {
        return false;
    };
    let head = &all[head_index];
    let head_widget = &tree.nodes[head.id].widget;
    let head_offset = if y < head.rect.y {
        0
    } else if y >= head.rect.y + head.rect.height {
        head_widget.selection_end(head.rect)
    } else {
        head_widget.selection_offset_at(head.rect, x, y)
    };

    let mut ranges = HashMap::new();
    let end_of = |t: &Target| tree.nodes[t.id].widget.selection_end(t.rect);
    if anchor_index == head_index {
        ranges.insert(
            head.key,
            (
                anchor.offset.min(head_offset),
                anchor.offset.max(head_offset),
            ),
        );
    } else if anchor_index < head_index {
        ranges.insert(
            all[anchor_index].key,
            (anchor.offset, end_of(&all[anchor_index])),
        );
        for t in &all[anchor_index + 1..head_index] {
            ranges.insert(t.key, (0, end_of(t)));
        }
        ranges.insert(head.key, (0, head_offset));
    } else {
        ranges.insert(head.key, (head_offset, end_of(head)));
        for t in &all[head_index + 1..anchor_index] {
            ranges.insert(t.key, (0, end_of(t)));
        }
        ranges.insert(all[anchor_index].key, (0, anchor.offset));
    }
    STATE.with(|s| {
        let mut s = s.borrow_mut();
        let changed = s.ranges != ranges;
        s.ranges = ranges;
        changed
    })
}

pub fn selected_text(tree: &WidgetTree) -> Option<String> {
    let ranges = STATE.with(|s| s.borrow().ranges.clone());
    let parts: Vec<String> = targets(tree)
        .iter()
        .filter_map(|t| {
            let (a, b) = *ranges.get(&t.key)?;
            (a != b).then(|| tree.nodes[t.id].widget.selection_text(t.rect, (a, b)))
        })
        .filter(|text| !text.is_empty())
        .collect();
    (!parts.is_empty()).then(|| parts.join("\n"))
}

pub fn copy(tree: &WidgetTree) -> bool {
    match selected_text(tree) {
        Some(text) => {
            clipboard::set_text(text);
            true
        }
        None => false,
    }
}
