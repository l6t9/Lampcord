use crate::control;
use crate::widget;
use std::rc::Rc;

use crate::signal::Signal;
use crate::widget::{Widget, WidgetId, WidgetTree};
use crate::widget::primitives::BoxWidget;

#[derive(Clone)]
pub struct ClassBinding {
    pub signal: Signal<bool>,
    pub class_if_true: String,
    pub class_if_false: String,
}

#[derive(Clone)]
pub struct ImageBinding {
    pub tick: Signal<u64>,
    pub resolve: Rc<dyn Fn() -> Option<skia_safe::Image>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Retain {
    Tag(u64, u64),

    Reuse(u64, u64),
}

pub struct Node {
    pub tag: &'static str,
    pub classes: Vec<String>,
    pub dom_id: Option<String>,
    pub widget: Box<dyn Widget>,
    pub children: Vec<Node>,

    pub wants_focus: bool,

    pub wants_focus_scope: bool,

    pub class_binding: Option<ClassBinding>,

    pub image_binding: Option<ImageBinding>,

    pub text_binding: Option<Signal<String>>,

    pub inline_style: Option<String>,

    pub style_binding: Option<Signal<String>>,

    pub size_observer: Option<Signal<(f32, f32)>>,

    pub rect_observer: Option<Signal<widget::AbsoluteRect>>,

    pub reveal: Option<(String, f32)>,

    pub reactive: Option<Rc<dyn Fn() -> Node>>,

    pub context_menu: Option<ContextMenuHandler>,

    pub retain: Option<Retain>,

    pub focus_ring_host: bool,

    pub mount_hook: Option<MountHook>,
}

pub type MountHook = Rc<dyn Fn(&mut WidgetTree, WidgetId)>;

pub type ContextMenuHandler = Rc<dyn Fn(f32, f32)>;

impl Node {
    pub fn new(tag: &'static str, widget: impl Widget + 'static) -> Self {
        Self {
            tag,
            classes: Vec::new(),
            dom_id: None,
            widget: Box::new(widget),
            children: Vec::new(),
            wants_focus: false,
            wants_focus_scope: false,
            class_binding: None,
            image_binding: None,
            text_binding: None,
            inline_style: None,
            style_binding: None,
            size_observer: None,
            rect_observer: None,
            reveal: None,
            reactive: None,
            context_menu: None,
            retain: None,
            focus_ring_host: false,
            mount_hook: None,
        }
    }

    pub fn on_mount(mut self, hook: impl Fn(&mut WidgetTree, WidgetId) + 'static) -> Self {
        self.mount_hook = Some(Rc::new(hook));
        self
    }

    pub fn retained(mut self, key: u64, version: u64) -> Self {
        self.retain = Some(Retain::Tag(key, version));
        self
    }

    pub fn reuse(key: u64, version: u64) -> Self {
        let mut node = Node::new("div", BoxWidget);
        node.retain = Some(Retain::Reuse(key, version));
        node
    }

    pub fn class(mut self, class: impl Into<String>) -> Self {
        self.classes.push(class.into());
        self
    }

    pub fn classes(mut self, classes: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.classes.extend(classes.into_iter().map(Into::into));
        self
    }

    pub fn style(mut self, css: impl AsRef<str>) -> Self {
        let css = css.as_ref();
        self.inline_style = Some(match self.inline_style.take() {
            Some(existing) => format!("{existing}; {css}"),
            None => css.to_string(),
        });
        self
    }

    pub fn bind_style(mut self, style: Signal<String>) -> Self {
        self.style_binding = Some(style);
        self
    }

    pub fn observe_size(mut self, size: Signal<(f32, f32)>) -> Self {
        self.size_observer = Some(size);
        self
    }

    pub fn observe_rect(mut self, rect: Signal<widget::AbsoluteRect>) -> Self {
        self.rect_observer = Some(rect);
        self
    }

    pub fn reveal_when_requested(mut self, key: impl Into<String>, padding: f32) -> Self {
        self.reveal = Some((key.into(), padding));
        self
    }

    pub fn dom_id(mut self, id: impl Into<String>) -> Self {
        self.dom_id = Some(id.into());
        self
    }

    pub fn child(mut self, child: Node) -> Self {
        self.children.push(child);
        self
    }

    pub fn children(mut self, children: impl IntoIterator<Item = Node>) -> Self {
        self.children.extend(children);
        self
    }

    pub fn wants_focus(mut self) -> Self {
        self.wants_focus = true;
        self
    }

    pub fn focus_scope(mut self) -> Self {
        self.wants_focus_scope = true;
        self
    }

    pub fn focus_ring_host(mut self) -> Self {
        self.focus_ring_host = true;
        self
    }

    pub fn bind_class(
        mut self,
        signal: Signal<bool>,
        class_if_true: impl Into<String>,
        class_if_false: impl Into<String>,
    ) -> Self {
        self.class_binding = Some(ClassBinding {
            signal,
            class_if_true: class_if_true.into(),
            class_if_false: class_if_false.into(),
        });
        self
    }

    pub fn bind_image(
        mut self,
        tick: Signal<u64>,
        resolve: impl Fn() -> Option<skia_safe::Image> + 'static,
    ) -> Self {
        self.image_binding = Some(ImageBinding {
            tick,
            resolve: Rc::new(resolve),
        });
        self
    }

    #[track_caller]
    pub fn reactive(render: impl Fn() -> Node + 'static) -> Node {
        let mut node = Node::new("div", BoxWidget).style("display: flex; flex-direction: column");
        let origin = std::panic::Location::caller();
        node.reactive = Some(Rc::new(move || {
            control::set_current_origin(origin);
            render()
        }));
        node
    }

    pub fn on_context_menu(mut self, handler: impl Fn(f32, f32) + 'static) -> Self {
        self.context_menu = Some(Rc::new(handler));
        self
    }

    pub fn bind_text(mut self, text: Signal<String>) -> Self {
        self.text_binding = Some(text);
        self
    }
}

impl Default for Node {
    fn default() -> Self {
        Node::new("div", BoxWidget)
    }
}

impl From<Vec<Node>> for Node {
    fn from(mut nodes: Vec<Node>) -> Self {
        match nodes.len() {
            0 => Node::default(),
            1 => nodes.pop().expect("len() == 1"),
            _ => Node::new("div", BoxWidget).children(nodes),
        }
    }
}
