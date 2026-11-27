mod helper;
mod page;

use base64::engine::general_purpose;
use requiem_core::ui;
use std::env;
use std::fmt;
use std::io::Write;
use std::process::{Command, Stdio};
use tokio::task;

use base64::Engine;
use requiem_core::auth::{CaptchaChallenge, CaptchaSolution};
use serde::{Deserialize, Serialize};

pub const FLAG: &str = "--captcha";

pub const EXIT_UNAVAILABLE: i32 = 2;

const FONTS: [(u16, &str); 4] = [
    (400, "font/ggsans"),
    (500, "font/ggsansmedium"),
    (600, "font/ggsanssemibold"),
    (700, "font/ggsansbold"),
];

#[derive(Serialize, Deserialize)]
struct Request {
    challenge: CaptchaChallenge,

    origin: String,

    language: String,

    graphic: Option<String>,
    fonts: Vec<FontFace>,
}

#[derive(Serialize, Deserialize)]
struct FontFace {
    weight: u16,

    woff2: String,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "result", rename_all = "snake_case")]
enum Response {
    Solved { key: String },

    Cancelled,

    Failed { code: String },

    Unavailable { reason: String },
}

#[derive(Debug)]
pub enum CaptchaError {
    Cancelled,
    Failed(String),

    Unavailable(String),
}

impl CaptchaError {
    pub fn message(&self) -> String {
        match self {
            Self::Cancelled => requiem_i18n::t!("captcha-cancelled"),
            Self::Failed(_) => requiem_i18n::t!("captcha-error"),
            Self::Unavailable(reason) => {
                requiem_i18n::t!("captcha-unavailable", reason = reason.as_str())
            }
        }
    }
}

pub async fn solve(
    challenge: CaptchaChallenge,
    origin: String,
) -> Result<CaptchaSolution, CaptchaError> {
    let request = Request {
        origin,
        language: requiem_i18n::locale()
            .split('-')
            .next()
            .unwrap_or("en")
            .to_string(),
        graphic: ui::asset_file("image/captcha_graphic").map(|svg| encode(&svg)),
        fonts: FONTS
            .iter()
            .filter_map(|(weight, name)| {
                Some(FontFace {
                    weight: *weight,
                    woff2: encode(&ui::asset_file(name)?),
                })
            })
            .collect(),
        challenge: challenge.clone(),
    };
    let response = task::spawn_blocking(move || run_helper(&request))
        .await
        .map_err(|err| CaptchaError::Unavailable(err.to_string()))??;
    match response {
        Response::Solved { key } => Ok(CaptchaSolution::new(&challenge, key)),
        Response::Cancelled => Err(CaptchaError::Cancelled),
        Response::Failed { code } => {
            log::warn!("the captcha widget failed: {code}");
            Err(CaptchaError::Failed(code))
        }
        Response::Unavailable { reason } => Err(CaptchaError::Unavailable(reason)),
    }
}

fn encode(bytes: &[u8]) -> String {
    general_purpose::STANDARD.encode(bytes)
}

fn run_helper(request: &Request) -> Result<Response, CaptchaError> {
    let unavailable = |err: &dyn fmt::Display| CaptchaError::Unavailable(err.to_string());
    let exe = env::current_exe().map_err(|err| unavailable(&err))?;
    let mut child = Command::new(exe)
        .arg(FLAG)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .map_err(|err| unavailable(&err))?;

    let mut stdin = child
        .stdin
        .take()
        .ok_or_else(|| CaptchaError::Unavailable("no stdin".into()))?;
    let line = serde_json::to_string(request).map_err(|err| unavailable(&err))?;
    writeln!(stdin, "{line}")
        .and_then(|()| stdin.flush())
        .map_err(|err| unavailable(&err))?;
    let output = child.wait_with_output().map_err(|err| unavailable(&err))?;
    drop(stdin);
    let answer = String::from_utf8_lossy(&output.stdout)
        .lines()
        .rev()
        .find(|line| !line.trim().is_empty())
        .and_then(|line| serde_json::from_str(line).ok());
    answer.ok_or_else(|| {
        CaptchaError::Unavailable(format!(
            "the captcha window ended without an answer ({})",
            output.status
        ))
    })
}

pub fn run() -> i32 {
    helper::run()
}
