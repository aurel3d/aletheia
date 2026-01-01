use crate::{
    certificate::verify_certificate_chain, signer::build_signature_input, AletheiaError,
    AletheiaFile, Result,
};
use ed25519_dalek::{Signature, Verifier, VerifyingKey};

/// Result of verifying an Aletheia file
#[derive(Debug, Clone)]
pub struct VerificationResult {
    /// Whether the verification succeeded
    pub valid: bool,
    /// The creator's ID from the certificate
    pub creator_id: String,
    /// The creator's name from the certificate
    pub creator_name: String,
    /// When the file was signed (Unix timestamp)
    pub signed_at: i64,
    /// Description from the header (if any)
    pub description: Option<String>,
}

/// Verify an Aletheia file's authenticity
///
/// This function:
/// 1. Verifies the certificate chain against trusted roots
/// 2. Verifies the signature over the entire file contents
///
/// # Arguments
/// * `file` - The Aletheia file to verify
/// * `trusted_root_keys` - List of trusted root CA public keys
///
/// # Returns
/// * `Ok(VerificationResult)` - If verification succeeds
/// * `Err(AletheiaError)` - If verification fails
pub fn verify(file: &AletheiaFile, trusted_root_keys: &[Vec<u8>]) -> Result<VerificationResult> {
    // Verify the certificate chain
    verify_certificate_chain(&file.certificate_chain, trusted_root_keys)?;

    // Get the creator's certificate (first in chain)
    let creator_cert = &file.certificate_chain[0];

    // Encode header and cert chain as they would have been signed
    let mut header_bytes = Vec::new();
    ciborium::into_writer(&file.header, &mut header_bytes)
        .map_err(|e| AletheiaError::CborEncode(e.to_string()))?;

    let mut cert_chain_bytes = Vec::new();
    ciborium::into_writer(&file.certificate_chain, &mut cert_chain_bytes)
        .map_err(|e| AletheiaError::CborEncode(e.to_string()))?;

    // Build the signature input
    let signature_input = build_signature_input(
        &file.flags,
        &header_bytes,
        &file.payload,
        &cert_chain_bytes,
    );

    // Verify the signature
    let verifying_key = VerifyingKey::try_from(creator_cert.public_key.as_slice())
        .map_err(|e| AletheiaError::InvalidCertificate(format!("Invalid public key: {}", e)))?;

    let signature = Signature::try_from(file.signature.as_slice())
        .map_err(|_| AletheiaError::InvalidSignature)?;

    verifying_key
        .verify(&signature_input, &signature)
        .map_err(|_| AletheiaError::InvalidSignature)?;

    Ok(VerificationResult {
        valid: true,
        creator_id: creator_cert.subject_id.clone(),
        creator_name: creator_cert.subject_name.clone(),
        signed_at: file.header.signed_at,
        description: file.header.description.clone(),
    })
}

/// Quick check if an Aletheia file has valid structure (without full verification)
pub fn validate_structure(file: &AletheiaFile) -> Result<()> {
    // Check version
    if file.version_major != 1 {
        return Err(AletheiaError::UnsupportedVersion {
            major: file.version_major,
            minor: file.version_minor,
        });
    }

    // Check certificate chain is not empty
    if file.certificate_chain.is_empty() {
        return Err(AletheiaError::CertificateChainInvalid(
            "Empty certificate chain".into(),
        ));
    }

    // Check signature length
    if file.signature.len() != 64 {
        return Err(AletheiaError::InvalidSignature);
    }

    // Verify creator ID in header matches certificate
    let creator_cert = &file.certificate_chain[0];
    if file.header.creator_id != creator_cert.subject_id {
        return Err(AletheiaError::InvalidHeader(format!(
            "Creator ID mismatch: header says '{}', certificate says '{}'",
            file.header.creator_id, creator_cert.subject_id
        )));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ca::{CertificateAuthority, SigningKeyPair},
        signer::Signer,
        Header,
    };

    fn create_test_file() -> (AletheiaFile, Vec<Vec<u8>>) {
        let timestamp = 1704067200;
        let ca = CertificateAuthority::new_root_with_timestamp(
            "root@example.com",
            "Root CA",
            timestamp,
        );
        let user_keys = SigningKeyPair::generate();

        let user_cert = ca
            .issue_certificate_with_timestamp(
                "alice@example.com",
                "Alice",
                &user_keys.public_key(),
                false,
                timestamp,
            )
            .unwrap();

        let chain = vec![user_cert, ca.certificate.clone()];
        let signer = Signer::new(user_keys, chain).unwrap();

        let payload = b"Test content";
        let header = Header::new_with_timestamp("alice@example.com", timestamp)
            .with_description("Test file");

        let file = signer.sign(payload, header).unwrap();
        let trusted_roots = vec![ca.public_key()];

        (file, trusted_roots)
    }

    #[test]
    fn test_verify_valid_file() {
        let (file, trusted_roots) = create_test_file();

        let result = verify(&file, &trusted_roots).unwrap();

        assert!(result.valid);
        assert_eq!(result.creator_id, "alice@example.com");
        assert_eq!(result.creator_name, "Alice");
        assert_eq!(result.description, Some("Test file".to_string()));
    }

    #[test]
    fn test_verify_untrusted_root() {
        let (file, _) = create_test_file();

        // Use a different trusted root
        let other_ca = CertificateAuthority::new_root_with_timestamp(
            "other@example.com",
            "Other CA",
            1704067200,
        );
        let wrong_roots = vec![other_ca.public_key()];

        let result = verify(&file, &wrong_roots);
        assert!(matches!(result, Err(AletheiaError::UntrustedRoot)));
    }

    #[test]
    fn test_verify_tampered_payload() {
        let (mut file, trusted_roots) = create_test_file();

        // Tamper with the payload
        file.payload = b"Tampered content".to_vec();

        let result = verify(&file, &trusted_roots);
        assert!(matches!(result, Err(AletheiaError::InvalidSignature)));
    }

    #[test]
    fn test_verify_tampered_header() {
        let (mut file, trusted_roots) = create_test_file();

        // Tamper with the header
        file.header.description = Some("Tampered description".to_string());

        let result = verify(&file, &trusted_roots);
        assert!(matches!(result, Err(AletheiaError::InvalidSignature)));
    }

    #[test]
    fn test_validate_structure() {
        let (file, _) = create_test_file();
        validate_structure(&file).unwrap();
    }
}
