use std::cell::RefCell;
use std::rc::Rc;

struct Popout {
    name: &'static str,
    is_open: Rc<dyn Fn() -> bool>,
    close: Rc<dyn Fn()>,

    was_open: bool,
}

thread_local! {
    static REGISTRY: RefCell<Vec<Popout>> = const { RefCell::new(Vec::new()) };
}

pub fn register(
    name: &'static str,
    is_open: impl Fn() -> bool + 'static,
    close: impl Fn() + 'static,
) {
    REGISTRY.with(|r| {
        r.borrow_mut().push(Popout {
            name,
            is_open: Rc::new(is_open),
            close: Rc::new(close),
            was_open: false,
        })
    });
}

pub fn close_open() -> bool {
    let entries: Vec<(Rc<dyn Fn() -> bool>, Rc<dyn Fn()>)> = REGISTRY.with(|r| {
        r.borrow()
            .iter()
            .map(|p| (p.is_open.clone(), p.close.clone()))
            .collect()
    });
    let mut closed = false;
    for (is_open, close) in entries {
        if is_open() {
            close();
            closed = true;
        }
    }
    if closed {
        REGISTRY.with(|r| r.borrow_mut().iter_mut().for_each(|p| p.was_open = false));
    }
    closed
}

pub fn reconcile() {
    let entries: Vec<(&'static str, Rc<dyn Fn() -> bool>, Rc<dyn Fn()>, bool)> =
        REGISTRY.with(|r| {
            r.borrow()
                .iter()
                .map(|p| (p.name, p.is_open.clone(), p.close.clone(), p.was_open))
                .collect()
        });
    let open: Vec<bool> = entries.iter().map(|(_, is_open, _, _)| is_open()).collect();
    let someone_opened = entries
        .iter()
        .zip(&open)
        .any(|((_, _, _, was), now)| *now && !*was);
    let mut still_open = open.clone();
    if someone_opened {
        for (index, (name, _, close, was)) in entries.iter().enumerate() {
            if open[index] && *was {
                log::debug!("popout {name:?} closed: another one opened");
                close();
                still_open[index] = false;
            }
        }
    }
    REGISTRY.with(|r| {
        for (popout, now) in r.borrow_mut().iter_mut().zip(still_open) {
            popout.was_open = now;
        }
    });
}
