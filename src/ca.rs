extern crate alloc;

use crate::{certificate::generate_serial, AletheiaError, Certificate, Result};
use ed25519_dalek::{Signer, SigningKey, VerifyingKey};
use rand::rngs::OsRng;

/// A Certificate Authority that can issue certificates
pub struct CertificateAuthority {
    /// The CA's signing key
    signing_key: SigningKey,
    /// The CA's certificate (self-signed for root CA)
    pub certificate: Certificate,
}

impl CertificateAuthority {
    /// Create a new root Certificate Authority
    ///
    /// This generates a new key pair and creates a self-signed root certificate.
    #[cfg(feature = "std")]
    pub fn new_root(subject_id: impl Into<String>, subject_name: impl Into<String>) -> Self {
        Self::new_root_with_timestamp(subject_id, subject_name, chrono::Utc::now().timestamp())
    }

    /// Create a new root Certificate Authority with a specific timestamp
    ///
    /// This generates a new key pair and creates a self-signed root certificate.
    /// Use this in no_std environments or when you need to control the timestamp.
    pub fn new_root_with_timestamp(
        subject_id: impl Into<String>,
        subject_name: impl Into<String>,
        issued_at: i64,
    ) -> Self {
        let signing_key = SigningKey::generate(&mut OsRng);
        let public_key = signing_key.verifying_key();
        let subject_id = subject_id.into();

        // Create self-signed root certificate
        let mut certificate = Certificate {
            version: 1,
            serial: generate_serial(),
            subject_id: subject_id.clone(),
            subject_name: subject_name.into(),
            public_key: public_key.to_bytes().to_vec(),
            issuer_id: subject_id, // Self-signed
            issued_at,
            is_ca: true,
            signature: Vec::new(),
        };

        // Sign the certificate with our own key (self-signed)
        let signable = certificate.signable_data();
        certificate.signature = signing_key.sign(&signable).to_bytes().to_vec();

        Self {
            signing_key,
            certificate,
        }
    }

    /// Create a CA from an existing signing key and certificate
    ///
    /// Used for loading a CA from storage.
    pub fn from_key_and_cert(signing_key_bytes: &[u8], certificate: Certificate) -> Result<Self> {
        let signing_key_array: [u8; 32] = signing_key_bytes.try_into().map_err(|_| {
            AletheiaError::KeyGeneration("Invalid signing key length".into())
        })?;

        let signing_key = SigningKey::from_bytes(&signing_key_array);

        // Verify the key matches the certificate
        let public_key = signing_key.verifying_key();
        if public_key.to_bytes() != certificate.public_key.as_slice() {
            return Err(AletheiaError::InvalidCertificate(
                "Signing key does not match certificate public key".into(),
            ));
        }

        Ok(Self {
            signing_key,
            certificate,
        })
    }

    /// Get the CA's public key
    pub fn public_key(&self) -> Vec<u8> {
        self.signing_key.verifying_key().to_bytes().to_vec()
    }

    /// Get the CA's private key bytes (for secure storage)
    pub fn private_key_bytes(&self) -> Vec<u8> {
        self.signing_key.to_bytes().to_vec()
    }

    /// Issue a certificate for a subject
    ///
    /// The subject provides their public key, and the CA signs a certificate
    /// binding their identity to that key.
    #[cfg(feature = "std")]
    pub fn issue_certificate(
        &self,
        subject_id: impl Into<String>,
        subject_name: impl Into<String>,
        subject_public_key: &[u8],
        is_ca: bool,
    ) -> Result<Certificate> {
        self.issue_certificate_with_timestamp(
            subject_id,
            subject_name,
            subject_public_key,
            is_ca,
            chrono::Utc::now().timestamp(),
        )
    }

