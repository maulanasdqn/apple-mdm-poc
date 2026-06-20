use sqlx::FromRow;

use crate::domain::entities::{Device, EnrollmentState};

#[derive(Debug, FromRow)]
pub struct DeviceRow {
    pub udid: String,
    pub push_token: Option<String>,
    pub push_magic: Option<String>,
    pub topic: Option<String>,
    pub unlock_token: Option<String>,
    pub enrollment_state: String,
}

impl From<DeviceRow> for Device {
    fn from(r: DeviceRow) -> Self {
        Device {
            udid: r.udid,
            push_token: r.push_token,
            push_magic: r.push_magic,
            topic: r.topic,
            unlock_token: r.unlock_token,
            enrollment_state: EnrollmentState::from_str(&r.enrollment_state),
        }
    }
}
