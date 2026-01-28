//! X509 Certificate Wrappers
//!
//! Certificate generation using rcgen and parsing using x509-parser.

use rcgen::{BasicConstraints, CertificateParams, DnType, IsCa, KeyPair};
use x509_parser::prelude::*;

use super::{OwnedBuffer, WrapperCategory, WrapperError, WrapperRegistry, WrapperResult};

// =============================================================================
// Key Generation
// =============================================================================

/// Generate an EC key pair (P-256)
pub fn keypair_generate_ec(curve: &str) -> WrapperResult<OwnedBuffer> {
    let alg = match curve.to_uppercase().as_str() {
        "P-256" | "P256" | "PRIME256V1" | "SECP256R1" => &rcgen::PKCS_ECDSA_P256_SHA256,
        "P-384" | "P384" | "SECP384R1" => &rcgen::PKCS_ECDSA_P384_SHA384,
        _ => {
            return Err(WrapperError::InvalidArg(format!(
                "Unsupported curve: {}",
                curve
            )))
        }
    };

    let keypair = KeyPair::generate_for(alg)
        .map_err(|e| WrapperError::X509Error(format!("Key generation failed: {}", e)))?;

    Ok(OwnedBuffer::from_string(keypair.serialize_pem()))
}

/// Generate an Ed25519 key pair
pub fn keypair_generate_ed25519() -> WrapperResult<OwnedBuffer> {
    let keypair = KeyPair::generate_for(&rcgen::PKCS_ED25519)
        .map_err(|e| WrapperError::X509Error(format!("Key generation failed: {}", e)))?;

    Ok(OwnedBuffer::from_string(keypair.serialize_pem()))
}

// =============================================================================
// Certificate Creation
// =============================================================================

/// Create a self-signed certificate
pub fn create_self_signed(
    subject: &str,
    keypair_pem: &OwnedBuffer,
    days: u32,
) -> WrapperResult<OwnedBuffer> {
    let keypair_str = keypair_pem.as_str()?;
    let keypair = KeyPair::from_pem(keypair_str)
        .map_err(|e| WrapperError::X509Error(format!("Invalid key: {}", e)))?;

    let mut params = CertificateParams::default();
    parse_subject_dn(subject, &mut params)?;

    // Set validity
    let now = ::time::OffsetDateTime::now_utc();
    params.not_before = now;
    params.not_after = now + ::time::Duration::days(days as i64);

    let cert = params
        .self_signed(&keypair)
        .map_err(|e| WrapperError::X509Error(format!("Certificate generation failed: {}", e)))?;

    Ok(OwnedBuffer::from_string(cert.pem()))
}

/// Create a CA certificate (with basicConstraints CA:TRUE)
pub fn create_ca(
    subject: &str,
    keypair_pem: &OwnedBuffer,
    days: u32,
) -> WrapperResult<OwnedBuffer> {
    let keypair_str = keypair_pem.as_str()?;
    let keypair = KeyPair::from_pem(keypair_str)
        .map_err(|e| WrapperError::X509Error(format!("Invalid key: {}", e)))?;

    let mut params = CertificateParams::default();
    parse_subject_dn(subject, &mut params)?;

    // Set as CA
    params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);

    // Set validity
    let now = ::time::OffsetDateTime::now_utc();
    params.not_before = now;
    params.not_after = now + ::time::Duration::days(days as i64);

    let cert = params
        .self_signed(&keypair)
        .map_err(|e| WrapperError::X509Error(format!("CA certificate generation failed: {}", e)))?;

    Ok(OwnedBuffer::from_string(cert.pem()))
}

/// Create a Certificate Signing Request (CSR)
pub fn create_csr(subject: &str, keypair_pem: &OwnedBuffer) -> WrapperResult<OwnedBuffer> {
    let keypair_str = keypair_pem.as_str()?;
    let keypair = KeyPair::from_pem(keypair_str)
        .map_err(|e| WrapperError::X509Error(format!("Invalid key: {}", e)))?;

    let mut params = CertificateParams::default();
    parse_subject_dn(subject, &mut params)?;

    let csr = params
        .serialize_request(&keypair)
        .map_err(|e| WrapperError::X509Error(format!("CSR generation failed: {}", e)))?;

    Ok(OwnedBuffer::from_string(
        csr.pem()
            .map_err(|e| WrapperError::X509Error(e.to_string()))?,
    ))
}

