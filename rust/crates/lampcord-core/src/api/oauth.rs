use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    rest::{Body, HttpMethod, RestConfig},
    rest_client::RestClient,
    traits::{CaptchaSolver, RestTransport},
};

use super::{ApiError, send_json};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct OAuthAuthorizationRequest {
    pub client_id: String,
    pub response_type: String,
    #[serde(default)]
    pub scopes: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub redirect_uri: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nonce: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub code_challenge: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub code_challenge_method: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub integration_type: Option<i32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct OAuthPreviewApplication {
    pub id: String,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct OAuthAuthorizePreview {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub application: Option<OAuthPreviewApplication>,
    #[serde(default)]
    pub authorized: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub integration_type: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub redirect_uri: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct OAuthAuthorizeResponse {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

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

fn build_query(request: &OAuthAuthorizationRequest) -> String {
    let mut params: Vec<(String, String)> = Vec::new();
    params.push(("client_id".to_owned(), request.client_id.clone()));
    params.push(("response_type".to_owned(), request.response_type.clone()));
    params.push(("scope".to_owned(), request.scopes.join(" ")));
    if let Some(v) = &request.redirect_uri {
        params.push(("redirect_uri".to_owned(), v.clone()));
    }
    if let Some(v) = &request.state {
        params.push(("state".to_owned(), v.clone()));
    }
    if let Some(v) = &request.nonce {
        params.push(("nonce".to_owned(), v.clone()));
    }
    if let Some(v) = &request.code_challenge {
        params.push(("code_challenge".to_owned(), v.clone()));
    }
    if let Some(v) = &request.code_challenge_method {
        params.push(("code_challenge_method".to_owned(), v.clone()));
    }
    if let Some(v) = request.integration_type {
        params.push(("integration_type".to_owned(), v.to_string()));
    }
    params
        .into_iter()
        .map(|(k, v)| format!("{}={}", url_encode_component(&k), url_encode_component(&v)))
        .collect::<Vec<_>>()
        .join("&")
}

pub async fn preview<T, S>(
    client: &RestClient<T, S>,
    config: &RestConfig,
    request: &OAuthAuthorizationRequest,
) -> Result<OAuthAuthorizePreview, ApiError>
where
    T: RestTransport,
    S: CaptchaSolver,
{
    let path = format!("/oauth2/authorize?{}", build_query(request));
    let req = config.request(
        HttpMethod::Get,
        &path,
        config.standard_headers(),
        Body::None,
    );
    send_json(client, req).await
}

pub async fn authorize<T, S>(
    client: &RestClient<T, S>,
    config: &RestConfig,
    request: &OAuthAuthorizationRequest,
    approve: bool,
) -> Result<String, ApiError>
where
    T: RestTransport,
    S: CaptchaSolver,
{
    let path = format!("/oauth2/authorize?{}", build_query(request));
    let mut body = serde_json::json!({ "authorize": approve });
    if let Some(it) = request.integration_type {
        body["integration_type"] = Value::Number(it.into());
    }
    let req = config.request(
        HttpMethod::Post,
        &path,
        config.standard_headers(),
        Body::Json(body.to_string()),
    );
    let response: OAuthAuthorizeResponse = send_json(client, req).await?;
    response.url.ok_or_else(|| ApiError::Http {
        status: 200,
        body: "authorize returned no redirect URL".to_owned(),
    })
}
