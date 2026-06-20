use serde::{Deserialize, Serialize};

use crate::domain::{
    entities::{
        profile::{build_restrictions_profile, RestrictionParams},
        Command, Device,
    },
    DomainError,
};

#[derive(Debug, Serialize)]
pub struct DeviceDto {
    pub udid: String,
    pub push_token: Option<String>,
    pub topic: Option<String>,
    pub enrollment_state: String,
    pub pushable: bool,
}

impl From<Device> for DeviceDto {
    fn from(d: Device) -> Self {
        let pushable = d.is_pushable();
        DeviceDto {
            udid: d.udid,
            push_token: d.push_token,
            topic: d.topic,
            enrollment_state: d.enrollment_state.as_str().to_string(),
            pushable,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum EnqueueCommandDto {
    DeviceLock {
        pin: Option<String>,
        message: Option<String>,
        phone_number: Option<String>,
    },
    EraseDevice {
        pin: Option<String>,
    },
    DeviceInformation {
        queries: Vec<String>,
    },

    Restrictions {
        #[serde(default)]
        allow_app_installation: Option<bool>,
        #[serde(default)]
        allow_camera: Option<bool>,
        #[serde(default)]
        allow_safari: Option<bool>,
        #[serde(default)]
        allow_screenshot: Option<bool>,
    },
}

impl EnqueueCommandDto {
    pub fn into_command(self) -> Result<Command, DomainError> {
        let cmd = match self {
            EnqueueCommandDto::DeviceLock {
                pin,
                message,
                phone_number,
            } => Command::DeviceLock {
                PIN: pin,
                Message: message,
                PhoneNumber: phone_number,
            },
            EnqueueCommandDto::EraseDevice { pin } => Command::EraseDevice { PIN: pin },
            EnqueueCommandDto::DeviceInformation { queries } => {
                Command::DeviceInformation { Queries: queries }
            }
            EnqueueCommandDto::Restrictions {
                allow_app_installation,
                allow_camera,
                allow_safari,
                allow_screenshot,
            } => {
                let defaults = RestrictionParams::default();
                let params = RestrictionParams {
                    allow_app_installation: allow_app_installation
                        .unwrap_or(defaults.allow_app_installation),
                    allow_camera: allow_camera.unwrap_or(defaults.allow_camera),
                    allow_safari: allow_safari.unwrap_or(defaults.allow_safari),
                    allow_screenshot: allow_screenshot.unwrap_or(defaults.allow_screenshot),
                    force_passcode: defaults.force_passcode,
                };
                let payload = build_restrictions_profile(&params)?;
                Command::InstallProfile { Payload: payload }
            }
        };
        Ok(cmd)
    }
}

#[derive(Debug, Serialize)]
pub struct EnqueueResponse {
    pub command_uuid: String,
    pub udid: String,
}
