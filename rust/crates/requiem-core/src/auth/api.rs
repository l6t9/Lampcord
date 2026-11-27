use crate::rest;
use requiem_ui::form;
use std::collections::HashMap;
use std::fmt as std_fmt;

use serde::Serialize;
use serde_json::Value;
use thiserror::Error;

use super::captcha::{CaptchaChallenge, CaptchaSolution};
use crate::config::BackendConfig;
use crate::properties::SuperProperties;

#[derive(Debug, Clone, Default)]
pub struct ApiError {
    pub status: u16,
    pub fields: HashMap<String, String>,
    pub message: Option<String>,

    pub captcha: Option<CaptchaChallenge>,
}

impl From<ApiError> for form::FormErrors {
    fn from(error: ApiError) -> Self {
        let general = error
            .fields
            .is_empty()
            .then(|| error.message.clone().unwrap_or_else(|| error.to_string()));
        Self {
            fields: error.fields,
            general,
        }
    }
}

impl std_fmt::Display for ApiError {
    fn fmt(&self, f: &mut std_fmt::Formatter<'_>) -> std_fmt::Result {
        match (&self.message, self.fields.iter().next()) {
            (_, Some((field, msg))) => write!(f, "{field}: {msg}"),
            (Some(message), None) => f.write_str(message),
            (None, None) => write!(f, "request failed with status {}", self.status),
        }
    }
}

impl ApiError {
    pub fn parse(status: u16, body: &str) -> Self {
        let mut error = ApiError {
            status,
            ..Default::default()
        };
        let Ok(json) = serde_json::from_str::<Value>(body) else {
            error.message = (!body.trim().is_empty()).then(|| body.trim().to_string());
            return error;
        };

        if json.get("captcha_key").is_some() {
            match CaptchaChallenge::from_body(&json).filter(|challenge| challenge.sitekey.is_some())
            {
                Some(challenge) => {
                    error.message = Some(requiem_i18n::t!("auth-captcha-failed"));
                    error.captcha = Some(challenge);
                }
                None => error.message = Some(requiem_i18n::t!("auth-captcha-unsupported")),
            }
            return error;
        }
        error.message = json
            .get("message")
            .and_then(Value::as_str)
            .map(str::to_string);
        if let Some(Value::Object(fields)) = json.get("errors") {
            for (field, value) in fields {
                if let Some(message) = first_error_message(value) {
                    error.fields.insert(field.clone(), message);
                }
            }
        }
        error
    }

    pub fn field(&self, field: &str) -> Option<String> {
        self.fields.get(field).cloned()
    }
}

fn first_error_message(value: &Value) -> Option<String> {
    if let Some(message) = value
        .get("_errors")
        .and_then(|e| e.get(0))
        .and_then(|e| e.get("message"))
        .and_then(Value::as_str)
    {
        return Some(message.to_string());
    }
    value.as_object()?.values().find_map(first_error_message)
}

#[derive(Debug, Error)]
pub enum AuthRequestError {
    #[error("request failed: {0}")]
    Request(#[from] wreq::Error),
    #[error("{0}")]
    Api(ApiError),
}

pub(crate) async fn post_json(
    client: &wreq::Client,
    config: &BackendConfig,
    super_properties: &SuperProperties,
    path: &str,
    body: &impl Serialize,
    captcha: Option<&CaptchaSolution>,
) -> Result<Value, AuthRequestError> {
    let url = format!("{}{}", config.rest_base_url.trim_end_matches('/'), path);
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
        .json(body)
        .send()
        .await?;
    let status = response.status();
    let text = response.text().await?;
    if !status.is_success() {
        return Err(AuthRequestError::Api(ApiError::parse(
            status.as_u16(),
            &text,
        )));
    }
    Ok(serde_json::from_str(&text).unwrap_or(Value::Null))
}

fn token_from(value: &Value, status_hint: u16) -> Result<String, AuthRequestError> {
    if value.get("token").and_then(Value::as_str).is_some() {
        super::fingerprint::forget();
    }
    value
        .get("token")
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| {
            AuthRequestError::Api(ApiError {
                status: status_hint,
                message: Some(requiem_i18n::t!("auth-no-token")),
                ..Default::default()
            })
        })
}

