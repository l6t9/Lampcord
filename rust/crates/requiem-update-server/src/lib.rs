pub mod auth;
pub mod error;
pub mod publish;
pub mod store;

use std::path::PathBuf;
use std::sync::Arc;

use axum::Router;
use axum::body::Body;
use axum::extract::{Request, State};
use axum::http::{HeaderValue, Method, StatusCode, header};
use axum::response::{IntoResponse, Response};
use requiem_update::{KeyEntry, KeySet, KeysDocument};
use tokio::sync::Mutex;
use tower::ServiceExt;
use tower_http::services::ServeDir;

use crate::auth::Authenticator;
use crate::error::ApiError;

#[derive(Debug, Clone)]
pub struct Config {
    pub root: PathBuf,

    pub public_base: String,
    pub max_release_bytes: u64,

    pub max_part_bytes: u64,
}

struct Inner {
    config: Config,
    keys: KeySet,
    auth: Box<dyn Authenticator>,
    publish_lock: Mutex<()>,
    prefix: String,
}

#[derive(Clone)]
pub struct AppState(Arc<Inner>);

impl AppState {
    pub fn new(mut config: Config, keys: KeySet, auth: Box<dyn Authenticator>) -> Self {
        config.public_base = config.public_base.trim_end_matches('/').to_string();
        let prefix = prefix_of(&config.public_base).to_string();
        Self(Arc::new(Inner {
            config,
            keys,
            auth,
            publish_lock: Mutex::new(()),
            prefix,
        }))
    }

    pub fn config(&self) -> &Config {
        &self.0.config
    }

    pub fn keys(&self) -> &KeySet {
        &self.0.keys
    }

    pub fn publish_lock(&self) -> &Mutex<()> {
        &self.0.publish_lock
    }
}

pub fn prefix_of(public_base: &str) -> &str {
    let base = public_base.trim_end_matches('/');
    base.split_once("://")
        .map_or("", |(_, rest)| rest.find('/').map_or("", |at| &rest[at..]))
}

pub fn router(state: AppState) -> Router {
    Router::new().fallback(handle).with_state(state)
}

fn json(status: StatusCode, body: serde_json::Value) -> Response {
    (status, axum::Json(body)).into_response()
}

async fn handle(State(state): State<AppState>, req: Request) -> Response {
    match dispatch(&state, req).await {
        Ok(response) => response,
        Err(error) => error.into_response(),
    }
}

async fn dispatch(state: &AppState, req: Request) -> Result<Response, ApiError> {
    let path = req.uri().path().to_string();
    let rest = path
        .strip_prefix(state.0.prefix.as_str())
        .ok_or(ApiError::NotFound)?
        .to_string();
    if rest.is_empty() || !rest.starts_with('/') {
        return Err(ApiError::NotFound);
    }
    match *req.method() {
        Method::GET | Method::HEAD => read(state, req, &rest).await,
        Method::PUT => write(state, req, &rest).await,
        _ => Ok(json(
            StatusCode::METHOD_NOT_ALLOWED,
            serde_json::json!({ "ok": false, "error": "method not allowed" }),
        )),
    }
}

async fn read(state: &AppState, req: Request, rest: &str) -> Result<Response, ApiError> {
    if rest == "/health" {
        return Ok(json(StatusCode::OK, serde_json::json!({ "ok": true })));
    }
    if rest == "/v1/keys.json" {
        let keys = state.keys();
        let entries: Vec<KeyEntry> = keys
            .ids()
            .filter_map(|id| keys.get(id).map(|key| KeyEntry::new(id, key)))
            .collect();
        return Ok(json(
            StatusCode::OK,
            serde_json::to_value(KeysDocument {
                schema: 1,
                keys: entries,
            })
            .expect("keys serialize"),
        ));
    }
    if !rest.starts_with("/v1/") {
        return Err(ApiError::NotFound);
    }

    let (mut parts, body) = req.into_parts();
    let query = parts
        .uri
        .query()
        .map(|q| format!("?{q}"))
        .unwrap_or_default();
    parts.uri = format!("{rest}{query}")
        .parse()
        .map_err(|_| ApiError::bad("bad path"))?;
    let mut response = ServeDir::new(&state.config().root)
        .oneshot(Request::from_parts(parts, body))
        .await
        .map_err(ApiError::internal)?
        .into_response();
    let cache = if rest.ends_with(".zip") || rest.ends_with(".exe") {
        "public, max-age=31536000, immutable"
    } else {
        "public, max-age=60"
    };
    if response.status().is_success() {
        response
            .headers_mut()
            .insert(header::CACHE_CONTROL, HeaderValue::from_static(cache));
    }
    Ok(response)
}

async fn write(state: &AppState, req: Request, rest: &str) -> Result<Response, ApiError> {
    let token = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or_else(|| ApiError::Unauthorized("a bearer token is required".into()))?
        .to_string();
    let who = state.0.auth.authenticate(&token).await?;
    log::info!("PUT {rest} by {who}");

    let content_length = req
        .headers()
        .get(header::CONTENT_LENGTH)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse::<u64>().ok());
    let body: Body = req.into_body();
    let segments: Vec<&str> = rest.trim_start_matches('/').split('/').collect();
    let outcome = match segments.as_slice() {
        ["v1", "releases", version, name] => {
            publish::put_release(state, version, name, content_length, body).await?
        }
        ["v1", "releases", version, name, "parts", index] => {
            publish::put_part(state, version, name, index, content_length, body).await?
        }
        ["v1", "releases", version, name, "complete"] => {
            publish::complete_parts(state, version, name, body).await?
        }
        ["v1", "channels", channel, file] => publish::put_feed(state, channel, file, body).await?,
        ["v1", "identity.json"] => publish::put_identity(state, body).await?,
        _ => return Err(ApiError::NotFound),
    };
    Ok(json(
        outcome.status,
        serde_json::json!({ "ok": true, "message": outcome.message }),
    ))
}
