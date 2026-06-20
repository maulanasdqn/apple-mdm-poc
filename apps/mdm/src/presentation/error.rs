use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

use crate::application::ApplicationError;
use crate::domain::DomainError;

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("bad request: {0}")]
    BadRequest(String),

    #[error("not found: {0}")]
    NotFound(String),

    #[error(transparent)]
    Application(#[from] ApplicationError),

    #[error(transparent)]
    Domain(#[from] DomainError),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            ApiError::BadRequest(m) => (StatusCode::BAD_REQUEST, m.clone()),
            ApiError::NotFound(m) => (StatusCode::NOT_FOUND, m.clone()),
            ApiError::Application(ApplicationError::DeviceNotReachable(u)) => {
                (StatusCode::NOT_FOUND, format!("device not reachable: {u}"))
            }
            other => {
                tracing::error!(error = %other, "internal error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal server error".to_string(),
                )
            }
        };
        (status, message).into_response()
    }
}
