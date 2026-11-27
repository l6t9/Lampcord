use std::collections::HashMap;
use std::sync::mpsc;
use std::sync::{Mutex, OnceLock, Weak};
use std::thread::Builder;
use std::time::{Duration, Instant};

pub fn monotonic_ms() -> u64 {
    static START: OnceLock<Instant> = OnceLock::new();
    START.get_or_init(Instant::now).elapsed().as_millis() as u64
}

const MAX_TIMERS: usize = 64;

enum Command {
    Set {
        plugin: String,
        id: String,
        delay: Duration,
        repeat: bool,
    },
    Clear {
        plugin: String,
        id: String,
    },
    ClearPlugin {
        plugin: String,
    },
}

pub(crate) struct Timers {
    tx: Mutex<mpsc::Sender<Command>>,
}

struct Timer {
    due: Instant,
    every: Option<Duration>,
}

impl Timers {
    pub fn start(host: Weak<crate::PluginHost>) -> Timers {
        let (tx, rx) = mpsc::channel::<Command>();
        Builder::new()
            .name("plugin-timers".into())
            .spawn(move || {
                let mut timers: HashMap<(String, String), Timer> = HashMap::new();
                loop {
                    let next = timers.values().map(|t| t.due).min();
                    let command = match next {
                        Some(due) => match rx.recv_timeout(due.saturating_duration_since(Instant::now())) {
                            Ok(command) => Some(command),
                            Err(mpsc::RecvTimeoutError::Timeout) => None,
                            Err(mpsc::RecvTimeoutError::Disconnected) => return,
                        },
                        None => match rx.recv() {
                            Ok(command) => Some(command),
                            Err(_) => return,
                        },
                    };
                    match command {
                        Some(Command::Set { plugin, id, delay, repeat }) => {
                            let count = timers.keys().filter(|(p, _)| *p == plugin).count();
                            let key = (plugin, id);
                            if count >= MAX_TIMERS && !timers.contains_key(&key) {
                                log::warn!(target: "plugin", "[{}] too many timers; `{}` ignored", key.0, key.1);
                                continue;
                            }
                            timers.insert(key, Timer { due: Instant::now() + delay, every: repeat.then_some(delay) });
                        }
                        Some(Command::Clear { plugin, id }) => {
                            timers.remove(&(plugin, id));
                        }
                        Some(Command::ClearPlugin { plugin }) => timers.retain(|(p, _), _| *p != plugin),
                        None => {}
                    }
                    let now = Instant::now();
                    let due: Vec<(String, String)> = timers.iter().filter(|(_, t)| t.due <= now).map(|(k, _)| k.clone()).collect();
                    for key in due {
                        let Some(host) = host.upgrade() else { return };
                        host.post_timer(&key.0, &key.1);
                        match timers.get_mut(&key).and_then(|t| t.every.map(|every| (t, every))) {
                            Some((timer, every)) => timer.due = now + every,
                            None => {
                                timers.remove(&key);
                            }
                        }
                    }
                }
            })
            .expect("spawning the plugin timer thread");
        Timers { tx: Mutex::new(tx) }
    }

    fn send(&self, command: Command) {
        let _ = self
            .tx
            .lock()
            .expect("timer channel lock poisoned")
            .send(command);
    }

    pub fn set(&self, plugin: &str, id: String, delay: Duration, repeat: bool) {
        self.send(Command::Set {
            plugin: plugin.to_string(),
            id,
            delay,
            repeat,
        });
    }

    pub fn clear(&self, plugin: &str, id: &str) {
        self.send(Command::Clear {
            plugin: plugin.to_string(),
            id: id.to_string(),
        });
    }

    pub fn clear_plugin(&self, plugin: &str) {
        self.send(Command::ClearPlugin {
            plugin: plugin.to_string(),
        });
    }
}
