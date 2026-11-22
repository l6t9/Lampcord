use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    auth::{self, LoginRequest, LoginResponse, MfaLoginRequest},
    rest::{Body, HttpMethod, RestConfig},
    rest_client::RestClient,
    traits::{CaptchaSolver, RestTransport},
};

use super::ApiError;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct FingerprintResponse {
    #[serde(default)]
    pub fingerprint: Option<String>,
}

pub async fn get_fingerprint<T, S>(
    client: &RestClient<T, S>,
    config: &RestConfig,
) -> Result<Option<String>, ApiError>
where
    T: RestTransport,
    S: CaptchaSolver,
{
    let req = config.request(
        HttpMethod::Get,
        "/experiments",
        config.login_headers(None),
        Body::None,
    );
    let response = client.send(req).await?;
    if !response.status.is_success_like() {
        return Err(ApiError::Http {
            status: response.status,
            body: response.body,
        });
    }
    let parsed: FingerprintResponse = serde_json::from_str(&response.body)?;
    Ok(parsed.fingerprint)
}

pub async fn login<T, S>(
    client: &RestClient<T, S>,
    config: &RestConfig,
    request: &LoginRequest,
    fingerprint: Option<&str>,
) -> Result<LoginResponse, ApiError>
where
    T: RestTransport,
    S: CaptchaSolver,
{
    let body = Body::Json(request.to_value().to_string());
    let req = config.request(
        HttpMethod::Post,
        "/auth/login",
        config.login_headers(fingerprint),
        body,
    );
    let response = client.send(req).await?;
    if (200..300).contains(&response.status) {
        let mut login = auth::decode_login(&response.body)?;
        let (completed, logout_token) = auth::complete_login(login);
        login = completed;
        // If password update required, best-effort logout with issued token.
        if let Some(token) = logout_token {
            let _ = logout_remote(client, config, &token).await;
        }
        return Ok(login);
    }

    let failure = auth::login_failure(&response.body, response.status);
    let err = auth::discord_error(&response.body);
    Ok(LoginResponse {
        code: err.as_ref().and_then(|e| e.code),
        message: Some(
            err.and_then(|e| e.message)
                .unwrap_or_else(|| auth::failure_message(failure).to_owned()),
        ),
        failure: Some(failure),
        body: Some(response.body),
        ..LoginResponse::default()
    })
}

pub async fn login_mfa<T, S>(
    client: &RestClient<T, S>,
    config: &RestConfig,
    request: &MfaLoginRequest,
    fingerprint: Option<&str>,
    mfa_type: &str,
) -> Result<LoginResponse, ApiError>
where
    T: RestTransport,
    S: CaptchaSolver,
{
    let body = Body::Json(request.to_value().to_string());
    let req = config.request(
        HttpMethod::Post,
        &format!("/auth/mfa/{mfa_type}"),
        config.login_headers(fingerprint),
        body,
    );
    let response = client.send(req).await?;
    if (200..300).contains(&response.status) {
        let login = auth::decode_login(&response.body)?;
        let (completed, logout_token) = auth::complete_login(login);
        if let Some(token) = logout_token {
            let _ = logout_remote(client, config, &token).await;
        }
        return Ok(completed);
    }

    let failure = auth::mfa_failure(&response.body, response.status);
    let err = auth::discord_error(&response.body);
    Ok(LoginResponse {
        code: err.as_ref().and_then(|e| e.code),
        message: Some(
            err.and_then(|e| e.message)
                .unwrap_or_else(|| auth::failure_message(failure).to_owned()),
        ),
        failure: Some(failure),
        body: Some(response.body),
        ..LoginResponse::default()
    })
}

pub async fn logout_remote<T, S>(
    client: &RestClient<T, S>,
    config: &RestConfig,
    token: &str,
) -> Result<(), ApiError>
where
    T: RestTransport,
    S: CaptchaSolver,
{
    let mut headers = config.login_headers(None);
    headers.push(("Authorization".to_owned(), token.to_owned()));
    let req = config.request(
        HttpMethod::Post,
        "/auth/logout",
        headers,
        Body::Json("{}".to_owned()),
    );
    let response = client.send(req).await?;
    if (200..300).contains(&response.status) {
        Ok(())
    } else {
        Err(ApiError::Http {
            status: response.status,
            body: response.body,
        })
    }
}

pub async fn get_current_user<T, S, U>(
    client: &RestClient<T, S>,
    config: &RestConfig,
    token: &str,
) -> Result<Option<U>, ApiError>
where
    T: RestTransport,
    S: CaptchaSolver,
    U: serde::de::DeserializeOwned,
{
    let mut headers = config.login_headers(None);
    headers.push(("Authorization".to_owned(), token.to_owned()));
    let req = config.request(HttpMethod::Get, "/users/@me", headers, Body::None);
    let response = client.send(req).await?;
    if (200..300).contains(&response.status) {
        Ok(Some(serde_json::from_str(&response.body)?))
    } else {
        Ok(None)
    }
}

pub async fn get_gateway_url<T, S>(
    client: &RestClient<T, S>,
    config: &RestConfig,
) -> Result<Option<String>, ApiError>
where
    T: RestTransport,
    S: CaptchaSolver,
{
    let req = config.request(
        HttpMethod::Get,
        "/gateway",
        config.standard_headers(),
        Body::None,
    );
    let response = client.send(req).await?;
    if !(200..300).contains(&response.status) {
        return Ok(None);
    }
    let value: Value = serde_json::from_str(&response.body)?;
    Ok(value.get("url").and_then(Value::as_str).map(str::to_owned))
}

