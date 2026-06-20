use crate::domain::{entities::Device, DomainError};

#[async_trait::async_trait]
pub trait DeviceRepository: Send + Sync {
    async fn upsert(&self, device: &Device) -> Result<(), DomainError>;

    async fn get(&self, udid: &str) -> Result<Option<Device>, DomainError>;

    async fn mark_checked_out(&self, udid: &str) -> Result<(), DomainError>;

    async fn list(&self) -> Result<Vec<Device>, DomainError>;
}
