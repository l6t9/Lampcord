use serde::{Deserialize, Serialize};

use super::super::{HttpClient, RestError};

#[derive(Clone, Debug, Deserialize)]
pub struct AuthSession {
    pub id_hash: String,

    pub approx_last_used_time: String,
    pub client_info: ClientInfo,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ClientInfo {
    pub os: Option<String>,
    pub platform: Option<String>,
    pub location: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct SessionsResponse {
    pub user_sessions: Vec<AuthSession>,
}

pub async fn get_sessions(http: &HttpClient) -> Result<SessionsResponse, RestError> {
    let response = http
        .request("GET /auth/sessions", wreq::Method::GET, "/auth/sessions")
        .await?;
    Ok(response.json().await?)
}

#[derive(Serialize)]
struct LogoutPayload {
    provider: Option<String>,
    voip_provider: Option<String>,
}

pub async fn logout(http: &HttpClient) -> Result<(), RestError> {
    http.request_with_json(
        "POST /auth/logout",
        wreq::Method::POST,
        "/auth/logout",
        &LogoutPayload {
            provider: None,
            voip_provider: None,
        },
    )
    .await?;
    Ok(())
}

#[derive(Serialize)]
struct LogoutSessionsPayload<'a> {
    session_id_hashes: &'a [String],
}

pub async fn revoke_sessions(
    http: &HttpClient,
    session_id_hashes: &[String],
) -> Result<(), RestError> {
    if session_id_hashes.is_empty() {
        return Ok(());
    }
    http.request_with_json(
        "POST /auth/sessions/logout",
        wreq::Method::POST,
        "/auth/sessions/logout",
        &LogoutSessionsPayload { session_id_hashes },
    )
    .await?;
    Ok(())
}
