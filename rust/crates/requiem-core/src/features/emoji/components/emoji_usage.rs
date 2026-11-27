use crate::ui::emoji_names;
use crate::ui::emoji_skin;
use crate::ui::reactions;
use crate::utils::time;
use std::cell::RefCell;
use std::time::Duration;

use crate::frecency::Frecency;
use crate::models::Id;
use crate::models::id::marker::GuildMarker;
use crate::rest::RestError;
use crate::rest::endpoints::{emoji_frecency, top_emojis};
use crate::ui::context_menus::MenuEnv;

const USED_KEY: &str = "emoji-frecency";
const TOP_TTL: Duration = Duration::from_secs(300);

pub const FREQUENT_LIMIT: usize = 9;

thread_local! {

    static WORKING: RefCell<Option<Frecency>> = const { RefCell::new(None) };
}

fn used(env: &MenuEnv) -> Frecency {
    if let Some(known) = WORKING.with(|w| w.borrow().clone()) {
        return known;
    }
    if let Some(http) = env.http.clone() {
        let loaded = env
            .queries
            .query_default::<Frecency, _, RestError>(USED_KEY, move || async move {
                emoji_frecency::get_used_frecency(&http).await
            });
        if let Some(loaded) = loaded.data {
            let list = if loaded.is_empty() {
                Frecency::default_used(time::now_ms())
            } else {
                (*loaded).clone()
            };
            WORKING.with(|w| *w.borrow_mut() = Some(list.clone()));
            return list;
        }
    }
    Frecency::default_used(time::now_ms())
}

pub fn frequent_keys(env: &MenuEnv, limit: usize) -> Vec<String> {
    used(env).frequently(time::now_ms(), limit)
}

fn key_of(picked: &str) -> Option<String> {
    match reactions::parse_custom_markup(picked) {
        Some(custom) => custom.id.map(|id| id.to_string()),

        None => emoji_names::primary_name(&emoji_skin::plain(picked))
            .map(str::to_string),
    }
}

pub fn track(env: &MenuEnv, picked: &str) {
    let Some(key) = key_of(picked) else { return };
    let mut list = used(env);
    list.track(&key, time::now_ms());
    WORKING.with(|w| *w.borrow_mut() = Some(list.clone()));
    env.queries.set_data(USED_KEY, list.clone());
    if let Some(http) = env.http.clone() {
        env.queries.mutate(Vec::new(), move || async move {
            emoji_frecency::set_used_frecency(&http, &list, time::now_ms()).await
        });
    }
}

pub fn top_emoji_ids(env: &MenuEnv, guild_id: Id<GuildMarker>) -> Vec<u64> {
    let Some(http) = env.http.clone() else {
        return Vec::new();
    };
    let loaded = env.queries.query::<Vec<u64>, _, RestError>(
        &format!("top-emojis:{guild_id}"),
        TOP_TTL,
        move || async move { top_emojis::get_top_emojis(&http, guild_id).await },
    );
    loaded.data.map(|ids| (*ids).clone()).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn picks_are_keyed_by_shortcode_or_custom_id() {
        assert_eq!(key_of("<:happy:123>").as_deref(), Some("123"));
        assert_eq!(key_of("<a:wiggle:9>").as_deref(), Some("9"));
        assert_eq!(key_of("\u{1F44D}").as_deref(), Some("thumbsup"));
        assert_eq!(key_of("hello"), None);
    }

    #[test]
    fn a_new_snowflake_cutoff_is_in_the_past() {
        assert!(
            time::snowflake_days_ago(60) < time::snowflake_days_ago(1)
        );
        assert!(time::snowflake_days_ago(60) > 0);
    }
}
