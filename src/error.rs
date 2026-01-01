use thiserror::Error;

#[derive(Error, Debug)]
pub enum AletheiaError {
    #[error("Invalid magic bytes: expected 'ALETHEIA'")]
    InvalidMagic,

    #[error("Unsupported version: {major}.{minor}")]
    UnsupportedVersion { major: u8, minor: u8 },

    #[error("Invalid signature")]
    InvalidSignature,

    #[error("Certificate chain verification failed: {0}")]
    CertificateChainInvalid(String),

    #[error("Certificate not found for subject: {0}")]
    CertificateNotFound(String),

    #[error("Untrusted root certificate")]
    UntrustedRoot,

    #[error("Certificate revoked: serial {0}")]
    CertificateRevoked(String),

    #[error("Invalid certificate: {0}")]
    InvalidCertificate(String),

    #[error("CBOR encoding error: {0}")]
    CborEncode(String),

    #[error("CBOR decoding error: {0}")]
    CborDecode(String),

    #[error("Compression error: {0}")]
    Compression(String),

    #[error("Decompression error: {0}")]
    Decompression(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid header: {0}")]
    InvalidHeader(String),

    #[error("Key generation failed: {0}")]
    KeyGeneration(String),
}

pub type Result<T> = std::result::Result<T, AletheiaError>;
