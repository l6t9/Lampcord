use crate::rest::RequestExtras;
use serde::{Deserialize, Serialize};

use crate::models::id::marker::ChannelMarker;
use crate::models::{Id, Message};

use super::super::{HttpClient, RestError};

pub const MAX_ATTACHMENTS_PER_MESSAGE: usize = 10;

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct UploadFileRequest {
    pub id: String,
    pub filename: String,
    pub file_size: u64,
    pub is_clip: bool,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub original_content_type: Option<String>,
}

#[derive(Serialize)]
struct CreateAttachmentsPayload<'a> {
    files: &'a [UploadFileRequest],
}

#[derive(Debug, Clone, Deserialize)]
pub struct CloudAttachment {
    #[serde(default)]
    pub id: Option<String>,
    pub upload_url: String,
    pub upload_filename: String,
}

#[derive(Deserialize)]
struct CreateAttachmentsResponse {
    attachments: Vec<CloudAttachment>,
}

pub async fn create_upload_urls(
    http: &HttpClient,
    channel_id: Id<ChannelMarker>,
    files: &[UploadFileRequest],
) -> Result<Vec<CloudAttachment>, RestError> {
    let response = http
        .request_with_json(
            "POST /channels/:id/attachments",
            wreq::Method::POST,
            &format!("/channels/{channel_id}/attachments"),
            &CreateAttachmentsPayload { files },
        )
        .await?;
    Ok(response
        .json::<CreateAttachmentsResponse>()
        .await?
        .attachments)
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct MessageAttachmentRef {
    pub id: String,
    pub filename: String,
    pub uploaded_filename: String,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub is_spoiler: bool,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub original_content_type: Option<String>,
}

pub async fn send_message_with_attachments(
    http: &HttpClient,
    channel_id: Id<ChannelMarker>,
    content: &str,
    nonce: &str,
    attachments: &[MessageAttachmentRef],
    reply: Option<&super::messages::ReplyTo>,
) -> Result<Message, RestError> {
    let response = http
        .request_with(
            "POST /channels/:id/messages",
            wreq::Method::POST,
            &format!("/channels/{channel_id}/messages"),
            Some(&super::messages::SendBody::new(
                content,
                nonce,
                reply,
                Some(attachments),
            )),
            &RequestExtras {
                retries: 3,
                requeue_on_429: true,
                ..RequestExtras::context("chat_input")
            },
        )
        .await?;
    Ok(response.json().await?)
}

pub async fn delete_upload(http: &HttpClient, upload_filename: &str) -> Result<(), RestError> {
    let segment =
        percent_encoding::utf8_percent_encode(upload_filename, percent_encoding::NON_ALPHANUMERIC);
    http.request(
        "DELETE /attachments/:name",
        wreq::Method::DELETE,
        &format!("/attachments/{segment}"),
    )
    .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn payloads_match_the_documented_shapes() {
        let files = [UploadFileRequest {
            id: "0".into(),
            filename: "cat.png".into(),
            file_size: 12,
            is_clip: false,
            original_content_type: Some("image/png".into()),
        }];
        assert_eq!(
            serde_json::to_value(CreateAttachmentsPayload { files: &files }).unwrap(),
            serde_json::json!({"files": [{"id": "0", "filename": "cat.png", "file_size": 12, "is_clip": false, "original_content_type": "image/png"}]})
        );
        let refs = [
            MessageAttachmentRef {
                id: "0".into(),
                filename: "cat.png".into(),
                uploaded_filename: "uploads/1/cat.png".into(),
                is_spoiler: true,
                description: Some("a cat".into()),
                original_content_type: Some("image/png".into()),
            },
            MessageAttachmentRef {
                id: "1".into(),
                filename: "a.txt".into(),
                uploaded_filename: "uploads/2/a.txt".into(),
                is_spoiler: false,
                description: None,
                original_content_type: None,
            },
        ];
        assert_eq!(
            serde_json::to_value(super::super::messages::SendBody::new(
                "hi",
                "42",
                None,
                Some(&refs)
            ))
            .unwrap(),
            serde_json::json!({"mobile_network_type": "unknown", "content": "hi", "nonce": "42", "tts": false, "flags": 0, "attachments": [
                {"id": "0", "filename": "cat.png", "uploaded_filename": "uploads/1/cat.png", "is_spoiler": true, "description": "a cat", "original_content_type": "image/png"},
                {"id": "1", "filename": "a.txt", "uploaded_filename": "uploads/2/a.txt"}
            ]})
        );
    }

    #[test]
    fn parses_the_upload_url_response() {
        let body = r#"{"attachments":[{"id":"0","upload_url":"https://s3.example/x?sig=1","upload_filename":"uploads/9/cat.png"}]}"#;
        let parsed: CreateAttachmentsResponse = serde_json::from_str(body).unwrap();
        assert_eq!(parsed.attachments[0].upload_filename, "uploads/9/cat.png");
        assert_eq!(parsed.attachments[0].id.as_deref(), Some("0"));
    }
}
