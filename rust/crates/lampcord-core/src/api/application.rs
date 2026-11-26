use serde_json::Value;

use crate::{
    rest::{Body, HttpMethod, RestConfig},
    rest_client::RestClient,
    traits::{CaptchaSolver, RestTransport},
};

use super::{ApiError, send_empty, send_json};

pub async fn get_applications<T, S>(
    client: &RestClient<T, S>,
    config: &RestConfig,
    application_ids: &[String],
) -> Result<Vec<Value>, ApiError>
where
    T: RestTransport,
    S: CaptchaSolver,
{
    if application_ids.is_empty() {
        return Ok(Vec::new());
    }
    let ids = application_ids.join(",");
    let path = format!("/applications/public?application_ids={ids}");
    let req = config.request(
        HttpMethod::Get,
        &path,
        config.standard_headers(),
        Body::None,
    );
    send_json(client, req).await
}

pub async fn send_interaction<T, S>(
    client: &RestClient<T, S>,
    config: &RestConfig,
    body: &Value,
) -> Result<(), ApiError>
where
    T: RestTransport,
    S: CaptchaSolver,
{
    let req = config.request(
        HttpMethod::Post,
        &format!("/interactions"),
        config.standard_headers(),
        Body::Json(body.to_string()),
    );
    send_empty(client, req).await
}

pub async fn get_guild_command_index<T, S>(
    client: &RestClient<T, S>,
    config: &RestConfig,
    guild_id: u64,
) -> Result<Value, ApiError>
where
    T: RestTransport,
    S: CaptchaSolver,
{
    let req = config.request(
        HttpMethod::Get,
        &format!("/guilds/{guild_id}/application-command-index"),
        config.standard_headers(),
        Body::None,
    );
    send_json(client, req).await
}

pub async fn get_channel_command_index<T, S>(
    client: &RestClient<T, S>,
    config: &RestConfig,
    channel_id: u64,
) -> Result<Value, ApiError>
where
    T: RestTransport,
    S: CaptchaSolver,
{
    let req = config.request(
        HttpMethod::Get,
        &format!("/channels/{channel_id}/application-command-index"),
        config.standard_headers(),
        Body::None,
    );
    send_json(client, req).await
}
