use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use std::fmt;

#[derive(Debug)]
pub enum ApiError {
    BadRequest(String),

    Unauthorized(String),

    Forbidden(String),

    Conflict(String),
    TooLarge(String),
    NotFound,
    Internal(String),
}

impl ApiError {
    pub fn bad(message: impl Into<String>) -> Self {
        Self::BadRequest(message.into())
    }

    pub fn internal(error: impl fmt::Display) -> Self {
        log::error!("internal error: {error}");
        Self::Internal("internal error".into())
    }

    fn parts(&self) -> (StatusCode, String) {
        match self {
            Self::BadRequest(m) => (StatusCode::BAD_REQUEST, m.clone()),
            Self::Unauthorized(m) => (StatusCode::UNAUTHORIZED, m.clone()),
            Self::Forbidden(m) => (StatusCode::FORBIDDEN, m.clone()),
            Self::Conflict(m) => (StatusCode::CONFLICT, m.clone()),
            Self::TooLarge(m) => (StatusCode::PAYLOAD_TOO_LARGE, m.clone()),
            Self::NotFound => (StatusCode::NOT_FOUND, "not found".into()),
            Self::Internal(m) => (StatusCode::INTERNAL_SERVER_ERROR, m.clone()),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = self.parts();
        let mut response = (
            status,
            axum::Json(serde_json::json!({ "ok": false, "error": message })),
        )
            .into_response();
        if status == StatusCode::UNAUTHORIZED {
            response
                .headers_mut()
                .insert("www-authenticate", "Bearer".parse().expect("static header"));
        }
        response
    }
}
