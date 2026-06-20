use std::sync::Arc;

use uuid::Uuid;

use crate::domain::{
    entities::{Command, CommandEnvelope},
    ports::{CommandQueue, DeviceRepository, PushNotifier},
};

use super::ApplicationError;

pub struct EnqueueCommand {
    queue: Arc<dyn CommandQueue>,
    devices: Arc<dyn DeviceRepository>,
    push: Arc<dyn PushNotifier>,
}

impl EnqueueCommand {
    pub fn new(
        queue: Arc<dyn CommandQueue>,
        devices: Arc<dyn DeviceRepository>,
        push: Arc<dyn PushNotifier>,
    ) -> Self {
        Self {
            queue,
            devices,
            push,
        }
    }

    pub async fn execute(&self, udid: &str, command: Command) -> Result<Uuid, ApplicationError> {
        let device = self
            .devices
            .get(udid)
            .await?
            .ok_or_else(|| ApplicationError::DeviceNotReachable(udid.to_string()))?;

        let envelope = CommandEnvelope::new(command);
        let uuid = self.queue.enqueue(udid, &envelope).await?;

        if device.is_pushable() {
            if let Err(e) = self.push.send_wakeup(&device).await {
                tracing::warn!(udid, error = %e, "APNs wake-up failed; command remains queued");
            }
        } else {
            tracing::warn!(
                udid,
                "device not pushable yet; command queued for next poll"
            );
        }

        Ok(uuid)
    }
}
