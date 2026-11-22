use serde_json::Value;

pub const RECAPTCHA_ENTERPRISE_ACTION: &str = "login";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaptchaService {
    Hcaptcha,
    Recaptcha,
    RecaptchaEnterprise,
}

impl CaptchaService {
    pub fn from_wire_name(name: Option<&str>) -> Option<Self> {
        match name {
            None | Some("hcaptcha") => Some(Self::Hcaptcha),
            Some("recaptcha") => Some(Self::Recaptcha),
            Some("recaptcha_enterprise") => Some(Self::RecaptchaEnterprise),
            _ => None,
        }
    }

    pub fn wire_name(self) -> &'static str {
        match self {
            Self::Hcaptcha => "hcaptcha",
            Self::Recaptcha => "recaptcha",
            Self::RecaptchaEnterprise => "recaptcha_enterprise",
        }
    }

    fn fixed_site_key(self) -> Option<&'static str> {
        match self {
            Self::Hcaptcha => None,
            Self::Recaptcha => Some("6Lef5iQTAAAAAKeIvIY-DeexoO3gj7ryl9rLMEnn"),
            Self::RecaptchaEnterprise => Some("6LeYqFcqAAAAAD6iZesmNgVulsO4PkpBdr6NVG6M"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaptchaChallenge {
    pub sitekey: String,
    pub service: CaptchaService,
    pub rqdata: Option<String>,
    pub rqtoken: Option<String>,
    pub session_id: Option<String>,
    pub should_serve_invisible: bool,
}

impl CaptchaChallenge {
    pub fn is_invisible(&self) -> bool {
        self.should_serve_invisible || self.service == CaptchaService::RecaptchaEnterprise
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaptchaHeaders {
    pub key: String,
    pub rqtoken: Option<String>,
    pub session_id: Option<String>,
}

pub fn is_captcha_response_body(body: &str) -> bool {
    parse_object(body)
        .map(|object| object.contains_key("captcha_key"))
        .unwrap_or(false)
}

pub fn parse_captcha_challenge(body: &str) -> Option<CaptchaChallenge> {
    let object = parse_object(body)?;
    if !object.contains_key("captcha_key") {
        return None;
    }

    let service = CaptchaService::from_wire_name(string_field(&object, "captcha_service"))?;
    let sitekey = string_field(&object, "captcha_sitekey")
        .map(str::to_owned)
        .or_else(|| service.fixed_site_key().map(str::to_owned))?;

    Some(CaptchaChallenge {
        sitekey,
        service,
        rqdata: string_field(&object, "captcha_rqdata").map(str::to_owned),
        rqtoken: string_field(&object, "captcha_rqtoken").map(str::to_owned),
        session_id: string_field(&object, "captcha_session_id").map(str::to_owned),
        should_serve_invisible: object
            .get("should_serve_invisible")
            .and_then(Value::as_bool)
            .unwrap_or(false),
    })
}

pub fn challenge_headers(challenge: &CaptchaChallenge, token: impl Into<String>) -> CaptchaHeaders {
    CaptchaHeaders {
        key: token.into(),
        rqtoken: challenge.rqtoken.clone(),
        session_id: challenge.session_id.clone(),
    }
}

fn parse_object(body: &str) -> Option<serde_json::Map<String, Value>> {
    serde_json::from_str::<Value>(body)
        .ok()?
        .as_object()
        .cloned()
}

fn string_field<'a>(object: &'a serde_json::Map<String, Value>, key: &str) -> Option<&'a str> {
    object.get(key).and_then(Value::as_str)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_hcaptcha_challenge_with_rqdata() {
        let body = r#"{
            "captcha_key": ["invalid-input-response"],
            "captcha_sitekey": "site-1",
            "captcha_service": "hcaptcha",
            "captcha_rqdata": "rq-data",
            "captcha_rqtoken": "rq-token",
            "captcha_session_id": "session-1",
            "should_serve_invisible": true
        }"#;

        let challenge = parse_captcha_challenge(body).expect("challenge");
        assert_eq!(challenge.service, CaptchaService::Hcaptcha);
        assert_eq!(challenge.sitekey, "site-1");
        assert_eq!(challenge.rqdata.as_deref(), Some("rq-data"));
        assert!(challenge.is_invisible());

        let headers = challenge_headers(&challenge, "token-1");
        assert_eq!(headers.key, "token-1");
        assert_eq!(headers.rqtoken.as_deref(), Some("rq-token"));
        assert_eq!(headers.session_id.as_deref(), Some("session-1"));
    }

    #[test]
    fn defaults_missing_service_to_hcaptcha() {
        let body = r#"{"captcha_key":["x"],"captcha_sitekey":"site-1"}"#;
        let challenge = parse_captcha_challenge(body).expect("challenge");
        assert_eq!(challenge.service, CaptchaService::Hcaptcha);
        assert!(!challenge.is_invisible());
    }

    #[test]
    fn enterprise_uses_fixed_site_key_and_is_invisible() {
        let body = r#"{"captcha_key":["x"],"captcha_service":"recaptcha_enterprise"}"#;
        let challenge = parse_captcha_challenge(body).expect("challenge");
        assert_eq!(challenge.service, CaptchaService::RecaptchaEnterprise);
        assert_eq!(
            challenge.sitekey,
            "6LeYqFcqAAAAAD6iZesmNgVulsO4PkpBdr6NVG6M"
        );
        assert!(challenge.is_invisible());
    }

    #[test]
    fn ignores_non_captcha_bodies() {
        let body = r#"{"message":"Invalid Form Body","code":50035}"#;
        assert!(!is_captcha_response_body(body));
        assert!(parse_captcha_challenge(body).is_none());
    }
}
