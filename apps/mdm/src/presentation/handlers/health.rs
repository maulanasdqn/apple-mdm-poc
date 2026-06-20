use std::sync::Arc;

use axum::extract::Extension;
use axum::http::StatusCode;

use crate::presentation::AppState;

pub async fn healthz() -> &'static str {
    "ok"
}

pub async fn readyz(Extension(state): Extension<Arc<AppState>>) -> StatusCode {
    match sqlx::query("SELECT 1").execute(&state.pool).await {
        Ok(_) => StatusCode::OK,
        Err(e) => {
            tracing::warn!(error = %e, "readyz: database ping failed");
            StatusCode::SERVICE_UNAVAILABLE
        }
    }
}
