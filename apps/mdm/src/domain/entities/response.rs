use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug, Clone, Deserialize)]
#[allow(non_snake_case)]
pub struct DeviceResponse {
    #[serde(rename = "UDID")]
    pub UDID: Option<String>,
    #[serde(rename = "Status")]
    pub Status: String,

    #[serde(rename = "CommandUUID", default)]
    pub CommandUUID: Option<Uuid>,

    #[serde(rename = "ErrorChain", default)]
    pub ErrorChain: Vec<plist::Value>,
}

impl DeviceResponse {
    pub fn from_plist(bytes: &[u8]) -> Result<Self, plist::Error> {
        plist::from_bytes(bytes)
    }

    pub fn is_idle(&self) -> bool {
        self.Status == "Idle"
    }

    pub fn is_not_now(&self) -> bool {
        self.Status == "NotNow"
    }
}
