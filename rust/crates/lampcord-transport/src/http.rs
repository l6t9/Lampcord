use async_trait::async_trait;
use lampcord_core::{
    rest::{Body, HttpMethod, RestRequest, RestResponse},
    traits::{RestTransport, TransportError},
};

#[derive(Clone)]
pub struct ReqwestRestTransport {
    client: reqwest::Client,
}

fn webpki_tls_config() -> Result<rustls::ClientConfig, TransportError> {
    let mut roots = rustls::RootCertStore::empty();
    roots.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
    let provider = std::sync::Arc::new(rustls::crypto::aws_lc_rs::default_provider());
    let builder = rustls::ClientConfig::builder_with_provider(provider)
        .with_safe_default_protocol_versions()
        .map_err(|error| TransportError(error.to_string()))?;
    Ok(builder.with_root_certificates(roots).with_no_client_auth())
}

impl ReqwestRestTransport {
    pub fn new() -> Result<Self, TransportError> {
        Self::new_with_cookie_store()
    }

    /// Create a transport with a persistent cookie jar, required for multi-step auth flows.
    pub fn new_with_cookie_store() -> Result<Self, TransportError> {
        let client = reqwest::Client::builder()
            .tls_backend_preconfigured(webpki_tls_config()?)
            .cookie_store(true)
            .connect_timeout(std::time::Duration::from_secs(15))
            .timeout(std::time::Duration::from_secs(30))
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
            Body::Bytes { content_type, data } => {
                builder.header("Content-Type", content_type).body(data)
            }
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::net::TcpListener;

    #[tokio::test]
    async fn cloned_transports_share_the_cookie_jar() {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind test server");
        let address = listener.local_addr().expect("test server address");
        let server = std::thread::spawn(move || {
            let mut requests = Vec::new();
            for _ in 0..2 {
                let (mut stream, _) = listener.accept().expect("accept request");
                let mut buffer = [0_u8; 4096];
                let read = stream.read(&mut buffer).expect("read request");
                requests.push(String::from_utf8_lossy(&buffer[..read]).into_owned());
                let body = "{}";
                let response = format!(
                    "HTTP/1.1 200 OK\r\nSet-Cookie: lampcord-test=1; Path=/\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    body.len(),
                    body
                );
                stream
                    .write_all(response.as_bytes())
                    .expect("write response");
            }
            requests
        });

        let transport = ReqwestRestTransport::new().expect("transport");
        let request = |path: &str| RestRequest {
            method: HttpMethod::Get,
            url: format!("http://{address}/{path}"),
            headers: Vec::new(),
            body: Body::None,
        };
        transport
            .clone()
            .execute(request("first"))
            .await
            .expect("first request");
        transport
            .clone()
            .execute(request("second"))
            .await
            .expect("second request");

        let requests = server.join().expect("test server thread");
        assert!(
            !requests[0].to_ascii_lowercase().contains("cookie:"),
            "first request cannot have the cookie yet: {}",
            requests[0]
        );
        assert!(
            requests[1].to_ascii_lowercase().contains("lampcord-test=1"),
            "cloned transport must reuse the cookie jar: {}",
            requests[1]
        );
    }
}
