use crate::domain::{entities::Device, DomainError};

#[async_trait::async_trait]
pub trait PushNotifier: Send + Sync {
    async fn send_wakeup(&self, device: &Device) -> Result<(), DomainError>;
}
