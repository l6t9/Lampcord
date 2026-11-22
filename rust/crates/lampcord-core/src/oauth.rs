use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OAuthRequestSource {
    AndroidDeeplink,
    DesktopUrl,
    DesktopRpc,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OAuthAuthorizationRequest {
    pub id: String,
    pub source: OAuthRequestSource,
    pub client_id: String,
    pub response_type: String,
    pub scopes: Vec<String>,
    pub redirect_uri: Option<String>,
    pub state: Option<String>,
    pub nonce: Option<String>,
    pub code_challenge: Option<String>,
    pub code_challenge_method: Option<String>,
    pub prompt: Option<String>,
    pub integration_type: Option<i32>,
    pub permissions: Option<String>,
    pub guild_id: Option<String>,
    pub channel_id: Option<String>,
    pub received_at_millis: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OAuthRedirectValidation {
    Success {
        code: String,
        redirect_url: Option<String>,
    },
    Error {
        error: String,
        error_description: Option<String>,
        redirect_url: Option<String>,
    },
}

impl OAuthRedirectValidation {
    pub fn code(&self) -> Option<&str> {
        match self {
            Self::Success { code, .. } => Some(code),
            Self::Error { .. } => None,
        }
    }

    pub fn error(&self) -> Option<&str> {
        match self {
            Self::Error { error, .. } => Some(error),
            Self::Success { .. } => None,
        }
    }

    pub fn error_description(&self) -> Option<&str> {
        match self {
            Self::Error {
                error_description, ..
            } => error_description.as_deref(),
            Self::Success { .. } => None,
        }
    }

    pub fn redirect_url(&self) -> Option<&str> {
        match self {
            Self::Success { redirect_url, .. } | Self::Error { redirect_url, .. } => {
                redirect_url.as_deref()
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ParsedUrl {
    scheme: String,
    authority: String,
    host: String,
    path: String,
    query: HashMap<String, String>,
}

const MAX_URL_LENGTH: usize = 2048;
const MAX_STATE_LENGTH: usize = 256;
const MAX_SCOPE_LENGTH: usize = 512;
const ISSUER: &str = "https://discord.com";

fn authorize_hosts() -> &'static [&'static str] {
    &[
        "discord.com",
        "www.discord.com",
        "discordapp.com",
        "www.discordapp.com",
        "canary.discord.com",
        "ptb.discord.com",
    ]
}

fn rejected_scopes() -> &'static [&'static str] {
    &[
        "bot",
        "webhook.incoming",
        "rpc",
        "applications.builds.upload",
        "applications.store.update",
        "applications.entitlements",
        "applications.commands.update",
        "guilds.join",
    ]
}

fn loopback_hosts() -> &'static [&'static str] {
    &["127.0.0.1", "localhost", "::1", "[::1]"]
}

fn is_iso_control(c: char) -> bool {
    c.is_control()
}

fn is_valid_scheme_char_start(c: char) -> bool {
    c.is_ascii_lowercase()
}

fn is_valid_scheme_char(c: char) -> bool {
    c.is_ascii_lowercase() || c.is_ascii_digit() || matches!(c, '+' | '-' | '.')
}

fn is_valid_custom_scheme(s: &str) -> bool {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) if is_valid_scheme_char_start(c) => {}
        _ => return false,
    }
    chars.all(is_valid_scheme_char)
}

fn is_snowflake(s: &str) -> bool {
    !s.is_empty() && s.len() <= 20 && s.chars().all(|c| c.is_ascii_digit())
}

fn is_client_id(s: &str) -> bool {
    if !(17..=20).contains(&s.len()) {
        return false;
    }
    let mut chars = s.chars();
    match chars.next() {
        Some(c) if c.is_ascii_digit() && c != '0' => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_digit())
}

fn is_code_challenge(s: &str) -> bool {
    s.len() == 43
        && s.chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
}

fn percent_decode(value: &str) -> Option<String> {
    let bytes = value.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'+' => {
                out.push(b' ');
                i += 1;
            }
            b'%' => {
                if i + 2 >= bytes.len() {
                    return None;
                }
                let high = (bytes[i + 1] as char).to_digit(16)? as u8;
                let low = (bytes[i + 2] as char).to_digit(16)? as u8;
                out.push((high << 4) | low);
                i += 3;
            }
            b => {
                out.push(b);
                i += 1;
            }
        }
    }
    String::from_utf8(out).ok()
}

