use crate::domain::{entities::Device, ports::PushNotifier, DomainError};

#[derive(Clone, Default)]
pub struct NoopPush;

#[async_trait::async_trait]
impl PushNotifier for NoopPush {
    async fn send_wakeup(&self, device: &Device) -> Result<(), DomainError> {
        tracing::info!(
            udid = %device.udid,
            token = device.push_token.as_deref().unwrap_or("<none>"),
            topic = device.topic.as_deref().unwrap_or("<none>"),
            "NoopPush: would send APNs wake-up (set APNS_MODE=real to deliver)"
        );
        Ok(())
    }
}
