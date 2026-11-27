use requiem_ui::components;
use requiem_ui::icons::Icon;
use requiem_ui::scroll_animation;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;

use requiem_ui::colors::{TEXT_SECONDARY, TEXT_STRONG};
use requiem_ui::components::{
    ControlItemProps, MessageTextProps, TextProps, TextVariant, div, icon,
};
use requiem_ui::node::Node;
use requiem_ui::signal::{Signal, signal};
use requiem_ui::widget::AbsoluteRect;
use requiem_ui::widget::anim::{ToggleMemo, ToggleState};

use crate::shell::classes::settings_page as cls;
use requiem_ui::icons::CHEVRON_LARGE_RIGHT;

pub fn rich(content: String, on_link: Option<Rc<dyn Fn(&str)>>) -> Node {
    let mut props = MessageTextProps {
        content,
        size: 14.0,
        line_height: Some(18.0),
        color: TEXT_SECONDARY,
        ..Default::default()
    };
    if let Some(on_link) = on_link {
        props.on_link = Some(Rc::new(move |url: &str, _| on_link(url)));
    }
    components::message_text(props)
}

pub fn plain(text: String) -> Option<Node> {
    Some(rich(text, None))
}

pub fn setting(node: Node) -> Node {
    div().class(cls::SETTING).child(node)
}

pub fn category(title: Option<String>, rows: Vec<Node>) -> Node {
    let mut block = div().class(cls::CATEGORY);
    if let Some(title) = title {
        block = block.child(
            div().class(cls::CATEGORY_HEADER).child(
                TextProps::styled(title, TextVariant::HEADING_XL_NORMAL)
                    .color(TEXT_STRONG)
                    .build(),
            ),
        );
    }
    block.child(div().class(cls::ROWS).children(rows))
}

pub fn divider() -> Node {
    div().class(cls::DIVIDER)
}

pub fn fieldset(legend: Vec<Node>, rows: Vec<Node>) -> Node {
    div()
        .class(cls::FIELDSET)
        .child(div().class(cls::LEGEND).children(legend))
        .children(rows)
}

pub fn related(
    title: String,
    subtitle: String,
    glyph: Icon,
    on_click: impl Fn() + 'static,
) -> Node {
    let leading = div()
        .class(cls::NAV_ICON)
        .child(icon(glyph, TEXT_STRONG).class(cls::NAV_ICON_GLYPH));
    let row = ControlItemProps::new(title)
        .subtitle(subtitle)
        .leading(leading)
        .trailing(icon(CHEVRON_LARGE_RIGHT, TEXT_SECONDARY).class(cls::NAV_CHEVRON))
        .on_click(on_click)
        .build();
    div().class(cls::RELATED).children([
        TextProps::styled(
            requiem_i18n::t!("voice-settings-related-settings"),
            TextVariant::HEADING_MD_SEMIBOLD,
        )
        .color(TEXT_STRONG)
        .build(),
        row,
    ])
}

pub fn panel(categories: Vec<Node>) -> Node {
    let mut children = Vec::with_capacity(categories.len() * 2);
    for (i, category) in categories.into_iter().enumerate() {
        if i > 0 {
            children.push(div().class(cls::CATEGORY_DIVIDER));
        }
        children.push(category);
    }
    div().class(cls::CATEGORIES).children(children)
}

pub fn page_container(content: Node) -> Node {
    div().class(cls::PANEL).child(content)
}

