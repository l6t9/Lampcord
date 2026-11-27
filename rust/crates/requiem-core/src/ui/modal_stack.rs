use requiem_ui::widget::anim::ToggleState;
use std::cell::RefCell;
use std::rc::Rc;

thread_local! {
    static MODALS: RefCell<Vec<(&'static str, Rc<dyn Fn() -> ToggleState>)>> = const { RefCell::new(Vec::new()) };
}

pub fn register(id: &'static str, read: impl Fn() -> ToggleState + 'static) {
    MODALS.with(|modals| {
        let mut modals = modals.borrow_mut();
        modals.retain(|(existing, _)| *existing != id);
        modals.push((id, Rc::new(read)));
    });
}

pub fn effective(id: &'static str, raw: ToggleState) -> ToggleState {
    let readers: Vec<(&'static str, Rc<dyn Fn() -> ToggleState>)> =
        MODALS.with(|modals| modals.borrow().clone());
    let states: Vec<(&'static str, ToggleState)> = readers
        .iter()
        .map(|(other, read)| (*other, if *other == id { raw } else { read() }))
        .collect();
    if !raw.on {
        return raw;
    }

    let top = states
        .iter()
        .filter(|(_, state)| state.on)
        .max_by_key(|(_, state)| state.changed_at)
        .map(|(top, _)| *top);

    let latest_other = states
        .iter()
        .filter(|(other, _)| *other != id)
        .map(|(_, state)| state.changed_at)
        .max();
    match (top, latest_other) {
        (Some(top), Some(at)) if top != id => ToggleState {
            on: false,
            changed_at: at.max(raw.changed_at),
        },
        (_, Some(at)) if at > raw.changed_at => ToggleState {
            on: true,
            changed_at: at,
        },
        _ => raw,
    }
}
