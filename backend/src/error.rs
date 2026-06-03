use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;
use serde_json::json;

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("unauthenticated")]
    Unauthenticated,
    #[error("forbidden")]
    Forbidden,
    #[error("not found")]
    NotFound,
    #[error("validation: {field}: {reason}")]
    Validation { field: String, reason: String },
    #[error("conflict")]
    Conflict,
    #[error("stale write")]
    StaleWrite,
    #[error("rate limited")]
    RateLimited,
    #[error("foreign key violation: {0}")]
    FkViolation(String),
    #[error("cursor filter mismatch")]
    CursorFilterMismatch,
    #[error(transparent)]
    Internal(#[from] anyhow::Error),
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
}

#[derive(Serialize)]
struct ErrorBody<'a> {
    error: ErrorDetail<'a>,
}

#[derive(Serialize)]
struct ErrorDetail<'a> {
    code: &'a str,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    details: Option<serde_json::Value>,
}

impl ApiError {
    fn parts(&self) -> (StatusCode, &'static str, Option<serde_json::Value>) {
        use ApiError::*;
        match self {
            Unauthenticated => (StatusCode::UNAUTHORIZED, "UNAUTHENTICATED", None),
            Forbidden => (StatusCode::FORBIDDEN, "FORBIDDEN", None),
            NotFound => (StatusCode::NOT_FOUND, "NOT_FOUND", None),
            Validation { field, reason } => (
                StatusCode::BAD_REQUEST,
                "VALIDATION_ERROR",
                Some(json!({ "field": field, "reason": reason })),
            ),
            Conflict => (StatusCode::CONFLICT, "CONFLICT", None),
            StaleWrite => (StatusCode::CONFLICT, "STALE_WRITE", None),
            RateLimited => (StatusCode::TOO_MANY_REQUESTS, "RATE_LIMITED", None),
            FkViolation(d) => (
                StatusCode::CONFLICT,
                "FK_VIOLATION",
                Some(json!({ "detail": d })),
            ),
            CursorFilterMismatch => (
                StatusCode::BAD_REQUEST,
                "CURSOR_FILTER_MISMATCH",
                None,
            ),
            Sqlx(_) | Internal(_) => (StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL", None),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        if matches!(self, ApiError::Internal(_) | ApiError::Sqlx(_)) {
            tracing::error!(error = %self, "internal error");
        }
        let (status, code, details) = self.parts();
        let body = ErrorBody {
            error: ErrorDetail {
                code,
                message: self.to_string(),
                details,
            },
        };
        (status, Json(body)).into_response()
    }
}

pub type ApiResult<T> = Result<T, ApiError>;