#[derive(Default)]
pub struct Switches(RefCell<HashMap<&'static str, ToggleMemo>>);

impl Switches {
    pub fn state(&self, key: &'static str, on: bool) -> ToggleState {
        self.0
            .borrow_mut()
            .entry(key)
            .or_insert_with(|| ToggleMemo::new(on))
            .track(on)
    }

    pub fn clear(&self) {
        self.0.borrow_mut().clear();
    }
}

pub struct Anchors {
    container: Signal<AbsoluteRect>,
    categories: Vec<Signal<AbsoluteRect>>,
    offset: Cell<f32>,
    active: Signal<usize>,

    snapshot: RefCell<Option<(Vec<AbsoluteRect>, AbsoluteRect, f32)>>,
}

fn content_span(rect: AbsoluteRect, container: AbsoluteRect, laid_out_at: f32) -> (f32, f32) {
    (rect.y - container.y + laid_out_at, rect.height)
}

impl Anchors {
    pub fn leak(count: usize) -> &'static Anchors {
        Box::leak(Box::new(Anchors {
            container: signal(Default::default()),
            categories: (0..count).map(|_| signal(Default::default())).collect(),
            offset: Cell::new(0.0),
            active: signal(0),
            snapshot: RefCell::new(None),
        }))
    }

    pub fn reset(&self) {
        self.offset.set(0.0);
        *self.snapshot.borrow_mut() = None;
        if self.active.peek() != 0 {
            self.active.set(0);
        }
    }

    pub fn active(&self) -> usize {
        self.active.get()
    }

    pub fn jump_to(&self, index: usize, rebuild: impl FnOnce()) {
        self.refresh_snapshot();
        let (top, _) = self
            .spans()
            .and_then(|spans| spans.get(index).copied())
            .unwrap_or((0.0, 0.0));

        let target = if index == 0 {
            0.0
        } else {
            (top - 16.0).max(0.0_f32)
        };
        scroll_animation::scroll_to(&self.identity(), target);
        self.active.set(index);
        rebuild();
    }

    fn identity(&self) -> String {
        format!("settings-anchors-{:p}", self)
    }

    fn refresh_snapshot(&self) {
        let container = self.container.peek();
        let rects: Vec<AbsoluteRect> = self.categories.iter().map(|c| c.peek()).collect();
        let mut snapshot = self.snapshot.borrow_mut();
        if snapshot
            .as_ref()
            .is_none_or(|(old, old_container, _)| *old != rects || *old_container != container)
        {
            *snapshot = Some((rects, container, self.offset.get()));
        }
    }

    fn spans(&self) -> Option<Vec<(f32, f32)>> {
        let snapshot = self.snapshot.borrow();
        let (rects, container, at) = snapshot.as_ref()?;
        Some(
            rects
                .iter()
                .map(|r| content_span(*r, *container, *at))
                .collect(),
        )
    }

    fn note_scroll(&self, offset: f32, max_offset: f32) {
        self.refresh_snapshot();
        self.offset.set(offset);

        if scroll_animation::is_programmatic() {
            return;
        }
        let Some(spans) = self.spans() else { return };
        let view = self
            .snapshot
            .borrow()
            .as_ref()
            .map_or(0.0, |(_, container, _)| container.height);
        let visible: Vec<usize> = spans
            .iter()
            .enumerate()
            .filter(|(_, (top, height))| {
                *height > 0.0 && *top - offset >= -0.5 && *top - offset + *height <= view + 0.5
            })
            .map(|(i, _)| i)
            .collect();
        let at_bottom = max_offset > 0.0 && offset >= max_offset - 0.5;
        let active = if at_bottom {
            visible.last()
        } else {
            visible.first()
        };
        if let Some(&active) = active {
            if self.active.peek() != active {
                self.active.set(active);
            }
        }
    }

    pub fn scroll_view(&'static self, class: &'static str, page: Node) -> Node {
        components::ScrollViewProps {
            initial_offset_y: self.offset.get(),
            on_scroll: (move |offset: f32, max: f32| self.note_scroll(offset, max)).into(),
            ..Default::default()
        }
        .build()
        .class(class)
        .dom_id(self.identity())
        .observe_rect(self.container.clone())
        .child(page)
    }

    pub fn anchored(&self, index: usize, node: Node) -> Node {
        node.observe_rect(self.categories[index].clone())
    }
}
