use requiem_ui::timer;
use std::cell::Cell;
use std::rc::Rc;
use std::time::Duration;

use requiem_ui::signal::{Signal, signal};
use requiem_ui::widget::anim::ToggleState;

const CLOSE_GRACE: Duration = Duration::from_millis(350);

#[derive(Clone)]
pub struct Dialog {
    toggle: Signal<ToggleState>,
    mounted: Signal<bool>,

    closing: Rc<Cell<u64>>,
}

impl Dialog {
    pub fn new() -> Self {
        Self {
            toggle: signal(ToggleState::new(false)),
            mounted: signal(false),
            closing: Rc::new(Cell::new(0)),
        }
    }

    pub fn is_open(&self) -> bool {
        self.toggle.peek().on
    }

    pub fn toggle(&self) -> ToggleState {
        self.toggle.get()
    }

    pub fn is_mounted(&self) -> bool {
        self.mounted.get()
    }

    pub fn open(&self) {
        self.mounted.set(true);
        self.toggle.update(|t| *t = t.set(true));
    }

    pub fn close(&self) {
        self.close_then(|| {});
    }

    pub fn close_then(&self, after: impl FnOnce() + 'static) {
        self.toggle.update(|t| *t = t.set(false));
        self.closing.set(self.closing.get() + 1);
        let ticket = self.closing.get();
        let this = self.clone();
        timer::set_timeout(CLOSE_GRACE, move || {
            if this.closing.get() == ticket && !this.is_open() {
                this.mounted.set(false);
                after();
            }
        });
    }
}

impl Default for Dialog {
    fn default() -> Self {
        Self::new()
    }
}
