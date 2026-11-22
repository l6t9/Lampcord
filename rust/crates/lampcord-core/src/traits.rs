use async_trait::async_trait;

use crate::{
    captcha::CaptchaChallenge,
    rest::{RestRequest, RestResponse},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransportError(pub String);

impl std::fmt::Display for TransportError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::error::Error for TransportError {}

#[async_trait]
pub trait RestTransport: Send + Sync {
    async fn execute(&self, request: RestRequest) -> Result<RestResponse, TransportError>;
}

#[async_trait]
pub trait CaptchaSolver: Send + Sync {
    async fn solve(&self, challenge: &CaptchaChallenge) -> Option<String>;
}

#[async_trait]
pub trait UnauthorizedSink: Send + Sync {
    async fn on_unauthorized(&self, token: String);
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GatewayMessage {
    Text(String),
    Closed,
}

#[async_trait]
pub trait GatewayTransport: Send {
    async fn connect(
        &mut self,
        url: &str,
        headers: &[(String, String)],
    ) -> Result<(), TransportError>;
    async fn receive(&mut self) -> Result<GatewayMessage, TransportError>;
    async fn send(&mut self, text: &str) -> Result<(), TransportError>;
    async fn close(&mut self, code: u16, reason: &str) -> Result<(), TransportError>;
    fn close_code(&self) -> Option<u16>;
}

#[async_trait]
pub trait GatewayUrlProvider: Send + Sync {
    async fn gateway_url(&self) -> Option<String>;
}
