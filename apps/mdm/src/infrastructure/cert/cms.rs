use openssl::cms::{CMSOptions, CmsContentInfo};
use openssl::pkey::{PKey, Private};
use openssl::x509::X509;

use crate::infrastructure::InfraError;

pub fn sign_profile(
    signing_cert: &X509,
    signing_key: &PKey<Private>,
    data: &[u8],
) -> Result<Vec<u8>, InfraError> {
    let flags = CMSOptions::BINARY;
    let cms = CmsContentInfo::sign(
        Some(signing_cert),
        Some(signing_key),
        None,
        Some(data),
        flags,
    )?;
    let der = cms.to_der()?;
    Ok(der)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dev_cert() -> (X509, PKey<Private>) {
        let certified = rcgen::generate_simple_self_signed(vec!["cms-test".to_string()]).unwrap();
        let cert = X509::from_der(certified.cert.der().as_ref()).unwrap();
        let key = PKey::private_key_from_der(&certified.key_pair.serialize_der()).unwrap();
        (cert, key)
    }

    #[test]
    fn signs_to_nonempty_der() {
        let (cert, key) = dev_cert();
        let signed = sign_profile(&cert, &key, b"<plist></plist>").unwrap();
        assert!(!signed.is_empty());

        assert!(CmsContentInfo::from_der(&signed).is_ok());
    }
}
