use crate::paint;
use crate::perf::Phase;
use crate::perf;
use std::any::Any;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::fmt as std_fmt;
use std::hash;
use std::mem;
use std::path;
use std::rc::{Rc, Weak};

use crate::widget::{Widget, WidgetId, WidgetTree};

thread_local! {

    static RUNNING: RefCell<Vec<Rc<EffectState>>> = const { RefCell::new(Vec::new()) };

    static PENDING: RefCell<HashMap<u64, Weak<EffectState>>> = RefCell::new(HashMap::new());
    static NEXT_EFFECT_ID: Cell<u64> = const { Cell::new(0) };

    static ACTIVE_OWNER: Cell<u64> = const { Cell::new(0) };
}

pub fn untrack<R>(f: impl FnOnce() -> R) -> R {
    let saved = RUNNING.with(|stack| mem::take(&mut *stack.borrow_mut()));
    let out = f();
    RUNNING.with(|stack| *stack.borrow_mut() = saved);
    out
}

pub fn set_owner(owner: u64) -> u64 {
    ACTIVE_OWNER.with(|o| o.replace(owner))
}

fn next_effect_id() -> u64 {
    NEXT_EFFECT_ID.with(|next| {
        let id = next.get();
        next.set(id + 1);
        id
    })
}

struct EffectState {
    id: u64,

    run: Box<dyn Fn(&mut WidgetTree)>,

    generation: Cell<u64>,

    owner: u64,
}

fn run_effect(effect: &Rc<EffectState>, tree: &mut WidgetTree) {
    effect.generation.set(effect.generation.get() + 1);
    RUNNING.with(|stack| stack.borrow_mut().push(effect.clone()));
    (effect.run)(tree);
    RUNNING.with(|stack| {
        stack.borrow_mut().pop();
    });
}

#[derive(Clone)]
#[allow(
    dead_code,
    reason = "the Rc is kept alive for its Drop/refcount side effect, never read directly"
)]
pub struct Effect(Rc<EffectState>);

pub fn create_effect(f: impl Fn(&mut WidgetTree) + 'static, tree: &mut WidgetTree) -> Effect {
    let effect = Rc::new(EffectState {
        id: next_effect_id(),
        run: Box::new(f),
        generation: Cell::new(0),
        owner: ACTIVE_OWNER.with(Cell::get),
    });
    run_effect(&effect, tree);
    Effect(effect)
}

pub fn flush(tree: &mut WidgetTree) -> bool {
    perf::time(Phase::Flush, || flush_pending(tree))
}

fn flush_pending(tree: &mut WidgetTree) -> bool {
    let mut ran_any = false;

    let mut foreign: Vec<(u64, Weak<EffectState>)> = Vec::new();
    let previous_owner = set_owner(tree.owner);
    loop {
        let batch: Vec<Weak<EffectState>> = PENDING.with(|pending| {
            let mut pending = pending.borrow_mut();
            if pending.is_empty() {
                return Vec::new();
            }
            pending.drain().map(|(_, effect)| effect).collect()
        });
        if batch.is_empty() {
            break;
        }

        for effect in batch {
            if let Some(effect) = effect.upgrade() {
                if effect.owner != tree.owner {
                    foreign.push((effect.id, Rc::downgrade(&effect)));
                    continue;
                }
                ran_any = true;
                run_effect(&effect, tree);
            }
        }
    }
    set_owner(previous_owner);
    PENDING.with(|pending| pending.borrow_mut().extend(foreign));
    if ran_any {
        paint::apply_focus_claim(tree, false);
    }
    ran_any
}

struct SignalInner<T> {
    value: T,

    subscribers: Vec<(Weak<EffectState>, u64)>,
}

pub struct Signal<T>(Rc<RefCell<SignalInner<T>>>);

impl<T> Clone for Signal<T> {
    fn clone(&self) -> Self {
        Signal(self.0.clone())
    }
}

impl<T> std_fmt::Debug for Signal<T> {
    fn fmt(&self, f: &mut std_fmt::Formatter<'_>) -> std_fmt::Result {
        f.write_str("Signal(..)")
    }
}

impl<T> Signal<T> {
    pub fn new(value: T) -> Self {
        Signal(Rc::new(RefCell::new(SignalInner {
            value,
            subscribers: Vec::new(),
        })))
    }

    fn track(&self) {
        RUNNING.with(|stack| {
            let Some(effect) = stack.borrow().last().cloned() else {
                return;
            };
            let generation = effect.generation.get();
            let mut inner = self.0.borrow_mut();
            match inner
                .subscribers
                .iter_mut()
                .find(|(weak, _)| weak.upgrade().is_some_and(|e| e.id == effect.id))
            {
                Some(entry) => entry.1 = generation,
                None => inner.subscribers.push((Rc::downgrade(&effect), generation)),
            }
        });
    }

