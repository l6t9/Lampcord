use serde::{Deserialize, Serialize};
use serde_json::Value;
use wreq::header::HeaderValue;
use wreq::header;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CaptchaService {
    Hcaptcha,
    Recaptcha,
    RecaptchaEnterprise,
    Turnstile,
}

impl CaptchaService {
    fn from_wire(service: Option<&str>) -> Option<Self> {
        match service {
            None => Some(Self::Recaptcha),
            Some("hcaptcha") => Some(Self::Hcaptcha),
            Some("recaptcha") => Some(Self::Recaptcha),
            Some("recaptcha_enterprise") => Some(Self::RecaptchaEnterprise),
            Some("turnstile") => Some(Self::Turnstile),
            Some(_) => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CaptchaChallenge {
    pub service: CaptchaService,

    pub sitekey: Option<String>,

    pub rqdata: Option<String>,

    pub rqtoken: Option<String>,

    pub session_id: Option<String>,

    pub invisible: bool,

    pub reasons: Vec<String>,
}

impl CaptchaChallenge {
    pub fn from_body(body: &Value) -> Option<Self> {
        let reasons = body
            .get("captcha_key")?
            .as_array()?
            .iter()
            .filter_map(Value::as_str)
            .map(str::to_string)
            .collect();
        let text = |field: &str| {
            body.get(field)
                .and_then(Value::as_str)
                .filter(|s| !s.is_empty())
                .map(str::to_string)
        };
        Some(Self {
            service: CaptchaService::from_wire(
                body.get("captcha_service").and_then(Value::as_str),
            )?,
            sitekey: text("captcha_sitekey"),
            rqdata: text("captcha_rqdata"),
            rqtoken: text("captcha_rqtoken"),
            session_id: text("captcha_session_id"),
            invisible: body
                .get("should_serve_invisible")
                .and_then(Value::as_bool)
                .unwrap_or(false),
            reasons,
        })
    }

    pub fn is_rejection(&self) -> bool {
        self.reasons
            .iter()
            .any(|reason| reason != "captcha-required")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CaptchaSolution {
    pub key: String,
    pub rqtoken: Option<String>,
    pub session_id: Option<String>,
}

impl CaptchaSolution {
    pub fn new(challenge: &CaptchaChallenge, key: String) -> Self {
        Self {
            key,
            rqtoken: challenge.rqtoken.clone(),
            session_id: challenge.session_id.clone(),
        }
    }

    pub fn header_pairs(&self) -> Vec<(&'static str, String)> {
        let mut pairs = vec![("x-captcha-key", self.key.clone())];
        if let Some(rqtoken) = &self.rqtoken {
            pairs.push(("x-captcha-rqtoken", rqtoken.clone()));
        }
        if let Some(session_id) = &self.session_id {
            pairs.push(("x-captcha-session-id", session_id.clone()));
        }
        pairs
    }

    pub fn apply(&self, headers: &mut header::HeaderMap) {
        let mut put = |name: &'static str, value: &str| {
            if let Ok(value) = HeaderValue::from_str(value) {
                headers.insert(name, value);
            }
        };
        put("x-captcha-key", &self.key);
        if let Some(rqtoken) = &self.rqtoken {
            put("x-captcha-rqtoken", rqtoken);
        }
        if let Some(session_id) = &self.session_id {
            put("x-captcha-session-id", session_id);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parses_an_hcaptcha_enterprise_challenge() {
        let body = json!({
            "captcha_key": ["captcha-required"], "captcha_service": "hcaptcha", "captcha_sitekey": "site",
            "captcha_rqdata": "data", "captcha_rqtoken": "token", "captcha_session_id": "session", "should_serve_invisible": true
        });
        let challenge = CaptchaChallenge::from_body(&body).unwrap();
        assert_eq!(challenge.service, CaptchaService::Hcaptcha);
        assert_eq!(challenge.sitekey.as_deref(), Some("site"));
        assert_eq!(
            (
                challenge.rqdata.as_deref(),
                challenge.rqtoken.as_deref(),
                challenge.session_id.as_deref()
            ),
            (Some("data"), Some("token"), Some("session"))
        );
        assert!(challenge.invisible);
        assert!(!challenge.is_rejection());
    }

    #[test]
    fn a_missing_service_is_recaptcha_and_an_unknown_one_is_unsupported() {
        let plain = CaptchaChallenge::from_body(
            &json!({"captcha_key": ["captcha-required"], "captcha_sitekey": "x"}),
        )
        .unwrap();
        assert_eq!(plain.service, CaptchaService::Recaptcha);
        assert!(
            CaptchaChallenge::from_body(
                &json!({"captcha_key": ["captcha-required"], "captcha_service": "mystery"})
            )
            .is_none()
        );
        assert!(CaptchaChallenge::from_body(&json!({"message": "nope"})).is_none());
    }

    #[test]
    fn another_reason_means_the_solution_was_refused() {
        let challenge = CaptchaChallenge::from_body(
            &json!({"captcha_key": ["incorrect-captcha-sol"], "captcha_service": "hcaptcha"}),
        )
        .unwrap();
        assert!(challenge.is_rejection());
    }

    #[test]
    fn the_answer_goes_in_headers_and_echoes_what_the_challenge_carried() {
        let challenge = CaptchaChallenge::from_body(&json!({"captcha_key": ["captcha-required"], "captcha_service": "hcaptcha", "captcha_rqtoken": "rq"})).unwrap();
        let mut headers = header::HeaderMap::new();
        CaptchaSolution::new(&challenge, "P1_key".into()).apply(&mut headers);
        assert_eq!(headers["x-captcha-key"], "P1_key");
        assert_eq!(headers["x-captcha-rqtoken"], "rq");
        assert!(!headers.contains_key("x-captcha-session-id"));
    }
}
