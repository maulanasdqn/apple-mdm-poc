use serde::Deserialize;
use serde_bytes::ByteBuf;

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "MessageType")]
#[allow(non_snake_case)]
pub enum CheckInMessage {
    Authenticate {
        #[serde(rename = "UDID")]
        UDID: String,
        #[serde(rename = "Topic", default)]
        Topic: Option<String>,
    },
    TokenUpdate {
        #[serde(rename = "UDID")]
        UDID: String,
        #[serde(rename = "Token")]
        Token: ByteBuf,
        #[serde(rename = "PushMagic")]
        PushMagic: String,
        #[serde(rename = "Topic")]
        Topic: String,
        #[serde(rename = "UnlockToken", default)]
        UnlockToken: Option<ByteBuf>,
    },
    CheckOut {
        #[serde(rename = "UDID")]
        UDID: String,
    },
}

impl CheckInMessage {
    pub fn from_plist(bytes: &[u8]) -> Result<Self, plist::Error> {
        plist::from_bytes(bytes)
    }

    pub fn udid(&self) -> &str {
        match self {
            CheckInMessage::Authenticate { UDID, .. }
            | CheckInMessage::TokenUpdate { UDID, .. }
            | CheckInMessage::CheckOut { UDID } => UDID,
        }
    }

    pub fn message_type(&self) -> &'static str {
        match self {
            CheckInMessage::Authenticate { .. } => "Authenticate",
            CheckInMessage::TokenUpdate { .. } => "TokenUpdate",
            CheckInMessage::CheckOut { .. } => "CheckOut",
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[allow(non_snake_case)]
pub struct DeviceResponse {
    #[serde(rename = "UDID")]
    pub UDID: Option<String>,
    #[serde(rename = "Status")]
    pub Status: String,
    #[serde(rename = "CommandUUID", default)]
    pub CommandUUID: Option<String>,
}

impl DeviceResponse {
    pub fn from_plist(bytes: &[u8]) -> Result<Self, plist::Error> {
        plist::from_bytes(bytes)
    }
}
