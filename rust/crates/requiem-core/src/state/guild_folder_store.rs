use requiem_ui::timer;
use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use std::time::{Duration, Instant};

use requiem_ui::signal::{Signal, signal};

use super::{Action, Store};
use crate::models::id::marker::GuildMarker;
use crate::models::{Guild, Id};
use crate::rest::endpoints::user_settings::{self, GuildFolderSettings};
use crate::state::wire::LegacyFolderWire;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GuildFolder {
    /// Folders the user made have an id; entries without one are lone servers.
    pub id: Option<i64>,
    pub name: Option<String>,
    /// 0xRRGGBB. Folders without one use the brand color.
    pub color: Option<u32>,
    pub guild_ids: Vec<Id<GuildMarker>>,
}

/// One slot of the server rail.
#[derive(Debug, Clone)]
pub enum RailEntry {
    Guild(Guild),
    Folder {
        folder: GuildFolder,
        guilds: Vec<Guild>,
    },
}

/// The web client grows an opening folder with a react-spring transition: linear, 150ms.
const OPEN_DURATION: Duration = Duration::from_millis(150);

/// What an open folder adds per server: its 40px icon and the 8px under it.
const ROW_PITCH: f32 = 48.0;

/// How one folder is animating. Folders keep their nodes whether open or closed, so the
/// stylesheet's transitions play; only the list's height needs driving from here.
struct Motion {
    open: Signal<bool>,
    list_style: Signal<String>,
    progress: Cell<f32>,
    rows: Cell<usize>,
    generation: Cell<u64>,
}

fn list_style(progress: f32, rows: usize) -> String {
    if progress <= 0.0 {
        "display: none".to_string()
    } else {
        format!("height: {}px", progress * ROW_PITCH * rows as f32)
    }
}

fn tween(motion: Rc<Motion>, from: f32, to: f32, start: Instant, generation: u64) {
    timer::request_animation_frame(move || {
        if motion.generation.get() != generation {
            return;
        }
        let t = (start.elapsed().as_secs_f32() / OPEN_DURATION.as_secs_f32()).min(1.0);
        let progress = from + (to - from) * t;
        motion.progress.set(progress);
        motion
            .list_style
            .set(list_style(progress, motion.rows.get()));
        if t < 1.0 {
            tween(motion, from, to, start, generation);
        }
    });
}

#[derive(Clone)]
pub struct GuildFolderStore {
    pub folders: Signal<Vec<GuildFolder>>,

    /// Where servers sit when the settings have no folders yet.
    positions: Signal<Vec<Id<GuildMarker>>>,

    /// Open folders. Local to this session, like the real client.
    pub expanded: Signal<HashSet<i64>>,

    motion: Rc<RefCell<HashMap<i64, Rc<Motion>>>>,
}

impl Default for GuildFolderStore {
    fn default() -> Self {
        Self::new()
    }
}

impl GuildFolderStore {
    pub fn new() -> Self {
        Self {
            folders: signal(Vec::new()),
            positions: signal(Vec::new()),
            expanded: signal(HashSet::new()),
            motion: Rc::default(),
        }
    }

    /// Whether the folder is open, and the style that sizes its list, for a folder with
    /// `rows` servers. The signals outlive rebuilds of the rail, so a rebuild mid-animation
    /// carries on from where it was.
    pub fn motion(&self, id: i64, rows: usize) -> (Signal<bool>, Signal<String>) {
        let open = self.expanded.with_untracked(|set| set.contains(&id));
        let motion = self
            .motion
            .borrow_mut()
            .entry(id)
            .or_insert_with(|| {
                let progress = if open { 1.0 } else { 0.0 };
                Rc::new(Motion {
                    open: signal(open),
                    list_style: signal(list_style(progress, rows)),
                    progress: Cell::new(progress),
                    rows: Cell::new(rows),
                    generation: Cell::new(0),
                })
            })
            .clone();
        if motion.rows.replace(rows) != rows {
            motion
                .list_style
                .set(list_style(motion.progress.get(), rows));
        }
        (motion.open.clone(), motion.list_style.clone())
    }

    pub fn toggle(&self, id: i64) {
        let open = !self.expanded.with_untracked(|set| set.contains(&id));
        self.set_open(id, open);
    }

