use std::sync::Arc;

use crate::domain::{
    entities::DeviceResponse,
    ports::{CommandQueue, QueuedCommand},
};

use super::ApplicationError;

pub struct PollCommand {
    queue: Arc<dyn CommandQueue>,
}

impl PollCommand {
    pub fn new(queue: Arc<dyn CommandQueue>) -> Self {
        Self { queue }
    }

    pub async fn execute(
        &self,
        response: &DeviceResponse,
        raw_body: &[u8],
    ) -> Result<Option<Vec<u8>>, ApplicationError> {
        let udid = match &response.UDID {
            Some(u) => u.clone(),

            None => return Ok(None),
        };

        if let Some(cmd_uuid) = response.CommandUUID {
            if response.is_not_now() {
                self.queue.requeue(&cmd_uuid).await?;
            } else if !response.is_idle() {
                self.queue
                    .acknowledge(&cmd_uuid, &udid, &response.Status, Some(raw_body))
                    .await?;
            }
        }

        let next: Option<QueuedCommand> = self.queue.next_for(&udid).await?;
        Ok(next.map(|c| c.payload_plist))
    }
}
