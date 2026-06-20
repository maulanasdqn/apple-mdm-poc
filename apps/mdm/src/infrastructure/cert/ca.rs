use config::Config;
use openssl::pkey::{PKey, Private};
use openssl::x509::X509;

use crate::infrastructure::InfraError;

pub struct CaMaterial {
    pub ca_cert: X509,
    pub ca_key: PKey<Private>,
    pub signing_cert: X509,
    pub signing_key: PKey<Private>,
}

impl CaMaterial {
    pub fn load_or_generate(config: &Config) -> Result<Self, InfraError> {
        let have_ca = std::path::Path::new(&config.ca_cert_path).exists()
            && std::path::Path::new(&config.ca_key_path).exists();
        let have_signing = std::path::Path::new(&config.signing_cert_path).exists()
            && std::path::Path::new(&config.signing_key_path).exists();

        if have_ca && have_signing {
            tracing::info!("loading certificate material from disk");
            let (ca_cert, ca_key) = load_pem(&config.ca_cert_path, &config.ca_key_path)?;
            let (signing_cert, signing_key) =
                load_pem(&config.signing_cert_path, &config.signing_key_path)?;
            Ok(Self {
                ca_cert,
                ca_key,
                signing_cert,
                signing_key,
            })
        } else {
            tracing::warn!(
                "certificate files missing; generating ephemeral in-memory dev CA \
                 (profiles will install as untrusted). Provide real certs via config for production."
            );
            let (ca_cert, ca_key) = generate_self_signed("Rust Apple MDM Dev CA")?;
            let (signing_cert, signing_key) =
                generate_self_signed("Rust Apple MDM Profile Signing")?;
            Ok(Self {
                ca_cert,
                ca_key,
                signing_cert,
                signing_key,
            })
        }
    }
}

fn load_pem(cert_path: &str, key_path: &str) -> Result<(X509, PKey<Private>), InfraError> {
    let cert_pem = std::fs::read(cert_path)?;
    let key_pem = std::fs::read(key_path)?;
    let cert = X509::from_pem(&cert_pem)?;
    let key = PKey::private_key_from_pem(&key_pem)?;
    Ok((cert, key))
}

fn generate_self_signed(common_name: &str) -> Result<(X509, PKey<Private>), InfraError> {
    let certified = rcgen::generate_simple_self_signed(vec![common_name.to_string()])
        .map_err(|e| InfraError::Other(format!("rcgen: {e}")))?;

    let cert_der = certified.cert.der();
    let key_der = certified.key_pair.serialize_der();

    let cert = X509::from_der(cert_der.as_ref())?;
    let key = PKey::private_key_from_der(&key_der)?;
    Ok((cert, key))
}
