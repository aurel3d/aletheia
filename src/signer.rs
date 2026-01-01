extern crate alloc;

use crate::{
    AletheiaError, AletheiaFile, Certificate, Flags, Header, MAGIC_BYTES, Result, VERSION_MAJOR,
    VERSION_MINOR, ca::SigningKeyPair,
};
use alloc::string::ToString;
use alloc::vec::Vec;

/// Builder for creating signed Aletheia files
pub struct Signer {
    signing_key: SigningKeyPair,
    certificate_chain: Vec<Certificate>,
    #[cfg(feature = "compression")]
    compress: bool,
}

impl Signer {
    /// Create a new signer with a key pair and certificate chain
    ///
    /// The certificate chain should be ordered: [creator_cert, ..., root_cert]
    /// The first certificate must contain the public key matching the signing key.
    pub fn new(signing_key: SigningKeyPair, certificate_chain: Vec<Certificate>) -> Result<Self> {
        if certificate_chain.is_empty() {
            return Err(AletheiaError::CertificateChainInvalid(
                "Certificate chain cannot be empty".into(),
            ));
        }

        // Verify the signing key matches the first certificate
        let creator_cert = &certificate_chain[0];
        if signing_key.public_key() != creator_cert.public_key {
            return Err(AletheiaError::InvalidCertificate(
                "Signing key does not match creator certificate".into(),
            ));
        }

        Ok(Self {
            signing_key,
            certificate_chain,
            #[cfg(feature = "compression")]
            compress: false,
        })
    }

    /// Enable compression for payloads
    #[cfg(feature = "compression")]
    pub fn with_compression(mut self) -> Self {
        self.compress = true;
        self
    }

    /// Sign data and create an Aletheia file structure
    pub fn sign(&self, payload: &[u8], header: Header) -> Result<AletheiaFile> {
        #[cfg(feature = "compression")]
        let (flags, processed_payload) = if self.compress {
            let compressed = lz4_flex::compress_prepend_size(payload);
            (Flags::new().with_compression(), compressed)
        } else {
            (Flags::new(), payload.to_vec())
        };

        #[cfg(not(feature = "compression"))]
        let (flags, processed_payload) = (Flags::new(), payload.to_vec());

        // Encode header as CBOR
        let mut header_bytes = Vec::new();
        ciborium::into_writer(&header, &mut header_bytes)
            .map_err(|e| AletheiaError::CborEncode(e.to_string()))?;

        // Encode certificate chain as CBOR
        let mut cert_chain_bytes = Vec::new();
        ciborium::into_writer(&self.certificate_chain, &mut cert_chain_bytes)
            .map_err(|e| AletheiaError::CborEncode(e.to_string()))?;

        // Build the data to sign
        let signature_input =
            build_signature_input(&flags, &header_bytes, &processed_payload, &cert_chain_bytes);

        // Sign it
        let signature = self.signing_key.sign(&signature_input);

        Ok(AletheiaFile {
            version_major: VERSION_MAJOR,
            version_minor: VERSION_MINOR,
            flags,
            header,
            payload: processed_payload,
            certificate_chain: self.certificate_chain.clone(),
            signature,
        })
    }

    /// Get the creator ID from the certificate
    pub fn creator_id(&self) -> &str {
        &self.certificate_chain[0].subject_id
    }
}

/// Build the input data for signature computation
pub(crate) fn build_signature_input(
    flags: &Flags,
    header_bytes: &[u8],
    payload: &[u8],
    cert_chain_bytes: &[u8],
) -> Vec<u8> {
    let mut input = Vec::new();

    // Magic bytes
    input.extend_from_slice(MAGIC_BYTES);

    // Version
    input.push(VERSION_MAJOR);
    input.push(VERSION_MINOR);

    // Flags
    input.extend_from_slice(&flags.to_bytes());

    // Header length + header
    input.extend_from_slice(&(header_bytes.len() as u32).to_le_bytes());
    input.extend_from_slice(header_bytes);

    // Payload length + payload
    input.extend_from_slice(&(payload.len() as u64).to_le_bytes());
    input.extend_from_slice(payload);

    // Certificate chain length + chain
    input.extend_from_slice(&(cert_chain_bytes.len() as u32).to_le_bytes());
    input.extend_from_slice(cert_chain_bytes);

    input
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ca::CertificateAuthority;

    #[test]
    fn test_sign_data() {
        // Create CA and user
        let timestamp = 1704067200;
        let ca =
            CertificateAuthority::new_root_with_timestamp("root@example.com", "Root CA", timestamp);
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

        // Create signer
        let signer = Signer::new(user_keys, chain).unwrap();

        // Sign some data
        let payload = b"Hello, World!";
        let header = Header::new_with_timestamp("alice@example.com", timestamp)
            .with_content_type("text/plain")
            .with_description("Test data");

        let file = signer.sign(payload, header).unwrap();

        assert_eq!(file.version_major, 1);
        assert_eq!(file.version_minor, 0);
        assert!(!file.flags.is_compressed());
        assert_eq!(file.payload, payload);
        assert_eq!(file.signature.len(), 64);
    }

    #[cfg(feature = "compression")]
    #[test]
    fn test_sign_with_compression() {
        let timestamp = 1704067200;
        let ca =
            CertificateAuthority::new_root_with_timestamp("root@example.com", "Root CA", timestamp);
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
        let signer = Signer::new(user_keys, chain).unwrap().with_compression();

        // Large repetitive data compresses well
        let payload = "Hello, World! ".repeat(1000);
        let header = Header::new_with_timestamp("alice@example.com", timestamp);

        let file = signer.sign(payload.as_bytes(), header).unwrap();

        assert!(file.flags.is_compressed());
        assert!(file.payload.len() < payload.len()); // Should be smaller

        // Verify we can decompress
        let decompressed = file.get_payload().unwrap();
        assert_eq!(decompressed, payload.as_bytes());
    }
}
