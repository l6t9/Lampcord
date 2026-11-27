use crate::classes::slides;
use crate::signal as crate_signal;
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::Instant;

use crate::node::Node;
use crate::signal::{Effect, Signal, signal};
use crate::timer;
use crate::widget::WidgetTree;

use super::container::div;

const MASS: f32 = 1.0;
const TENSION: f32 = 300.0;
const FRICTION: f32 = 28.0;

struct Anim {
    to: usize,
    dir: f32,
    from_height: f32,
    to_height: f32,
    pos: f32,
    vel: f32,
    last: Instant,
}

pub struct SlideShow {
    width: f32,
    current: Cell<usize>,
    active: Cell<usize>,
    slots: [Signal<Option<usize>>; 2],
    styles: [Signal<String>; 2],
    sizes: [Signal<(f32, f32)>; 2],
    host: Signal<String>,
    host_height: Cell<f32>,
    target: Signal<usize>,
    anim: RefCell<Option<Anim>>,
}

impl SlideShow {
    pub fn new(width: f32) -> Rc<Self> {
        Rc::new(Self {
            width,
            current: Cell::new(0),
            active: Cell::new(0),
            slots: [signal(Some(0)), signal(None)],
            styles: [signal(offset(0.0)), signal(offset(0.0))],
            sizes: [signal((0.0, 0.0)), signal((0.0, 0.0))],
            host: signal("height: 0px".to_string()),
            host_height: Cell::new(0.0),
            target: signal(0),
            anim: RefCell::new(None),
        })
    }

    pub fn reset(&self, index: usize) {
        *self.anim.borrow_mut() = None;
        self.current.set(index);
        self.active.set(0);

        self.target.set(0);
        self.slots[0].set(Some(index));
        self.slots[1].set(None);
        self.styles[0].set(offset(0.0));
        self.styles[1].set(offset(0.0));
        self.sizes[0].set((0.0, 0.0));
        self.sizes[1].set((0.0, 0.0));
        self.host_height.set(0.0);
        self.host.set(String::new());
    }

    pub fn index(&self) -> usize {
        self.anim
            .borrow()
            .as_ref()
            .map_or(self.current.get(), |anim| anim.to)
    }

    pub fn view(self: &Rc<Self>, step: impl Fn(usize) -> Node) -> Node {
        let _showing = self.index();
        let mut host = div()
            .class(slides::HOST)
            .bind_style(self.host.clone());
        for slot in 0..2 {
            let content = self.slots[slot].get().map(&step).unwrap_or_else(div);
            host = host.child(
                div()
                    .class(slides::SLOT)
                    .bind_style(self.styles[slot].clone())
                    .observe_size(self.sizes[slot].clone())
                    .child(content),
            );
        }
        host
    }

    pub fn go(self: &Rc<Self>, index: usize) {
        if self.anim.borrow().is_some() || self.current.get() == index {
            return;
        }
        let from_height = self.host_height.get();
        let leaving = self.active.get();
        let entering = 1 - leaving;
        let dir = if index > self.current.get() {
            1.0
        } else {
            -1.0
        };
        self.slots[entering].set(Some(index));
        self.styles[leaving].set(offset(0.0));
        self.styles[entering].set(offset(dir * self.width));
        self.target.set(entering);
        *self.anim.borrow_mut() = Some(Anim {
            to: index,
            dir,
            from_height,
            to_height: from_height,
            pos: 0.0,
            vel: 0.0,
            last: Instant::now(),
        });
        let show = self.clone();
        timer::request_animation_frame(move || show.tick());
    }

    pub fn effect(self: &Rc<Self>, tree: &mut WidgetTree) -> Effect {
        let show = self.clone();
        crate_signal::create_effect(move |_tree| show.follow(), tree)
    }

    fn follow(&self) {
        let slot = self.target.get();
        let (_, height) = self.sizes[slot].get();
        if height <= 0.0 {
            return;
        }
        let sliding = {
            let mut anim = self.anim.borrow_mut();
            if let Some(anim) = anim.as_mut() {
                anim.to_height = height;
                true
            } else {
                false
            }
        };
        if !sliding {
            self.set_height(height);
        }
    }

    fn set_height(&self, height: f32) {
        self.host_height.set(height);
        self.host.set(format!(
            "height: {height}px; overflow-x: hidden; overflow-y: hidden"
        ));
    }

    fn tick(self: &Rc<Self>) {
        let finished = {
            let mut anim = self.anim.borrow_mut();
            let Some(anim) = anim.as_mut() else { return };
            let now = Instant::now();
            let mut remaining = now.duration_since(anim.last).as_secs_f32().min(0.1);
            anim.last = now;
            while remaining > 0.0 {
                let dt = remaining.min(0.004);
                anim.vel += (-TENSION * (anim.pos - 1.0) - FRICTION * anim.vel) / MASS * dt;
                anim.pos += anim.vel * dt;
                remaining -= dt;
                if anim.pos >= 1.0 {
                    anim.pos = 1.0;
                    break;
                }
            }
            let leaving = self.active.get();
            let entering = 1 - leaving;
            self.styles[leaving].set(offset(-anim.dir * self.width * anim.pos));
            self.styles[entering].set(offset(anim.dir * self.width * (1.0 - anim.pos)));
            let height = anim.from_height + (anim.to_height - anim.from_height) * anim.pos;
            self.set_height(height);
            (anim.pos >= 1.0).then_some(anim.to)
        };
        match finished {
            Some(to) => {
                let leaving = self.active.get();
                self.slots[leaving].set(None);
                self.active.set(1 - leaving);
                self.current.set(to);
                *self.anim.borrow_mut() = None;
                let height = self.sizes[self.target.peek()].peek().1;
                if height > 0.0 {
                    self.set_height(height);
                }
            }
            None => {
                let show = self.clone();
                timer::request_animation_frame(move || show.tick());
            }
        }
    }
}

fn offset(x: f32) -> String {
    format!("left: {x}px")
}
