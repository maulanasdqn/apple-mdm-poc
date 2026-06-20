use openssl::asn1::Asn1Time;
use openssl::bn::{BigNum, MsbOption};
use openssl::hash::MessageDigest;
use openssl::pkcs12::Pkcs12;
use openssl::pkey::{PKey, Private};
use openssl::rsa::Rsa;
use openssl::x509::extension::{BasicConstraints, ExtendedKeyUsage, KeyUsage};
use openssl::x509::{X509Builder, X509NameBuilder, X509Req, X509};

use crate::infrastructure::InfraError;

pub fn generate_device_identity(
    ca_cert: &X509,
    ca_key: &PKey<Private>,
    common_name: &str,
    validity_days: u32,
) -> Result<(X509, PKey<Private>), InfraError> {
    let key = PKey::from_rsa(Rsa::generate(2048)?)?;

    let mut name = X509NameBuilder::new()?;
    name.append_entry_by_text("CN", common_name)?;
    let name = name.build();

    let mut builder = X509Builder::new()?;
    builder.set_version(2)?;
    let serial = {
        let mut bn = BigNum::new()?;
        bn.rand(159, MsbOption::MAYBE_ZERO, false)?;
        bn.to_asn1_integer()?
    };
    builder.set_serial_number(&serial)?;
    builder.set_subject_name(&name)?;
    builder.set_issuer_name(ca_cert.subject_name())?;
    builder.set_pubkey(&key)?;
    let not_before = Asn1Time::days_from_now(0)?;
    let not_after = Asn1Time::days_from_now(validity_days)?;
    builder.set_not_before(&not_before)?;
    builder.set_not_after(&not_after)?;
    builder.append_extension(BasicConstraints::new().build()?)?;
    builder.append_extension(
        KeyUsage::new()
            .digital_signature()
            .key_encipherment()
            .build()?,
    )?;
    builder.append_extension(ExtendedKeyUsage::new().client_auth().build()?)?;
    builder.sign(ca_key, MessageDigest::sha256())?;

    Ok((builder.build(), key))
}

pub fn build_pkcs12(
    cert: &X509,
    key: &PKey<Private>,
    ca_cert: &X509,
    password: &str,
    friendly_name: &str,
) -> Result<Vec<u8>, InfraError> {
    let mut ca_stack = openssl::stack::Stack::new()?;
    ca_stack.push(ca_cert.to_owned())?;

    let mut builder = Pkcs12::builder();
    builder.name(friendly_name);
    builder.pkey(key);
    builder.cert(cert);
    builder.ca(ca_stack);
    let p12 = builder.build2(password)?;
    Ok(p12.to_der()?)
}

pub fn normalize_csr(bytes: &[u8]) -> Result<Vec<u8>, InfraError> {
    if bytes.starts_with(b"-----BEGIN") {
        Ok(X509Req::from_pem(bytes)?.to_der()?)
    } else {
        Ok(bytes.to_vec())
    }
}

pub fn issue_from_csr(
    ca_cert: &X509,
    ca_key: &PKey<Private>,
    csr_der: &[u8],
    validity_days: u32,
) -> Result<X509, InfraError> {
    let req = X509Req::from_der(csr_der)?;

    let req_pubkey = req.public_key()?;
    if !req.verify(&req_pubkey)? {
        return Err(InfraError::Other(
            "CSR signature verification failed".into(),
        ));
    }

    let mut builder = X509Builder::new()?;
    builder.set_version(2)?;

    let serial = {
        let mut bn = BigNum::new()?;
        bn.rand(159, MsbOption::MAYBE_ZERO, false)?;
        bn.to_asn1_integer()?
    };
    builder.set_serial_number(&serial)?;

    builder.set_subject_name(req.subject_name())?;
    builder.set_issuer_name(ca_cert.subject_name())?;
    builder.set_pubkey(&req_pubkey)?;

    let not_before = Asn1Time::days_from_now(0)?;
    let not_after = Asn1Time::days_from_now(validity_days)?;
    builder.set_not_before(&not_before)?;
    builder.set_not_after(&not_after)?;

    builder.append_extension(BasicConstraints::new().build()?)?;
    builder.append_extension(
        KeyUsage::new()
            .digital_signature()
            .key_encipherment()
            .build()?,
    )?;
    builder.append_extension(ExtendedKeyUsage::new().client_auth().build()?)?;

    builder.sign(ca_key, MessageDigest::sha256())?;
    Ok(builder.build())
}

