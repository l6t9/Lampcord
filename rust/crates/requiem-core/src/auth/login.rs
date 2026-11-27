use crate::rest;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::api::ApiError;
use super::captcha::CaptchaSolution;
use crate::config::BackendConfig;
use crate::properties::SuperProperties;

#[derive(Debug, Error)]
pub enum LoginError {
    #[error("request failed: {0}")]
    Request(#[from] wreq::Error),
    #[error("login requires MFA (ticket: {ticket})")]
    MfaRequired { ticket: String },
    #[error("login failed: {0}")]
    Rejected(String),

    #[error("POST {url} failed: {error}")]
    Api { url: String, error: ApiError },
}

#[derive(Serialize)]
struct LoginRequest<'a> {
    login: &'a str,
    password: &'a str,
    undelete: bool,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum LoginResponse {
    Success { token: String },
    Mfa { mfa: bool, ticket: String },
    Error(serde_json::Value),
}

pub async fn login(
    client: &wreq::Client,
    config: &BackendConfig,
    super_properties: &SuperProperties,
    username: &str,
    password: &str,
    captcha: Option<&CaptchaSolution>,
) -> Result<String, LoginError> {
    let url = format!("{}/auth/login", config.rest_base_url.trim_end_matches('/'));

    super::fingerprint::ensure(client, config, super_properties).await;
    let extras = rest::RequestExtras {
        headers: captcha
            .map(CaptchaSolution::header_pairs)
            .unwrap_or_default(),
        ..Default::default()
    };
    let (headers, order) = rest::anonymous_request_headers(
        &config.web_base_url(),
        super_properties,
        &wreq::Method::POST,
        true,
        &extras,
        super::fingerprint::current().as_deref(),
    );
    let response = client
        .post(&url)
        .headers(headers)
        .orig_headers(order)
        .json(&LoginRequest {
            login: username,
            password,
            undelete: false,
        })
        .send()
        .await?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(LoginError::Api {
            url,
            error: ApiError::parse(status.as_u16(), &body),
        });
    }

    let body: LoginResponse = response.json().await?;
    match body {
        LoginResponse::Success { token } => {
            super::fingerprint::forget();
            Ok(token)
        }
        LoginResponse::Mfa { mfa: true, ticket } => Err(LoginError::MfaRequired { ticket }),
        LoginResponse::Mfa { ticket, .. } => Err(LoginError::MfaRequired { ticket }),
        LoginResponse::Error(value) => Err(LoginError::Rejected(value.to_string())),
    }
}
