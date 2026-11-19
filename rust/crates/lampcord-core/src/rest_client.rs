use std::sync::Arc;

use async_trait::async_trait;
use serde_json::Value;

use crate::{
    rest::{
        Body, CaptchaRetry, HttpMethod, RestConfig, RestRequest, RestResponse,
        apply_captcha_headers, unauthorized_token,
    },
    traits::{
        CaptchaSolver, GatewayUrlProvider, RestTransport as Transport, TransportError,
        UnauthorizedSink,
    },
};

#[derive(Debug)]
pub enum RestError {
    Transport(TransportError),
    CaptchaCancelled,
}

impl std::fmt::Display for RestError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Transport(error) => write!(formatter, "REST transport failed: {error}"),
            Self::CaptchaCancelled => formatter.write_str("captcha was cancelled"),
        }
    }
}

impl std::error::Error for RestError {}

pub struct RestClient<T: Transport, S: CaptchaSolver> {
    config: RestConfig,
    transport: T,
    solver: S,
    unauthorized: Option<Arc<dyn UnauthorizedSink>>,
}

impl<T: Transport, S: CaptchaSolver> RestClient<T, S> {
    pub fn new(config: RestConfig, transport: T, solver: S) -> Self {
        Self {
            config,
            transport,
            solver,
            unauthorized: None,
        }
    }

    pub fn with_unauthorized_sink(mut self, sink: Arc<dyn UnauthorizedSink>) -> Self {
        self.unauthorized = Some(sink);
        self
    }

    pub fn config(&self) -> &RestConfig {
        &self.config
    }

    pub async fn send(&self, mut request: RestRequest) -> Result<RestResponse, RestError> {
        let mut retry = CaptchaRetry::new();

        loop {
            let response = self
                .transport
                .execute(request.clone())
                .await
                .map_err(RestError::Transport)?;

            if let Some(challenge) = retry.next_challenge(&response, &request) {
                let token = self
                    .solver
                    .solve(&challenge)
                    .await
                    .ok_or(RestError::CaptchaCancelled)?;
                apply_captcha_headers(&mut request, &challenge, token);
                continue;
            }

            if let Some(token) = unauthorized_token(&response, &request) {
                if let Some(sink) = self.unauthorized.as_deref() {
                    sink.on_unauthorized(token).await;
                }
            }

            return Ok(response);
        }
    }

    pub async fn gateway_url(&self) -> Option<String> {
        let request = self.config.request(
            HttpMethod::Get,
            "/gateway",
            self.config.standard_headers(),
            Body::None,
        );
        let response = self.send(request).await.ok()?;
        let value: Value = serde_json::from_str(&response.body).ok()?;
        value.get("url")?.as_str().map(str::to_owned)
    }
}

#[async_trait]
impl<T: Transport, S: CaptchaSolver> GatewayUrlProvider for RestClient<T, S> {
    async fn gateway_url(&self) -> Option<String> {
        RestClient::gateway_url(self).await
    }
}

#[cfg(test)]
mod tests {
    use std::{
        collections::VecDeque,
        sync::{Arc, Mutex},
    };

    use super::*;
    use crate::captcha::CaptchaChallenge;
    use crate::traits::{CaptchaSolver, RestTransport};

    #[derive(Default)]
    struct ScriptedRestTransport {
        responses: Mutex<VecDeque<RestResponse>>,
        requests: Mutex<Vec<RestRequest>>,
    }

    #[async_trait]
    impl RestTransport for ScriptedRestTransport {
        async fn execute(&self, request: RestRequest) -> Result<RestResponse, TransportError> {
            self.requests.lock().expect("requests lock").push(request);
            self.responses
                .lock()
                .expect("responses lock")
                .pop_front()
                .ok_or_else(|| TransportError("no scripted response".to_owned()))
        }
    }

    struct StaticSolver;

    #[async_trait]
    impl CaptchaSolver for StaticSolver {
        async fn solve(&self, _challenge: &CaptchaChallenge) -> Option<String> {
            Some("captcha-token".to_owned())
        }
    }

    #[derive(Default)]
    struct CapturingUnauthorizedSink {
        tokens: Mutex<Vec<String>>,
    }

    #[async_trait]
    impl UnauthorizedSink for CapturingUnauthorizedSink {
        async fn on_unauthorized(&self, token: String) {
            self.tokens.lock().expect("tokens lock").push(token);
        }
    }

    fn config() -> RestConfig {
        RestConfig::new(Some("token-1".to_owned()), "super", "install", "UTC")
    }

    fn request() -> RestRequest {
        config().request(
            HttpMethod::Post,
            "/auth/login",
            config().standard_headers(),
            Body::Json("{}".to_owned()),
        )
    }

    fn header<'a>(request: &'a RestRequest, name: &str) -> Option<&'a str> {
        request
            .headers
            .iter()
            .find(|(key, _)| key.eq_ignore_ascii_case(name))
            .map(|(_, value)| value.as_str())
    }

    #[tokio::test]
    async fn retries_with_captcha_headers() {
        let transport = ScriptedRestTransport::default();
        transport.responses.lock().expect("responses lock").extend([
            RestResponse {
                status: 400,
                body: r#"{"captcha_key":["x"],"captcha_sitekey":"site-1","captcha_session_id":"session-1"}"#.to_owned(),
            },
            RestResponse {
                status: 200,
                body: r#"{"token":"new-token"}"#.to_owned(),
            },
        ]);
        let client = RestClient::new(config(), transport, StaticSolver);

        let response = client.send(request()).await.expect("response");
        assert_eq!(response.status, 200);

        let requests = client.transport.requests.lock().expect("requests lock");
        assert_eq!(requests.len(), 2);
        assert_eq!(header(&requests[0], "X-Captcha-Key"), None);
        assert_eq!(header(&requests[1], "X-Captcha-Key"), Some("captcha-token"));
        assert_eq!(
            header(&requests[1], "X-Captcha-Session-Id"),
            Some("session-1")
        );
    }

    #[tokio::test]
    async fn reports_current_unauthorized_token() {
        let transport = ScriptedRestTransport::default();
        transport
            .responses
            .lock()
            .expect("responses lock")
            .push_back(RestResponse {
                status: 401,
                body: r#"{"message":"401: Unauthorized"}"#.to_owned(),
            });
        let sink = Arc::new(CapturingUnauthorizedSink::default());
        let client =
            RestClient::new(config(), transport, StaticSolver).with_unauthorized_sink(sink.clone());

        let response = client.send(request()).await.expect("response");
        assert_eq!(response.status, 401);
        assert_eq!(
            sink.tokens.lock().expect("tokens lock").as_slice(),
            &["token-1".to_owned()]
        );
    }
}