    pub fn with<R>(&self, f: impl FnOnce(&T) -> R) -> R {
        self.track();
        f(&self.0.borrow().value)
    }

    pub fn with_untracked<R>(&self, f: impl FnOnce(&T) -> R) -> R {
        f(&self.0.borrow().value)
    }

    #[track_caller]
    pub fn set(&self, value: T) {
        if perf::profile_paint() {
            let at = std::panic::Location::caller();
            perf::record_widget_paint(
                format!(
                    "SIGNAL-SET {}:{}",
                    at.file()
                        .rsplit(['/', path::MAIN_SEPARATOR])
                        .next()
                        .unwrap_or(""),
                    at.line()
                ),
                0.05,
            );
        }
        self.0.borrow_mut().value = value;
        self.schedule_subscribers();
    }

    #[track_caller]
    pub fn update(&self, f: impl FnOnce(&mut T)) {
        if perf::profile_paint() {
            let at = std::panic::Location::caller();
            perf::record_widget_paint(
                format!(
                    "SIGNAL-SET {}:{}",
                    at.file()
                        .rsplit(['/', path::MAIN_SEPARATOR])
                        .next()
                        .unwrap_or(""),
                    at.line()
                ),
                0.05,
            );
        }
        f(&mut self.0.borrow_mut().value);
        self.schedule_subscribers();
    }

    /// Changes the value without waking anyone. Pair it with `notify` or a narrower signal.
    pub fn update_silent(&self, f: impl FnOnce(&mut T)) {
        f(&mut self.0.borrow_mut().value);
    }

    /// Wakes everything that read this signal.
    pub fn notify(&self) {
        self.schedule_subscribers();
    }

    fn schedule_subscribers(&self) {
        let mut inner = self.0.borrow_mut();

        inner.subscribers.retain(|(weak, generation)| {
            weak.upgrade()
                .is_some_and(|e| e.generation.get() == *generation)
        });
        let live: Vec<Rc<EffectState>> = inner
            .subscribers
            .iter()
            .filter_map(|(weak, _)| weak.upgrade())
            .collect();
        drop(inner);
        PENDING.with(|pending| {
            let mut pending = pending.borrow_mut();
            for effect in live {
                pending.insert(effect.id, Rc::downgrade(&effect));
            }
        });
    }
}

impl<T: Clone> Signal<T> {
    pub fn get(&self) -> T {
        self.with(Clone::clone)
    }

    pub fn holders(&self) -> usize {
        Rc::strong_count(&self.0)
    }

    pub fn peek(&self) -> T {
        self.with_untracked(Clone::clone)
    }
}

pub fn signal<T>(value: T) -> Signal<T> {
    Signal::new(value)
}

/// A map that readers can follow one key at a time. `watch(&key)` subscribes to that key
/// only, so a change under another key leaves the reader alone. Readers that need the whole
/// map still use `with`/`get`, and `update_key` wakes them as well.
pub struct KeyedSignal<K, V> {
    map: Signal<HashMap<K, V>>,
    keys: Rc<RefCell<HashMap<K, Signal<()>>>>,
}

impl<K, V> Clone for KeyedSignal<K, V> {
    fn clone(&self) -> Self {
        Self {
            map: self.map.clone(),
            keys: self.keys.clone(),
        }
    }
}

impl<K: hash::Hash + Eq + Clone, V> KeyedSignal<K, V> {
    pub fn new() -> Self {
        Self {
            map: Signal::new(HashMap::new()),
            keys: Rc::default(),
        }
    }

    /// Tracks `key` and nothing else.
    pub fn watch(&self, key: &K) {
        let watched = self
            .keys
            .borrow_mut()
            .entry(key.clone())
            .or_insert_with(|| Signal::new(()))
            .clone();
        watched.with(|_| ());
    }

    /// Reads the whole map, tracked on every key.
    pub fn with<R>(&self, f: impl FnOnce(&HashMap<K, V>) -> R) -> R {
        self.map.with(f)
    }

    pub fn with_untracked<R>(&self, f: impl FnOnce(&HashMap<K, V>) -> R) -> R {
        self.map.with_untracked(f)
    }

    /// Changes what is stored under `key` and wakes that key's readers and the whole-map readers.
    pub fn update_key(&self, key: &K, f: impl FnOnce(&mut HashMap<K, V>)) {
        self.map.update_silent(f);
        let watched = self.keys.borrow().get(key).cloned();
        if let Some(watched) = watched {
            watched.set(());
        }
        self.map.notify();
    }

