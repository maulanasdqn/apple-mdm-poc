use crate::domain::DomainError;

pub struct EnrollmentIdentity {
    pub pkcs12_der: Vec<u8>,

    pub password: String,
}

#[async_trait::async_trait]
pub trait CertProvider: Send + Sync {
    fn sign_profile(&self, profile_plist: &[u8]) -> Result<Vec<u8>, DomainError>;

    fn ca_cert_der(&self) -> Result<Vec<u8>, DomainError>;

    fn issue_identity(&self, csr_der: &[u8]) -> Result<Vec<u8>, DomainError>;

    fn generate_enrollment_identity(
        &self,
        common_name: &str,
    ) -> Result<EnrollmentIdentity, DomainError>;
}
