use std::sync::Arc;

use axum::extract::Extension;
use axum::http::header;
use axum::response::{IntoResponse, Response};

use crate::application::poll_command::PollCommand;
use crate::domain::entities::DeviceResponse;
use crate::presentation::{error::ApiError, extract::PlistBody, AppState};

use super::content_type;

pub async fn command(
    Extension(state): Extension<Arc<AppState>>,
    PlistBody(body): PlistBody,
) -> Result<Response, ApiError> {
    let response = DeviceResponse::from_plist(&body)
        .map_err(|e| ApiError::BadRequest(format!("invalid response plist: {e}")))?;

    tracing::info!(
        status = %response.Status,
        udid = response.UDID.as_deref().unwrap_or("<none>"),
        "command poll received"
    );

    let use_case = PollCommand::new(state.queue.clone());
    match use_case.execute(&response, &body).await? {
        Some(command_plist) => Ok((
            [(header::CONTENT_TYPE, content_type::MDM_COMMAND)],
            command_plist,
        )
            .into_response()),

        None => Ok(().into_response()),
    }
}
