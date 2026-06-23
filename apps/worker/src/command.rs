use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::profile::{build_restrictions_profile, RestrictionParams};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandEnvelope {
    #[serde(rename = "CommandUUID")]
    pub command_uuid: Uuid,
    #[serde(rename = "Command")]
    pub command: Command,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "RequestType")]
#[allow(non_snake_case)]
pub enum Command {
    DeviceLock {
        #[serde(rename = "PIN", skip_serializing_if = "Option::is_none")]
        PIN: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        Message: Option<String>,
        #[serde(rename = "PhoneNumber", skip_serializing_if = "Option::is_none")]
        PhoneNumber: Option<String>,
    },
    EraseDevice {
        #[serde(rename = "PIN", skip_serializing_if = "Option::is_none")]
        PIN: Option<String>,
    },
    DeviceInformation {
        #[serde(rename = "Queries")]
        Queries: Vec<String>,
    },
    InstallProfile {
        #[serde(rename = "Payload", with = "serde_bytes")]
        Payload: Vec<u8>,
    },
    RemoveProfile {
        #[serde(rename = "Identifier")]
        Identifier: String,
    },
}

impl Command {
    pub fn request_type(&self) -> &'static str {
        match self {
            Command::DeviceLock { .. } => "DeviceLock",
            Command::EraseDevice { .. } => "EraseDevice",
            Command::DeviceInformation { .. } => "DeviceInformation",
            Command::InstallProfile { .. } => "InstallProfile",
            Command::RemoveProfile { .. } => "RemoveProfile",
        }
    }
}

impl CommandEnvelope {
    pub fn new(command: Command) -> Self {
        Self {
            command_uuid: Uuid::new_v4(),
            command,
        }
    }

    pub fn to_xml(&self) -> Result<Vec<u8>, String> {
        let mut buf = Vec::new();
        plist::to_writer_xml(&mut buf, self).map_err(|e| e.to_string())?;
        Ok(buf)
    }
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum AdminCommand {
    DeviceLock {
        pin: Option<String>,
        message: Option<String>,
        phone_number: Option<String>,
    },
    EraseDevice {
        pin: Option<String>,
    },
    DeviceInformation {
        queries: Option<Vec<String>>,
    },
    Restrictions {
        allow_app_installation: Option<bool>,
        allow_camera: Option<bool>,
        allow_safari: Option<bool>,
        allow_screenshot: Option<bool>,
    },
    RemoveProfile {
        identifier: String,
    },
}

impl AdminCommand {
    pub fn into_command(self) -> Result<Command, String> {
        Ok(match self {
            AdminCommand::DeviceLock { pin, message, phone_number } => Command::DeviceLock {
                PIN: pin,
                Message: message,
                PhoneNumber: phone_number,
            },
            AdminCommand::EraseDevice { pin } => Command::EraseDevice { PIN: pin },
            AdminCommand::DeviceInformation { queries } => Command::DeviceInformation {
                Queries: queries.unwrap_or_else(|| {
                    vec![
                        "DeviceName".into(),
                        "OSVersion".into(),
                        "ProductName".into(),
                        "SerialNumber".into(),
                    ]
                }),
            },
            AdminCommand::Restrictions {
                allow_app_installation,
                allow_camera,
                allow_safari,
                allow_screenshot,
            } => {
                let payload = build_restrictions_profile(&RestrictionParams {
                    allow_app_installation: allow_app_installation.unwrap_or(true),
                    allow_camera: allow_camera.unwrap_or(true),
                    allow_safari: allow_safari.unwrap_or(true),
                    allow_screenshot: allow_screenshot.unwrap_or(true),
                })?;
                Command::InstallProfile { Payload: payload }
            }
            AdminCommand::RemoveProfile { identifier } => Command::RemoveProfile {
                Identifier: identifier,
            },
        })
    }
}
