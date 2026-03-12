use axum::{
    http::{HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Config error: {0}")]
    Config(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Range not satisfiable")]
    RangeNotSatisfiable,

    #[error("Internal error: {0}")]
    Internal(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, msg, error_code) = match &self {
            AppError::Config(m) => (StatusCode::INTERNAL_SERVER_ERROR, m.clone(), None),
            AppError::NotFound(m) => (
                StatusCode::NOT_FOUND,
                m.clone(),
                Some(classify_not_found_error(m)),
            ),
            AppError::BadRequest(m) => (StatusCode::BAD_REQUEST, m.clone(), None),
            AppError::Unauthorized(m) => (StatusCode::UNAUTHORIZED, m.clone(), None),
            AppError::Forbidden(m) => (StatusCode::FORBIDDEN, m.clone(), None),
            AppError::Conflict(m) => (StatusCode::CONFLICT, m.clone(), None),
            AppError::RangeNotSatisfiable => (
                StatusCode::RANGE_NOT_SATISFIABLE,
                "Range not satisfiable".into(),
                None,
            ),
            AppError::Internal(m) => (StatusCode::INTERNAL_SERVER_ERROR, m.clone(), None),
        };
        tracing::error!(error = %self, "Request error");

        let mut response = (status, Json(json!({ "error": msg }))).into_response();
        if let Some(error_code) = error_code {
            response
                .headers_mut()
                .insert("X-Error-Code", HeaderValue::from_static(error_code));
        }
        if let Ok(error_message) = HeaderValue::from_str(&msg) {
            response
                .headers_mut()
                .insert("X-Error-Message", error_message);
        }
        response
    }
}

fn classify_not_found_error(message: &str) -> &'static str {
    let lower = message.to_ascii_lowercase();
    if lower.contains("revision") {
        "RevisionNotFound"
    } else if lower.contains("repo") {
        "RepoNotFound"
    } else {
        "EntryNotFound"
    }
}

/// Convert any anyhow/boxed error to AppError::Internal.
impl From<anyhow::Error> for AppError {
    fn from(e: anyhow::Error) -> Self {
        AppError::Internal(e.to_string())
    }
}
