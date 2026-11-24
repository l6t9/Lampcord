use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::captcha::is_captcha_response_body;

pub const NETWORK_ERROR_MESSAGE: &str =
    "Could not reach Discord. Check your connection and try again.";
pub const PASSWORD_UPDATE_MESSAGE: &str =
    "Your account requires a password change at discord.com before you can continue.";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LoginFailure {
    Suspended,
    RateLimited,
    ServerError,
    CaptchaRejected,
    VerificationRequired,
    AccountDisabled,
    AccountScheduledForDeletion,
    InvalidCredentials,
    PasswordUpdateRequired,
    CaptchaCancelled,
    Network,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LoginRequest {
    pub login: String,
    pub password: String,
    #[serde(skip_serializing_if = "is_false")]
    pub undelete: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub login_source: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gift_code_sku_id: Option<String>,
}

impl LoginRequest {
    pub fn new(login: impl Into<String>, password: impl Into<String>) -> Self {
        Self {
            login: login.into(),
            password: password.into(),
            undelete: false,
            login_source: None,
            gift_code_sku_id: None,
        }
    }

    pub fn to_value(&self) -> Value {
        serde_json::to_value(self).unwrap_or(Value::Null)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MfaLoginRequest {
    pub code: String,
    pub ticket: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub login_source: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gift_code_sku_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub login_instance_id: Option<String>,
}

impl MfaLoginRequest {
    pub fn new(code: impl Into<String>, ticket: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            ticket: ticket.into(),
            login_source: None,
            gift_code_sku_id: None,
            login_instance_id: None,
        }
    }

    pub fn with_login_instance_id(mut self, id: Option<&str>) -> Self {
        self.login_instance_id = id.filter(|id| !id.is_empty()).map(str::to_owned);
        self
    }

    pub fn to_value(&self) -> Value {
        serde_json::to_value(self).unwrap_or(Value::Null)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct LoginResponse {
    pub user_id: Option<String>,
    pub token: Option<String>,
    pub ticket: Option<String>,
    pub login_instance_id: Option<String>,
    pub mfa: Option<Value>,
    #[serde(default)]
    pub totp: Option<bool>,
    #[serde(default)]
    pub sms: Option<bool>,
    #[serde(default)]
    pub backup: Option<bool>,
    #[serde(default)]
    pub webauthn: Option<String>,
    pub required_actions: Vec<Value>,
    pub code: Option<i32>,
    pub message: Option<String>,
    pub failure: Option<LoginFailure>,
    pub captcha_cancelled: bool,
    #[serde(default, skip_serializing)]
    pub body: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DiscordError {
    pub code: Option<i32>,
    pub message: Option<String>,
}

pub fn decode_login(body: &str) -> Result<LoginResponse, serde_json::Error> {
    serde_json::from_str(body)
}

pub fn discord_error(body: &str) -> Option<DiscordError> {
    let value: Value = serde_json::from_str(body).ok()?;
    let object = value.as_object()?;
    Some(DiscordError {
        code: object
            .get("code")
            .and_then(Value::as_i64)
            .map(|code| code as i32),
        message: object
            .get("message")
            .and_then(Value::as_str)
            .map(str::to_owned),
    })
}

pub fn login_failure(body: &str, status: u16) -> LoginFailure {
    if status == 403 && body.contains("suspended_user_token") {
        return LoginFailure::Suspended;
    }
    if status == 429 {
        return LoginFailure::RateLimited;
    }
    if (500..=599).contains(&status) {
        return LoginFailure::ServerError;
    }
    if is_captcha_response_body(body) {
        return LoginFailure::CaptchaRejected;
    }

    let code = discord_error(body)
        .and_then(|error| error.code)
        .unwrap_or_default();
    match code {
        70007 => LoginFailure::VerificationRequired,
        20013 => LoginFailure::AccountDisabled,
        20011 => LoginFailure::AccountScheduledForDeletion,
        _ => {
            let lower = body.to_ascii_lowercase();
            if status == 401 || code == 50035 || (status == 400 && lower.contains("password")) {
                LoginFailure::InvalidCredentials
            } else {
                LoginFailure::Unknown
            }
        }
    }
}

pub fn mfa_failure(body: &str, status: u16) -> LoginFailure {
    let code = discord_error(body)
        .and_then(|error| error.code)
        .unwrap_or_default();
    if status == 401 || code == 50035 {
        return LoginFailure::InvalidCredentials;
    }
    if is_captcha_response_body(body) {
        return LoginFailure::CaptchaRejected;
    }
    if status == 429 {
        return LoginFailure::RateLimited;
    }
    if (500..=599).contains(&status) {
        return LoginFailure::ServerError;
    }
    LoginFailure::Unknown
}

pub fn failure_message(failure: LoginFailure) -> &'static str {
    match failure {
        LoginFailure::Suspended => "This account is suspended.",
        LoginFailure::RateLimited => "Too many attempts. Please try again later.",
        LoginFailure::ServerError => "Discord is unavailable. Please try again later.",
        LoginFailure::CaptchaRejected => "The captcha was rejected. Please try again.",
        LoginFailure::VerificationRequired => {
            "Phone verification is required before you can log in."
        }
        LoginFailure::AccountDisabled => "This account has been disabled.",
        LoginFailure::AccountScheduledForDeletion => "This account is scheduled for deletion.",
        LoginFailure::InvalidCredentials => "Invalid credentials or authentication code.",
        LoginFailure::PasswordUpdateRequired => PASSWORD_UPDATE_MESSAGE,
        LoginFailure::CaptchaCancelled => "Captcha was cancelled.",
        LoginFailure::Network => NETWORK_ERROR_MESSAGE,
        LoginFailure::Unknown => "Login failed. Please try again.",
    }
}

pub fn network_failure() -> LoginResponse {
    LoginResponse {
        failure: Some(LoginFailure::Network),
        message: Some(NETWORK_ERROR_MESSAGE.to_owned()),
        ..LoginResponse::default()
    }
}

pub fn captcha_cancelled() -> LoginResponse {
    LoginResponse {
        failure: Some(LoginFailure::CaptchaCancelled),
        message: Some(failure_message(LoginFailure::CaptchaCancelled).to_owned()),
        captcha_cancelled: true,
        ..LoginResponse::default()
    }
}

pub fn complete_login(response: LoginResponse) -> (LoginResponse, Option<String>) {
    let requires_password_update = response.token.as_deref().is_some_and(|_| {
        response.required_actions.iter().any(|action| match action {
            Value::String(s) => s.contains("update_password"),
            other => other.to_string().contains("update_password"),
        })
    });

    if !requires_password_update {
        return (response, None);
    }

    let token_to_logout = response.token.clone();
    let completed = LoginResponse {
        user_id: response.user_id,
        token: None,
        ticket: None,
        login_instance_id: None,
        mfa: None,
        totp: None,
        sms: None,
        backup: None,
        webauthn: None,
        required_actions: Vec::new(),
        code: None,
        message: Some(PASSWORD_UPDATE_MESSAGE.to_owned()),
        failure: Some(LoginFailure::PasswordUpdateRequired),
        captcha_cancelled: false,
        body: response.body,
    };
    (completed, token_to_logout)
}

fn is_false(value: &bool) -> bool {
    !*value
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_login_failures_in_priority_order() {
        assert_eq!(
            login_failure(r#"{"message":"suspended_user_token"}"#, 403),
            LoginFailure::Suspended
        );
        assert_eq!(login_failure("{}", 429), LoginFailure::RateLimited);
        assert_eq!(login_failure("{}", 503), LoginFailure::ServerError);
        assert_eq!(
            login_failure(r#"{"captcha_key":["x"],"captcha_sitekey":"site"}"#, 400),
            LoginFailure::CaptchaRejected
        );
        assert_eq!(
            login_failure(r#"{"code":70007}"#, 400),
            LoginFailure::VerificationRequired
        );
        assert_eq!(
            login_failure(r#"{"code":50035}"#, 400),
            LoginFailure::InvalidCredentials
        );
        assert_eq!(login_failure("{}", 400), LoginFailure::Unknown);
    }

    #[test]
    fn mfa_failures_treat_401_as_invalid_code() {
        assert_eq!(
            mfa_failure(r#"{"code":50035}"#, 400),
            LoginFailure::InvalidCredentials
        );
        assert_eq!(mfa_failure("{}", 401), LoginFailure::InvalidCredentials);
        assert_eq!(
            mfa_failure(r#"{"captcha_key":["x"],"captcha_sitekey":"site"}"#, 400),
            LoginFailure::CaptchaRejected
        );
    }

    #[test]
    fn password_update_logs_out_the_issued_token() {
        let response = decode_login(
            r#"{"token":"new-token","user_id":"123","required_actions":["update_password"]}"#,
        )
        .expect("login response");
        let (response, logout_token) = complete_login(response);
        assert_eq!(logout_token.as_deref(), Some("new-token"));
        assert_eq!(response.failure, Some(LoginFailure::PasswordUpdateRequired));
        assert_eq!(response.token, None);
        assert_eq!(response.user_id.as_deref(), Some("123"));
    }

    #[test]
    fn normal_login_is_untouched() {
        let response =
            decode_login(r#"{"token":"new-token","user_id":"123"}"#).expect("login response");
        let (completed, logout_token) = complete_login(response);
        assert_eq!(logout_token, None);
        assert_eq!(completed.token.as_deref(), Some("new-token"));
        assert_eq!(completed.failure, None);
    }

    #[test]
    fn mfa_request_keeps_login_instance_id() {
        let request = MfaLoginRequest::new("123456", "ticket")
            .with_login_instance_id(Some("instance-1"));
        assert_eq!(
            request.to_value(),
            serde_json::json!({
                "code": "123456",
                "ticket": "ticket",
                "login_instance_id": "instance-1"
            })
        );

        let without = MfaLoginRequest::new("123456", "ticket").with_login_instance_id(Some(""));
        assert_eq!(without.login_instance_id, None);
    }
}
