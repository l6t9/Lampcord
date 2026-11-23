use crate::captcha::{CaptchaChallenge, challenge_headers, parse_captcha_challenge};

pub const API_BASE: &str = "https://discord.com/api/v9";
pub const DESKTOP_USER_AGENT: &str = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) discord/0.0.398 Chrome/138.0.7204.251 Electron/37.6.0 Safari/537.36";
pub const ACCEPT_LANGUAGE: &str = "en-US,en;q=0.9";
pub const DISCORD_LOCALE: &str = "en-US";
pub const DISCORD_ORIGIN: &str = "https://discord.com";
pub const LOGIN_REFERER: &str = "https://discord.com/login";
pub const CAPTCHA_KEY_HEADER: &str = "X-Captcha-Key";
pub const CAPTCHA_RQTOKEN_HEADER: &str = "X-Captcha-Rqtoken";
pub const CAPTCHA_SESSION_HEADER: &str = "X-Captcha-Session-Id";
pub const MAX_CAPTCHA_ATTEMPTS: usize = 3;

#[derive(Debug, Clone)]
pub struct RestConfig {
    token: Option<String>,
    api_base: String,
    user_agent: String,
    super_properties: String,
    installation_id: String,
    timezone: String,
    fingerprint: Option<String>,
}

impl RestConfig {
    pub fn new(
        token: Option<String>,
        super_properties: impl Into<String>,
        installation_id: impl Into<String>,
        timezone: impl Into<String>,
    ) -> Self {
        Self {
            token,
            api_base: API_BASE.to_owned(),
            user_agent: DESKTOP_USER_AGENT.to_owned(),
            super_properties: super_properties.into(),
            installation_id: installation_id.into(),
            timezone: timezone.into(),
            fingerprint: None,
        }
    }

    pub fn with_api_base(mut self, api_base: impl Into<String>) -> Self {
        self.api_base = api_base.into();
        self
    }

    pub fn with_user_agent(mut self, user_agent: impl Into<String>) -> Self {
        self.user_agent = user_agent.into();
        self
    }

    pub fn with_fingerprint(mut self, fingerprint: Option<String>) -> Self {
        self.fingerprint = fingerprint;
        self
    }

    pub fn set_token(&mut self, token: Option<String>) {
        self.token = token;
    }

    pub fn token(&self) -> Option<&str> {
        self.token.as_deref()
    }

    pub fn endpoint(&self, path: &str) -> String {
        format!(
            "{}/{}",
            self.api_base.trim_end_matches('/'),
            path.trim_start_matches('/')
        )
    }

    pub fn standard_headers(&self) -> Vec<(String, String)> {
        let mut headers = vec![
            ("User-Agent".to_owned(), self.user_agent.clone()),
            ("Accept-Language".to_owned(), ACCEPT_LANGUAGE.to_owned()),
            (
                "X-Super-Properties".to_owned(),
                self.super_properties.clone(),
            ),
            ("X-Discord-Locale".to_owned(), DISCORD_LOCALE.to_owned()),
        ];
        if let Some(token) = self.token.as_deref() {
            set_header(&mut headers, "Authorization", token);
        }
        headers
    }

    pub fn login_headers(&self, fingerprint: Option<&str>) -> Vec<(String, String)> {
        let mut headers = vec![
            ("User-Agent".to_owned(), self.user_agent.clone()),
            ("Accept-Language".to_owned(), ACCEPT_LANGUAGE.to_owned()),
            (
                "X-Super-Properties".to_owned(),
                self.super_properties.clone(),
            ),
            ("X-Discord-Locale".to_owned(), DISCORD_LOCALE.to_owned()),
            ("X-Discord-Timezone".to_owned(), self.timezone.clone()),
            ("X-Installation-Id".to_owned(), self.installation_id.clone()),
            ("Origin".to_owned(), DISCORD_ORIGIN.to_owned()),
            ("Referer".to_owned(), LOGIN_REFERER.to_owned()),
        ];
        let fingerprint = fingerprint.or(self.fingerprint.as_deref());
        if let Some(fingerprint) = fingerprint {
            set_header(&mut headers, "X-Fingerprint", fingerprint);
        }
        headers
    }