#[cfg(test)]
mod tests {
    use super::*;
    use openssl::nid::Nid;
    use openssl::x509::X509ReqBuilder;

    fn make_ca() -> (X509, PKey<Private>) {
        let key = PKey::from_rsa(Rsa::generate(2048).unwrap()).unwrap();
        let mut name = X509NameBuilder::new().unwrap();
        name.append_entry_by_text("CN", "Test MDM CA").unwrap();
        let name = name.build();

        let mut b = X509Builder::new().unwrap();
        b.set_version(2).unwrap();
        let serial = {
            let mut bn = BigNum::new().unwrap();
            bn.rand(128, MsbOption::MAYBE_ZERO, false).unwrap();
            bn.to_asn1_integer().unwrap()
        };
        b.set_serial_number(&serial).unwrap();
        b.set_subject_name(&name).unwrap();
        b.set_issuer_name(&name).unwrap();
        b.set_pubkey(&key).unwrap();
        b.set_not_before(&Asn1Time::days_from_now(0).unwrap())
            .unwrap();
        b.set_not_after(&Asn1Time::days_from_now(3650).unwrap())
            .unwrap();
        b.append_extension(BasicConstraints::new().critical().ca().build().unwrap())
            .unwrap();
        b.sign(&key, MessageDigest::sha256()).unwrap();
        (b.build(), key)
    }

    fn make_csr(cn: &str) -> Vec<u8> {
        let key = PKey::from_rsa(Rsa::generate(2048).unwrap()).unwrap();
        let mut name = X509NameBuilder::new().unwrap();
        name.append_entry_by_text("CN", cn).unwrap();
        let name = name.build();
        let mut rb = X509ReqBuilder::new().unwrap();
        rb.set_pubkey(&key).unwrap();
        rb.set_subject_name(&name).unwrap();
        rb.sign(&key, MessageDigest::sha256()).unwrap();
        rb.build().to_der().unwrap()
    }

    #[test]
    fn issues_cert_chained_to_ca() {
        let (ca_cert, ca_key) = make_ca();
        let csr = make_csr("00008030-DEVICE-UDID");

        let issued = issue_from_csr(&ca_cert, &ca_key, &csr, 365).unwrap();

        let ca_pubkey = ca_cert.public_key().unwrap();
        assert!(issued.verify(&ca_pubkey).unwrap());

        let subj_cn = issued
            .subject_name()
            .entries_by_nid(Nid::COMMONNAME)
            .next()
            .unwrap()
            .data()
            .to_string()
            .unwrap();
        assert_eq!(subj_cn, "00008030-DEVICE-UDID");
    }

    #[test]
    fn generates_identity_and_pkcs12() {
        let (ca_cert, ca_key) = make_ca();
        let (cert, key) =
            generate_device_identity(&ca_cert, &ca_key, "00008030-IDENTITY", 365).unwrap();

        assert!(cert.verify(&ca_cert.public_key().unwrap()).unwrap());

        let password = "test-pass-123";
        let der = build_pkcs12(&cert, &key, &ca_cert, password, "MDM Identity").unwrap();
        assert!(!der.is_empty());
        let parsed = Pkcs12::from_der(&der).unwrap().parse2(password).unwrap();
        assert!(parsed.cert.is_some());
        assert!(parsed.pkey.is_some());
    }

    #[test]
    fn rejects_tampered_csr() {
        let (ca_cert, ca_key) = make_ca();
        let mut csr = make_csr("tamper");

        let n = csr.len();
        csr[n - 5] ^= 0xff;
        assert!(issue_from_csr(&ca_cert, &ca_key, &csr, 365).is_err());
    }
}