// =============================================================================
// Certificate Parsing
// =============================================================================

/// Certificate information structure
#[derive(Debug, Clone)]
pub struct CertInfo {
    pub subject: String,
    pub issuer: String,
    pub serial: String,
    pub not_before: String,
    pub not_after: String,
    pub is_ca: bool,
}

/// Parse a PEM-encoded certificate
pub fn parse_pem(cert_pem: &OwnedBuffer) -> WrapperResult<CertInfo> {
    let pem_str = cert_pem.as_str()?;
    let (_, pem) = x509_parser::pem::parse_x509_pem(pem_str.as_bytes())
        .map_err(|e| WrapperError::X509Error(format!("Invalid PEM: {:?}", e)))?;

    let (_, cert) = X509Certificate::from_der(&pem.contents)
        .map_err(|e| WrapperError::X509Error(format!("Invalid certificate: {:?}", e)))?;

    Ok(CertInfo {
        subject: cert.subject().to_string(),
        issuer: cert.issuer().to_string(),
        serial: hex::encode(cert.raw_serial()),
        not_before: cert
            .validity()
            .not_before
            .to_rfc2822()
            .unwrap_or_else(|_| "unknown".to_string()),
        not_after: cert
            .validity()
            .not_after
            .to_rfc2822()
            .unwrap_or_else(|_| "unknown".to_string()),
        is_ca: cert.is_ca(),
    })
}

/// Get certificate subject
pub fn get_subject(cert_pem: &OwnedBuffer) -> WrapperResult<OwnedBuffer> {
    let info = parse_pem(cert_pem)?;
    Ok(OwnedBuffer::from_string(info.subject))
}

/// Get certificate issuer
pub fn get_issuer(cert_pem: &OwnedBuffer) -> WrapperResult<OwnedBuffer> {
    let info = parse_pem(cert_pem)?;
    Ok(OwnedBuffer::from_string(info.issuer))
}

/// Get certificate serial number (hex)
pub fn get_serial(cert_pem: &OwnedBuffer) -> WrapperResult<OwnedBuffer> {
    let info = parse_pem(cert_pem)?;
    Ok(OwnedBuffer::from_string(info.serial))
}

/// Get certificate expiry as string
pub fn get_expiry(cert_pem: &OwnedBuffer) -> WrapperResult<OwnedBuffer> {
    let info = parse_pem(cert_pem)?;
    Ok(OwnedBuffer::from_string(info.not_after))
}

/// Check if certificate is expired
pub fn is_expired(cert_pem: &OwnedBuffer) -> WrapperResult<bool> {
    let pem_str = cert_pem.as_str()?;
    let (_, pem) = x509_parser::pem::parse_x509_pem(pem_str.as_bytes())
        .map_err(|e| WrapperError::X509Error(format!("Invalid PEM: {:?}", e)))?;

    let (_, cert) = X509Certificate::from_der(&pem.contents)
        .map_err(|e| WrapperError::X509Error(format!("Invalid certificate: {:?}", e)))?;

    let now = x509_parser::time::ASN1Time::now();
    Ok(cert.validity().not_after < now)
}

/// Check if certificate is a CA
pub fn is_ca(cert_pem: &OwnedBuffer) -> WrapperResult<bool> {
    let info = parse_pem(cert_pem)?;
    Ok(info.is_ca)
}

// =============================================================================
// Helpers
// =============================================================================

