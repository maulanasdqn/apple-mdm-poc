pub mod ca;
pub mod cms;
pub mod issue;
pub mod scep;

use std::sync::Arc;

use config::Config;

use crate::domain::{
    ports::{CertProvider, EnrollmentIdentity},
    DomainError,
};
use crate::infrastructure::InfraError;

use ca::CaMaterial;

#[derive(Clone)]
pub struct CertManager {
    material: Arc<CaMaterial>,
}

impl CertManager {
    pub fn load_or_generate(config: &Config) -> Result<Self, InfraError> {
        let material = CaMaterial::load_or_generate(config)?;
        Ok(Self {
            material: Arc::new(material),
        })
    }
}

#[async_trait::async_trait]
impl CertProvider for CertManager {
    fn sign_profile(&self, profile_plist: &[u8]) -> Result<Vec<u8>, DomainError> {
        let der = cms::sign_profile(
            &self.material.signing_cert,
            &self.material.signing_key,
            profile_plist,
        )
        .map_err(DomainError::from)?;
        Ok(der)
    }

    fn ca_cert_der(&self) -> Result<Vec<u8>, DomainError> {
        let der = self
            .material
            .ca_cert
            .to_der()
            .map_err(InfraError::from)
            .map_err(DomainError::from)?;
        Ok(der)
    }

    fn issue_identity(&self, csr_der: &[u8]) -> Result<Vec<u8>, DomainError> {
        let der = issue::normalize_csr(csr_der).map_err(DomainError::from)?;
        let cert = issue::issue_from_csr(&self.material.ca_cert, &self.material.ca_key, &der, 365)
            .map_err(DomainError::from)?;
        let der = cert
            .to_der()
            .map_err(InfraError::from)
            .map_err(DomainError::from)?;
        Ok(der)
    }

    fn generate_enrollment_identity(
        &self,
        common_name: &str,
    ) -> Result<EnrollmentIdentity, DomainError> {
        let (cert, key) = issue::generate_device_identity(
            &self.material.ca_cert,
            &self.material.ca_key,
            common_name,
            365,
        )
        .map_err(DomainError::from)?;

        let password = uuid::Uuid::new_v4().to_string();
        let pkcs12_der =
            issue::build_pkcs12(&cert, &key, &self.material.ca_cert, &password, common_name)
                .map_err(DomainError::from)?;
        Ok(EnrollmentIdentity {
            pkcs12_der,
            password,
        })
    }
}
