use std::sync::Arc;

use futures::future::BoxFuture;
use gpui::http_client::{self, AsyncBody, HttpClient, Url};

pub struct ReqwestHttpClient {
    client: reqwest::Client,
    user_agent: Option<http_client::http::HeaderValue>,
    runtime: tokio::runtime::Handle,
    // Keep the runtime alive for the lifetime of the client.
    _runtime_thread: Option<std::thread::JoinHandle<()>>,
}

impl ReqwestHttpClient {
    pub fn new() -> anyhow::Result<Arc<Self>> {
        let user_agent = http_client::http::HeaderValue::from_static(
            "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) discord/0.0.309 Chrome/128.0.6613.186 Electron/32.2.7 Safari/537.36",
        );
        let client = reqwest::Client::builder()
            .user_agent(user_agent.to_str().unwrap_or("Lampcord"))
            .build()?;

        // gpui polls image-loading futures on its own executor threads (no Tokio reactor).
        // Run reqwest on a dedicated Tokio runtime and bridge results back.
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .thread_name("lampcord-http")
            .build()?;
        let handle = runtime.handle().clone();
        let runtime_thread = std::thread::spawn(move || {
            runtime.block_on(std::future::pending::<()>());
        });

        Ok(Arc::new(Self {
            client,
            user_agent: Some(user_agent),
            runtime: handle,
            _runtime_thread: Some(runtime_thread),
        }))
    }
}

impl HttpClient for ReqwestHttpClient {
    fn user_agent(&self) -> Option<&http_client::http::HeaderValue> {
        self.user_agent.as_ref()
    }

    fn proxy(&self) -> Option<&Url> {
        None
    }

    fn send(
        &self,
        req: http_client::http::Request<AsyncBody>,
    ) -> BoxFuture<'static, anyhow::Result<http_client::http::Response<AsyncBody>>> {
        let client = self.client.clone();
        let handle = self.runtime.clone();
        let (tx, rx) = tokio::sync::oneshot::channel();

        handle.spawn(async move {
            let result = async {
                let (parts, body) = req.into_parts();
                let method = reqwest::Method::from_bytes(parts.method.as_str().as_bytes())
                    .unwrap_or(reqwest::Method::GET);
                let uri = parts.uri.to_string();
                let mut builder = client.request(method, &uri);
                for (name, value) in parts.headers.iter() {
                    if let Ok(v) = value.to_str() {
                        builder = builder.header(name.as_str(), v);
                    }
                }

                let body_bytes = read_async_body(body).await.unwrap_or_default();
                if !body_bytes.is_empty() {
                    builder = builder.body(body_bytes);
                }

                let resp = builder.send().await?;
                let status = resp.status().as_u16();
                let headers: Vec<(String, String)> = resp
                    .headers()
                    .iter()
                    .filter_map(|(n, v)| v.to_str().ok().map(|vv| (n.to_string(), vv.to_owned())))
                    .collect();
                let bytes = resp.bytes().await?;
                Ok::<_, anyhow::Error>((uri, status, headers, bytes))
            }
            .await;

            let _ = tx.send(result);
        });

        Box::pin(async move {
            match rx
                .await
                .map_err(|e| anyhow::anyhow!("http request task cancelled: {e}"))?
            {
                Ok((uri, status, headers, bytes)) => {
                    super::debug_stats::record_http_response(&uri, bytes.len(), status < 400);
                    let mut response_builder =
                        http_client::http::Response::builder().status(status);
                    if let Some(map) = response_builder.headers_mut() {
                        for (name, value) in headers {
                            if let (Ok(n), Ok(v)) = (
                                http_client::http::HeaderName::from_bytes(name.as_bytes()),
                                http_client::http::HeaderValue::from_str(&value),
                            ) {
                                map.insert(n, v);
                            }
                        }
                    }
                    Ok(response_builder.body(AsyncBody::from_bytes(bytes))?)
                }
                Err(e) => {
                    super::debug_stats::record_http_response("<error>", 0, false);
                    Err(e)
                }
            }
        })
    }
}

async fn read_async_body(mut body: AsyncBody) -> std::io::Result<Vec<u8>> {
    use futures::AsyncReadExt;
    let mut buf = Vec::new();
    body.read_to_end(&mut buf).await?;
    Ok(buf)
}
