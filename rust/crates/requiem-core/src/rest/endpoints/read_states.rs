use std::future;
use std::rc::Rc;
use std::sync::Arc;
use std::time::Duration;
use tokio::runtime;
use tokio::sync::Mutex;
use tokio::time;

use serde::{Deserialize, Serialize};

use crate::models::id::marker::{ChannelMarker, GuildMarker, MessageMarker};
use crate::models::{Id, Message};
use crate::state::AckRequest;

use super::super::{HttpClient, RestError};

#[derive(Serialize)]
struct AckBody<'a> {
    token: Option<&'a str>,
    last_viewed: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    flags: Option<u8>,
}

#[derive(Deserialize)]
struct AckResponse {
    token: Option<String>,
}

pub async fn ack_message(
    http: &HttpClient,
    channel_id: Id<ChannelMarker>,
    message_id: Id<MessageMarker>,
    token: Option<&str>,
    last_viewed: u32,
    flags: Option<u8>,
) -> Result<Option<String>, RestError> {
    let response = http
        .request_with_json(
            "POST /channels/:id/messages/:id/ack",
            wreq::Method::POST,
            &format!("/channels/{channel_id}/messages/{message_id}/ack"),
            &AckBody {
                token,
                last_viewed,
                flags,
            },
        )
        .await?;

    Ok(response
        .json::<AckResponse>()
        .await
        .ok()
        .and_then(|body| body.token))
}

pub const BULK_ACK_CHUNK: usize = 100;

#[derive(Serialize)]
struct BulkEntry {
    channel_id: Id<ChannelMarker>,
    message_id: Id<MessageMarker>,

    read_state_type: u8,
}

#[derive(Serialize)]
struct BulkBody {
    read_states: Vec<BulkEntry>,
}

pub async fn ack_bulk(
    http: &HttpClient,
    channels: &[(Id<ChannelMarker>, Id<MessageMarker>)],
) -> Result<(), RestError> {
    let body = BulkBody {
        read_states: channels
            .iter()
            .map(|(channel_id, message_id)| BulkEntry {
                channel_id: *channel_id,
                message_id: *message_id,
                read_state_type: 0,
            })
            .collect(),
    };
    http.request_with_json(
        "POST /read-states/ack-bulk",
        wreq::Method::POST,
        "/read-states/ack-bulk",
        &body,
    )
    .await?;
    Ok(())
}

async fn retried<T, Fut: future::Future<Output = Result<T, RestError>>>(
    what: &str,
    mut call: impl FnMut() -> Fut,
) -> Option<T> {
    for attempt in 0..3u64 {
        match call().await {
            Ok(value) => return Some(value),
            Err(err) => {
                log::warn!("{what} failed (attempt {}): {err}", attempt + 1);
                time::sleep(Duration::from_secs((attempt + 1) * 2)).await;
            }
        }
    }
    None
}

pub fn ack_sender(runtime: runtime::Handle, http: HttpClient) -> Rc<dyn Fn(AckRequest)> {
    let token: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
    Rc::new(move |request| {
        let (http, token) = (http.clone(), token.clone());
        runtime.spawn(async move {
            match request {
                AckRequest::Message {
                    channel_id,
                    message_id,
                    last_viewed,
                    flags,
                } => {
                    let mut token = token.lock().await;
                    let current = token.clone();
                    if let Some(next) = retried("acknowledging a channel", || {
                        ack_message(
                            &http,
                            channel_id,
                            message_id,
                            current.as_deref(),
                            last_viewed,
                            flags,
                        )
                    })
                    .await
                    {
                        if next.is_some() {
                            *token = next;
                        }
                    }
                }
                AckRequest::Bulk(channels) => {
                    for (index, chunk) in channels.chunks(BULK_ACK_CHUNK).enumerate() {
                        if index > 0 {
                            time::sleep(Duration::from_secs(1)).await;
                        }
                        retried("acknowledging channels", || ack_bulk(&http, chunk)).await;
                    }
                }
            }
        });
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MentionsFilter {
    pub guild_id: Option<Id<GuildMarker>>,

    pub roles: bool,

    pub everyone: bool,
}

pub const MENTIONS_PAGE: u8 = 25;

pub async fn recent_mentions(
    http: &HttpClient,
    filter: MentionsFilter,
    before: Option<Id<MessageMarker>>,
) -> Result<Vec<Message>, RestError> {
    let mut path = format!(
        "/users/@me/mentions?limit={MENTIONS_PAGE}&roles={}&everyone={}",
        filter.roles, filter.everyone
    );
    if let Some(guild_id) = filter.guild_id {
        path.push_str(&format!("&guild_id={guild_id}"));
    }
    if let Some(before) = before {
        path.push_str(&format!("&before={before}"));
    }
    let response = http
        .request("GET /users/@me/mentions", wreq::Method::GET, &path)
        .await?;
    Ok(response.json().await?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_ack_body_has_the_clients_shape() {
        let body = serde_json::to_value(AckBody {
            token: None,
            last_viewed: 4100,
            flags: Some(1),
        })
        .unwrap();
        assert_eq!(
            body,
            serde_json::json!({ "token": null, "last_viewed": 4100, "flags": 1 })
        );

        assert_eq!(
            serde_json::to_string(&AckBody {
                token: Some("t"),
                last_viewed: 4100,
                flags: None
            })
            .unwrap(),
            r#"{"token":"t","last_viewed":4100}"#
        );
    }

    #[test]
    fn a_bulk_ack_lists_channel_message_and_type() {
        let body = BulkBody {
            read_states: vec![BulkEntry {
                channel_id: Id::new(1),
                message_id: Id::new(2),
                read_state_type: 0,
            }],
        };
        assert_eq!(
            serde_json::to_value(body).unwrap(),
            serde_json::json!({ "read_states": [{ "channel_id": "1", "message_id": "2", "read_state_type": 0 }] })
        );
    }
}
