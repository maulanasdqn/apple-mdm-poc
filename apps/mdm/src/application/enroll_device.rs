use std::sync::Arc;

use config::Config;

use crate::domain::{
    entities::profile::{build_enrollment_profile, EnrollmentParams},
    ports::CertProvider,
};

use super::ApplicationError;

pub struct EnrollDevice {
    cert: Arc<dyn CertProvider>,
    config: Arc<Config>,
}

impl EnrollDevice {
    pub fn new(cert: Arc<dyn CertProvider>, config: Arc<Config>) -> Self {
        Self { cert, config }
    }

    pub fn execute(&self) -> Result<Vec<u8>, ApplicationError> {
        let common_name = format!("mdm-enroll-{}", uuid::Uuid::new_v4());
        let identity = self.cert.generate_enrollment_identity(&common_name)?;

        let params = EnrollmentParams {
            server_url: self.config.server_url(),
            checkin_url: self.config.checkin_url(),
            topic: self.config.mdm_topic.clone(),
            pkcs12_der: identity.pkcs12_der,
            pkcs12_password: identity.password,
            ca_cert_der: self.cert.ca_cert_der()?,
        };
        let unsigned = build_enrollment_profile(&params)?;
        let signed = self.cert.sign_profile(&unsigned)?;
        Ok(signed)
    }
}
