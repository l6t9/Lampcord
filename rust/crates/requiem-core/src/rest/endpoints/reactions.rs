use crate::models::id::marker::{ChannelMarker, MessageMarker};
use crate::models::{Id, ReactionEmoji};

use super::super::{HttpClient, RestError};

pub const LOCATION_HOVER_BAR: &str = "Message%20Hover%20Bar";

pub const LOCATION_INLINE_BUTTON: &str = "Message%20Inline%20Button";

pub const LOCATION_CONTEXT_MENU: &str = "Message%20Context%20Menu";

pub const LOCATION_REACTION_PICKER: &str = "Message%20Reaction%20Picker";

pub async fn add_reaction(
    http: &HttpClient,
    channel_id: Id<ChannelMarker>,
    message_id: Id<MessageMarker>,
    emoji: &ReactionEmoji,
    location: &str,
) -> Result<(), RestError> {
    let path = add_path(channel_id, message_id, emoji, location);
    http.request(
        "PUT /channels/:id/messages/:id/reactions/:emoji/@me",
        wreq::Method::PUT,
        &path,
    )
    .await?;
    Ok(())
}

pub async fn remove_reaction(
    http: &HttpClient,
    channel_id: Id<ChannelMarker>,
    message_id: Id<MessageMarker>,
    emoji: &ReactionEmoji,
    location: &str,
) -> Result<(), RestError> {
    let path = remove_path(channel_id, message_id, emoji, location);
    http.request(
        "DELETE /channels/:id/messages/:id/reactions/:emoji/:type/@me",
        wreq::Method::DELETE,
        &path,
    )
    .await?;
    Ok(())
}

fn add_path(
    channel_id: Id<ChannelMarker>,
    message_id: Id<MessageMarker>,
    emoji: &ReactionEmoji,
    location: &str,
) -> String {
    format!(
        "/channels/{channel_id}/messages/{message_id}/reactions/{}/@me?location={location}&type=0",
        emoji.path_segment()
    )
}

fn remove_path(
    channel_id: Id<ChannelMarker>,
    message_id: Id<MessageMarker>,
    emoji: &ReactionEmoji,
    location: &str,
) -> String {
    format!(
        "/channels/{channel_id}/messages/{message_id}/reactions/{}/0/@me?location={location}&burst=false",
        emoji.path_segment()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_and_remove_queries_are_the_ones_the_real_client_sends() {
        let emoji = ReactionEmoji::unicode("\u{1F44D}");
        let (channel, message) = (Id::new(1), Id::new(2));
        assert_eq!(
            add_path(channel, message, &emoji, LOCATION_HOVER_BAR),
            "/channels/1/messages/2/reactions/%F0%9F%91%8D/@me?location=Message%20Hover%20Bar&type=0"
        );
        assert_eq!(
            remove_path(channel, message, &emoji, LOCATION_INLINE_BUTTON),
            "/channels/1/messages/2/reactions/%F0%9F%91%8D/0/@me?location=Message%20Inline%20Button&burst=false"
        );
    }

    #[test]
    fn unicode_emoji_is_one_percent_encoded_segment() {
        assert_eq!(
            ReactionEmoji::unicode("\u{1F44D}").path_segment(),
            "%F0%9F%91%8D"
        );
    }

    #[test]
    fn custom_emoji_is_name_colon_id() {
        let emoji = ReactionEmoji {
            id: Some(Id::new(123)),
            name: Some("pepe".into()),
            animated: false,
        };
        assert_eq!(emoji.path_segment(), "pepe%3A123");
    }
}