pub struct RegisterForm<'a> {
    pub email: &'a str,
    pub global_name: Option<&'a str>,
    pub username: &'a str,
    pub password: &'a str,

    pub date_of_birth: &'a str,
    pub consent: bool,
}

#[derive(Serialize)]
struct RegisterRequest<'a> {
    email: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    global_name: Option<&'a str>,
    username: &'a str,
    password: &'a str,
    date_of_birth: &'a str,
    consent: bool,
    invite: Option<()>,
    gift_code_sku_id: Option<()>,
}

pub async fn register(
    client: &wreq::Client,
    config: &BackendConfig,
    super_properties: &SuperProperties,
    form: RegisterForm<'_>,
    captcha: Option<&CaptchaSolution>,
) -> Result<String, AuthRequestError> {
    let body = RegisterRequest {
        email: form.email,
        global_name: form.global_name.filter(|n| !n.trim().is_empty()),
        username: form.username,
        password: form.password,
        date_of_birth: form.date_of_birth,
        consent: form.consent,
        invite: None,
        gift_code_sku_id: None,
    };
    let value = post_json(
        client,
        config,
        super_properties,
        "/auth/register",
        &body,
        captcha,
    )
    .await?;
    token_from(&value, 201)
}

#[derive(Serialize)]
struct MfaRequest<'a> {
    ticket: &'a str,
    code: &'a str,
}

pub async fn mfa_login(
    client: &wreq::Client,
    config: &BackendConfig,
    super_properties: &SuperProperties,
    ticket: &str,
    code: &str,
) -> Result<String, AuthRequestError> {
    let code = code.trim().replace([' ', '-'], "");
    let path = if code.len() == 6 && code.chars().all(|c| c.is_ascii_digit()) {
        "/auth/mfa/totp"
    } else {
        "/auth/mfa/backup"
    };
    let value = post_json(
        client,
        config,
        super_properties,
        path,
        &MfaRequest {
            ticket,
            code: &code,
        },
        None,
    )
    .await?;
    token_from(&value, 200)
}

#[derive(Serialize)]
struct ForgotRequest<'a> {
    login: &'a str,
}

pub async fn forgot_password(
    client: &wreq::Client,
    config: &BackendConfig,
    super_properties: &SuperProperties,
    login: &str,
    captcha: Option<&CaptchaSolution>,
) -> Result<(), AuthRequestError> {
    post_json(
        client,
        config,
        super_properties,
        "/auth/forgot",
        &ForgotRequest { login },
        captcha,
    )
    .await
    .map(|_| ())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_field_errors() {
        let body = r#"{"message":"Invalid Form Body","code":50035,"errors":{"login":{"_errors":[{"code":"INVALID_LOGIN","message":"Login or password is invalid."}]}}}"#;
        let error = ApiError::parse(400, body);
        assert_eq!(
            error.field("login").as_deref(),
            Some("Login or password is invalid.")
        );
        assert_eq!(error.message.as_deref(), Some("Invalid Form Body"));
    }

    #[test]
    fn parses_nested_field_errors() {
        let body =
            r#"{"errors":{"date_of_birth":{"year":{"_errors":[{"message":"Too young."}]}}}}"#;
        assert_eq!(
            ApiError::parse(400, body).field("date_of_birth").as_deref(),
            Some("Too young.")
        );
    }

    #[test]
    fn a_renderable_captcha_is_kept_as_a_challenge() {
        let error = ApiError::parse(
            400,
            r#"{"captcha_key":["captcha-required"],"captcha_service":"hcaptcha","captcha_sitekey":"x"}"#,
        );
        assert_eq!(error.captcha.unwrap().sitekey.as_deref(), Some("x"));
    }

    #[test]
    fn a_captcha_that_cannot_be_rendered_becomes_a_top_level_message() {
        let error = ApiError::parse(
            400,
            r#"{"captcha_key":["captcha-required"],"captcha_service":"mystery","captcha_sitekey":"x"}"#,
        );
        assert!(error.captcha.is_none());
        assert!(error.message.unwrap().contains("captcha"));
        let error = ApiError::parse(400, r#"{"captcha_key":["captcha-required"]}"#);
        assert!(error.captcha.is_none(), "no sitekey, nothing to render");
        assert!(error.fields.is_empty());
    }
}
