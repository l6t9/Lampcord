use crate::models::id::marker;
use serde::Deserialize;

use crate::models::Id;
use crate::models::id::marker::GuildMarker;
use crate::rest::{HttpClient, RestError};

#[derive(Deserialize)]
struct TopEmojis {
    items: Vec<Item>,
}

#[derive(Deserialize)]
struct Item {
    emoji_id: Id<marker::EmojiMarker>,
    #[serde(default)]
    emoji_rank: i64,
}

pub async fn get_top_emojis(
    http: &HttpClient,
    guild_id: Id<GuildMarker>,
) -> Result<Vec<u64>, RestError> {
    let path = format!("/guilds/{guild_id}/top-emojis");
    let response = http
        .request("GET /guilds/:id/top-emojis", wreq::Method::GET, &path)
        .await?;
    let parsed: TopEmojis = response.json().await?;
    Ok(ranked_ids(parsed))
}

fn ranked_ids(mut parsed: TopEmojis) -> Vec<u64> {
    parsed.items.sort_by_key(|item| item.emoji_rank);
    parsed
        .items
        .iter()
        .map(|item| item.emoji_id.get())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_come_back_in_rank_order() {
        let parsed: TopEmojis = serde_json::from_str(
            r#"{"items":[{"emoji_id":"20","emoji_rank":2},{"emoji_id":"10","emoji_rank":1}]}"#,
        )
        .unwrap();
        assert_eq!(ranked_ids(parsed), [10, 20]);
    }
}
