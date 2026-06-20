use std::sync::Arc;

use axum::extract::Extension;
use axum::http::StatusCode;

use crate::application::handle_checkin::HandleCheckin;
use crate::domain::entities::CheckInMessage;
use crate::presentation::{error::ApiError, extract::PlistBody, AppState};

pub async fn checkin(
    Extension(state): Extension<Arc<AppState>>,
    PlistBody(body): PlistBody,
) -> Result<StatusCode, ApiError> {
    let msg = CheckInMessage::from_plist(&body)
        .map_err(|e| ApiError::BadRequest(format!("invalid check-in plist: {e}")))?;

    tracing::info!(
        message_type = msg.message_type(),
        udid = msg.udid(),
        "check-in received"
    );

    let use_case = HandleCheckin::new(state.devices.clone());
    use_case.execute(msg).await?;

    Ok(StatusCode::OK)
}
