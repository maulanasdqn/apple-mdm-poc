use std::sync::Arc;

use axum::extract::{Extension, Path};
use axum::Json;

use crate::application::enqueue_command::EnqueueCommand;
use crate::presentation::dto::{DeviceDto, EnqueueCommandDto, EnqueueResponse};
use crate::presentation::{error::ApiError, AppState};

pub async fn list_devices(
    Extension(state): Extension<Arc<AppState>>,
) -> Result<Json<Vec<DeviceDto>>, ApiError> {
    let devices = state.devices.list().await?;
    Ok(Json(devices.into_iter().map(DeviceDto::from).collect()))
}

pub async fn enqueue_command(
    Extension(state): Extension<Arc<AppState>>,
    Path(udid): Path<String>,
    Json(dto): Json<EnqueueCommandDto>,
) -> Result<Json<EnqueueResponse>, ApiError> {
    let command = dto.into_command()?;
    let use_case = EnqueueCommand::new(
        state.queue.clone(),
        state.devices.clone(),
        state.push.clone(),
    );
    let uuid = use_case.execute(&udid, command).await?;
    Ok(Json(EnqueueResponse {
        command_uuid: uuid.to_string(),
        udid,
    }))
}
