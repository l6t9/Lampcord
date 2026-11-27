use std::io;
use std::result;

pub mod cache;
pub mod compose;
pub mod desktop;
pub mod discord;
pub mod health;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("network: {0}")]
    Http(#[from] reqwest::Error),
    #[error("io: {0}")]
    Io(#[from] io::Error),
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
    #[error("{0}")]
    Parse(#[from] discord::ParseError),
    #[error("{0}")]
    Other(String),
    #[error("download of `{name}` failed: HTTP {status}")]
    Status { name: String, status: u16 },
    #[error("{} recipe(s) do not resolve against this build", .0.len())]
    Unresolved(Vec<(String, String)>),
    #[error("{0}")]
    Invalid(String),
    #[error("{0}")]
    NotFound(String),
}

pub type Result<T, E = Error> = result::Result<T, E>;