fn split_url(url: &str) -> Option<ParsedUrl> {
    if url.is_empty() || url.len() > MAX_URL_LENGTH {
        return None;
    }
    if url
        .chars()
        .any(|c| c == '\\' || c.is_whitespace() || is_iso_control(c))
    {
        return None;
    }

    let (scheme, remainder, has_double_slash) = if let Some(idx) = url.find("://") {
        if idx == 0 {
            return None;
        }
        (url[..idx].to_ascii_lowercase(), &url[idx + 3..], true)
    } else {
        let idx = url.find(':')?;
        if idx == 0 {
            return None;
        }
        (url[..idx].to_ascii_lowercase(), &url[idx + 1..], false)
    };

    if !is_valid_custom_scheme(&scheme) {
        return None;
    }

    // Reject fragments (fragments would appear after # in remainder/afterAuthority; KMP checks after stripping fragment equality)
    if remainder.contains('#') {
        // KMP: withoutFragment = remainder.substringBefore('#'); if different length return null
        // so any '#' means fragment present -> reject
        return None;
    }

    let (authority, after_authority) =
        if has_double_slash && (scheme == "https" || scheme == "http") {
            let end = remainder.find('/').unwrap_or(remainder.len());
            (&remainder[..end], &remainder[end..])
        } else {
            // For custom schemes or non-http(s) with ://, KMP treats differently; but best-effort:
            // KMP logic:
            // - if http/https: authority up to '/', afterAuthority from '/'
            // - else: authority up to '/' in withoutFragment, afterAuthority from '/' or ""
            if scheme == "https" || scheme == "http" {
                // shouldn't happen without :// handled above, but keep safe
                let end = remainder.find('/').unwrap_or(remainder.len());
                (&remainder[..end], &remainder[end..])
            } else {
                let end = remainder.find('/');
                match end {
                    Some(e) => (&remainder[..e], &remainder[e..]),
                    None => (remainder, ""),
                }
            }
        };

    if authority.contains('@') {
        return None;
    }

    let host = if authority.starts_with('[') {
        // [ipv6] or [ipv6]:port
        if let Some(end) = authority.find(']') {
            authority[..=end].to_ascii_lowercase()
        } else {
            authority.to_ascii_lowercase()
        }
    } else if let Some(pos) = authority.rfind(':') {
        // could be host:port (not perfect for raw ipv6 without brackets, but KMP does this)
        authority[..pos].to_ascii_lowercase()
    } else {
        authority.to_ascii_lowercase()
    };

    let (path, query_str) = if let Some(q) = after_authority.find('?') {
        (&after_authority[..q], Some(&after_authority[q + 1..]))
    } else {
        (after_authority, None)
    };

    let mut query = HashMap::new();
    if let Some(qs) = query_str {
        if !qs.is_empty() {
            for pair in qs.split('&') {
                if pair.is_empty() {
                    continue;
                }
                let (raw_key, raw_val) = if let Some(eq) = pair.find('=') {
                    (&pair[..eq], &pair[eq + 1..])
                } else {
                    (pair, "")
                };
                let key = percent_decode(raw_key)?;
                let value = percent_decode(raw_val)?;
                if query.insert(key, value).is_some() {
                    return None; // duplicate keys rejected
                }
            }
        }
    }

    Some(ParsedUrl {
        scheme,
        authority: authority.to_owned(),
        host,
        path: path.to_owned(),
        query,
    })
}

fn is_http_redirect(uri: &str) -> bool {
    uri.to_ascii_lowercase().starts_with("https://")
        || uri.to_ascii_lowercase().starts_with("http://")
}

fn is_plausible_redirect(uri: &str) -> bool {
    if is_http_redirect(uri) {
        return split_url(uri).is_some_and(|p| !p.host.is_empty());
    }
    let Some(colon) = uri.find(':') else {
        return false;
    };
    if colon == 0 {
        return false;
    }
    let scheme = uri[..colon].to_ascii_lowercase();
    if !is_valid_custom_scheme(&scheme) {
        return false;
    }
    if matches!(scheme.as_str(), "javascript" | "data" | "file" | "content") {
        return false;
    }
    uri[colon + 1..].trim().len() > 0
}