/// Parse a subject DN string like "CN=localhost,O=Test" into certificate params
fn parse_subject_dn(subject: &str, params: &mut CertificateParams) -> WrapperResult<()> {
    for part in subject.split(',') {
        let part = part.trim();
        if let Some((key, value)) = part.split_once('=') {
            let key = key.trim().to_uppercase();
            let value = value.trim();

            match key.as_str() {
                "CN" => params.distinguished_name.push(DnType::CommonName, value),
                "O" => params
                    .distinguished_name
                    .push(DnType::OrganizationName, value),
                "OU" => params
                    .distinguished_name
                    .push(DnType::OrganizationalUnitName, value),
                "C" => params.distinguished_name.push(DnType::CountryName, value),
                "ST" => params
                    .distinguished_name
                    .push(DnType::StateOrProvinceName, value),
                "L" => params.distinguished_name.push(DnType::LocalityName, value),
                _ => {} // Ignore unknown attributes
            }
        }
    }
    Ok(())
}

// =============================================================================
// Registration
// =============================================================================

/// Register all X509 wrappers with the registry
pub fn register(registry: &mut WrapperRegistry) {
    registry.register_wrapper(
        "keypair_generate_ec",
        "Generate an EC key pair (P-256 or P-384)",
        WrapperCategory::X509,
        1,
        &["keypair", "key", "generate", "ec", "ecdsa", "p256", "p384"],
        |args| {
            let curve = if args.is_empty() {
                "P-256"
            } else {
                args[0].as_str().unwrap_or("P-256")
            };
            keypair_generate_ec(curve)
        },
    );

    registry.register_wrapper(
        "keypair_generate_ed25519",
        "Generate an Ed25519 key pair",
        WrapperCategory::X509,
        0,
        &["ed25519", "eddsa", "generate ed25519"],
        |_args| keypair_generate_ed25519(),
    );

    registry.register_wrapper(
        "x509_create_self_signed",
        "Create a self-signed certificate",
        WrapperCategory::X509,
        3,
        &[
            "x509",
            "certificate",
            "cert",
            "self-signed",
            "create",
            "generate",
        ],
        |args| {
            if args.len() < 3 {
                return Err(WrapperError::InvalidArg(
                    "Need subject, keypair, and days".to_string(),
                ));
            }
            let subject = args[0].as_str()?;
            let days = u32::from_le_bytes(
                args[2]
                    .as_slice()
                    .get(0..4)
                    .ok_or_else(|| WrapperError::InvalidArg("Invalid days".to_string()))?
                    .try_into()
                    .unwrap(),
            );
            create_self_signed(subject, &args[1], days)
        },
    );

    registry.register_wrapper(
        "x509_create_ca",
        "Create a CA certificate",
        WrapperCategory::X509,
        3,
        &["ca", "certificate authority", "root cert"],
        |args| {
            if args.len() < 3 {
                return Err(WrapperError::InvalidArg(
                    "Need subject, keypair, and days".to_string(),
                ));
            }
            let subject = args[0].as_str()?;
            let days = u32::from_le_bytes(
                args[2]
                    .as_slice()
                    .get(0..4)
                    .ok_or_else(|| WrapperError::InvalidArg("Invalid days".to_string()))?
                    .try_into()
                    .unwrap(),
            );
            create_ca(subject, &args[1], days)
        },
    );

    registry.register_wrapper(
        "x509_create_csr",
        "Create a Certificate Signing Request",
        WrapperCategory::X509,
        2,
        &["csr", "certificate request", "signing request"],
        |args| {
            if args.len() < 2 {
                return Err(WrapperError::InvalidArg(
                    "Need subject and keypair".to_string(),
                ));
            }
            let subject = args[0].as_str()?;
            create_csr(subject, &args[1])
        },
    );

    registry.register_wrapper(
        "x509_get_subject",
        "Get certificate subject",
        WrapperCategory::X509,
        1,
        &["subject", "cert subject", "get subject"],
        |args| {
            if args.is_empty() {
                return Err(WrapperError::InvalidArg(
                    "No certificate provided".to_string(),
                ));
            }
            get_subject(&args[0])
        },
    );

    registry.register_wrapper(
        "x509_get_issuer",
        "Get certificate issuer",
        WrapperCategory::X509,
        1,
        &["issuer", "cert issuer", "get issuer"],
        |args| {
            if args.is_empty() {
                return Err(WrapperError::InvalidArg(
                    "No certificate provided".to_string(),
                ));
            }
            get_issuer(&args[0])
        },
    );

    registry.register_wrapper(
        "x509_get_expiry",
        "Get certificate expiry",
        WrapperCategory::X509,
        1,
        &["expiry", "expiration", "not after", "valid until"],
        |args| {
            if args.is_empty() {
                return Err(WrapperError::InvalidArg(
                    "No certificate provided".to_string(),
                ));
            }
            get_expiry(&args[0])
        },
    );

    registry.register_wrapper(
        "x509_is_expired",
        "Check if certificate is expired",
        WrapperCategory::X509,
        1,
        &["is expired", "expired", "check expiry"],
        |args| {
            if args.is_empty() {
                return Err(WrapperError::InvalidArg(
                    "No certificate provided".to_string(),
                ));
            }
            let expired = is_expired(&args[0])?;
            Ok(OwnedBuffer::from_slice(&[if expired { 1 } else { 0 }]))
        },
    );

    registry.register_wrapper(
        "x509_is_ca",
        "Check if certificate is a CA",
        WrapperCategory::X509,
        1,
        &["is ca", "is certificate authority"],
        |args| {
            if args.is_empty() {
                return Err(WrapperError::InvalidArg(
                    "No certificate provided".to_string(),
                ));
            }
            let ca = is_ca(&args[0])?;
            Ok(OwnedBuffer::from_slice(&[if ca { 1 } else { 0 }]))
        },
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keypair_generate_ec() {
        let keypair = keypair_generate_ec("P-256").unwrap();
        let pem = keypair.as_str().unwrap();
        assert!(pem.contains("BEGIN PRIVATE KEY"));
    }

    #[test]
    fn test_keypair_generate_ed25519() {
        let keypair = keypair_generate_ed25519().unwrap();
        let pem = keypair.as_str().unwrap();
        assert!(pem.contains("BEGIN PRIVATE KEY"));
    }

    #[test]
    fn test_create_self_signed() {
        let keypair = keypair_generate_ec("P-256").unwrap();
        let cert = create_self_signed("CN=localhost,O=Test", &keypair, 365).unwrap();

        let pem = cert.as_str().unwrap();
        assert!(pem.contains("BEGIN CERTIFICATE"));
    }

    #[test]
    fn test_parse_certificate() {
        let keypair = keypair_generate_ec("P-256").unwrap();
        let cert = create_self_signed("CN=test.example.com,O=Test Org", &keypair, 365).unwrap();

        let info = parse_pem(&cert).unwrap();
        assert!(info.subject.contains("test.example.com"));
        assert!(!info.is_ca);
    }

    #[test]
    fn test_create_ca() {
        let keypair = keypair_generate_ec("P-256").unwrap();
        let cert = create_ca("CN=My CA,O=Test", &keypair, 3650).unwrap();

        let info = parse_pem(&cert).unwrap();
        assert!(info.is_ca);
    }

    #[test]
    fn test_create_csr() {
        let keypair = keypair_generate_ec("P-256").unwrap();
        let csr = create_csr("CN=server.example.com", &keypair).unwrap();

        let pem = csr.as_str().unwrap();
        assert!(pem.contains("BEGIN CERTIFICATE REQUEST"));
    }

    #[test]
    fn test_is_expired() {
        let keypair = keypair_generate_ec("P-256").unwrap();
        let cert = create_self_signed("CN=test", &keypair, 365).unwrap();

        // Newly created cert should not be expired
        assert!(!is_expired(&cert).unwrap());
    }

    #[test]
    fn test_get_subject() {
        let keypair = keypair_generate_ec("P-256").unwrap();
        let cert = create_self_signed("CN=myhost.local", &keypair, 365).unwrap();

        let subject = get_subject(&cert).unwrap();
        assert!(subject.as_str().unwrap().contains("myhost.local"));
    }
}
