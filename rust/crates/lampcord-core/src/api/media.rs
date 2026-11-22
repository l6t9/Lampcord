use serde_json::Value;

use crate::{
    rest::{Body, HttpMethod, RestConfig},
    rest_client::RestClient,
    traits::{CaptchaSolver, RestTransport},
};

use super::{ApiError, send_json};

fn url_encode_component(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char)
            }
            b' ' => out.push_str("%20"),
            _ => out.push_str(&format!("%{:02X}", b)),
        }
    }
    out
}

pub async fn get_sticker_packs<T, S>(
    client: &RestClient<T, S>,
    config: &RestConfig,
) -> Result<Value, ApiError>
where
    T: RestTransport,
    S: CaptchaSolver,
{
    let req = config.request(
        HttpMethod::Get,
        "/sticker-packs",
        config.standard_headers(),
        Body::None,
    );
    send_json(client, req).await
}

pub async fn get_sticker_pack<T, S>(
    client: &RestClient<T, S>,
    config: &RestConfig,
    pack_id: &str,
) -> Result<Value, ApiError>
where
    T: RestTransport,
    S: CaptchaSolver,
{
    let req = config.request(
        HttpMethod::Get,
        &format!("/sticker-packs/{pack_id}"),
        config.standard_headers(),
        Body::None,
    );
    send_json(client, req).await
}

pub async fn get_trending_gif_categories<T, S>(
    client: &RestClient<T, S>,
    config: &RestConfig,
    locale: &str,
    media_format: &str,
) -> Result<Value, ApiError>
where
    T: RestTransport,
    S: CaptchaSolver,
{
    let path = format!(
        "/gifs/trending?locale={}&media_format={}",
        url_encode_component(locale),
        url_encode_component(media_format)
    );
    let req = config.request(
        HttpMethod::Get,
        &path,
        config.standard_headers(),
        Body::None,
    );
    send_json(client, req).await
}

pub async fn search_gifs<T, S>(
    client: &RestClient<T, S>,
    config: &RestConfig,
    query: &str,
    locale: &str,
    media_format: &str,
    limit: u16,
) -> Result<Value, ApiError>
where
    T: RestTransport,
    S: CaptchaSolver,
{
    let path = format!(
        "/gifs/search?q={}&provider=klipy&locale={}&media_format={}&limit={limit}",
        url_encode_component(query),
        url_encode_component(locale),
        url_encode_component(media_format)
    );
    let req = config.request(
        HttpMethod::Get,
        &path,
        config.standard_headers(),
        Body::None,
    );
    send_json(client, req).await
}
