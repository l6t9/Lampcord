use std::cell::{Cell, RefCell};
use std::time::{Duration, Instant};

struct Pending {
    id: u64,
    deadline: Instant,
    callback: Box<dyn FnOnce()>,
}

thread_local! {
    static QUEUE: RefCell<Vec<Pending>> = const { RefCell::new(Vec::new()) };
    static NEXT_ID: Cell<u64> = const { Cell::new(1) };
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TimeoutHandle(u64);

pub fn set_timeout(delay: Duration, callback: impl FnOnce() + 'static) -> TimeoutHandle {
    set_deadline(Instant::now() + delay, callback)
}

pub fn set_deadline(deadline: Instant, callback: impl FnOnce() + 'static) -> TimeoutHandle {
    let id = NEXT_ID.with(|n| {
        let id = n.get();
        n.set(id + 1);
        id
    });
    QUEUE.with(|q| {
        q.borrow_mut().push(Pending {
            id,
            deadline,
            callback: Box::new(callback),
        })
    });
    TimeoutHandle(id)
}

pub fn request_animation_frame(callback: impl FnOnce() + 'static) -> TimeoutHandle {
    set_timeout(Duration::ZERO, callback)
}

pub fn clear_timeout(handle: TimeoutHandle) {
    QUEUE.with(|q| q.borrow_mut().retain(|p| p.id != handle.0));
}

pub fn run_due() -> bool {
    let now = Instant::now();
    let mut due: Vec<Pending> = QUEUE.with(|q| {
        let mut q = q.borrow_mut();
        let (due, rest): (Vec<_>, Vec<_>) = q.drain(..).partition(|p| p.deadline <= now);
        *q = rest;
        due
    });
    due.sort_by_key(|p| p.deadline);
    let ran = !due.is_empty();
    for pending in due {
        (pending.callback)();
    }
    ran
}

pub(crate) fn next_deadline() -> Option<Instant> {
    QUEUE.with(|q| q.borrow().iter().map(|p| p.deadline).min())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::rc::Rc;

    #[test]
    fn runs_only_due_and_respects_clear() {
        let hits = Rc::new(Cell::new(0));
        let h = hits.clone();
        set_timeout(Duration::ZERO, move || h.set(h.get() + 1));
        let h = hits.clone();
        let cancelled = set_timeout(Duration::ZERO, move || h.set(h.get() + 10));
        let h = hits.clone();
        set_timeout(Duration::from_secs(3600), move || h.set(h.get() + 100));
        clear_timeout(cancelled);
        assert!(run_due());
        assert_eq!(hits.get(), 1);
        assert!(next_deadline().is_some());
    }
}
