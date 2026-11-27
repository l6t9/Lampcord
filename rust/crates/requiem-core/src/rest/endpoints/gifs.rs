use serde::Deserialize;

use super::super::{HttpClient, RestError};

#[derive(Clone, Deserialize)]
pub struct GifCategory {
    pub name: String,
    pub src: String,
}

#[derive(Clone, Default, Deserialize)]
pub struct Trending {
    pub categories: Vec<GifCategory>,
    #[serde(default)]
    pub gifs: Vec<GifResult>,
}

#[derive(Clone, Deserialize)]
pub struct GifResult {
    pub id: String,
    #[allow(
        dead_code,
        reason = "not currently displayed anywhere, kept since it's real response data a future alt-text/hover-title feature would want"
    )]
    pub title: String,
    pub url: String,
    pub gif_src: String,
    pub preview: String,
    pub width: u32,
    pub height: u32,
}

pub async fn get_trending(http: &HttpClient) -> Result<Trending, RestError> {
    let response = http
        .request(
            "GET /gifs/trending",
            wreq::Method::GET,
            "/gifs/trending?locale=en-US&media_format=gif",
        )
        .await?;
    Ok(response.json().await?)
}

pub async fn search_gifs(http: &HttpClient, query: &str) -> Result<Vec<GifResult>, RestError> {
    let encoded_query =
        percent_encoding::utf8_percent_encode(query, percent_encoding::NON_ALPHANUMERIC);
    let path = format!("/gifs/search?q={encoded_query}&locale=en-US&media_format=gif");
    let response = http
        .request("GET /gifs/search", wreq::Method::GET, &path)
        .await?;
    Ok(response.json().await?)
}
