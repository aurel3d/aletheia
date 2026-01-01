extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

pub const MAGIC_BYTES: &[u8; 8] = b"ALETHEIA";
pub const VERSION_MAJOR: u8 = 1;
pub const VERSION_MINOR: u8 = 0;

/// Flags for the Aletheia file format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Flags(u16);

impl Flags {
    pub const COMPRESSED: u16 = 0b0000_0000_0000_0001;

    pub fn new() -> Self {
        Self(0)
    }

    #[cfg(feature = "compression")]
    pub fn with_compression(mut self) -> Self {
        self.0 |= Self::COMPRESSED;
        self
    }

    pub fn is_compressed(&self) -> bool {
        self.0 & Self::COMPRESSED != 0
    }

    pub fn to_bytes(&self) -> [u8; 2] {
        self.0.to_le_bytes()
    }

    pub fn from_bytes(bytes: [u8; 2]) -> Self {
        Self(u16::from_le_bytes(bytes))
    }
}

/// Header metadata for an Aletheia file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Header {
    /// MIME type of the payload (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_type: Option<String>,

    /// Unix timestamp when the data was signed
    pub signed_at: i64,

    /// Unique identifier of the signer
    pub creator_id: String,

    /// Original filename if applicable (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub original_name: Option<String>,

    /// Human-readable description (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Application-specific custom metadata (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom: Option<BTreeMap<String, serde_cbor_value::Value>>,
}

/// Workaround for custom CBOR values in the header
pub mod serde_cbor_value {
    extern crate alloc;

    use alloc::string::String;
    use alloc::vec::Vec;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(untagged)]
    pub enum Value {
        Null,
        Bool(bool),
        Integer(i64),
        Float(f64),
        Text(String),
        Bytes(Vec<u8>),
        Array(Vec<Value>),
        Map(Vec<(String, Value)>),
    }
}

impl Header {
    #[cfg(feature = "std")]
    pub fn new(creator_id: impl Into<String>) -> Self {
        Self {
            content_type: None,
            signed_at: chrono::Utc::now().timestamp(),
            creator_id: creator_id.into(),
            original_name: None,
            description: None,
            custom: None,
        }
    }

    /// Create a header with a specific timestamp (useful for WASM/no_std)
    pub fn new_with_timestamp(creator_id: impl Into<String>, signed_at: i64) -> Self {
        Self {
            content_type: None,
            signed_at,
            creator_id: creator_id.into(),
            original_name: None,
            description: None,
            custom: None,
        }
    }

    pub fn with_content_type(mut self, content_type: impl Into<String>) -> Self {
        self.content_type = Some(content_type.into());
        self
    }

    pub fn with_original_name(mut self, name: impl Into<String>) -> Self {
        self.original_name = Some(name.into());
        self
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }
}

/// A certificate that attests to a subject's identity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Certificate {
    /// Certificate format version
    pub version: u8,

    /// Unique serial number
    #[serde(with = "serde_bytes")]
    pub serial: Vec<u8>,

    /// Identity of the certificate holder (e.g., email)
    pub subject_id: String,

    /// Human-readable name of the holder
    pub subject_name: String,

    /// Ed25519 public key (32 bytes)
    #[serde(with = "serde_bytes")]
    pub public_key: Vec<u8>,

    /// Identity of the issuing CA
    pub issuer_id: String,

    /// Unix timestamp when issued
    pub issued_at: i64,

    /// Whether this certificate can issue other certificates
    pub is_ca: bool,

    /// Ed25519 signature by the issuer (64 bytes)
    #[serde(with = "serde_bytes")]
    pub signature: Vec<u8>,
}

impl Certificate {
    /// Get the data that is signed by the issuer (everything except the signature)
    pub fn signable_data(&self) -> Vec<u8> {
        let unsigned = UnsignedCertificate {
            version: self.version,
            serial: self.serial.clone(),
            subject_id: self.subject_id.clone(),
            subject_name: self.subject_name.clone(),
            public_key: self.public_key.clone(),
            issuer_id: self.issuer_id.clone(),
            issued_at: self.issued_at,
            is_ca: self.is_ca,
        };
        let mut data = Vec::new();
        ciborium::into_writer(&unsigned, &mut data).expect("CBOR encoding failed");
        data
    }
}

/// Certificate data without signature (used for signing)
#[derive(Serialize)]
struct UnsignedCertificate {
    version: u8,
    #[serde(with = "serde_bytes")]
    serial: Vec<u8>,
    subject_id: String,
    subject_name: String,
    #[serde(with = "serde_bytes")]
    public_key: Vec<u8>,
    issuer_id: String,
    issued_at: i64,
    is_ca: bool,
}

/// A complete Aletheia file structure
#[derive(Debug, Clone)]
pub struct AletheiaFile {
    pub version_major: u8,
    pub version_minor: u8,
    pub flags: Flags,
    pub header: Header,
    pub payload: Vec<u8>,
    pub certificate_chain: Vec<Certificate>,
    pub signature: Vec<u8>,
}

impl AletheiaFile {
    /// Get the original (decompressed) payload
    pub fn get_payload(&self) -> crate::Result<Vec<u8>> {
        if self.flags.is_compressed() {
            #[cfg(feature = "compression")]
            {
                lz4_flex::decompress_size_prepended(&self.payload)
                    .map_err(|e| crate::AletheiaError::Decompression(alloc::format!("{}", e)))
            }
            #[cfg(not(feature = "compression"))]
            {
                Err(crate::AletheiaError::Decompression(
                    "Compression feature not enabled".into(),
                ))
            }
        } else {
            Ok(self.payload.clone())
        }
    }
}
