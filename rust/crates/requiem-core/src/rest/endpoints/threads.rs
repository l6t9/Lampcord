use serde::Deserialize;

use crate::models::channel::ThreadMember;
use crate::models::id::marker::{ChannelMarker, MessageMarker};
use crate::models::{Channel, Id, Message};

use super::super::HttpClient;

#[derive(Clone, Debug, Default, Deserialize)]
pub struct ThreadSearch {
    #[serde(default)]
    pub threads: Vec<Channel>,
    /// The current user's membership in the returned threads.
    #[serde(default)]
    pub members: Vec<ThreadMember>,
    #[serde(default)]
    pub has_more: bool,
    #[serde(default, deserialize_with = "crate::state::wire::lenient_vec")]
    pub most_recent_messages: Vec<Message>,
}

pub async fn create_from_message(
    http: &HttpClient,
    channel_id: Id<ChannelMarker>,
    message_id: Id<MessageMarker>,
    name: &str,
    auto_archive_duration: u32,
) -> Result<Channel, String> {
    let response = http
        .request_with_json(
            "POST /channels/:id/messages/:id/threads",
            wreq::Method::POST,
            &format!("/channels/{channel_id}/messages/{message_id}/threads"),
            &serde_json::json!({
                "name": name,
                "auto_archive_duration": auto_archive_duration,
            }),
        )
        .await
        .map_err(|e| super::error_message(&e))?;
    response.json().await.map_err(|e| e.to_string())
}

pub async fn create(
    http: &HttpClient,
    channel_id: Id<ChannelMarker>,
    name: &str,
    kind: u8,
    auto_archive_duration: u32,
    invitable: bool,
) -> Result<Channel, String> {
    let response = http
        .request_with_json(
            "POST /channels/:id/threads",
            wreq::Method::POST,
            &format!("/channels/{channel_id}/threads"),
            &serde_json::json!({
                "name": name,
                "type": kind,
                "auto_archive_duration": auto_archive_duration,
                "invitable": invitable,
            }),
        )
        .await
        .map_err(|e| super::error_message(&e))?;
    response.json().await.map_err(|e| e.to_string())
}

/// What the thread browser asks `/threads/search` for.
pub enum ThreadSearchQuery<'a> {
    /// Threads whose name matches.
    Name(&'a str),
    /// One page of archived threads, newest activity first.
    Archived { offset: usize },
}

pub const ARCHIVED_PAGE_SIZE: usize = 25;

pub async fn search(
    http: &HttpClient,
    channel_id: Id<ChannelMarker>,
    query: ThreadSearchQuery<'_>,
) -> Result<ThreadSearch, String> {
    let query = match query {
        ThreadSearchQuery::Name(name) => format!(
            "name={}&tag_setting=match_some",
            percent_encoding::utf8_percent_encode(name, percent_encoding::NON_ALPHANUMERIC)
        ),
        ThreadSearchQuery::Archived { offset } => format!(
            "archived=true&sort_by=last_message_time&sort_order=desc&limit={ARCHIVED_PAGE_SIZE}&tag_setting=match_some&offset={offset}"
        ),
    };
    let response = http
        .request(
            "GET /channels/:id/threads/search",
            wreq::Method::GET,
            &format!("/channels/{channel_id}/threads/search?{query}"),
        )
        .await
        .map_err(|e| super::error_message(&e))?;
    response.json().await.map_err(|e| e.to_string())
}

pub async fn join(http: &HttpClient, thread_id: Id<ChannelMarker>) -> Result<(), String> {
    http.request(
        "PUT /channels/:id/thread-members/@me",
        wreq::Method::PUT,
        &format!("/channels/{thread_id}/thread-members/@me"),
    )
    .await
    .map_err(|e| super::error_message(&e))?;
    Ok(())
}

pub async fn leave(http: &HttpClient, thread_id: Id<ChannelMarker>) -> Result<(), String> {
    http.request(
        "DELETE /channels/:id/thread-members/@me",
        wreq::Method::DELETE,
        &format!("/channels/{thread_id}/thread-members/@me"),
    )
    .await
    .map_err(|e| super::error_message(&e))?;
    Ok(())
}

pub async fn update(
    http: &HttpClient,
    thread_id: Id<ChannelMarker>,
    fields: &serde_json::Value,
) -> Result<Channel, String> {
    let response = http
        .request_with_json(
            "PATCH /channels/:id",
            wreq::Method::PATCH,
            &format!("/channels/{thread_id}"),
            fields,
        )
        .await
        .map_err(|e| super::error_message(&e))?;
    response.json().await.map_err(|e| e.to_string())
}