    pub fn request(
        &self,
        method: HttpMethod,
        path: &str,
        headers: Vec<(String, String)>,
        body: Body,
    ) -> RestRequest {
        RestRequest {
            method,
            url: self.endpoint(path),
            headers,
            body,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Patch,
    Delete,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Body {
    None,
    Json(String),
    Replayable(String),
    Streaming,
    /// Raw bytes with their content type, e.g. a file PUT to an upload URL.
    Bytes {
        content_type: String,
        data: Vec<u8>,
    },
}

impl Body {
    pub fn json(value: &serde_json::Value) -> Self {
        Self::Json(value.to_string())
    }

    pub fn is_replayable(&self) -> bool {
        !matches!(self, Self::Streaming)
    }

    pub fn as_text(&self) -> Option<&str> {
        match self {
            Self::Json(value) | Self::Replayable(value) => Some(value),
            Self::None | Self::Streaming | Self::Bytes { .. } => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RestRequest {
    pub method: HttpMethod,
    pub url: String,
    pub headers: Vec<(String, String)>,
    pub body: Body,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RestResponse {
    pub status: u16,
    pub body: String,
}

#[derive(Debug, Default)]
pub struct CaptchaRetry {
    attempts: usize,
    max_attempts: usize,
}

impl CaptchaRetry {
    pub fn new() -> Self {
        Self {
            attempts: 0,
            max_attempts: MAX_CAPTCHA_ATTEMPTS,
        }
    }

    pub fn with_max_attempts(max_attempts: usize) -> Self {
        Self {
            attempts: 0,
            max_attempts,
        }
    }

    pub fn attempts(&self) -> usize {
        self.attempts
    }

    pub fn reset(&mut self) {
        self.attempts = 0;
    }

    pub fn next_challenge(
        &mut self,
        response: &RestResponse,
        request: &RestRequest,
    ) -> Option<CaptchaChallenge> {
        if self.attempts >= self.max_attempts || response.status != 400 {
            return None;
        }
        if !request.body.is_replayable() {
            return None;
        }
        let challenge = parse_captcha_challenge(&response.body)?;
        self.attempts += 1;
        Some(challenge)
    }
}

pub fn apply_captcha_headers(
    request: &mut RestRequest,
    challenge: &CaptchaChallenge,
    token: impl Into<String>,
) {
    let headers = challenge_headers(challenge, token);
    set_header(&mut request.headers, CAPTCHA_KEY_HEADER, headers.key);
    match headers.rqtoken {
        Some(value) => set_header(&mut request.headers, CAPTCHA_RQTOKEN_HEADER, value),
        None => remove_header(&mut request.headers, CAPTCHA_RQTOKEN_HEADER),
    }
    match headers.session_id {
        Some(value) => set_header(&mut request.headers, CAPTCHA_SESSION_HEADER, value),
        None => remove_header(&mut request.headers, CAPTCHA_SESSION_HEADER),
    }
}

pub fn unauthorized_token(response: &RestResponse, request: &RestRequest) -> Option<String> {
    if response.status != 401 {
        return None;
    }
    let token = header(&request.headers, "Authorization")?.trim();
    (!token.is_empty()).then(|| token.to_owned())
}

pub fn is_current_token(rejected: &str, current: Option<&str>) -> bool {
    let rejected = rejected.trim();
    current
        .map(str::trim)
        .is_some_and(|current| !rejected.is_empty() && current == rejected)
}

fn header<'a>(headers: &'a [(String, String)], name: &str) -> Option<&'a str> {
    headers
        .iter()
        .find(|(key, _)| key.eq_ignore_ascii_case(name))
        .map(|(_, value)| value.as_str())
}

fn set_header(headers: &mut Vec<(String, String)>, name: &str, value: impl Into<String>) {
    remove_header(headers, name);
    headers.push((name.to_owned(), value.into()));
}

fn remove_header(headers: &mut Vec<(String, String)>, name: &str) {
    headers.retain(|(key, _)| !key.eq_ignore_ascii_case(name));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config() -> RestConfig {
        RestConfig::new(
            Some("token-1".to_owned()),
            "super-1",
            "install-1",
            "Europe/Berlin",
        )
    }

    #[test]
    fn standard_headers_include_authorization_only_with_token() {
        let headers = config().standard_headers();
        assert_eq!(header(&headers, "Authorization"), Some("token-1"));
        assert_eq!(header(&headers, "Accept-Language"), Some(ACCEPT_LANGUAGE));

        let mut config = config();
        config.set_token(None);
        assert_eq!(header(&config.standard_headers(), "Authorization"), None);
    }

    #[test]
    fn login_headers_never_include_authorization() {
        let config = config().with_fingerprint(Some("fingerprint-1".to_owned()));
        let headers = config.login_headers(None);
        assert_eq!(header(&headers, "Authorization"), None);
        assert_eq!(header(&headers, "X-Fingerprint"), Some("fingerprint-1"));
        assert_eq!(header(&headers, "Origin"), Some(DISCORD_ORIGIN));
        assert_eq!(header(&headers, "Referer"), Some(LOGIN_REFERER));
    }

    #[test]
    fn captcha_retry_stops_after_three_challenges() {
        let body = r#"{"captcha_key":["x"],"captcha_sitekey":"site-1"}"#;
        let response = RestResponse {
            status: 400,
            body: body.to_owned(),
        };
        let request = config().request(
            HttpMethod::Post,
            "/auth/login",
            Vec::new(),
            Body::Json("{}".to_owned()),
        );
        let mut retry = CaptchaRetry::new();

        for _ in 0..3 {
            assert!(retry.next_challenge(&response, &request).is_some());
        }
        assert!(retry.next_challenge(&response, &request).is_none());
        assert_eq!(retry.attempts(), 3);
    }

    #[test]
    fn captcha_retry_skips_streaming_bodies() {
        let response = RestResponse {
            status: 400,
            body: r#"{"captcha_key":["x"],"captcha_sitekey":"site-1"}"#.to_owned(),
        };
        let request = config().request(
            HttpMethod::Post,
            "/channels/1/messages",
            Vec::new(),
            Body::Streaming,
        );
        let mut retry = CaptchaRetry::new();
        assert!(retry.next_challenge(&response, &request).is_none());
    }

    #[test]
    fn only_authenticated_401s_report_token() {
        let response = RestResponse {
            status: 401,
            body: "{}".to_owned(),
        };
        let mut request = config().request(HttpMethod::Get, "/users/@me", Vec::new(), Body::None);
        assert_eq!(unauthorized_token(&response, &request), None);

        request
            .headers
            .push(("Authorization".to_owned(), "token-1".to_owned()));
        assert_eq!(
            unauthorized_token(&response, &request).as_deref(),
            Some("token-1")
        );
        assert!(is_current_token("token-1", Some("token-1")));
        assert!(!is_current_token("token-1", Some("token-2")));
    }

    #[test]
    fn captcha_headers_replace_previous_values() {
        let challenge = parse_captcha_challenge(
            r#"{"captcha_key":["x"],"captcha_sitekey":"site-1","captcha_session_id":"session-1"}"#,
        )
        .expect("challenge");
        let mut request = config().request(
            HttpMethod::Post,
            "/auth/login",
            vec![(CAPTCHA_KEY_HEADER.to_owned(), "old-token".to_owned())],
            Body::Json("{}".to_owned()),
        );
        apply_captcha_headers(&mut request, &challenge, "new-token");

        assert_eq!(
            header(&request.headers, CAPTCHA_KEY_HEADER),
            Some("new-token")
        );
        assert_eq!(
            header(&request.headers, CAPTCHA_SESSION_HEADER),
            Some("session-1")
        );
    }
}