pub fn parse_authorization_url(
    url: &str,
    source: OAuthRequestSource,
    id: impl Into<String>,
    received_at_millis: u64,
) -> Option<OAuthAuthorizationRequest> {
    let parsed = split_url(url)?;

    let is_web = parsed.scheme == "https" && authorize_hosts().contains(&parsed.host.as_str());
    let is_discord_deeplink = parsed.scheme == "discord" && parsed.host == "action";
    if (!is_web && !is_discord_deeplink) || parsed.path != "/oauth2/authorize" {
        return None;
    }

    let q = &parsed.query;
    if q.get("response_type").map(String::as_str) != Some("code") {
        return None;
    }

    let client_id = q.get("client_id")?.trim().to_owned();
    if !is_client_id(&client_id) {
        return None;
    }

    let state = q.get("state")?.trim().to_owned();
    if state.is_empty() || state.len() > MAX_STATE_LENGTH {
        return None;
    }
    if state
        .chars()
        .any(|c| is_iso_control(c) || c.is_whitespace())
    {
        return None;
    }

    let scope_value = q.get("scope")?.trim().to_owned();
    if scope_value.is_empty() || scope_value.len() > MAX_SCOPE_LENGTH {
        return None;
    }
    let mut scopes: Vec<String> = Vec::new();
    for scope in scope_value.split(' ').filter(|s| !s.is_empty()) {
        if !scopes.iter().any(|s| s == scope) {
            scopes.push(scope.to_owned());
        }
    }
    if scopes.is_empty() {
        return None;
    }
    if scopes
        .iter()
        .any(|s| rejected_scopes().contains(&s.as_str()))
    {
        return None;
    }

    let redirect_uri = q
        .get("redirect_uri")
        .map(|s| s.trim().to_owned())
        .filter(|s| !s.is_empty());
    if let Some(ref uri) = redirect_uri {
        if uri.len() > MAX_URL_LENGTH {
            return None;
        }
        if !is_plausible_redirect(uri) {
            return None;
        }
    }

    let code_challenge = q
        .get("code_challenge")
        .map(|s| s.trim().to_owned())
        .filter(|s| !s.is_empty());
    let code_challenge_method = q
        .get("code_challenge_method")
        .map(|s| s.trim().to_owned())
        .filter(|s| !s.is_empty());
    let has_pkce = code_challenge.is_some();
    if let Some(ref cc) = code_challenge {
        if code_challenge_method.as_deref() != Some("S256") || !is_code_challenge(cc) {
            return None;
        }
    }
    if let Some(ref uri) = redirect_uri {
        if !is_http_redirect(uri) && !has_pkce {
            return None;
        }
    }

    let guild_id = q
        .get("guild_id")
        .map(|s| s.trim().to_owned())
        .filter(|s| is_snowflake(s));
    let channel_id = q
        .get("channel_id")
        .map(|s| s.trim().to_owned())
        .filter(|s| is_snowflake(s));
    let permissions = q
        .get("permissions")
        .map(|s| s.trim().to_owned())
        .filter(|s| !s.is_empty() && s.len() <= 64 && s.chars().all(|c| c.is_ascii_digit()));
    let integration_type = q
        .get("integration_type")
        .and_then(|s| s.trim().parse::<i32>().ok())
        .filter(|v| (0..=1).contains(v));
    let nonce = q
        .get("nonce")
        .map(|s| s.trim().to_owned())
        .filter(|s| s.len() <= MAX_STATE_LENGTH && !s.is_empty());
    let prompt = q
        .get("prompt")
        .map(|s| s.trim().to_owned())
        .filter(|s| s == "consent" || s == "none");

    Some(OAuthAuthorizationRequest {
        id: id.into(),
        source,
        client_id,
        response_type: "code".to_owned(),
        scopes,
        redirect_uri,
        state: Some(state),
        nonce,
        code_challenge,
        code_challenge_method,
        prompt,
        integration_type,
        permissions,
        guild_id,
        channel_id,
        received_at_millis,
    })
}

fn normalize_path(path: &str) -> String {
    let trimmed = path.trim().trim_end_matches('/');
    if trimmed.is_empty() {
        "/".to_owned()
    } else {
        trimmed.to_owned()
    }
}

fn matches_requested_redirect(returned: &ParsedUrl, requested: &str) -> bool {
    let Some(expected) = split_url(requested) else {
        return false;
    };
    if expected.scheme != returned.scheme {
        return false;
    }
    if expected.scheme == "https" || expected.scheme == "http" {
        return expected.host == returned.host
            && normalize_path(&expected.path) == normalize_path(&returned.path);
    }
    normalize_path(&expected.path) == normalize_path(&returned.path)
}

