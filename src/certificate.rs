use crate::{AletheiaError, Certificate, Result};
use ed25519_dalek::{Signature, Verifier, VerifyingKey};

/// Verify that a certificate was properly signed by its issuer
pub fn verify_certificate_signature(cert: &Certificate, issuer_public_key: &[u8]) -> Result<()> {
    let verifying_key = VerifyingKey::try_from(issuer_public_key).map_err(|e| {
        AletheiaError::InvalidCertificate(format!("Invalid issuer public key: {}", e))
    })?;

    let signature = Signature::try_from(cert.signature.as_slice()).map_err(|e| {
        AletheiaError::InvalidCertificate(format!("Invalid signature format: {}", e))
    })?;

    let signable = cert.signable_data();
    verifying_key
        .verify(&signable, &signature)
        .map_err(|_| AletheiaError::InvalidCertificate("Signature verification failed".into()))
}

/// Verify a complete certificate chain
///
/// The chain should be ordered: [creator_cert, ..., root_cert]
/// Each certificate is verified against the next one in the chain.
/// The root certificate must be self-signed.
pub fn verify_certificate_chain(
    chain: &[Certificate],
    trusted_root_keys: &[Vec<u8>],
) -> Result<()> {
    if chain.is_empty() {
        return Err(AletheiaError::CertificateChainInvalid(
            "Empty certificate chain".into(),
        ));
    }

    // Verify each certificate in the chain
    for i in 0..chain.len() {
        let cert = &chain[i];

        // Get the issuer's public key
        let issuer_key = if i + 1 < chain.len() {
            // Issuer is the next certificate in the chain
            let issuer = &chain[i + 1];

            // Verify the issuer is allowed to issue certificates
            if !issuer.is_ca {
                return Err(AletheiaError::CertificateChainInvalid(format!(
                    "Certificate '{}' is not a CA but issued '{}'",
                    issuer.subject_id, cert.subject_id
                )));
            }

            // Verify issuer ID matches
            if cert.issuer_id != issuer.subject_id {
                return Err(AletheiaError::CertificateChainInvalid(format!(
                    "Issuer ID mismatch: cert says '{}', chain has '{}'",
                    cert.issuer_id, issuer.subject_id
                )));
            }

            &issuer.public_key
        } else {
            // This is the root certificate - must be self-signed
            if cert.issuer_id != cert.subject_id {
                return Err(AletheiaError::CertificateChainInvalid(
                    "Root certificate is not self-signed".into(),
                ));
            }

            // Root must be a CA
            if !cert.is_ca {
                return Err(AletheiaError::CertificateChainInvalid(
                    "Root certificate is not marked as CA".into(),
                ));
            }

            // Verify root is trusted
            if !trusted_root_keys.contains(&cert.public_key) {
                return Err(AletheiaError::UntrustedRoot);
            }

            &cert.public_key
        };

        // Verify this certificate's signature
        verify_certificate_signature(cert, issuer_key)?;
    }

    Ok(())
}

/// Generate a unique serial number for a certificate
pub fn generate_serial() -> Vec<u8> {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let mut serial = vec![0u8; 16];
    rng.fill(&mut serial[..]);
    serial
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_serial() {
        let s1 = generate_serial();
        let s2 = generate_serial();
        assert_eq!(s1.len(), 16);
        assert_ne!(s1, s2);
    }
}
