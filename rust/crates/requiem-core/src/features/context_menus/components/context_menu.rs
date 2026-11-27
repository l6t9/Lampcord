use requiem_ui::clipboard;
use requiem_ui::components::ModalSize;
use requiem_ui::components;
use requiem_ui::timer;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use requiem_ui::components::{ConfirmModalProps, ContextItem, div};
use requiem_ui::control::section;
use requiem_ui::node::Node;
use requiem_ui::signal::{Effect, Signal, signal};
use requiem_ui::widget::anim::ToggleState;
use requiem_ui::widget::{WidgetId, WidgetTree};

use crate::models::Id;
use crate::models::id::marker::{ChannelMarker, GuildMarker, UserMarker};
use requiem_ui::components::toast::{ToastKind, ToastState};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EditorCommand {
    Cut,
    Copy,
    Paste,
    SelectAll,
    Undo,
    Redo,

    Focus,
}

#[derive(Clone)]
pub struct ConfirmRequest {
    pub title: String,
    pub body: String,
    pub confirm_label: String,
    pub on_confirm: Rc<dyn Fn()>,

    pub critical: bool,

    pub content: Option<Rc<dyn Fn() -> Node>>,
}

#[derive(Clone)]
pub struct Actions {
    pub toasts: ToastState,

    pub open_profile: Rc<dyn Fn(Id<UserMarker>, Option<Id<GuildMarker>>, bool)>,

    pub confirm_block: Rc<dyn Fn(Id<UserMarker>, String, &'static str)>,

    pub open_settings: Rc<dyn Fn(&str)>,

    pub open_guild: Rc<dyn Fn(Id<GuildMarker>)>,

    pub open_media: Rc<dyn Fn(String)>,
    pub log_out: Rc<dyn Fn()>,

    pub dm_channel: Rc<dyn Fn(Id<UserMarker>) -> Option<Id<ChannelMarker>>>,

    pub editor_command: Rc<dyn Fn(EditorCommand)>,
}

impl Default for Actions {
    fn default() -> Self {
        Self {
            toasts: ToastState::new(),
            open_profile: Rc::new(|_, _, _| {}),
            confirm_block: Rc::new(|_, _, _| {}),
            open_settings: Rc::new(|_| {}),
            open_guild: Rc::new(|_| {}),
            open_media: Rc::new(|_| {}),
            log_out: Rc::new(|| {}),
            dm_channel: Rc::new(|_| None),
            editor_command: Rc::new(|_| {}),
        }
    }
}

struct Open {
    x: f32,
    y: f32,

    above: bool,
    items: Box<dyn Fn() -> Vec<ContextItem>>,
}

thread_local! {
    static OPEN: Signal<Option<Rc<Open>>> = signal(None);
    static ACTIONS: RefCell<Actions> = RefCell::new(Actions::default());
    static WEB_BASE_URL: RefCell<String> = const { RefCell::new(String::new()) };
    static CONFIRM: Signal<Option<ConfirmRequest>> = signal(None);
    static CONFIRM_OPEN: Signal<ToggleState> = signal(ToggleState::new(false));
}

pub fn open(x: f32, y: f32, items: impl Fn() -> Vec<ContextItem> + 'static) {
    OPEN.with(|open| {
        open.set(Some(Rc::new(Open {
            x,
            y,
            above: false,
            items: Box::new(items),
        })))
    });
}

pub fn open_above(x: f32, y: f32, items: impl Fn() -> Vec<ContextItem> + 'static) {
    OPEN.with(|open| {
        open.set(Some(Rc::new(Open {
            x,
            y,
            above: true,
            items: Box::new(items),
        })))
    });
}

pub fn origin_tracked() -> Option<(f32, f32)> {
    OPEN.with(|open| open.get().map(|open| (open.x, open.y)))
}

pub fn origin() -> Option<(f32, f32)> {
    OPEN.with(|open| open.peek().map(|open| (open.x, open.y)))
}

pub fn close() {
    OPEN.with(|open| {
        if open.peek().is_some() {
            open.set(None);
        }
    });
}

pub fn is_open() -> bool {
    OPEN.with(|open| open.peek().is_some())
}

pub fn actions() -> Actions {
    ACTIONS.with(|a| a.borrow().clone())
}

pub fn set_web_base_url(url: String) {
    WEB_BASE_URL.with(|u| *u.borrow_mut() = url);
}

pub fn web_base_url() -> String {
    WEB_BASE_URL.with(|u| u.borrow().clone())
}

pub fn notify(message: &str) {
    actions().toasts.show(message, ToastKind::Success);
}

pub fn notify_failure(message: &str) {
    actions().toasts.show(message, ToastKind::Failure);
}

pub fn copy_with_toast(text: impl Into<String>, success: &str) {
    if clipboard::set_text(text) {
        notify(success);
    } else {
        notify_failure(&requiem_i18n::t!("context-menu-unable-to-copy"));
    }
}

thread_local! {
    /// Cancel button text for the open confirm, when it isn't the default "Cancel".
    static CONFIRM_CANCEL: RefCell<Option<String>> = const { RefCell::new(None) };
}

/// Like [`confirm`], with its own text on the cancel button.
pub fn confirm_labeled(request: ConfirmRequest, cancel_label: String) {
    CONFIRM_CANCEL.with(|c| *c.borrow_mut() = Some(cancel_label));
    confirm(request);
}

pub fn confirm(request: ConfirmRequest) {
    CONFIRM.with(|c| c.set(Some(request)));
    CONFIRM_OPEN.with(|open| open.update(|t| *t = t.set(true)));
}

const CONFIRM_CLOSE_GRACE: Duration = Duration::from_millis(350);

fn close_confirm() {
    CONFIRM_OPEN.with(|open| open.update(|t| *t = t.set(false)));
    timer::set_timeout(CONFIRM_CLOSE_GRACE, || {
        if !CONFIRM_OPEN.with(|open| open.peek().on) {
            CONFIRM_CANCEL.with(|c| *c.borrow_mut() = None);
            CONFIRM.with(|c| c.set(None));
        }
    });
}

fn build_confirm(request: &ConfirmRequest, open: ToggleState) -> Node {
    let on_confirm = request.on_confirm.clone();
    let mut props = ConfirmModalProps::new(
        request.title.clone(),
        request.confirm_label.clone(),
        close_confirm,
        open,
    );
    props = match &request.content {
        Some(content) => props
            .subtitle(request.body.clone())
            .content(content())
            .size(ModalSize::Medium),
        None => props.message(request.body.clone()),
    };
    if let Some(label) = CONFIRM_CANCEL.with(|c| c.borrow().clone()) {
        props = props.cancel_label(label);
    }
    if request.critical {
        props = props.critical();
    }
    props.on_confirm(move || on_confirm()).build()
}

pub fn install(actions: Actions) {
    ACTIONS.with(|a| *a.borrow_mut() = actions);
}

pub fn mount(tree: &mut WidgetTree, parent: WidgetId) -> Vec<Effect> {
    let menu = section(tree, parent, move || match OPEN.with(Signal::get) {
        Some(menu) => {
            let on_dismiss: Rc<dyn Fn()> = Rc::new(close);
            components::context_menu_layer_at(
                (menu.x, menu.y),
                menu.above,
                (menu.items)(),
                on_dismiss,
            )
        }
        None => div(),
    });

    let confirm = section(tree, parent, move || match CONFIRM.with(Signal::get) {
        Some(request) => build_confirm(&request, CONFIRM_OPEN.with(Signal::get)),
        None => div(),
    });
    vec![menu, confirm]
}
