//! Names for channels mentioned as `<#id>`, for any channel the account can reach: in the
//! store when it's there, otherwise fetched once.

use crate::models::Permissions;
use crate::rest::RequestExtras;
use crate::rest::RestError;
use crate::ui::home;
use crate::ui;
use requiem_ui::widget::primitives;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::models::Id;
use crate::ui::home::HomeEnv;

/// What a fetch found: the name, or that the channel exists but isn't open to this
/// account. A channel that doesn't exist is recorded as `None`.
#[derive(Clone)]
enum Fetched {
    Name(String),
    NoAccess,
}

static FETCHED: Mutex<Option<HashMap<u64, Option<Fetched>>>> = Mutex::new(None);
static REQUESTED: Mutex<Option<HashSet<u64>>> = Mutex::new(None);
static REVISION: AtomicU64 = AtomicU64::new(0);

/// Changes whenever a fetched name arrives, so rows that showed a placeholder rebuild.
pub fn revision() -> u64 {
    REVISION.load(Ordering::Relaxed)
}

pub fn install(env: Rc<dyn Fn() -> HomeEnv>) {
    primitives::set_channel_name_resolver(Rc::new(move |id| {
        name_of(&env(), id)
    }));
}

fn name_of(env: &HomeEnv, id: u64) -> Option<String> {
    let channel_id = Id::new(id);
    if let Some((_, channel)) = ui::find_channel(&env.state, channel_id) {
        if !env.state.can(&channel, Permissions::VIEW_CHANNEL) {
            return Some(no_access());
        }
        if let Some(name) = channel.name.filter(|name| !name.is_empty()) {
            return Some(name);
        }
    }
    if let Some(dm) = env.state.dms.find(channel_id) {
        return Some(home::dm_title(&env.state, &dm));
    }
    if let Some(known) = FETCHED
        .lock()
        .expect("channel names poisoned")
        .as_ref()
        .and_then(|fetched| fetched.get(&id).cloned())
    {
        return match known {
            Some(Fetched::Name(name)) => Some(name),
            Some(Fetched::NoAccess) => Some(no_access()),
            None => None,
        };
    }
    fetch(env, id);
    None
}

fn no_access() -> String {
    requiem_i18n::t!("channel-mention-no-access").to_string()
}

fn fetch(env: &HomeEnv, id: u64) {
    let first = REQUESTED
        .lock()
        .expect("channel names poisoned")
        .get_or_insert_with(HashSet::new)
        .insert(id);
    if !first {
        return;
    }
    let (http, wake) = (env.http.clone(), env.images.ready_notifier());
    env.queries.mutate(Vec::new(), move || async move {
        let fetched = match http
            .request_with::<()>(
                "GET /channels/:id",
                wreq::Method::GET,
                &format!("/channels/{id}"),
                None,
                &RequestExtras::default(),
            )
            .await
        {
            Ok(response) => response
                .json::<serde_json::Value>()
                .await
                .ok()
                .and_then(|channel| channel.get("name")?.as_str().map(str::to_string))
                .map(Fetched::Name),
            // Forbidden: it exists, this account just can't open it.
            Err(RestError::Status { status, .. }) if status.as_u16() == 403 => {
                Some(Fetched::NoAccess)
            }
            Err(err) => {
                log::debug!("channel names: could not fetch {id}: {err}");
                None
            }
        };
        FETCHED
            .lock()
            .expect("channel names poisoned")
            .get_or_insert_with(HashMap::new)
            .insert(id, fetched);
        REVISION.fetch_add(1, Ordering::Relaxed);
        wake();
        Ok::<(), String>(())
    });
}
