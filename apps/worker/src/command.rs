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
    EnableLostMode {
        #[serde(rename = "Message", skip_serializing_if = "Option::is_none")]
        Message: Option<String>,
        #[serde(rename = "PhoneNumber", skip_serializing_if = "Option::is_none")]
        PhoneNumber: Option<String>,
        #[serde(rename = "Footnote", skip_serializing_if = "Option::is_none")]
        Footnote: Option<String>,
    },
    DisableLostMode {},
    PlayLostModeSound {},
    DeviceLocation {},
}

impl Command {
    pub fn request_type(&self) -> &'static str {
        match self {
            Command::DeviceLock { .. } => "DeviceLock",
            Command::EraseDevice { .. } => "EraseDevice",
            Command::DeviceInformation { .. } => "DeviceInformation",
            Command::InstallProfile { .. } => "InstallProfile",
            Command::RemoveProfile { .. } => "RemoveProfile",
            Command::EnableLostMode { .. } => "EnableLostMode",
            Command::DisableLostMode {} => "DisableLostMode",
            Command::PlayLostModeSound {} => "PlayLostModeSound",
            Command::DeviceLocation {} => "DeviceLocation",
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
        allow_app_removal: Option<bool>,
        allow_camera: Option<bool>,
        allow_safari: Option<bool>,
        allow_screenshot: Option<bool>,
        allow_erase_content_and_settings: Option<bool>,
        allow_account_modification: Option<bool>,
        allow_ui_configuration_profile_installation: Option<bool>,
        allow_activation_lock: Option<bool>,
        force_automatic_date_and_time: Option<bool>,
    },
    Lockdown {},
    RemoveProfile {
        identifier: String,
    },
    EnableLostMode {
        message: Option<String>,
        phone_number: Option<String>,
        footnote: Option<String>,
    },
    DisableLostMode {},
    PlayLostModeSound {},
    DeviceLocation {},
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
                allow_app_removal,
                allow_camera,
                allow_safari,
                allow_screenshot,
                allow_erase_content_and_settings,
                allow_account_modification,
                allow_ui_configuration_profile_installation,
                allow_activation_lock,
                force_automatic_date_and_time,
            } => {
                let mut p = RestrictionParams::default();
                if let Some(v) = allow_app_installation { p.allow_app_installation = v; }
                if let Some(v) = allow_app_removal { p.allow_app_removal = v; }
                if let Some(v) = allow_camera { p.allow_camera = v; }
                if let Some(v) = allow_safari { p.allow_safari = v; }
                if let Some(v) = allow_screenshot { p.allow_screenshot = v; }
                if let Some(v) = allow_erase_content_and_settings { p.allow_erase_content_and_settings = v; }
                if let Some(v) = allow_account_modification { p.allow_account_modification = v; }
                if let Some(v) = allow_ui_configuration_profile_installation { p.allow_ui_configuration_profile_installation = v; }
                if let Some(v) = allow_activation_lock { p.allow_activation_lock = v; }
                if let Some(v) = force_automatic_date_and_time { p.force_automatic_date_and_time = v; }
                let payload = build_restrictions_profile(&p)?;
                Command::InstallProfile { Payload: payload }
            }
            AdminCommand::Lockdown {} => {
                let payload = build_restrictions_profile(&RestrictionParams::lockdown())?;
                Command::InstallProfile { Payload: payload }
            }
            AdminCommand::RemoveProfile { identifier } => Command::RemoveProfile {
                Identifier: identifier,
            },
            AdminCommand::EnableLostMode {
                message,
                phone_number,
                footnote,
            } => Command::EnableLostMode {
                Message: message,
                PhoneNumber: phone_number,
                Footnote: footnote,
            },
            AdminCommand::DisableLostMode {} => Command::DisableLostMode {},
            AdminCommand::PlayLostModeSound {} => Command::PlayLostModeSound {},
            AdminCommand::DeviceLocation {} => Command::DeviceLocation {},
        })
    }
}
