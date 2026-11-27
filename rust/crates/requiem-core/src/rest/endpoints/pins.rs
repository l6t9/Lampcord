use serde::Deserialize;

use crate::models::id::marker::{ChannelMarker, MessageMarker};
use crate::models::{Id, Message};
use crate::state::Pin;

use super::super::{HttpClient, RequestExtras, RestError};

pub const PAGE_SIZE: u32 = 25;

#[derive(Deserialize)]
struct PinsPage {
    #[serde(default)]
    items: Vec<PinWire>,
    #[serde(default)]
    has_more: bool,
}

#[derive(Deserialize)]
struct PinWire {
    pinned_at: String,
    message: Message,
}

pub async fn list(
    http: &HttpClient,
    channel_id: Id<ChannelMarker>,
    before: Option<&str>,
) -> Result<(Vec<Pin>, bool), RestError> {
    let mut path = format!("/channels/{channel_id}/messages/pins?limit={PAGE_SIZE}");
    if let Some(before) = before {
        path.push_str("&before=");
        path.push_str(
            &percent_encoding::utf8_percent_encode(before, percent_encoding::NON_ALPHANUMERIC)
                .to_string(),
        );
    }
    let response = http
        .request_with::<()>(
            "GET /channels/:id/messages/pins",
            wreq::Method::GET,
            &path,
            None,
            &RequestExtras {
                retries: 2,
                ..RequestExtras::default()
            },
        )
        .await?;
    let page: PinsPage = response.json().await?;
    let pins = page
        .items
        .into_iter()
        .map(|pin| Pin {
            pinned_at: pin.pinned_at,
            message: pin.message,
        })
        .collect();
    Ok((pins, page.has_more))
}

pub async fn pin(
    http: &HttpClient,
    channel_id: Id<ChannelMarker>,
    message_id: Id<MessageMarker>,
) -> Result<(), RestError> {
    http.request_with::<()>(
        "PUT /channels/:id/messages/pins/:id",
        wreq::Method::PUT,
        &format!("/channels/{channel_id}/messages/pins/{message_id}"),
        None,
        &RequestExtras::default(),
    )
    .await?;
    Ok(())
}

pub async fn unpin(
    http: &HttpClient,
    channel_id: Id<ChannelMarker>,
    message_id: Id<MessageMarker>,
) -> Result<(), RestError> {
    http.request_with::<()>(
        "DELETE /channels/:id/messages/pins/:id",
        wreq::Method::DELETE,
        &format!("/channels/{channel_id}/messages/pins/{message_id}"),
        None,
        &RequestExtras::default(),
    )
    .await?;
    Ok(())
}

pub async fn ack(http: &HttpClient, channel_id: Id<ChannelMarker>) -> Result<(), RestError> {
    http.request_with::<()>(
        "POST /channels/:id/pins/ack",
        wreq::Method::POST,
        &format!("/channels/{channel_id}/pins/ack"),
        None,
        &RequestExtras::default(),
    )
    .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn a_page_decodes_from_the_live_shape() {
        let body = r#"{"items":[{"pinned_at":"2026-10-14T00:54:43.850269+00:00","message":{"id":"1559731026756894720","channel_id":"1559657301076353024","author":{"id":"1527666663510511616","username":"testing","global_name":"testing","discriminator":"0","avatar":null},"content":"pin me please","type":0,"timestamp":"2026-10-14T00:54:15.182623+00:00","pinned":true,"flags":0}}],"has_more":false}"#;
        let page: super::PinsPage = serde_json::from_str(body).unwrap();
        assert_eq!(page.items.len(), 1);
        assert!(page.items[0].message.pinned);
        assert!(!page.has_more);
    }
}
