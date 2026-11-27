#![allow(clippy::print_stdout, clippy::print_stderr)]

use std::sync::Arc;

use env_logger::Builder;
use env_logger::Env;
use requiem_update_server::auth::{Authenticator, GithubJwks, GithubOidc, OidcPolicy, StaticToken};
use requiem_update_server::store;
use requiem_update_server::{AppState, Config, router};
use reqwest::Client;
use std::env;
use std::process;
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::signal;

fn env(name: &str) -> Option<String> {
    env::var(name).ok().filter(|v| !v.trim().is_empty())
}

fn required(name: &str) -> String {
    env(name).unwrap_or_else(|| {
        eprintln!("requiem-update-server: {name} is required");
        process::exit(2);
    })
}

async fn healthcheck() -> ! {
    let listen = env("UPDATE_SERVER_LISTEN").unwrap_or_else(|| "0.0.0.0:8080".into());
    let port = listen.rsplit(':').next().unwrap_or("8080");
    let prefix = env("UPDATE_SERVER_PUBLIC_BASE")
        .map(|base| requiem_update_server::prefix_of(&base).to_string())
        .unwrap_or_default();
    let ok = Client::new()
        .get(format!("http://127.0.0.1:{port}{prefix}/health"))
        .timeout(Duration::from_secs(3))
        .send()
        .await
        .is_ok_and(|r| r.status().is_success());
    process::exit(if ok { 0 } else { 1 });
}

#[tokio::main]
async fn main() {
    if env::args().nth(1).as_deref() == Some("healthcheck") {
        healthcheck().await;
    }
    Builder::from_env(Env::default().default_filter_or("info")).init();

    let public_base = required("UPDATE_SERVER_PUBLIC_BASE");
    let config = Config {
        root: env("UPDATE_SERVER_ROOT")
            .unwrap_or_else(|| "/data".into())
            .into(),
        public_base: public_base.clone(),
        max_release_bytes: env("UPDATE_SERVER_MAX_RELEASE_MB")
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(1024)
            * 1024
            * 1024,
        max_part_bytes: env("UPDATE_SERVER_MAX_PART_MB")
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(64)
            * 1024
            * 1024,
    };
    let auth: Box<dyn Authenticator> = match env("UPDATE_SERVER_STATIC_TOKEN") {
        Some(token) => {
            log::warn!(
                "authenticating with a static token (UPDATE_SERVER_STATIC_TOKEN): for local testing only"
            );
            Box::new(StaticToken(token))
        }
        None => {
            let policy = OidcPolicy {
                issuer: "https://token.actions.githubusercontent.com".into(),
                audience: public_base.trim_end_matches('/').to_string(),
                repository: required("UPDATE_SERVER_GITHUB_REPOSITORY"),
                environment: match env::var("UPDATE_SERVER_ENVIRONMENT") {
                    Ok(v) => Some(v).filter(|v| !v.trim().is_empty()),
                    Err(_) => Some("update-signing".into()),
                },
                workflows: env("UPDATE_SERVER_WORKFLOWS")
                    .map(|v| {
                        v.split(',')
                            .map(|s| s.trim().to_string())
                            .filter(|s| !s.is_empty())
                            .collect()
                    })
                    .unwrap_or_default(),
            };
            log::info!(
                "publishing allowed for {} (environment {:?}, workflows {:?})",
                policy.repository,
                policy.environment,
                policy.workflows
            );
            Box::new(GithubOidc {
                policy,
                keys: Arc::new(GithubJwks::new(
                    "https://token.actions.githubusercontent.com/.well-known/jwks",
                )),
            })
        }
    };

    store::clear_tmp(&config.root).await;
    let mut keys = requiem_update::official_keys();
    if env("UPDATE_SERVER_STATIC_TOKEN").is_some() {
        for item in env("UPDATE_SERVER_TEST_KEYS")
            .unwrap_or_default()
            .split(',')
            .filter_map(|i| i.split_once('='))
        {
            keys.insert(&requiem_update::KeyEntry {
                id: item.0.trim().into(),
                public: item.1.trim().into(),
            })
            .expect("UPDATE_SERVER_TEST_KEYS holds valid keys");
            log::warn!("trusting the test key `{}`", item.0.trim());
        }
    }
    let state = AppState::new(config.clone(), keys, auth);
    let listen = env("UPDATE_SERVER_LISTEN").unwrap_or_else(|| "0.0.0.0:8080".into());
    let listener = TcpListener::bind(&listen).await.unwrap_or_else(|e| {
        eprintln!("requiem-update-server: cannot listen on {listen}: {e}");
        process::exit(1);
    });
    log::info!(
        "serving {} from {} on {listen}",
        public_base,
        config.root.display()
    );
    axum::serve(listener, router(state))
        .with_graceful_shutdown(async {
            let _ = signal::ctrl_c().await;
        })
        .await
        .expect("server ran");
}
