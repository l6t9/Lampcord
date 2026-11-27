use std::time::{SystemTime, UNIX_EPOCH};

use requiem_db::Db;
use serde::{Deserialize, Serialize};

const KEY: &str = "update.resume";

pub const MAX_AGE_SECS: u64 = 120;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Resume {
    pub guild_id: Option<u64>,
    pub channel_id: u64,
    pub muted: bool,
    pub deafened: bool,

    pub saved_at: u64,
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

pub fn save(
    db: &Db,
    guild_id: Option<u64>,
    channel_id: u64,
    muted: bool,
    deafened: bool,
) {
    let resume = Resume {
        guild_id,
        channel_id,
        muted,
        deafened,
        saved_at: now(),
    };
    let _ = db.set_meta(
        KEY,
        &serde_json::to_string(&resume).expect("resume serializes"),
    );
}

pub fn take(db: &Db) -> Option<Resume> {
    take_at(db, now())
}

pub fn take_at(db: &Db, now: u64) -> Option<Resume> {
    let raw = db.meta(KEY)?;
    let _ = db.set_meta(KEY, "");
    let resume: Resume = serde_json::from_str(&raw).ok()?;
    (now.saturating_sub(resume.saved_at) <= MAX_AGE_SECS).then_some(resume)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_fresh_record_is_used_once() {
        let db = Db::open_in_memory().unwrap();
        save(&db, Some(1), 2, true, false);
        let got = take(&db).unwrap();
        assert_eq!(
            (got.guild_id, got.channel_id, got.muted, got.deafened),
            (Some(1), 2, true, false)
        );
        assert!(take(&db).is_none(), "used once");
    }

    #[test]
    fn a_stale_record_is_ignored_and_cleared() {
        let db = Db::open_in_memory().unwrap();
        save(&db, None, 5, false, false);
        let saved_at = serde_json::from_str::<Resume>(&db.meta(KEY).unwrap())
            .unwrap()
            .saved_at;
        assert!(take_at(&db, saved_at + MAX_AGE_SECS + 1).is_none());
        assert!(db.meta(KEY).as_deref().is_none_or(str::is_empty));
    }

    #[test]
    fn nothing_saved_means_nothing_to_resume() {
        assert!(take(&Db::open_in_memory().unwrap()).is_none());
    }
}
