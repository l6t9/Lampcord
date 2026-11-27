use requiem_ui::timer;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;

const TYPING_TIMEOUT: Duration = Duration::from_millis(10_000);

const DELAY_LOCAL: Duration = Duration::from_millis(1_500);

#[derive(Debug)]
struct Local {
    channel: u64,

    pending: bool,
    prev_send: Instant,
}

#[derive(Debug, Default)]
pub struct LocalTyping {
    state: Option<Local>,
}

impl LocalTyping {
    pub fn start(&mut self, channel: u64, now: Instant) -> Option<Duration> {
        if self.state.as_ref().is_some_and(|s| s.channel != channel) {
            self.state = None;
        }
        let next_send = TYPING_TIMEOUT.mul_f64(0.8);
        if let Some(s) = &self.state {
            if s.pending || s.prev_send + next_send > now {
                return None;
            }
        }

        let recent = self
            .state
            .as_ref()
            .is_none_or(|s| s.prev_send + next_send * 2 > now);
        self.state = Some(Local {
            channel,
            pending: true,
            prev_send: now,
        });
        Some(if recent { DELAY_LOCAL } else { Duration::ZERO })
    }

    pub fn fire(&mut self, channel: u64) -> bool {
        match &mut self.state {
            Some(s) if s.channel == channel && s.pending => {
                s.pending = false;
                true
            }
            _ => false,
        }
    }

    pub fn stop(&mut self, channel: u64) {
        if self
            .state
            .as_ref()
            .is_some_and(|s| s.channel == channel && s.pending)
        {
            self.state = None;
        }
    }
}

pub fn is_reaction_text(text: &str) -> bool {
    let Some(rest) = text.strip_prefix('+') else {
        return false;
    };
    let is_word = |c: char| c.is_alphanumeric() || c == '_';
    if rest.chars().next().is_some_and(is_word) {
        return false;
    }
    let rest = rest.strip_prefix(':').unwrap_or(rest);
    if rest.starts_with(':') {
        return false;
    }
    let rest = rest.trim_start_matches(is_word);
    let rest = rest.strip_prefix(':').unwrap_or(rest);
    rest.is_empty()
}

pub fn is_meaningful_change(previous: &str, text: &str, first_change: bool) -> bool {
    text != previous
        && !text.is_empty()
        && !is_reaction_text(text)
        && !text.starts_with('/')
        && !(first_change && text == previous)
}

static STATE: Mutex<Option<LocalTyping>> = Mutex::new(None);
static POSTS: Mutex<Option<mpsc::UnboundedSender<u64>>> = Mutex::new(None);

pub fn set_poster(tx: mpsc::UnboundedSender<u64>) {
    *POSTS.lock().unwrap_or_else(|e| e.into_inner()) = Some(tx);
}

fn with<R>(f: impl FnOnce(&mut LocalTyping) -> R) -> R {
    f(STATE
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .get_or_insert_with(LocalTyping::default))
}

pub fn composer_changed(channel: u64, previous: &str, text: &str, first_change: bool) {
    if is_meaningful_change(previous, text, first_change) {
        if let Some(delay) = with(|t| t.start(channel, Instant::now())) {
            timer::set_timeout(delay, move || {
                if with(|t| t.fire(channel)) {
                    if let Some(tx) = POSTS.lock().unwrap_or_else(|e| e.into_inner()).as_ref() {
                        let _ = tx.send(channel);
                    }
                }
            });
        }
    } else if text.is_empty() {
        with(|t| t.stop(channel));
    }
}

pub fn stopped(channel: u64) {
    with(|t| t.stop(channel));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_first_keystroke_schedules_a_post_after_a_second_and_a_half() {
        let mut t = LocalTyping::default();
        let now = Instant::now();
        assert_eq!(t.start(1, now), Some(Duration::from_millis(1_500)));
        assert_eq!(
            t.start(1, now + Duration::from_millis(100)),
            None,
            "already scheduled"
        );
        assert!(t.fire(1));
        assert!(!t.fire(1), "fires once");
    }

    #[test]
    fn typing_goes_on_posting_every_eight_seconds_not_more() {
        let mut t = LocalTyping::default();
        let start = Instant::now();
        t.start(1, start);
        t.fire(1);
        assert_eq!(
            t.start(1, start + Duration::from_secs(5)),
            None,
            "less than 8 s since the last one"
        );
        assert_eq!(
            t.start(1, start + Duration::from_secs(9)),
            Some(Duration::from_millis(1_500)),
            "8 s have passed"
        );
    }

    #[test]
    fn after_a_long_pause_the_post_is_immediate() {
        let mut t = LocalTyping::default();
        let start = Instant::now();
        t.start(1, start);
        t.fire(1);
        assert_eq!(
            t.start(1, start + Duration::from_secs(30)),
            Some(Duration::ZERO),
            "more than 16 s since the last post"
        );
    }

    #[test]
    fn stopping_before_the_post_cancels_it() {
        let mut t = LocalTyping::default();
        t.start(1, Instant::now());
        t.stop(1);
        assert!(!t.fire(1));
    }

    #[test]
    fn typing_in_another_channel_drops_the_first_ones_post() {
        let mut t = LocalTyping::default();
        let now = Instant::now();
        t.start(1, now);
        assert_eq!(t.start(2, now), Some(Duration::from_millis(1_500)));
        assert!(!t.fire(1));
        assert!(t.fire(2));
    }

    #[test]
    fn reactions_commands_and_unchanged_text_are_not_typing() {
        for text in ["+", "+:", "+:smi", "+:smile:", "+1x"] {
            assert_eq!(is_reaction_text(text), text != "+1x", "{text}");
        }
        assert!(!is_meaningful_change("", "/giphy cat", false));
        assert!(!is_meaningful_change("", "+:smile", false));
        assert!(!is_meaningful_change("abc", "abc", false));
        assert!(!is_meaningful_change("abc", "", false));
        assert!(is_meaningful_change("", "h", false));
        assert!(is_meaningful_change("h", "hi", false));
    }
}