pub async fn send_mfa_sms<T, S>(
    client: &RestClient<T, S>,
    config: &RestConfig,
    ticket: &str,
    fingerprint: Option<&str>,
) -> Result<String, ApiError>
where
    T: RestTransport,
    S: CaptchaSolver,
{
    let body = serde_json::json!({ "ticket": ticket });
    let req = config.request(
        HttpMethod::Post,
        "/auth/mfa/sms/send",
        config.login_headers(fingerprint),
        Body::Json(body.to_string()),
    );
    let response = client.send(req).await?;
    if !(200..300).contains(&response.status) {
        return Err(ApiError::Http {
            status: response.status,
            body: response.body,
        });
    }
    let value: Value = serde_json::from_str(&response.body)?;
    value
        .get("phone")
        .and_then(Value::as_str)
        .map(str::to_owned)
        .ok_or_else(|| ApiError::Http {
            status: response.status,
            body: response.body,
        })
}

trait StatusExt {
    fn is_success_like(&self) -> bool;
}
impl StatusExt for u16 {
    fn is_success_like(&self) -> bool {
        (200..300).contains(self)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ForgotPasswordResult {
    Sent,
    PhoneVerificationRequired,
    Failed(String),
}

pub async fn forgot_password<T, S>(
    client: &RestClient<T, S>,
    config: &RestConfig,
    login: &str,
    fingerprint: Option<&str>,
) -> Result<ForgotPasswordResult, ApiError>
where
    T: RestTransport,
    S: CaptchaSolver,
{
    let body = serde_json::json!({ "login": login });
    let req = config.request(
        HttpMethod::Post,
        "/auth/forgot",
        config.login_headers(fingerprint),
        Body::Json(body.to_string()),
    );
    let response = client.send(req).await?;
    if (200..300).contains(&response.status) {
        return Ok(ForgotPasswordResult::Sent);
    }
    let err = auth::discord_error(&response.body);
    if err.as_ref().and_then(|e| e.code) == Some(70007) {
        return Ok(ForgotPasswordResult::PhoneVerificationRequired);
    }
    let msg = err
        .and_then(|e| e.message)
        .filter(|m| !m.is_empty())
        .unwrap_or_else(|| {
            format!(
                "Could not start a password reset (HTTP {}).",
                response.status
            )
        });
    Ok(ForgotPasswordResult::Failed(msg))
}

pub async fn verify_phone_for_login<T, S>(
    client: &RestClient<T, S>,
    config: &RestConfig,
    phone: &str,
    code: &str,
    fingerprint: Option<&str>,
) -> Result<String, ApiError>
where
    T: RestTransport,
    S: CaptchaSolver,
{
    let body = serde_json::json!({ "phone": phone, "code": code });
    let req = config.request(
        HttpMethod::Post,
        "/phone-verifications/verify",
        config.login_headers(fingerprint),
        Body::Json(body.to_string()),
    );
    let response = client.send(req).await?;
    if !(200..300).contains(&response.status) {
        return Err(ApiError::Http {
            status: response.status,
            body: response.body,
        });
    }
    let value: Value = serde_json::from_str(&response.body)?;
    value
        .get("token")
        .and_then(Value::as_str)
        .map(str::to_owned)
        .ok_or_else(|| ApiError::Http {
            status: response.status,
            body: response.body,
        })
}

pub async fn resend_phone_code<T, S>(
    client: &RestClient<T, S>,
    config: &RestConfig,
    phone: &str,
    fingerprint: Option<&str>,
) -> Result<(), ApiError>
where
    T: RestTransport,
    S: CaptchaSolver,
{
    let body = serde_json::json!({ "phone": phone });
    let req = config.request(
        HttpMethod::Post,
        "/phone-verifications/resend",
        config.login_headers(fingerprint),
        Body::Json(body.to_string()),
    );
    let response = client.send(req).await?;
    if (200..300).contains(&response.status) {
        Ok(())
    } else {
        Err(ApiError::Http {
            status: response.status,
            body: response.body,
        })
    }
}

pub async fn authorize_ip<T, S>(
    client: &RestClient<T, S>,
    config: &RestConfig,
    token: &str,
    fingerprint: Option<&str>,
) -> Result<(), ApiError>
where
    T: RestTransport,
    S: CaptchaSolver,
{
    let body = serde_json::json!({ "token": token });
    let req = config.request(
        HttpMethod::Post,
        "/auth/authorize-ip",
        config.login_headers(fingerprint),
        Body::Json(body.to_string()),
    );
    let response = client.send(req).await?;
    if (200..300).contains(&response.status) {
        Ok(())
    } else {
        Err(ApiError::Http {
            status: response.status,
            body: response.body,
        })
    }
}

pub async fn exchange_remote_auth_ticket<T, S>(
    client: &RestClient<T, S>,
    config: &RestConfig,
    ticket: &str,
) -> Result<Option<String>, ApiError>
where
    T: RestTransport,
    S: CaptchaSolver,
{
    let body = serde_json::json!({ "ticket": ticket });
    let req = config.request(
        HttpMethod::Post,
        "/users/@me/remote-auth/login",
        config.login_headers(None),
        Body::Json(body.to_string()),
    );
    let response = client.send(req).await?;
    if !(200..300).contains(&response.status) {
        return Ok(None);
    }
    let value: Value = serde_json::from_str(&response.body)?;
    Ok(value
        .get("encrypted_token")
        .and_then(Value::as_str)
        .map(str::to_owned))
}
