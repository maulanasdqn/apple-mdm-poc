use serde::Serialize;
use uuid::Uuid;

use crate::domain::DomainError;

#[derive(Debug, Clone, Serialize)]
pub struct CommandEnvelope {
    #[serde(rename = "CommandUUID")]
    pub command_uuid: Uuid,
    #[serde(rename = "Command")]
    pub command: Command,
}

#[derive(Debug, Clone, Serialize)]
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

    pub fn to_xml(&self) -> Result<Vec<u8>, DomainError> {
        let mut buf = Vec::new();
        plist::to_writer_xml(&mut buf, self).map_err(|e| DomainError::Plist(e.to_string()))?;
        Ok(buf)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn device_lock_round_trips_with_request_type() {
        let env = CommandEnvelope::new(Command::DeviceLock {
            PIN: Some("123456".into()),
            Message: Some("Locked by MDM".into()),
            PhoneNumber: None,
        });
        let xml = env.to_xml().expect("serialize");
        let text = String::from_utf8(xml).unwrap();
        assert!(text.contains("RequestType"));
        assert!(text.contains("DeviceLock"));
        assert!(text.contains("CommandUUID"));

        assert!(!text.contains("PhoneNumber"));
    }

    #[test]
    fn device_information_serializes_queries() {
        let env = CommandEnvelope::new(Command::DeviceInformation {
            Queries: vec!["DeviceName".into(), "OSVersion".into()],
        });
        let text = String::from_utf8(env.to_xml().unwrap()).unwrap();
        assert!(text.contains("Queries"));
        assert!(text.contains("OSVersion"));
        assert_eq!(env.command.request_type(), "DeviceInformation");
    }
}
