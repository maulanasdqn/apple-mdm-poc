use crate::domain::{entities::Device, ports::PushNotifier, DomainError};
use crate::infrastructure::InfraError;

pub struct RealApnsPush {
    #[allow(dead_code)]
    cert_path: String,
    #[allow(dead_code)]
    cert_password: String,
}

impl RealApnsPush {
    pub fn new(cert_path: String, cert_password: String) -> Self {
        Self {
            cert_path,
            cert_password,
        }
    }
}

#[async_trait::async_trait]
impl PushNotifier for RealApnsPush {
    #[cfg(feature = "apns-real")]
    async fn send_wakeup(&self, device: &Device) -> Result<(), DomainError> {
        use a2::{Client, ClientConfig, DefaultNotificationBuilder, NotificationBuilder};

        let token = device
            .push_token
            .as_deref()
            .ok_or_else(|| InfraError::Push("device has no push token".into()))?;
        let magic = device
            .push_magic
            .as_deref()
            .ok_or_else(|| InfraError::Push("device has no push magic".into()))?;
        let topic = device
            .topic
            .as_deref()
            .ok_or_else(|| InfraError::Push("device has no topic".into()))?;

        let mut file =
            std::fs::File::open(&self.cert_path).map_err(|e| InfraError::Push(e.to_string()))?;
        let client = Client::certificate(&mut file, &self.cert_password, ClientConfig::default())
            .map_err(|e| InfraError::Push(format!("apns client: {e}")))?;

        let mut payload = DefaultNotificationBuilder::new().build(token, Default::default());
        payload.options.apns_topic = Some(topic);
        payload
            .add_custom_data("mdm", &magic)
            .map_err(|e| InfraError::Push(format!("payload: {e}")))?;

        client
            .send(payload)
            .await
            .map_err(|e| InfraError::Push(format!("apns send: {e}")))?;

        tracing::info!(udid = %device.udid, "APNs wake-up sent");
        Ok(())
    }

    #[cfg(not(feature = "apns-real"))]
    async fn send_wakeup(&self, _device: &Device) -> Result<(), DomainError> {
        Err(InfraError::Push(
            "real APNs push requires building with `--features apns-real` and a vendor push cert"
                .into(),
        )
        .into())
    }
}
