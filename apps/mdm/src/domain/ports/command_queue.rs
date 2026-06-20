use uuid::Uuid;

use crate::domain::{entities::CommandEnvelope, DomainError};

#[derive(Debug, Clone)]
pub struct QueuedCommand {
    pub command_uuid: Uuid,
    pub udid: String,
    pub request_type: String,

    pub payload_plist: Vec<u8>,
}

#[async_trait::async_trait]
pub trait CommandQueue: Send + Sync {
    async fn enqueue(&self, udid: &str, command: &CommandEnvelope) -> Result<Uuid, DomainError>;

    async fn next_for(&self, udid: &str) -> Result<Option<QueuedCommand>, DomainError>;

    async fn acknowledge(
        &self,
        command_uuid: &Uuid,
        udid: &str,
        status: &str,
        response_plist: Option<&[u8]>,
    ) -> Result<(), DomainError>;

    async fn requeue(&self, command_uuid: &Uuid) -> Result<(), DomainError>;
}