    /// Changes the map in ways that can touch any key; wakes every reader.
    pub fn update(&self, f: impl FnOnce(&mut HashMap<K, V>)) {
        self.map.update_silent(f);
        self.wake_all();
    }

    pub fn set(&self, value: HashMap<K, V>) {
        self.map.update_silent(|current| *current = value);
        self.wake_all();
    }

    fn wake_all(&self) {
        let watched: Vec<Signal<()>> = self.keys.borrow().values().cloned().collect();
        for watched in watched {
            watched.set(());
        }
        self.map.notify();
    }
}

impl<K: hash::Hash + Eq + Clone, V> Default for KeyedSignal<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K: hash::Hash + Eq + Clone, V: Clone> KeyedSignal<K, V> {
    pub fn get(&self) -> HashMap<K, V> {
        self.map.get()
    }

    pub fn peek(&self) -> HashMap<K, V> {
        self.map.peek()
    }
}

pub struct Memo<T> {
    signal: Signal<Option<T>>,
    _effect: Effect,
}

impl<T: Clone> Memo<T> {
    pub fn get(&self) -> T {
        self.signal
            .with(|value| value.clone())
            .expect("Memo's effect always runs once, synchronously, before computed() returns")
    }
}

pub fn computed<T: Clone + 'static>(f: impl Fn() -> T + 'static, tree: &mut WidgetTree) -> Memo<T> {
    let signal = Signal::new(None);
    let write = signal.clone();
    let effect = create_effect(
        move |_tree| {
            write.set(Some(f()));
        },
        tree,
    );
    Memo {
        signal,
        _effect: effect,
    }
}

pub fn bind<W: Widget, T: Clone + 'static>(
    tree: &mut WidgetTree,
    id: WidgetId,
    sig: Signal<T>,
    apply: impl Fn(&mut W, T) + 'static,
) -> Effect {
    create_effect(
        move |tree| {
            let value = sig.get();
            if let Some(node) = tree.nodes.get_mut(id) {
                if let Some(widget) = (&mut *node.widget as &mut dyn Any).downcast_mut::<W>() {
                    apply(widget, value);
                    tree.mark_dirty(id);
                }
            }
        },
        tree,
    )
}

#[cfg(test)]
mod tests {
    use crate::font_system::make_font_system;
    use crate::glyph_cache::GlyphCache;
    use crate::widget::primitives::TextWidget;
    use skia_safe::Color;
    use std::cell::RefCell;
    use std::rc::Rc;
    use super::*;
    use crate::style::ComputedStyle;
    use crate::widget::primitives::BoxWidget;

    fn test_tree() -> WidgetTree {
        WidgetTree::new(Box::new(BoxWidget), ComputedStyle::default())
    }

    #[test]
    fn peek_does_not_subscribe_the_running_effect() {
        let mut tree = test_tree();
        let count = signal(0);
        let runs = Rc::new(RefCell::new(0));
        let (runs_write, count_read) = (runs.clone(), count.clone());
        let _effect = create_effect(
            move |_tree| {
                *runs_write.borrow_mut() += 1;
                count_read.peek();
            },
            &mut tree,
        );
        assert_eq!(*runs.borrow(), 1);

        count.set(1);
        flush(&mut tree);
        assert_eq!(
            *runs.borrow(),
            1,
            "peek() must not schedule a rerun when the peeked signal is written"
        );
    }

    #[test]
    fn effect_runs_immediately_then_reruns_only_after_flush() {
        let mut tree = test_tree();
        let count = signal(0);
        let seen = Rc::new(RefCell::new(Vec::new()));
        let (seen_write, count_read) = (seen.clone(), count.clone());
        let _effect = create_effect(
            move |_tree| seen_write.borrow_mut().push(count_read.get()),
            &mut tree,
        );
        assert_eq!(
            *seen.borrow(),
            vec![0],
            "runs once immediately, synchronously"
        );

        count.set(1);
        assert_eq!(
            *seen.borrow(),
            vec![0],
            "set() alone must not rerun subscribers"
        );
        flush(&mut tree);
        assert_eq!(
            *seen.borrow(),
            vec![0, 1],
            "flush() reruns exactly the subscribed effect"
        );
    }

