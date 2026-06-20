use plist::{Dictionary, Value};
use uuid::Uuid;

use crate::domain::DomainError;

pub struct EnrollmentParams {
    pub server_url: String,
    pub checkin_url: String,
    pub topic: String,

    pub pkcs12_der: Vec<u8>,

    pub pkcs12_password: String,
}

#[derive(Debug, Clone)]
pub struct RestrictionParams {
    pub allow_app_installation: bool,
    pub allow_camera: bool,
    pub allow_safari: bool,
    pub allow_screenshot: bool,
    pub force_passcode: bool,
}

impl Default for RestrictionParams {
    fn default() -> Self {
        Self {
            allow_app_installation: true,
            allow_camera: true,
            allow_safari: true,
            allow_screenshot: true,
            force_passcode: false,
        }
    }
}

fn serialize(dict: Dictionary) -> Result<Vec<u8>, DomainError> {
    let mut buf = Vec::new();
    plist::to_writer_xml(&mut buf, &Value::Dictionary(dict))
        .map_err(|e| DomainError::Plist(e.to_string()))?;
    Ok(buf)
}

fn base_profile(identifier: &str, display_name: &str, payloads: Vec<Value>) -> Dictionary {
    let mut dict = Dictionary::new();
    dict.insert("PayloadContent".into(), Value::Array(payloads));
    dict.insert("PayloadDisplayName".into(), display_name.into());
    dict.insert("PayloadIdentifier".into(), identifier.into());
    dict.insert("PayloadType".into(), "Configuration".into());
    dict.insert("PayloadUUID".into(), Uuid::new_v4().to_string().into());
    dict.insert("PayloadVersion".into(), 1.into());
    dict
}

pub fn build_enrollment_profile(params: &EnrollmentParams) -> Result<Vec<u8>, DomainError> {
    let identity_uuid = Uuid::new_v4().to_string();

    let mut p12_payload = Dictionary::new();
    p12_payload.insert(
        "PayloadContent".into(),
        Value::Data(params.pkcs12_der.clone()),
    );
    p12_payload.insert("Password".into(), params.pkcs12_password.clone().into());
    p12_payload.insert("PayloadType".into(), "com.apple.security.pkcs12".into());
    p12_payload.insert(
        "PayloadIdentifier".into(),
        "com.rust-apple-mdm.identity".into(),
    );
    p12_payload.insert("PayloadUUID".into(), identity_uuid.clone().into());
    p12_payload.insert("PayloadVersion".into(), 1.into());
    p12_payload.insert(
        "PayloadDisplayName".into(),
        "MDM Identity Certificate".into(),
    );
    p12_payload.insert("PayloadCertificateFileName".into(), "identity.p12".into());

    let mut mdm_payload = Dictionary::new();
    mdm_payload.insert("ServerURL".into(), params.server_url.clone().into());
    mdm_payload.insert("CheckInURL".into(), params.checkin_url.clone().into());
    mdm_payload.insert("Topic".into(), params.topic.clone().into());
    mdm_payload.insert("IdentityCertificateUUID".into(), identity_uuid.into());
    mdm_payload.insert("SignMessage".into(), true.into());
    mdm_payload.insert("CheckOutWhenRemoved".into(), true.into());
    mdm_payload.insert("AccessRights".into(), 8191.into());
    mdm_payload.insert("PayloadType".into(), "com.apple.mdm".into());
    mdm_payload.insert("PayloadIdentifier".into(), "com.rust-apple-mdm.mdm".into());
    mdm_payload.insert("PayloadUUID".into(), Uuid::new_v4().to_string().into());
    mdm_payload.insert("PayloadVersion".into(), 1.into());
    mdm_payload.insert("PayloadDisplayName".into(), "MDM Enrollment".into());

    let profile = base_profile(
        "com.rust-apple-mdm.enroll",
        "Rust Apple MDM Enrollment",
        vec![
            Value::Dictionary(p12_payload),
            Value::Dictionary(mdm_payload),
        ],
    );
    serialize(profile)
}

pub fn build_restrictions_profile(params: &RestrictionParams) -> Result<Vec<u8>, DomainError> {
    let mut content = Dictionary::new();
    content.insert(
        "allowAppInstallation".into(),
        params.allow_app_installation.into(),
    );
    content.insert("allowCamera".into(), params.allow_camera.into());
    content.insert("allowSafari".into(), params.allow_safari.into());
    content.insert("allowScreenShot".into(), params.allow_screenshot.into());
    content.insert("forceITunesStorePasswordEntry".into(), false.into());

    let mut payload = Dictionary::new();

    for (k, v) in content.into_iter() {
        payload.insert(k, v);
    }
    payload.insert("PayloadType".into(), "com.apple.applicationaccess".into());
    payload.insert(
        "PayloadIdentifier".into(),
        "com.rust-apple-mdm.restrictions".into(),
    );
    payload.insert("PayloadUUID".into(), Uuid::new_v4().to_string().into());
    payload.insert("PayloadVersion".into(), 1.into());
    payload.insert("PayloadDisplayName".into(), "Restrictions".into());

    let profile = base_profile(
        "com.rust-apple-mdm.restrictions.profile",
        "Device Lockdown Restrictions",
        vec![Value::Dictionary(payload)],
    );
    serialize(profile)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enrollment_profile_contains_mdm_and_identity() {
        let bytes = build_enrollment_profile(&EnrollmentParams {
            server_url: "https://h/server".into(),
            checkin_url: "https://h/checkin".into(),
            topic: "com.apple.mgmt.External.x".into(),
            pkcs12_der: vec![1, 2, 3, 4],
            pkcs12_password: "secret".into(),
        })
        .unwrap();
        let text = String::from_utf8(bytes).unwrap();
        assert!(text.contains("com.apple.mdm"));
        assert!(text.contains("com.apple.security.pkcs12"));
        assert!(text.contains("IdentityCertificateUUID"));
        assert!(text.contains("CheckInURL"));
    }

    #[test]
    fn restrictions_profile_has_applicationaccess() {
        let mut p = RestrictionParams::default();
        p.allow_camera = false;
        let text = String::from_utf8(build_restrictions_profile(&p).unwrap()).unwrap();
        assert!(text.contains("com.apple.applicationaccess"));
        assert!(text.contains("allowCamera"));
    }
}
