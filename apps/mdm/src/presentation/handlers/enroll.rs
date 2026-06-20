use std::sync::Arc;

use axum::extract::Extension;
use axum::http::header;
use axum::response::{IntoResponse, Response};

use crate::application::enroll_device::EnrollDevice;
use crate::presentation::{error::ApiError, AppState};

use super::content_type;

pub async fn enroll(Extension(state): Extension<Arc<AppState>>) -> Result<Response, ApiError> {
    let use_case = EnrollDevice::new(state.cert.clone(), state.config.clone());
    let signed = use_case.execute()?;

    Ok((
        [
            (header::CONTENT_TYPE, content_type::ENROLL_PROFILE),
            (
                header::CONTENT_DISPOSITION,
                "attachment; filename=\"enroll.mobileconfig\"",
            ),
        ],
        signed,
    )
        .into_response())
}