    #[test]
    fn dropping_the_effect_handle_stops_future_reruns() {
        let mut tree = test_tree();
        let count = signal(0);
        let runs = Rc::new(RefCell::new(0));
        let (runs_write, count_read) = (runs.clone(), count.clone());
        let effect = create_effect(
            move |_tree| {
                *runs_write.borrow_mut() += 1;
                count_read.get();
            },
            &mut tree,
        );
        assert_eq!(*runs.borrow(), 1);
        drop(effect);
        count.set(1);
        flush(&mut tree);
        assert_eq!(
            *runs.borrow(),
            1,
            "no strong Effect handle left, so it must not run again"
        );
    }

    #[test]
    fn update_mutates_in_place_and_still_schedules() {
        let mut tree = test_tree();
        let items = signal(Vec::<i32>::new());
        let seen_len = Rc::new(RefCell::new(0));
        let (seen_write, items_read) = (seen_len.clone(), items.clone());
        let _effect = create_effect(
            move |_tree| *seen_write.borrow_mut() = items_read.with(Vec::len),
            &mut tree,
        );
        assert_eq!(*seen_len.borrow(), 0);

        items.update(|v| v.push(1));
        flush(&mut tree);
        assert_eq!(*seen_len.borrow(), 1);
    }

    #[test]
    fn dropping_an_effect_cancels_its_queued_run() {
        let mut tree = test_tree();
        let sig = Signal::new(0u64);
        let runs = Rc::new(Cell::new(0));
        let (read, runs_write) = (sig.clone(), runs.clone());
        let effect = create_effect(
            move |_| {
                let _ = read.get();
                runs_write.set(runs_write.get() + 1);
            },
            &mut tree,
        );
        assert_eq!(runs.get(), 1);

        sig.set(1);
        drop(effect);
        assert!(!flush(&mut tree), "nothing live was left to run");
        assert_eq!(
            runs.get(),
            1,
            "a dropped effect never runs again, even if it was already queued"
        );
    }

    #[test]
    fn effect_stops_rerunning_for_a_signal_it_no_longer_reads() {
        let mut tree = test_tree();
        let open = Signal::new(true);
        let tick = Signal::new(0u64);
        let runs = Rc::new(Cell::new(0));
        let (open_read, tick_read, runs_write) = (open.clone(), tick.clone(), runs.clone());
        let _effect = create_effect(
            move |_| {
                runs_write.set(runs_write.get() + 1);
                if open_read.get() {
                    let _ = tick_read.get();
                }
            },
            &mut tree,
        );
        assert_eq!(runs.get(), 1);

        tick.set(1);
        flush(&mut tree);
        assert_eq!(runs.get(), 2, "reads tick while open");

        open.set(false);
        flush(&mut tree);
        assert_eq!(runs.get(), 3);

        tick.set(2);
        flush(&mut tree);
        assert_eq!(
            runs.get(),
            3,
            "closed: the last run didn't read tick, so tick no longer reruns it"
        );
    }

    #[test]
    fn computed_recomputes_when_its_dependency_changes() {
        let mut tree = test_tree();
        let base = signal(2);
        let base_read = base.clone();
        let doubled = computed(move || base_read.get() * 2, &mut tree);
        assert_eq!(doubled.get(), 4);

        base.set(5);
        flush(&mut tree);
        assert_eq!(doubled.get(), 10);
    }

    #[test]
    fn bind_mutates_the_mounted_widget_in_place_without_remounting() {
        let mut tree = test_tree();
        let root = tree.root;
        let font_system = Rc::new(RefCell::new(make_font_system()));
        let glyph_cache = Rc::new(GlyphCache::new(1024 * 1024));
        let id = tree.add_child(
            root,
            "span",
            Box::new(TextWidget::new(
                "initial",
                16.0,
                Color::WHITE,
                font_system,
                glyph_cache,
            )),
            ComputedStyle::default(),
        );

        let content = signal("initial".to_string());
        let content_read = content.clone();
        let _effect = bind(
            &mut tree,
            id,
            content_read,
            |widget: &mut TextWidget, value: String| widget.set_text(value),
        );

        content.set("updated".to_string());
        flush(&mut tree);

        assert_eq!(tree.nodes[root].children, vec![id]);
    }

    #[test]
    fn unrelated_signal_write_does_not_schedule_effect() {
        let mut tree = test_tree();
        let watched = signal(0);
        let unwatched = signal(0);
        let runs = Rc::new(RefCell::new(0));
        let (runs_write, watched_read) = (runs.clone(), watched.clone());
        let _effect = create_effect(
            move |_tree| {
                *runs_write.borrow_mut() += 1;
                watched_read.get();
            },
            &mut tree,
        );
        assert_eq!(*runs.borrow(), 1);

        unwatched.set(42);
        flush(&mut tree);
        assert_eq!(
            *runs.borrow(),
            1,
            "an effect must not rerun for a signal it never read"
        );
    }
}