    /// Issue a certificate for a subject with a specific timestamp
    ///
    /// The subject provides their public key, and the CA signs a certificate
    /// binding their identity to that key.
    /// Use this in no_std environments or when you need to control the timestamp.
    pub fn issue_certificate_with_timestamp(
        &self,
        subject_id: impl Into<String>,
        subject_name: impl Into<String>,
        subject_public_key: &[u8],
        is_ca: bool,
        issued_at: i64,
    ) -> Result<Certificate> {
        // Validate the public key
        VerifyingKey::try_from(subject_public_key)
            .map_err(|e| AletheiaError::InvalidCertificate(alloc::format!("Invalid public key: {}", e)))?;

        let mut certificate = Certificate {
            version: 1,
            serial: generate_serial(),
            subject_id: subject_id.into(),
            subject_name: subject_name.into(),
            public_key: subject_public_key.to_vec(),
            issuer_id: self.certificate.subject_id.clone(),
            issued_at,
            is_ca,
            signature: Vec::new(),
        };

        // Sign the certificate
        let signable = certificate.signable_data();
        certificate.signature = self.signing_key.sign(&signable).to_bytes().to_vec();

        Ok(certificate)
    }
}

/// A key pair for signing data (used by content creators)
pub struct SigningKeyPair {
    signing_key: SigningKey,
}

impl SigningKeyPair {
    /// Generate a new random key pair
    pub fn generate() -> Self {
        Self {
            signing_key: SigningKey::generate(&mut OsRng),
        }
    }

    /// Load a key pair from private key bytes
    pub fn from_bytes(private_key: &[u8]) -> Result<Self> {
        let key_array: [u8; 32] = private_key.try_into().map_err(|_| {
            AletheiaError::KeyGeneration("Invalid private key length".into())
        })?;

        Ok(Self {
            signing_key: SigningKey::from_bytes(&key_array),
        })
    }

    /// Get the public key bytes
    pub fn public_key(&self) -> Vec<u8> {
        self.signing_key.verifying_key().to_bytes().to_vec()
    }

    /// Get the private key bytes (for secure storage)
    pub fn private_key_bytes(&self) -> Vec<u8> {
        self.signing_key.to_bytes().to_vec()
    }

    /// Sign data and return the signature bytes
    pub fn sign(&self, data: &[u8]) -> Vec<u8> {
        self.signing_key.sign(data).to_bytes().to_vec()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::certificate::{verify_certificate_chain, verify_certificate_signature};

    #[test]
    fn test_create_root_ca() {
        let ca = CertificateAuthority::new_root("root@example.com", "Root CA");

        assert_eq!(ca.certificate.subject_id, "root@example.com");
        assert_eq!(ca.certificate.issuer_id, "root@example.com");
        assert!(ca.certificate.is_ca);

        // Verify self-signature
        verify_certificate_signature(&ca.certificate, &ca.public_key()).unwrap();
    }

    #[test]
    fn test_issue_certificate() {
        let ca = CertificateAuthority::new_root("root@example.com", "Root CA");
        let user_keys = SigningKeyPair::generate();

        let cert = ca
            .issue_certificate(
                "alice@example.com",
                "Alice",
                &user_keys.public_key(),
                false,
            )
            .unwrap();

        assert_eq!(cert.subject_id, "alice@example.com");
        assert_eq!(cert.issuer_id, "root@example.com");
        assert!(!cert.is_ca);

        // Verify signature
        verify_certificate_signature(&cert, &ca.public_key()).unwrap();
    }

    #[test]
    fn test_certificate_chain() {
        let root_ca = CertificateAuthority::new_root("root@example.com", "Root CA");
        let user_keys = SigningKeyPair::generate();

        let user_cert = root_ca
            .issue_certificate(
                "alice@example.com",
                "Alice",
                &user_keys.public_key(),
                false,
            )
            .unwrap();

        let chain = vec![user_cert, root_ca.certificate.clone()];
        let trusted_roots = vec![root_ca.public_key()];

        verify_certificate_chain(&chain, &trusted_roots).unwrap();
    }
}
