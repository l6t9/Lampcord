use std::sync::Arc;
use std::time::{Duration, Instant};

use async_trait::async_trait;
use jsonwebtoken::jwk::JwkSet;
use jsonwebtoken::{Algorithm, DecodingKey, Validation};
use serde::Deserialize;
use tokio::sync::Mutex;

use crate::error::ApiError;
use reqwest::Client;

#[async_trait]
pub trait Authenticator: Send + Sync {
    async fn authenticate(&self, token: &str) -> Result<String, ApiError>;
}

pub struct StaticToken(pub String);

#[async_trait]
impl Authenticator for StaticToken {
    async fn authenticate(&self, token: &str) -> Result<String, ApiError> {
        let (a, b) = (self.0.as_bytes(), token.as_bytes());
        let mut diff = a.len() ^ b.len();
        for i in 0..a.len().min(b.len()) {
            diff |= usize::from(a[i] ^ b[i]);
        }
        if diff == 0 {
            Ok("static token".into())
        } else {
            Err(ApiError::Unauthorized("invalid token".into()))
        }
    }
}

#[async_trait]
pub trait KeyProvider: Send + Sync {
    async fn key(&self, kid: &str) -> Option<DecodingKey>;
}

pub struct GithubJwks {
    url: String,
    client: Client,
    cache: Mutex<Option<(Instant, JwkSet)>>,
}

impl GithubJwks {
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            client: Client::builder()
                .timeout(Duration::from_secs(10))
                .build()
                .expect("http client"),
            cache: Mutex::new(None),
        }
    }

    async fn fetch(&self) -> Result<JwkSet, String> {
        self.client
            .get(&self.url)
            .send()
            .await
            .map_err(|e| e.to_string())?
            .error_for_status()
            .map_err(|e| e.to_string())?
            .json::<JwkSet>()
            .await
            .map_err(|e| e.to_string())
    }
}

#[async_trait]
impl KeyProvider for GithubJwks {
    async fn key(&self, kid: &str) -> Option<DecodingKey> {
        let mut cache = self.cache.lock().await;
        let find = |set: &JwkSet| {
            set.find(kid)
                .and_then(|jwk| DecodingKey::from_jwk(jwk).ok())
        };
        if let Some((_, set)) = cache.as_ref() {
            if let Some(key) = find(set) {
                return Some(key);
            }
        }
        if cache
            .as_ref()
            .is_some_and(|(at, _)| at.elapsed() < Duration::from_secs(60))
        {
            return None;
        }
        match self.fetch().await {
            Ok(set) => {
                let key = find(&set);
                *cache = Some((Instant::now(), set));
                key
            }
            Err(error) => {
                log::warn!("could not fetch the token keys from {}: {error}", self.url);
                None
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct OidcPolicy {
    pub issuer: String,

    pub audience: String,

    pub repository: String,

    pub environment: Option<String>,

    pub workflows: Vec<String>,
}

pub struct GithubOidc {
    pub policy: OidcPolicy,
    pub keys: Arc<dyn KeyProvider>,
}

#[derive(Deserialize)]
struct Claims {
    repository: Option<String>,
    environment: Option<String>,
    job_workflow_ref: Option<String>,
    sub: Option<String>,
}

#[async_trait]
impl Authenticator for GithubOidc {
    async fn authenticate(&self, token: &str) -> Result<String, ApiError> {
        let denied = |why: &str| ApiError::Unauthorized(format!("token rejected: {why}"));
        let header = jsonwebtoken::decode_header(token).map_err(|_| denied("not a token"))?;
        if header.alg != Algorithm::RS256 {
            return Err(denied("unexpected algorithm"));
        }
        let kid = header.kid.ok_or_else(|| denied("no key id"))?;
        let key = self
            .keys
            .key(&kid)
            .await
            .ok_or_else(|| denied("unknown signing key"))?;

        let mut validation = Validation::new(Algorithm::RS256);
        validation.set_audience(&[&self.policy.audience]);
        validation.set_issuer(&[&self.policy.issuer]);
        validation.set_required_spec_claims(&["exp", "iss", "aud"]);
        validation.leeway = 30;
        let claims = jsonwebtoken::decode::<Claims>(token, &key, &validation)
            .map_err(|e| denied(&e.to_string()))?
            .claims;

        if claims.repository.as_deref() != Some(self.policy.repository.as_str()) {
            return Err(ApiError::Forbidden(
                "this repository may not publish".into(),
            ));
        }
        if let Some(environment) = &self.policy.environment {
            if claims.environment.as_deref() != Some(environment.as_str()) {
                return Err(ApiError::Forbidden(format!(
                    "the job must run in the `{environment}` environment"
                )));
            }
        }
        if !self.policy.workflows.is_empty() {
            let workflow = claims.job_workflow_ref.as_deref().unwrap_or("");
            if !self
                .policy
                .workflows
                .iter()
                .any(|allowed| workflow.starts_with(allowed.as_str()))
            {
                return Err(ApiError::Forbidden("this workflow may not publish".into()));
            }
        }
        Ok(claims.sub.unwrap_or_else(|| "unknown".into()))
    }
}