    pub fn collapse_all(&self) {
        let open: Vec<i64> = self
            .expanded
            .with_untracked(|set| set.iter().copied().collect());
        for id in open {
            self.set_open(id, false);
        }
    }

    fn set_open(&self, id: i64, open: bool) {
        self.expanded.update(|set| {
            if open {
                set.insert(id);
            } else {
                set.remove(&id);
            }
        });
        let Some(motion) = self.motion.borrow().get(&id).cloned() else {
            return;
        };
        motion.open.set(open);
        let generation = motion.generation.get() + 1;
        motion.generation.set(generation);
        let from = motion.progress.get();
        tween(
            motion,
            from,
            if open { 1.0 } else { 0.0 },
            Instant::now(),
            generation,
        );
    }

    pub fn is_expanded(&self, id: i64) -> bool {
        self.expanded.with(|open| open.contains(&id))
    }

    /// The folder holding `guild`, if the user filed it.
    pub fn folder_of(&self, guild: Id<GuildMarker>) -> Option<GuildFolder> {
        self.folders.with_untracked(|folders| {
            folders
                .iter()
                .find(|f| f.id.is_some() && f.guild_ids.contains(&guild))
                .cloned()
        })
    }

    /// The rail in the user's order. Servers the settings don't mention yet, such as one just
    /// joined, follow the rest.
    pub fn arrange(&self, guilds: &[Guild]) -> Vec<RailEntry> {
        let folders = self.folders.get();
        let positions = self.positions.get();
        arrange(&folders, &positions, guilds)
    }

    fn apply_legacy(&self, folders: &[LegacyFolderWire]) {
        self.folders.set(
            folders
                .iter()
                .map(|f| GuildFolder {
                    id: f.id,
                    name: f.name.clone().filter(|n| !n.is_empty()),
                    color: f.color.map(|c| c as u32),
                    guild_ids: f.guild_ids.clone(),
                })
                .collect(),
        );
        self.positions.set(Vec::new());
    }

    fn apply(&self, settings: GuildFolderSettings) {
        let id = |raw: &u64| Id::new(*raw);
        self.folders.set(
            settings
                .folders
                .iter()
                .map(|f| GuildFolder {
                    id: f.id,
                    name: f.name.clone(),
                    color: f.color,
                    guild_ids: f.guild_ids.iter().map(id).collect(),
                })
                .collect(),
        );
        self.positions
            .set(settings.positions.iter().map(id).collect());
    }
}

fn arrange(
    folders: &[GuildFolder],
    positions: &[Id<GuildMarker>],
    guilds: &[Guild],
) -> Vec<RailEntry> {
    let mut placed: HashSet<Id<GuildMarker>> = HashSet::new();
    let mut out = Vec::new();
    let find = |id: &Id<GuildMarker>| guilds.iter().find(|g| g.id == *id).cloned();

    for folder in folders {
        let members: Vec<Guild> = folder
            .guild_ids
            .iter()
            .filter(|id| !placed.contains(*id))
            .filter_map(find)
            .collect();
        if members.is_empty() {
            continue;
        }
        placed.extend(members.iter().map(|g| g.id));
        match folder.id {
            Some(_) => out.push(RailEntry::Folder {
                folder: folder.clone(),
                guilds: members,
            }),
            None => out.extend(members.into_iter().map(RailEntry::Guild)),
        }
    }

    // Older settings only carry a flat order.
    for id in positions {
        if let Some(guild) = find(id).filter(|_| placed.insert(*id)) {
            out.push(RailEntry::Guild(guild));
        }
    }
    for guild in guilds.iter().filter(|g| !placed.contains(&g.id)) {
        out.push(RailEntry::Guild(guild.clone()));
    }
    out
}

impl Store for GuildFolderStore {
    fn handle(&self, action: &Action) {
        let (blob, legacy) = match action {
            Action::ConnectionOpen {
                user_settings_proto,
                ready,
                ..
            } => (
                user_settings_proto.as_deref(),
                ready.legacy_folders.as_deref(),
            ),
            Action::UserSettingsProtoUpdate { proto } => (Some(proto.as_str()), None),
            Action::GuildFoldersUpdate(folders) => (None, Some(folders.as_slice())),
            _ => return,
        };
        // An update that doesn't touch the folders leaves them as they were. The settings
        // blob wins; servers that only send the older JSON settings are read as a fallback.
        if let Some(settings) = blob.and_then(user_settings::guild_folders_from_base64) {
            self.apply(settings);
        } else if let Some(folders) = legacy {
            self.apply_legacy(folders);
        }
    }

