use crate::perf;
use crate::signal;
use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::panic;
use std::path;
use std::rc::Rc;
use std::time::Duration;
use std::time::Instant;

use crate::node::Node;
use crate::signal::{Effect, Signal, create_effect};
use crate::widget::{WidgetId, WidgetTree};

fn recompute_if_styled(tree: &mut WidgetTree) {
    tree.invalidate_styles();
}

pub fn show(
    tree: &mut WidgetTree,
    parent: WidgetId,
    cond: impl Fn() -> bool + 'static,
    render: impl Fn() -> Node + 'static,
) -> Effect {
    let mounted: Rc<Cell<Option<WidgetId>>> = Rc::new(Cell::new(None));
    create_effect(
        move |tree| {
            match (cond(), mounted.get()) {
                (true, None) => mounted.set(Some(tree.mount(parent, render()))),
                (false, Some(id)) => {
                    tree.unmount(id);
                    mounted.set(None);
                }
                _ => return,
            }
            recompute_if_styled(tree);
        },
        tree,
    )
}

thread_local! {

    static CURRENT_ORIGIN: Cell<Option<&'static panic::Location<'static>>> = const { Cell::new(None) };

    static REBUILDS: RefCell<HashMap<usize, (Instant, u32, &'static panic::Location<'static>)>> = RefCell::new(HashMap::new());
}

thread_local! {

    static AVAILABLE_RETAINED: RefCell<HashSet<(u64, u64)>> = RefCell::new(HashSet::new());
}

pub fn retained_available(key: u64, version: u64) -> bool {
    AVAILABLE_RETAINED.with(|set| set.borrow().contains(&(key, version)))
}

pub fn locale_epoch() -> u64 {
    LOCALE_EPOCH.with(|epoch| epoch.peek())
}

pub fn track_locale() {
    LOCALE_EPOCH.with(Signal::get);
}

pub(crate) fn set_current_origin(origin: &'static panic::Location<'static>) {
    CURRENT_ORIGIN.with(|c| c.set(Some(origin)));
}

const HOT_REBUILDS_PER_SECOND: u32 = 60;

fn note_rebuild(origin: &'static panic::Location<'static>) {
    REBUILDS.with(|rebuilds| {
        let mut rebuilds = rebuilds.borrow_mut();
        let now = Instant::now();
        let entry = rebuilds.entry(origin as *const _ as usize).or_insert((now, 0, origin));
        entry.1 += 1;
        if now.duration_since(entry.0) >= Duration::from_secs(1) {
            if entry.1 >= HOT_REBUILDS_PER_SECOND {
                log::warn!(target: "requiem_ui::rebuild", "{}:{} rebuilt {} times in {:.1}s: it reads a signal that changes far more often than it needs to", entry.2.file().rsplit(['/', path::MAIN_SEPARATOR]).next().unwrap_or(""), entry.2.line(), entry.1, now.duration_since(entry.0).as_secs_f64());
            }
            *entry = (now, 0, origin);
        }
    });
}

thread_local! {

    static LOCALE_EPOCH: Signal<u64> = signal::signal(0);
}

pub fn locale_changed() {
    LOCALE_EPOCH.with(|epoch| epoch.update(|n| *n += 1));
}

#[track_caller]
pub fn section(
    tree: &mut WidgetTree,
    parent: WidgetId,
    render: impl Fn() -> Node + 'static,
) -> Effect {
    section_at(panic::Location::caller(), tree, parent, render)
}

pub fn section_at(
    origin: &'static panic::Location<'static>,
    tree: &mut WidgetTree,
    parent: WidgetId,
    render: impl Fn() -> Node + 'static,
) -> Effect {
    let mounted: Rc<Cell<Option<WidgetId>>> = Rc::new(Cell::new(None));

    let index: Rc<Cell<Option<usize>>> = Rc::new(Cell::new(None));
    create_effect(
        move |tree| {
            LOCALE_EPOCH.with(Signal::get);
            let mut focus = None;
            let is_rerun = mounted.get().is_some();

            let available = mounted
                .get()
                .map(|old| tree.collect_retained(old))
                .unwrap_or_default();
            AVAILABLE_RETAINED.with(|set| *set.borrow_mut() = available);
            let started = Instant::now();
            CURRENT_ORIGIN.with(|c| c.set(None));
            let node = render();
            AVAILABLE_RETAINED.with(|set| set.borrow_mut().clear());
            if let Some(old) = mounted.take() {
                index.set(
                    tree.nodes[parent]
                        .children
                        .iter()
                        .position(|&child| child == old),
                );

                focus = tree.remember_text_focus(old);
                let unmount_started = Instant::now();
                tree.pool_retained(old);
                tree.unmount(old);
                if perf::profile_paint() {
                    perf::record_widget_paint(
                        format!("BUILD unmount {}:{}", short_file(origin), origin.line()),
                        unmount_started.elapsed().as_secs_f64() * 1000.0,
                    );
                }
            }
            let at = index.get().unwrap_or(tree.nodes[parent].children.len());
            let origin = CURRENT_ORIGIN.with(|c| c.take()).unwrap_or(origin);
            if is_rerun {
                note_rebuild(origin);
                log::debug!(target: "requiem_ui::rebuild", "rebuilt {}:{}", short_file(origin), origin.line());
            }
            if perf::profile_paint() {
                perf::record_widget_paint(
                    format!("BUILD render {}:{}", short_file(origin), origin.line()),
                    started.elapsed().as_secs_f64() * 1000.0,
                );
            }
            let started = Instant::now();
            let new_root = tree.mount_at(parent, at, node);
            tree.drop_retained_pool();
            if perf::profile_paint() {
                perf::record_widget_paint(
                    format!("BUILD mount {}:{}", short_file(origin), origin.line()),
                    started.elapsed().as_secs_f64() * 1000.0,
                );
            }
            if let Some(focus) = focus {
                tree.restore_text_focus(new_root, focus);
            }
            mounted.set(Some(new_root));
            recompute_if_styled(tree);
        },
        tree,
    )
}

