use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use winit::window::Window;

#[derive(Clone, Default)]
pub struct FrameWaker {
    inner: Arc<Inner>,
}

#[derive(Default)]
struct Inner {
    window: Mutex<Option<Arc<Window>>>,
    woken: AtomicBool,
}

impl FrameWaker {
    pub fn wake(&self) {
        self.inner.woken.store(true, Ordering::Release);
        if let Some(window) = self
            .inner
            .window
            .lock()
            .expect("frame waker mutex poisoned")
            .as_ref()
        {
            window.request_redraw();
        }
    }

    pub(crate) fn attach(&self, window: Option<Arc<Window>>) {
        *self
            .inner
            .window
            .lock()
            .expect("frame waker mutex poisoned") = window;
    }

    pub(crate) fn take_woken(&self) -> bool {
        self.inner.woken.swap(false, Ordering::AcqRel)
    }
}