pub fn validate_redirect(
    url: &str,
    request: &OAuthAuthorizationRequest,
) -> OAuthRedirectValidation {
    let Some(parsed) = split_url(url) else {
        return OAuthRedirectValidation::Error {
            error: "invalid_redirect".to_owned(),
            error_description: None,
            redirect_url: None,
        };
    };
    if parsed.authority.contains('@') {
        return OAuthRedirectValidation::Error {
            error: "invalid_redirect".to_owned(),
            error_description: None,
            redirect_url: None,
        };
    }

    let is_https = parsed.scheme == "https";
    let is_loopback_http =
        parsed.scheme == "http" && loopback_hosts().contains(&parsed.host.as_str());
    let is_custom = !is_https && !is_loopback_http;
    if is_custom && parsed.path != "/authorize/callback" {
        return OAuthRedirectValidation::Error {
            error: "invalid_redirect".to_owned(),
            error_description: None,
            redirect_url: None,
        };
    }

    if let Some(ref requested) = request.redirect_uri {
        if !matches_requested_redirect(&parsed, requested) {
            return OAuthRedirectValidation::Error {
                error: "redirect_mismatch".to_owned(),
                error_description: None,
                redirect_url: None,
            };
        }
    }

    let query = &parsed.query;
    if let Some(error) = query.get("error") {
        return OAuthRedirectValidation::Error {
            error: error.clone(),
            error_description: query.get("error_description").cloned(),
            redirect_url: Some(url.to_owned()),
        };
    }

    if let Some(ref expected_state) = request.state {
        if query.get("state") != Some(expected_state) {
            return OAuthRedirectValidation::Error {
                error: "invalid_state".to_owned(),
                error_description: None,
                redirect_url: None,
            };
        }
    }

    if let Some(iss) = query.get("iss") {
        if iss.trim_end_matches('/') != ISSUER {
            return OAuthRedirectValidation::Error {
                error: "invalid_issuer".to_owned(),
                error_description: None,
                redirect_url: None,
            };
        }
    }

    let code = query.get("code").map(|s| s.trim().to_owned());
    match code {
        Some(c) if !c.is_empty() && c.len() <= 512 => OAuthRedirectValidation::Success {
            code: c,
            redirect_url: Some(url.to_owned()),
        },
        _ => OAuthRedirectValidation::Error {
            error: "missing_code".to_owned(),
            error_description: None,
            redirect_url: Some(url.to_owned()),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request(
        redirect_uri: Option<&str>,
        state: Option<&str>,
        code_challenge: Option<&str>,
        code_challenge_method: Option<&str>,
    ) -> OAuthAuthorizationRequest {
        OAuthAuthorizationRequest {
            id: "request-1".to_owned(),
            source: OAuthRequestSource::AndroidDeeplink,
            client_id: "157730590492196864".to_owned(),
            response_type: "code".to_owned(),
            scopes: vec!["openid".to_owned(), "sdk.social_layer_presence".to_owned()],
            redirect_uri: redirect_uri.map(str::to_owned),
            state: state.map(str::to_owned),
            nonce: None,
            code_challenge: code_challenge.map(str::to_owned),
            code_challenge_method: code_challenge_method.map(str::to_owned),
            prompt: Some("consent".to_owned()),
            integration_type: None,
            permissions: None,
            guild_id: None,
            channel_id: None,
            received_at_millis: 0,
        }
    }

    #[test]
    fn parses_documented_web_authorization_url() {
        let base = "https://discord.com/oauth2/authorize";
        let req = parse_authorization_url(
            &format!(
                "{base}?response_type=code&client_id=157730590492196864\
                 &scope=openid%20sdk.social_layer_presence\
                 &redirect_uri=http%3A%2F%2F127.0.0.1%3A12345%2Fcallback\
                 &state=state-1234567890&prompt=consent"
            ),
            OAuthRequestSource::AndroidDeeplink,
            "id",
            0,
        )
        .unwrap();
        assert_eq!(req.client_id, "157730590492196864");
        assert_eq!(
            req.scopes,
            vec!["openid".to_owned(), "sdk.social_layer_presence".to_owned()]
        );
        assert_eq!(
            req.redirect_uri.as_deref(),
            Some("http://127.0.0.1:12345/callback")
        );
        assert_eq!(req.state.as_deref(), Some("state-1234567890"));
    }

    #[test]
    fn parses_discord_deep_link_authorization_url() {
        let challenge = "a".repeat(43);
        let req = parse_authorization_url(
            &format!(
                "discord://action/oauth2/authorize?response_type=code&client_id=157730590492196864\
                 &scope=sdk.social_layer&state=state-1234567890\
                 &redirect_uri=discord-157730590492196864%3A%2Fauthorize%2Fcallback\
                 &code_challenge={challenge}&code_challenge_method=S256"
            ),
            OAuthRequestSource::AndroidDeeplink,
            "id",
            0,
        )
        .unwrap();
        assert_eq!(req.source, OAuthRequestSource::AndroidDeeplink);
        assert_eq!(req.code_challenge_method.as_deref(), Some("S256"));
    }

    #[test]
    fn rejects_unsafe_or_malformed_requests() {
        let base = "https://discord.com/oauth2/authorize";
        assert!(parse_authorization_url(
            &format!("{base}?response_type=token&client_id=157730590492196864&scope=identify&state=state-1234567890"),
            OAuthRequestSource::DesktopUrl, "id", 0
        ).is_none());
        assert!(
            parse_authorization_url(
                &format!(
                    "{base}?response_type=code&client_id=123&scope=identify&state=state-1234567890"
                ),
                OAuthRequestSource::DesktopUrl,
                "id",
                0
            )
            .is_none()
        );
        assert!(parse_authorization_url(
            &format!("{base}?response_type=code&client_id=157730590492196864&scope=bot&state=state-1234567890"),
            OAuthRequestSource::DesktopUrl, "id", 0
        ).is_none());
        assert!(parse_authorization_url(
            "https://evil.example/oauth2/authorize?response_type=code&client_id=157730590492196864&scope=identify&state=state-1234567890",
            OAuthRequestSource::DesktopUrl, "id", 0
        ).is_none());
        assert!(parse_authorization_url(
            &format!("{base}?response_type=code&client_id=157730590492196864&scope=identify&state=state-1234567890&redirect_uri=mygame%3A%2Fauthorize%2Fcallback"),
            OAuthRequestSource::DesktopUrl, "id", 0
        ).is_none());
    }

    #[test]
    fn validates_discord_issued_redirect_and_rejects_tampering() {
        let req = request(
            Some("http://127.0.0.1:12345/callback"),
            Some("state-1234567890"),
            None,
            None,
        );
        let successful = validate_redirect(
            "http://127.0.0.1:12345/callback?code=abc123&state=state-1234567890&iss=https%3A%2F%2Fdiscord.com",
            &req,
        );
        assert_eq!(successful.code(), Some("abc123"));
        assert_eq!(successful.error(), None);

        let wrong_state = validate_redirect(
            "http://127.0.0.1:12345/callback?code=abc123&state=other&iss=https%3A%2F%2Fdiscord.com",
            &req,
        );
        assert_eq!(wrong_state.error(), Some("invalid_state"));

        let denied = validate_redirect(
            "http://127.0.0.1:12345/callback?error=access_denied&state=state-1234567890",
            &req,
        );
        assert_eq!(denied.error(), Some("access_denied"));

        let remote_http = validate_redirect(
            "http://evil.example/callback?code=abc123&state=state-1234567890",
            &request(
                Some("http://evil.example/callback"),
                Some("state-1234567890"),
                None,
                None,
            ),
        );
        assert_eq!(remote_http.error(), Some("invalid_redirect"));
    }

    #[test]
    fn custom_scheme_redirects_require_authorize_callback_path() {
        let custom = "discord-157730590492196864:/authorize/callback";
        let challenge = "a".repeat(43);
        let valid = validate_redirect(
            &format!("{custom}?code=abc123&state=state-1234567890"),
            &request(
                Some(custom),
                Some("state-1234567890"),
                Some(&challenge),
                Some("S256"),
            ),
        );
        assert_eq!(valid.code(), Some("abc123"));

        let invalid = validate_redirect(
            "discord-157730590492196864:/evil?code=abc123&state=state-1234567890",
            &request(
                Some("discord-157730590492196864:/evil"),
                Some("state-1234567890"),
                Some(&challenge),
                Some("S256"),
            ),
        );
        assert_eq!(invalid.error(), Some("invalid_redirect"));
    }
}
