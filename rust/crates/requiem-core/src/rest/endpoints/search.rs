use std::time::Duration;
use tokio::time;

use serde::Deserialize;

use crate::models::id::marker::{ChannelMarker, GuildMarker};
use crate::models::{Id, Message};

use super::super::{HttpClient, RestError};

pub const PAGE_SIZE: u32 = 25;

const MAX_INDEXING_RETRIES: u32 = 12;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SearchScope {
    Guild(Id<GuildMarker>),
    Channel(Id<ChannelMarker>),
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum SortMode {
    #[default]
    Newest,
    Oldest,
    MostRelevant,
}

impl SortMode {
    pub const ALL: [SortMode; 3] = [SortMode::Newest, SortMode::Oldest, SortMode::MostRelevant];

    pub fn label(self) -> &'static str {
        match self {
            SortMode::Newest => "Newest",
            SortMode::Oldest => "Oldest",
            SortMode::MostRelevant => "Most Relevant",
        }
    }

    fn params(self) -> (&'static str, &'static str) {
        match self {
            SortMode::Newest => ("timestamp", "desc"),
            SortMode::Oldest => ("timestamp", "asc"),
            SortMode::MostRelevant => ("relevance", "desc"),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct SearchParams {
    pub content: String,
    pub author_ids: Vec<u64>,
    pub channel_ids: Vec<u64>,
    pub mentions: Vec<u64>,
    pub has: Vec<String>,
    pub author_types: Vec<String>,
    pub pinned: Option<bool>,

    pub min_id: Option<u64>,

    pub max_id: Option<u64>,
    pub sort: SortMode,
}

impl SearchParams {
    pub fn filter_count(&self) -> usize {
        self.author_ids.len()
            + self.channel_ids.len()
            + self.mentions.len()
            + self.has.len()
            + self.author_types.len()
            + usize::from(self.pinned.is_some())
            + usize::from(self.min_id.is_some())
            + usize::from(self.max_id.is_some())
    }

    fn query_string(&self, offset: u32) -> String {
        fn join(ids: impl IntoIterator<Item = String>) -> String {
            ids.into_iter().collect::<Vec<_>>().join(",")
        }
        let mut parts: Vec<String> = Vec::new();
        let encode = |s: &str| {
            percent_encoding::utf8_percent_encode(s, percent_encoding::NON_ALPHANUMERIC).to_string()
        };
        if !self.content.is_empty() {
            parts.push(format!("content={}", encode(&self.content)));
        }
        if !self.author_ids.is_empty() {
            parts.push(format!(
                "author_id={}",
                join(self.author_ids.iter().map(u64::to_string))
            ));
        }
        if !self.channel_ids.is_empty() {
            parts.push(format!(
                "channel_id={}",
                join(self.channel_ids.iter().map(u64::to_string))
            ));
        }
        if !self.mentions.is_empty() {
            parts.push(format!(
                "mentions={}",
                join(self.mentions.iter().map(u64::to_string))
            ));
        }
        if !self.has.is_empty() {
            parts.push(format!("has={}", join(self.has.iter().cloned())));
        }
        if !self.author_types.is_empty() {
            parts.push(format!(
                "author_type={}",
                join(self.author_types.iter().cloned())
            ));
        }
        if let Some(pinned) = self.pinned {
            parts.push(format!("pinned={pinned}"));
        }
        if let Some(min_id) = self.min_id {
            parts.push(format!("min_id={min_id}"));
        }
        if let Some(max_id) = self.max_id {
            parts.push(format!("max_id={max_id}"));
        }
        let (sort_by, sort_order) = self.sort.params();
        parts.push(format!("sort_by={sort_by}"));
        parts.push(format!("sort_order={sort_order}"));
        parts.push(format!("offset={offset}"));
        parts.join("&")
    }
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct SearchPage {
    #[serde(default)]
    pub total_results: u32,

    #[serde(default)]
    pub messages: Vec<Vec<Message>>,
}

impl SearchPage {
    pub fn hits(&self) -> impl Iterator<Item = &Message> {
        self.messages.iter().filter_map(|group| group.first())
    }
}

#[derive(Deserialize)]
struct IndexingBody {
    #[serde(default)]
    retry_after: Option<f64>,
}

pub async fn search(
    http: &HttpClient,
    scope: SearchScope,
    params: &SearchParams,
    page: u32,
) -> Result<SearchPage, RestError> {
    let (route, base) = match scope {
        SearchScope::Guild(id) => (
            "GET /guilds/:id/messages/search",
            format!("/guilds/{id}/messages/search"),
        ),
        SearchScope::Channel(id) => (
            "GET /channels/:id/messages/search",
            format!("/channels/{id}/messages/search"),
        ),
    };
    let path = format!(
        "{base}?{}",
        params.query_string(page.saturating_sub(1) * PAGE_SIZE)
    );
    let mut attempts = 0;
    loop {
        let response = http.request(route, wreq::Method::GET, &path).await?;
        if response.status() != wreq::StatusCode::ACCEPTED || attempts >= MAX_INDEXING_RETRIES {
            return Ok(response.json().await?);
        }
        attempts += 1;
        let wait = response
            .json::<IndexingBody>()
            .await
            .ok()
            .and_then(|b| b.retry_after)
            .unwrap_or(2.0)
            .clamp(0.5, 10.0);
        time::sleep(Duration::from_secs_f64(wait)).await;
    }
}
