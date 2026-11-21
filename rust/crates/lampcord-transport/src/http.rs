use async_trait::async_trait;
use lampcord_core::{
    rest::{Body, HttpMethod, RestRequest, RestResponse},
    traits::{RestTransport, TransportError},
};

#[derive(Clone)]
pub struct ReqwestRestTransport {
    client: reqwest::Client,
}

impl ReqwestRestTransport {
    pub fn new() -> Result<Self, TransportError> {
        Self::new_with_cookie_store()
    }

    /// Create a transport with a persistent cookie jar, required for multi-step auth flows.
    pub fn new_with_cookie_store() -> Result<Self, TransportError> {
        let client = reqwest::Client::builder()
            .cookie_store(true)
            .build()
            .map_err(|error| TransportError(error.to_string()))?;
        Ok(Self::with_client(client))
    }

    pub fn with_client(client: reqwest::Client) -> Self {
        Self { client }
    }
}

#[async_trait]
impl RestTransport for ReqwestRestTransport {
    async fn execute(&self, request: RestRequest) -> Result<RestResponse, TransportError> {
        let RestRequest {
            method,
            url,
            headers,
            body,
        } = request;

        let method = match method {
            HttpMethod::Get => reqwest::Method::GET,
            HttpMethod::Post => reqwest::Method::POST,
            HttpMethod::Put => reqwest::Method::PUT,
            HttpMethod::Patch => reqwest::Method::PATCH,
            HttpMethod::Delete => reqwest::Method::DELETE,
        };

        let mut builder = self.client.request(method, url);
        for (name, value) in headers {
            builder = builder.header(name, value);
        }

        builder = match body {
            Body::None => builder,
            Body::Json(text) | Body::Replayable(text) => builder
                .header("Content-Type", "application/json")
                .body(text),
            Body::Streaming => {
                return Err(TransportError(
                    "streaming bodies are not supported by ReqwestRestTransport".to_owned(),
                ));
            }
        };

        let response = builder
            .send()
            .await
            .map_err(|error| TransportError(error.to_string()))?;
        let status = response.status().as_u16();
        let body = response
            .text()
            .await
            .map_err(|error| TransportError(error.to_string()))?;
        Ok(RestResponse { status, body })
    }
}
