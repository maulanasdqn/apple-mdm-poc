use plist::{Dictionary, Value};
use uuid::Uuid;

pub struct EnrollmentParams<'a> {
    pub server_url: String,
    pub checkin_url: String,
    pub topic: String,
    pub pkcs12_der: &'a [u8],
    pub pkcs12_password: String,
    pub ca_cert_der: &'a [u8],
}

fn serialize(dict: Dictionary) -> Result<Vec<u8>, String> {
    let mut buf = Vec::new();
    plist::to_writer_xml(&mut buf, &Value::Dictionary(dict)).map_err(|e| e.to_string())?;
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

pub fn build_enrollment_profile(params: &EnrollmentParams) -> Result<Vec<u8>, String> {
    let identity_uuid = Uuid::new_v4().to_string();

    let mut ca_payload = Dictionary::new();
    ca_payload.insert("PayloadContent".into(), Value::Data(params.ca_cert_der.to_vec()));
    ca_payload.insert("PayloadType".into(), "com.apple.security.root".into());
    ca_payload.insert("PayloadIdentifier".into(), "com.rust-apple-mdm.ca".into());
    ca_payload.insert("PayloadUUID".into(), Uuid::new_v4().to_string().into());
    ca_payload.insert("PayloadVersion".into(), 1.into());
    ca_payload.insert("PayloadDisplayName".into(), "MDM Root CA".into());
    ca_payload.insert("PayloadCertificateFileName".into(), "ca.cer".into());

    let mut p12_payload = Dictionary::new();
    p12_payload.insert("PayloadContent".into(), Value::Data(params.pkcs12_der.to_vec()));
    p12_payload.insert("Password".into(), params.pkcs12_password.clone().into());
    p12_payload.insert("PayloadType".into(), "com.apple.security.pkcs12".into());
    p12_payload.insert("PayloadIdentifier".into(), "com.rust-apple-mdm.identity".into());
    p12_payload.insert("PayloadUUID".into(), identity_uuid.clone().into());
    p12_payload.insert("PayloadVersion".into(), 1.into());
    p12_payload.insert("PayloadDisplayName".into(), "MDM Identity Certificate".into());
    p12_payload.insert("PayloadCertificateFileName".into(), "identity.p12".into());

    let mut mdm_payload = Dictionary::new();
    mdm_payload.insert("ServerURL".into(), params.server_url.clone().into());
    mdm_payload.insert("CheckInURL".into(), params.checkin_url.clone().into());
    mdm_payload.insert("Topic".into(), params.topic.clone().into());
    mdm_payload.insert("IdentityCertificateUUID".into(), identity_uuid.into());
    mdm_payload.insert("SignMessage".into(), true.into());
    mdm_payload.insert("CheckOutWhenRemoved".into(), true.into());
    mdm_payload.insert("AccessRights".into(), 8191.into());
    mdm_payload.insert(
        "ServerCapabilities".into(),
        Value::Array(vec!["com.apple.mdm.per-user-connections".into()]),
    );
    mdm_payload.insert("PayloadType".into(), "com.apple.mdm".into());
    mdm_payload.insert("PayloadIdentifier".into(), "com.rust-apple-mdm.mdm".into());
    mdm_payload.insert("PayloadUUID".into(), Uuid::new_v4().to_string().into());
    mdm_payload.insert("PayloadVersion".into(), 1.into());
    mdm_payload.insert("PayloadDisplayName".into(), "MDM Enrollment".into());

    let profile = base_profile(
        "com.rust-apple-mdm.enroll",
        "Rust Apple MDM Enrollment",
        vec![
            Value::Dictionary(ca_payload),
            Value::Dictionary(p12_payload),
            Value::Dictionary(mdm_payload),
        ],
    );
    serialize(profile)
}

#[derive(Debug, Clone)]
pub struct RestrictionParams {
    pub allow_app_installation: bool,
    pub allow_app_removal: bool,
    pub allow_camera: bool,
    pub allow_safari: bool,
    pub allow_screenshot: bool,
    pub allow_erase_content_and_settings: bool,
    pub allow_account_modification: bool,
    pub allow_ui_configuration_profile_installation: bool,
    pub allow_activation_lock: bool,
    pub force_automatic_date_and_time: bool,
}

impl Default for RestrictionParams {
    fn default() -> Self {
        Self {
            allow_app_installation: true,
            allow_app_removal: true,
            allow_camera: true,
            allow_safari: true,
            allow_screenshot: true,
            allow_erase_content_and_settings: true,
            allow_account_modification: true,
            allow_ui_configuration_profile_installation: true,
            allow_activation_lock: true,
            force_automatic_date_and_time: false,
        }
    }
}

impl RestrictionParams {
    pub fn lockdown() -> Self {
        Self {
            allow_app_installation: false,
            allow_app_removal: false,
            allow_camera: true,
            allow_safari: true,
            allow_screenshot: true,
            allow_erase_content_and_settings: false,
            allow_account_modification: false,
            allow_ui_configuration_profile_installation: false,
            allow_activation_lock: true,
            force_automatic_date_and_time: true,
        }
    }
}

pub fn build_restrictions_profile(params: &RestrictionParams) -> Result<Vec<u8>, String> {
    let mut payload = Dictionary::new();
    payload.insert("allowAppInstallation".into(), params.allow_app_installation.into());
    payload.insert("allowAppRemoval".into(), params.allow_app_removal.into());
    payload.insert("allowCamera".into(), params.allow_camera.into());
    payload.insert("allowSafari".into(), params.allow_safari.into());
    payload.insert("allowScreenShot".into(), params.allow_screenshot.into());
    payload.insert("allowEraseContentAndSettings".into(), params.allow_erase_content_and_settings.into());
    payload.insert("allowAccountModification".into(), params.allow_account_modification.into());
    payload.insert(
        "allowUIConfigurationProfileInstallation".into(),
        params.allow_ui_configuration_profile_installation.into(),
    );
    payload.insert("allowActivationLock".into(), params.allow_activation_lock.into());
    payload.insert("forceAutomaticDateAndTime".into(), params.force_automatic_date_and_time.into());
    payload.insert("PayloadType".into(), "com.apple.applicationaccess".into());
    payload.insert("PayloadIdentifier".into(), "com.rust-apple-mdm.restrictions".into());
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
