use crate::models::id;
use crate::plugins::IncomingSource;
use crate::plugins;
use crate::rest::RequestExtras;
use serde::Serialize;
use std::time::SystemTime;
use std::time;

use crate::models::id::marker::{ChannelMarker, GuildMarker, MessageMarker};
use crate::models::{Id, Message};

use super::super::{HttpClient, RestError};

async fn history(messages: Vec<Message>) -> Result<Vec<Message>, RestError> {
    plugins::transform(&messages, IncomingSource::History).await;
    Ok(messages)
}

pub const INITIAL_PAGE_SIZE: u8 = 10;

pub const OLDER_PAGE_SIZE: u8 = 20;

pub const AROUND_PAGE_SIZE: u8 = 50;

pub async fn get_channel_messages_before(
    http: &HttpClient,
    channel_id: Id<ChannelMarker>,
    before: Id<MessageMarker>,
    limit: u8,
) -> Result<Vec<Message>, RestError> {
    let response = http
        .request_with::<()>(
            "GET /channels/:id/messages",
            wreq::Method::GET,
            &format!("/channels/{channel_id}/messages?before={before}&limit={limit}"),
            None,
            &RequestExtras {
                retries: 2,
                ..Default::default()
            },
        )
        .await?;
    history(response.json().await?).await
}

pub async fn get_channel_messages_after(
    http: &HttpClient,
    channel_id: Id<ChannelMarker>,
    after: Id<MessageMarker>,
    limit: u8,
) -> Result<Vec<Message>, RestError> {
    let response = http
        .request_with::<()>(
            "GET /channels/:id/messages",
            wreq::Method::GET,
            &format!("/channels/{channel_id}/messages?after={after}&limit={limit}"),
            None,
            &RequestExtras {
                retries: 2,
                ..Default::default()
            },
        )
        .await?;
    history(response.json().await?).await
}

pub async fn get_channel_messages_around(
    http: &HttpClient,
    channel_id: Id<ChannelMarker>,
    around: Id<MessageMarker>,
    limit: u8,
) -> Result<Vec<Message>, RestError> {
    let response = http
        .request_with::<()>(
            "GET /channels/:id/messages",
            wreq::Method::GET,
            &format!("/channels/{channel_id}/messages?around={around}&limit={limit}"),
            None,
            &RequestExtras {
                retries: 2,
                ..Default::default()
            },
        )
        .await?;
    history(response.json().await?).await
}

pub async fn get_channel_messages(
    http: &HttpClient,
    channel_id: Id<ChannelMarker>,
    limit: u8,
) -> Result<Vec<Message>, RestError> {
    let response = http
        .request_with::<()>(
            "GET /channels/:id/messages",
            wreq::Method::GET,
            &format!("/channels/{channel_id}/messages?limit={limit}"),
            None,
            &RequestExtras {
                retries: 2,
                ..Default::default()
            },
        )
        .await?;
    history(response.json().await?).await
}

#[derive(Debug, Clone)]
pub struct ReplyTo {
    pub message_id: Id<MessageMarker>,
    pub channel_id: Id<ChannelMarker>,

    pub guild_id: Option<Id<GuildMarker>>,
    pub pings: bool,
}

#[derive(Serialize)]
struct MessageReferencePayload {
    #[serde(skip_serializing_if = "Option::is_none")]
    guild_id: Option<String>,
    channel_id: String,
    message_id: String,
}

#[derive(Serialize)]
struct AllowedMentionsPayload {
    parse: [&'static str; 3],
    replied_user: bool,
}

#[derive(Serialize)]
pub struct SendBody<'a> {
    mobile_network_type: &'static str,
    content: &'a str,
    nonce: &'a str,
    tts: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    message_reference: Option<MessageReferencePayload>,

    #[serde(skip_serializing_if = "Option::is_none")]
    allowed_mentions: Option<AllowedMentionsPayload>,
    flags: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    attachments: Option<&'a [super::attachments::MessageAttachmentRef]>,
}

impl<'a> SendBody<'a> {
    pub fn new(
        content: &'a str,
        nonce: &'a str,
        reply: Option<&ReplyTo>,
        attachments: Option<&'a [super::attachments::MessageAttachmentRef]>,
    ) -> Self {
        Self {
            mobile_network_type: "unknown",
            content,
            nonce,
            tts: false,
            message_reference: reply.map(|r| MessageReferencePayload {
                guild_id: r.guild_id.map(|g| g.to_string()),
                channel_id: r.channel_id.to_string(),
                message_id: r.message_id.to_string(),
            }),
            allowed_mentions: reply.filter(|r| !r.pings).map(|_| AllowedMentionsPayload {
                parse: ["users", "roles", "everyone"],
                replied_user: false,
            }),
            flags: 0,
            attachments,
        }
    }
}

const EDIT_EXTRAS: RequestExtras = RequestExtras {
    context: None,
    reason: None,
    headers: Vec::new(),
    failed: 0,
    retries: 1,
    requeue_on_429: true,
};

pub fn new_nonce() -> String {
    let now_ms = SystemTime::now()
        .duration_since(time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);
    (now_ms.saturating_sub(id::DISCORD_EPOCH_MS as u64) << 22).to_string()
}