pub fn for_each<T, K>(
    tree: &mut WidgetTree,
    parent: WidgetId,
    items: Signal<Vec<T>>,
    key: impl Fn(&T) -> K + 'static,
    render: impl Fn(&T) -> Node + 'static,
) -> Effect
where
    T: Clone + 'static,
    K: Eq + Hash + Clone + 'static,
{
    let mounted: Rc<RefCell<HashMap<K, WidgetId>>> = Rc::new(RefCell::new(HashMap::new()));
    create_effect(
        move |tree| {
            let current = items.get();
            let new_keys: Vec<K> = current.iter().map(&key).collect();

            let mut mounted = mounted.borrow_mut();

            let new_key_set: HashSet<&K> = new_keys.iter().collect();
            let stale: Vec<K> = mounted
                .keys()
                .filter(|k| !new_key_set.contains(k))
                .cloned()
                .collect();
            for stale_key in stale {
                if let Some(id) = mounted.remove(&stale_key) {
                    tree.unmount(id);
                }
            }

            for (item, item_key) in current.iter().zip(new_keys.iter()) {
                if !mounted.contains_key(item_key) {
                    let id = tree.mount(parent, render(item));
                    mounted.insert(item_key.clone(), id);
                }
            }

            let order: Vec<WidgetId> = new_keys.iter().map(|k| mounted[k]).collect();
            tree.reorder_children(parent, &order);
            recompute_if_styled(tree);
        },
        tree,
    )
}

