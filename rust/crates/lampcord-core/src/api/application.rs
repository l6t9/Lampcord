use serde_json::Value;

use crate::{
    rest::{Body, HttpMethod, RestConfig},
    rest_client::RestClient,
    traits::{CaptchaSolver, RestTransport},
};

use super::{ApiError, send_json};

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
