use std::cell::Cell;
use std::rc::Rc;
use std::time::{Duration, Instant};

use requiem_ui::signal::KeyedSignal;
use requiem_ui::timer::set_timeout;

use super::{Action, Store};
use crate::models::Id;
use crate::models::id::marker::{ChannelMarker, UserMarker};

/// How long a `TYPING_START` counts: Discord shows someone as typing for ten seconds unless
/// they send the message or start again.
const TYPING_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Clone, Copy)]
struct Typer {
    user_id: Id<UserMarker>,
    until: Instant,
}

/// Who is typing in which channel.
#[derive(Clone)]
pub struct TypingStore {
    channels: KeyedSignal<Id<ChannelMarker>, Vec<Typer>>,
    me: Rc<Cell<Option<Id<UserMarker>>>>,
}

impl Default for TypingStore {
    fn default() -> Self {
        Self::new()
    }
}

impl TypingStore {
    pub fn new() -> Self {
        Self {
            channels: KeyedSignal::new(),
            me: Rc::default(),
        }
    }

    /// Everyone but you who is typing in `channel`, longest first. Follows that channel only.
    pub fn typing_in(&self, channel: Id<ChannelMarker>) -> Vec<Id<UserMarker>> {
        self.channels.watch(&channel);
        let now = Instant::now();
        self.channels.with_untracked(|all| {
            all.get(&channel)
                .into_iter()
                .flatten()
                .filter(|typer| typer.until > now && Some(typer.user_id) != self.me.get())
                .map(|typer| typer.user_id)
                .collect()
        })
    }

    fn start(&self, channel: Id<ChannelMarker>, user_id: Id<UserMarker>) {
        let until = Instant::now() + TYPING_TIMEOUT;
        self.channels.update_key(&channel, |all| {
            let typers = all.entry(channel).or_default();
            match typers.iter_mut().find(|typer| typer.user_id == user_id) {
                Some(typer) => typer.until = until,
                None => typers.push(Typer { user_id, until }),
            }
        });
        let store = self.clone();
        set_timeout(TYPING_TIMEOUT + Duration::from_millis(50), move || {
            store.expire(channel)
        });
    }

    fn stop(&self, channel: Id<ChannelMarker>, user_id: Id<UserMarker>) {
        let typing = self.channels.with_untracked(|all| {
            all.get(&channel)
                .is_some_and(|typers| typers.iter().any(|typer| typer.user_id == user_id))
        });
        if typing {
            self.channels.update_key(&channel, |all| {
                if let Some(typers) = all.get_mut(&channel) {
                    typers.retain(|typer| typer.user_id != user_id);
                }
            });
        }
    }

    fn expire(&self, channel: Id<ChannelMarker>) {
        let now = Instant::now();
        let stale = self.channels.with_untracked(|all| {
            all.get(&channel)
                .is_some_and(|typers| typers.iter().any(|typer| typer.until <= now))
        });
        if stale {
            self.channels.update_key(&channel, |all| {
                if let Some(typers) = all.get_mut(&channel) {
                    typers.retain(|typer| typer.until > now);
                }
            });
        }
    }
}

impl Store for TypingStore {
    fn handle(&self, action: &Action) {
        match action {
            Action::ConnectionOpen { user, .. } => {
                self.me.set(Some(user.id));
                self.channels.set(Default::default());
            }
            Action::TypingStart(typing) => self.start(typing.channel_id, typing.user_id),
            // Sending a message ends the typing that led to it.
            Action::MessageCreate(message) => self.stop(message.channel_id, message.author.id),
            _ => {}
        }
    }

    fn reset(&self) {
        self.me.set(None);
        self.channels.set(Default::default());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn store_with(me: u64) -> TypingStore {
        let store = TypingStore::new();
        store.me.set(Some(Id::new(me)));
        store
    }

    #[test]
    fn people_typing_show_up_and_you_do_not() {
        let store = store_with(1);
        let channel = Id::new(10);
        store.start(channel, Id::new(2));
        store.start(channel, Id::new(1));
        store.start(channel, Id::new(3));
        store.start(channel, Id::new(2));
        assert_eq!(store.typing_in(channel), vec![Id::new(2), Id::new(3)]);
        assert!(store.typing_in(Id::new(11)).is_empty());
    }

    #[test]
    fn sending_a_message_or_running_out_clears_the_indicator() {
        let store = store_with(1);
        let channel = Id::new(10);
        store.start(channel, Id::new(2));
        store.start(channel, Id::new(3));
        store.stop(channel, Id::new(2));
        assert_eq!(store.typing_in(channel), vec![Id::new(3)]);

        store.channels.update_key(&channel, |all| {
            for typer in all.get_mut(&channel).into_iter().flatten() {
                typer.until = Instant::now() - Duration::from_secs(1);
            }
        });
        assert!(store.typing_in(channel).is_empty(), "expired entries are hidden");
        store.expire(channel);
        assert!(store.channels.with_untracked(|all| all[&channel].is_empty()));
    }
}
