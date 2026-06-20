use std::sync::Arc;

use base64::Engine;

use crate::domain::{
    entities::{CheckInMessage, Device, EnrollmentState},
    ports::DeviceRepository,
};

use super::ApplicationError;

pub struct HandleCheckin {
    devices: Arc<dyn DeviceRepository>,
}

impl HandleCheckin {
    pub fn new(devices: Arc<dyn DeviceRepository>) -> Self {
        Self { devices }
    }

    pub async fn execute(&self, msg: CheckInMessage) -> Result<(), ApplicationError> {
        match msg {
            CheckInMessage::Authenticate { UDID, .. } => {
                let device = Device::authenticated(UDID);
                self.devices.upsert(&device).await?;
            }
            CheckInMessage::TokenUpdate {
                UDID,
                Token,
                PushMagic,
                Topic,
                UnlockToken,
            } => {
                let mut device = self
                    .devices
                    .get(&UDID)
                    .await?
                    .unwrap_or_else(|| Device::authenticated(&UDID));
                device.push_token = Some(hex_encode(&Token));
                device.push_magic = Some(PushMagic);
                device.topic = Some(Topic);
                if let Some(unlock) = UnlockToken {
                    device.unlock_token =
                        Some(base64::engine::general_purpose::STANDARD.encode(&unlock));
                }
                device.enrollment_state = EnrollmentState::Enrolled;
                self.devices.upsert(&device).await?;
            }
            CheckInMessage::CheckOut { UDID } => {
                self.devices.mark_checked_out(&UDID).await?;
            }
        }
        Ok(())
    }
}

fn hex_encode(bytes: &[u8]) -> String {
    use std::fmt::Write;
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        let _ = write!(s, "{:02x}", b);
    }
    s
}
