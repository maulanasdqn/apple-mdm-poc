use serde::Deserialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ApnsMode {
    Noop,

    Real,
}

impl Default for ApnsMode {
    fn default() -> Self {
        ApnsMode::Noop
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    #[serde(default = "default_server_addr")]
    pub server_addr: String,

    #[serde(default = "default_base_url")]
    pub base_url: String,

    #[serde(default = "default_database_url")]
    pub database_url: String,

    #[serde(default = "default_topic")]
    pub mdm_topic: String,

    #[serde(default)]
    pub apns_mode: ApnsMode,

    pub apns_cert_path: Option<String>,

    pub apns_cert_password: Option<String>,

    #[serde(default = "default_ca_cert")]
    pub ca_cert_path: String,
    #[serde(default = "default_ca_key")]
    pub ca_key_path: String,

    #[serde(default = "default_signing_cert")]
    pub signing_cert_path: String,
    #[serde(default = "default_signing_key")]
    pub signing_key_path: String,
}

fn default_server_addr() -> String {
    "0.0.0.0:8080".into()
}
fn default_base_url() -> String {
    "https://mdm.example.com".into()
}
fn default_database_url() -> String {
    "sqlite://mdm.db?mode=rwc".into()
}
fn default_topic() -> String {
    "com.apple.mgmt.External.00000000-0000-0000-0000-000000000000".into()
}
fn default_ca_cert() -> String {
    "certs/ca.pem".into()
}
fn default_ca_key() -> String {
    "certs/ca.key".into()
}
fn default_signing_cert() -> String {
    "certs/signing.pem".into()
}
fn default_signing_key() -> String {
    "certs/signing.key".into()
}

impl Config {
    pub fn from_env() -> Result<Self, figment::Error> {
        use figment::{providers::Env, Figment};
        Figment::new().merge(Env::raw()).extract()
    }

    pub fn server_url(&self) -> String {
        format!("{}/server", self.base_url.trim_end_matches('/'))
    }

    pub fn checkin_url(&self) -> String {
        format!("{}/checkin", self.base_url.trim_end_matches('/'))
    }

    pub fn scep_url(&self) -> String {
        format!("{}/scep", self.base_url.trim_end_matches('/'))
    }
}