    fn reset(&self) {
        self.folders.set(Vec::new());
        self.positions.set(Vec::new());
        self.expanded.set(HashSet::new());
        self.motion.borrow_mut().clear();
    }
}

#[cfg(test)]
mod tests {
    use std::thread;
    use super::*;
    use crate::models::Guild;

    fn guild(id: u64) -> Guild {
        serde_json::from_value(
            serde_json::json!({ "id": id.to_string(), "name": format!("g{id}") }),
        )
        .expect("guild")
    }

    fn folder(id: Option<i64>, ids: &[u64]) -> GuildFolder {
        GuildFolder {
            id,
            name: None,
            color: None,
            guild_ids: ids.iter().map(|i| Id::new(*i)).collect(),
        }
    }

    fn shape(entries: &[RailEntry]) -> Vec<String> {
        entries
            .iter()
            .map(|e| match e {
                RailEntry::Guild(g) => format!("g{}", g.id.get()),
                RailEntry::Folder { folder, guilds } => format!(
                    "f{}[{}]",
                    folder.id.unwrap_or(0),
                    guilds
                        .iter()
                        .map(|g| g.id.get().to_string())
                        .collect::<Vec<_>>()
                        .join(",")
                ),
            })
            .collect()
    }

    #[test]
    fn folders_keep_their_order_and_unfiled_servers_follow() {
        let guilds = [guild(1), guild(2), guild(3), guild(4)];
        let folders = [folder(None, &[3]), folder(Some(7), &[2, 1])];
        assert_eq!(
            shape(&arrange(&folders, &[], &guilds)),
            ["g3", "f7[2,1]", "g4"]
        );
    }

    #[test]
    fn servers_you_left_drop_out_and_empty_folders_vanish() {
        let guilds = [guild(1)];
        let folders = [folder(Some(7), &[9]), folder(Some(8), &[9, 1])];
        assert_eq!(shape(&arrange(&folders, &[], &guilds)), ["f8[1]"]);
    }

    #[test]
    fn opening_a_folder_grows_its_list_linearly_over_150ms() {
        let store = GuildFolderStore::new();
        let (open, style) = store.motion(7, 2);
        assert_eq!((open.get(), style.get().as_str()), (false, "display: none"));

        store.toggle(7);
        assert!(
            open.get(),
            "the open flag flips at once so transitions start"
        );

        let mut heights = Vec::new();
        let started = Instant::now();
        while started.elapsed() < Duration::from_millis(400) {
            timer::run_due();
            if let Some(h) = style
                .get()
                .strip_prefix("height: ")
                .and_then(|h| h.strip_suffix("px"))
                .and_then(|h| h.parse::<f32>().ok())
            {
                heights.push(h);
                if h >= 96.0 {
                    break;
                }
            }
            thread::sleep(Duration::from_millis(4));
        }
        assert_eq!(heights.last(), Some(&96.0), "{heights:?}");
        assert!(heights.windows(2).all(|w| w[0] <= w[1]), "{heights:?}");
        assert!(
            heights.iter().any(|h| *h > 0.0 && *h < 96.0),
            "it passes through in-between sizes: {heights:?}"
        );
        assert!(
            started.elapsed() >= Duration::from_millis(140),
            "it takes the full duration"
        );

        store.toggle(7);
        assert!(!open.get());
        let started = Instant::now();
        while started.elapsed() < Duration::from_millis(400) && style.get() != "display: none" {
            timer::run_due();
            thread::sleep(Duration::from_millis(4));
        }
        assert_eq!(style.get(), "display: none", "closed folders take no room");
    }

    #[test]
    fn flat_positions_order_a_list_without_folders() {
        let guilds = [guild(1), guild(2), guild(3)];
        let positions = [Id::new(3), Id::new(1)];
        assert_eq!(
            shape(&arrange(&[], &positions, &guilds)),
            ["g3", "g1", "g2"]
        );
    }
}