pub async fn send_message(
    http: &HttpClient,
    channel_id: Id<ChannelMarker>,
    content: &str,
) -> Result<Message, RestError> {
    send_reply(http, channel_id, content, None).await
}

pub async fn send_reply(
    http: &HttpClient,
    channel_id: Id<ChannelMarker>,
    content: &str,
    reply: Option<&ReplyTo>,
) -> Result<Message, RestError> {
    send_reply_from(http, channel_id, content, reply, send_location::CHAT_INPUT).await
}

pub mod send_location {

    pub const CHAT_INPUT: &str = "chat_input";

    pub const USER_PROFILE: &str = "user_profile";

    pub const INVITE_SUGGESTION: &str = "Guild Create Invite Suggestion";
}

pub async fn send_reply_from(
    http: &HttpClient,
    channel_id: Id<ChannelMarker>,
    content: &str,
    reply: Option<&ReplyTo>,
    location: &str,
) -> Result<Message, RestError> {
    let (content, reply) = plugins::before_send(channel_id, content, reply).await?;
    let nonce = new_nonce();
    let response = http
        .request_with(
            "POST /channels/:id/messages",
            wreq::Method::POST,
            &format!("/channels/{channel_id}/messages"),
            Some(&SendBody::new(&content, &nonce, reply.as_ref(), None)),
            &RequestExtras {
                retries: 3,
                requeue_on_429: true,
                ..RequestExtras::context(location)
            },
        )
        .await?;
    Ok(response.json().await?)
}

pub async fn delete_message(
    http: &HttpClient,
    channel_id: Id<ChannelMarker>,
    message_id: Id<MessageMarker>,
) -> Result<(), RestError> {
    http.request(
        "DELETE /channels/:id/messages/:id",
        wreq::Method::DELETE,
        &format!("/channels/{channel_id}/messages/{message_id}"),
    )
    .await?;
    Ok(())
}

#[derive(Serialize)]
struct EditContentPayload<'a> {
    content: &'a str,
}

pub async fn edit_message_content(
    http: &HttpClient,
    channel_id: Id<ChannelMarker>,
    message_id: Id<MessageMarker>,
    original: &str,
    content: &str,
) -> Result<(), RestError> {
    let content = plugins::before_edit(channel_id, message_id, original, content).await?;
    http.request_with(
        "PATCH /channels/:id/messages/:id",
        wreq::Method::PATCH,
        &format!("/channels/{channel_id}/messages/{message_id}"),
        Some(&EditContentPayload { content: &content }),
        &EDIT_EXTRAS,
    )
    .await?;
    Ok(())
}

#[derive(Serialize)]
pub struct AttachmentPatch {
    pub id: String,
    pub filename: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[derive(Serialize)]
struct EditAttachmentsPayload<'a> {
    attachments: &'a [AttachmentPatch],
}

pub async fn edit_message_attachments(
    http: &HttpClient,
    channel_id: Id<ChannelMarker>,
    message_id: Id<MessageMarker>,
    keep: &[AttachmentPatch],
) -> Result<(), RestError> {
    http.request_with(
        "PATCH /channels/:id/messages/:id",
        wreq::Method::PATCH,
        &format!("/channels/{channel_id}/messages/{message_id}"),
        Some(&EditAttachmentsPayload { attachments: keep }),
        &EDIT_EXTRAS,
    )
    .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_plain_send_has_discords_fields_in_discords_order() {
        let body = serde_json::to_string(&SendBody::new("hi", "1234", None, None)).unwrap();
        assert_eq!(
            body,
            r#"{"mobile_network_type":"unknown","content":"hi","nonce":"1234","tts":false,"flags":0}"#
        );
    }

    #[test]
    fn a_reply_names_its_guild_and_only_a_silent_reply_sets_allowed_mentions() {
        let silent = ReplyTo {
            message_id: Id::new(7),
            channel_id: Id::new(9),
            guild_id: Some(Id::new(3)),
            pings: false,
        };
        assert_eq!(
            serde_json::to_string(&SendBody::new("hi", "1", Some(&silent), None)).unwrap(),
            r#"{"mobile_network_type":"unknown","content":"hi","nonce":"1","tts":false,"message_reference":{"guild_id":"3","channel_id":"9","message_id":"7"},"allowed_mentions":{"parse":["users","roles","everyone"],"replied_user":false},"flags":0}"#
        );
        let pinging = ReplyTo {
            pings: true,
            guild_id: None,
            ..silent
        };
        assert_eq!(
            serde_json::to_string(&SendBody::new("hi", "1", Some(&pinging), None)).unwrap(),
            r#"{"mobile_network_type":"unknown","content":"hi","nonce":"1","tts":false,"message_reference":{"channel_id":"9","message_id":"7"},"flags":0}"#
        );
    }

    #[test]
    fn nonces_are_snowflakes_of_the_current_time() {
        let nonce: u64 = new_nonce().parse().unwrap();
        let ms = (nonce >> 22) + id::DISCORD_EPOCH_MS as u64;
        let now = SystemTime::now()
            .duration_since(time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        assert!(now.abs_diff(ms) < 2000);
        assert_eq!(nonce & ((1 << 22) - 1), 0);
    }
}