fn short_file(location: &panic::Location<'_>) -> String {
    location
        .file()
        .rsplit(['/', path::MAIN_SEPARATOR])
        .next()
        .unwrap_or("")
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::style::ComputedStyle;
    use crate::widget::primitives::BoxWidget;

    fn test_tree() -> WidgetTree {
        WidgetTree::new(Box::new(BoxWidget), ComputedStyle::default())
    }

    fn div() -> Node {
        Node::new("div", BoxWidget)
    }

    fn row_section(
        tree: &mut WidgetTree,
        rows: Signal<Vec<(u64, u64)>>,
        built: Rc<Cell<u32>>,
    ) -> Effect {
        let root = tree.root;
        section(tree, root, move || {
            let mut list = div();
            for (key, version) in rows.get() {
                list = list.child(if retained_available(key, version) {
                    Node::reuse(key, version)
                } else {
                    built.set(built.get() + 1);
                    div()
                        .class(format!("row-{key}-{version}"))
                        .retained(key, version)
                });
            }
            list
        })
    }

    #[test]
    fn a_rebuild_moves_rows_with_the_same_key_and_version_instead_of_building_them_again() {
        let mut tree = test_tree();
        let root = tree.root;
        let rows = Signal::new(vec![(1, 0), (2, 0), (3, 0)]);
        let built = Rc::new(Cell::new(0));
        let _effect = row_section(&mut tree, rows.clone(), built.clone());
        let before = tree.nodes[tree.nodes[root].children[0]].children.clone();
        assert_eq!(built.get(), 3);

        rows.set(vec![(2, 0), (3, 0), (4, 0)]);
        signal::flush(&mut tree);

        let after = tree.nodes[tree.nodes[root].children[0]].children.clone();
        assert_eq!(built.get(), 4, "only the new row was built");
        assert_eq!(after.len(), 3);
        assert_eq!(after[0], before[1], "row 2 is the very same widget");
        assert_eq!(after[1], before[2], "row 3 is the very same widget");
        assert!(!tree.nodes.contains_key(before[0]), "row 1 was dropped");
        assert_eq!(tree.nodes[after[2]].classes, vec!["row-4-0".to_string()]);
        assert_eq!(
            tree.nodes[after[0]].parent,
            Some(tree.nodes[root].children[0]),
            "reattached to the new list"
        );
    }

    #[test]
    fn a_row_whose_version_changed_is_built_again() {
        let mut tree = test_tree();
        let root = tree.root;
        let rows = Signal::new(vec![(1, 0), (2, 0)]);
        let built = Rc::new(Cell::new(0));
        let _effect = row_section(&mut tree, rows.clone(), built.clone());
        let before = tree.nodes[tree.nodes[root].children[0]].children.clone();

        rows.set(vec![(1, 0), (2, 1)]);
        signal::flush(&mut tree);

        let after = tree.nodes[tree.nodes[root].children[0]].children.clone();
        assert_eq!(
            built.get(),
            3,
            "row 2 changed and was rebuilt, row 1 was not"
        );
        assert_eq!(after[0], before[0]);
        assert_ne!(after[1], before[1]);
        assert!(
            !tree.nodes.contains_key(before[1]),
            "the stale version is gone"
        );
        assert_eq!(tree.nodes[after[1]].classes, vec!["row-2-1".to_string()]);
    }

    #[test]
    fn section_remounts_a_fresh_subtree_when_its_dependency_changes() {
        let mut tree = test_tree();
        let root = tree.root;
        let label = Signal::new("first".to_string());
        let label_read = label.clone();
        let _effect = section(&mut tree, root, move || div().class(label_read.get()));

        assert_eq!(tree.nodes[root].children.len(), 1);
        let first_id = tree.nodes[root].children[0];

        label.set("second".to_string());
        signal::flush(&mut tree);

        assert_eq!(
            tree.nodes[root].children.len(),
            1,
            "still exactly one child, not appended"
        );
        let second_id = tree.nodes[root].children[0];
        assert_ne!(
            first_id, second_id,
            "a dependency change remounts a fresh subtree, unlike bind's in-place mutation"
        );
        assert_eq!(tree.nodes[second_id].classes, vec!["second".to_string()]);
    }

    #[test]
    fn section_remount_preserves_position_among_sibling_sections() {
        let mut tree = test_tree();
        let root = tree.root;
        let first_label = Signal::new("a".to_string());
        let first_label_read = first_label.clone();
        let _first = section(&mut tree, root, move || div().class(first_label_read.get()));
        let _second = section(&mut tree, root, || div().class("static"));

        assert_eq!(tree.nodes[root].children.len(), 2);
        let first_id_before = tree.nodes[root].children[0];
        let second_id = tree.nodes[root].children[1];

        first_label.set("b".to_string());
        signal::flush(&mut tree);

        assert_eq!(tree.nodes[root].children.len(), 2);
        assert_ne!(
            tree.nodes[root].children[0], first_id_before,
            "first section did remount"
        );
        assert_eq!(
            tree.nodes[root].children[1], second_id,
            "second section's own id/position is untouched"
        );
        assert_eq!(
            tree.nodes[tree.nodes[root].children[0]].classes,
            vec!["b".to_string()],
            "remounted section is still at index 0, not appended after the second"
        );
    }

    #[test]
    fn show_mounts_and_unmounts_on_condition_change() {
        let mut tree = test_tree();
        let root = tree.root;
        let open = Signal::new(false);
        let open_read = open.clone();
        let _effect = show(&mut tree, root, move || open_read.get(), div);
        assert!(
            tree.nodes[root].children.is_empty(),
            "starts closed, nothing mounted"
        );

        open.set(true);
        signal::flush(&mut tree);
        assert_eq!(
            tree.nodes[root].children.len(),
            1,
            "flush() mounts once cond() becomes true"
        );

        open.set(false);
        signal::flush(&mut tree);
        assert!(
            tree.nodes[root].children.is_empty(),
            "flush() unmounts once cond() becomes false"
        );
    }

    #[test]
    fn for_each_adds_removes_and_reorders_without_rebuilding_survivors() {
        let mut tree = test_tree();
        let root = tree.root;
        let items = Signal::new(vec![1, 2, 3]);
        let items_read = items.clone();
        let _effect = for_each(&mut tree, root, items_read, |n: &i32| *n, |_n| div());

        assert_eq!(tree.nodes[root].children.len(), 3);
        let id_for_2 = tree.nodes[root].children[1];

        items.set(vec![3, 2, 4]);
        signal::flush(&mut tree);

        assert_eq!(
            tree.nodes[root].children.len(),
            3,
            "one removed (1), one added (4): net still 3"
        );
        assert!(
            tree.nodes[root].children.contains(&id_for_2),
            "row for key 2 must be the same WidgetId, not rebuilt"
        );

        assert_eq!(tree.nodes[root].children[1], id_for_2);
    }
}
