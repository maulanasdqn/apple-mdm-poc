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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_authenticate() {
        let xml = br#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
  <key>MessageType</key><string>Authenticate</string>
  <key>UDID</key><string>00008030-0011</string>
  <key>Topic</key><string>com.apple.mgmt.External.test</string>
</dict></plist>"#;
        let msg = CheckInMessage::from_plist(xml).expect("parse");
        assert_eq!(msg.message_type(), "Authenticate");
        assert_eq!(msg.udid(), "00008030-0011");
    }
}
