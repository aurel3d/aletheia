//! WASM bindings for browser use

use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use crate::{
    ca::{CertificateAuthority, SigningKeyPair},
    file::{from_bytes, to_bytes},
    signer::Signer,
    verifier::verify,
    Certificate, Header,
};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

// Re-export types with serde
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WasmHeader {
    pub creator_id: String,
    pub signed_at: i64,
    pub content_type: Option<String>,
    pub original_name: Option<String>,
    pub description: Option<String>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WasmCertificate {
    pub version: u8,
    #[serde(with = "serde_bytes")]
    pub serial: Vec<u8>,
    pub subject_id: String,
    pub subject_name: String,
    #[serde(with = "serde_bytes")]
    pub public_key: Vec<u8>,
    pub issuer_id: String,
    pub issued_at: i64,
    pub is_ca: bool,
    #[serde(with = "serde_bytes")]
    pub signature: Vec<u8>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WasmParsedFile {
    pub version_major: u8,
    pub version_minor: u8,
    pub is_compressed: bool,
    pub header: WasmHeader,
    #[serde(with = "serde_bytes")]
    pub payload: Vec<u8>,
    pub certificate_chain: Vec<WasmCertificate>,
    #[serde(with = "serde_bytes")]
    pub signature: Vec<u8>,

    // Byte ranges for hex highlighting
    pub magic_range: (usize, usize),
    pub version_range: (usize, usize),
    pub flags_range: (usize, usize),
    pub header_range: (usize, usize),
    pub payload_range: (usize, usize),
    pub cert_chain_range: (usize, usize),
    pub signature_range: (usize, usize),
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WasmVerificationResult {
    pub valid: bool,
    pub creator_id: String,
    pub creator_name: String,
    pub signed_at: i64,
    pub description: Option<String>,
}

/// Parse an Aletheia file from bytes
#[wasm_bindgen]
pub fn parse_aletheia_file(data: &[u8]) -> Result<JsValue, JsValue> {
    let file = from_bytes(data).map_err(|e| JsValue::from_str(&format!("Parse error: {}", e)))?;

    // Calculate byte ranges
    let mut offset = 0;

    // Magic (8 bytes)
    let magic_range = (offset, offset + 8);
    offset += 8;

    // Version (2 bytes)
    let version_range = (offset, offset + 2);
    offset += 2;

    // Flags (2 bytes)
    let flags_range = (offset, offset + 2);
    offset += 2;

    // Header (4-byte length + content)
    let header_start = offset;
    let mut header_bytes = Vec::new();
    ciborium::into_writer(&file.header, &mut header_bytes).unwrap();
    let header_len = header_bytes.len();
    offset += 4 + header_len;
    let header_range = (header_start, offset);

    // Payload (8-byte length + content)
    let payload_start = offset;
    offset += 8 + file.payload.len();
    let payload_range = (payload_start, offset);

    // Cert chain (4-byte length + content)
    let cert_start = offset;
    let mut cert_bytes = Vec::new();
    ciborium::into_writer(&file.certificate_chain, &mut cert_bytes).unwrap();
    let cert_len = cert_bytes.len();
    offset += 4 + cert_len;
    let cert_chain_range = (cert_start, offset);

    // Signature (64 bytes)
    let signature_range = (offset, offset + 64);

    let parsed = WasmParsedFile {
        version_major: file.version_major,
        version_minor: file.version_minor,
        is_compressed: file.flags.is_compressed(),
        header: WasmHeader {
            creator_id: file.header.creator_id,
            signed_at: file.header.signed_at,
            content_type: file.header.content_type,
            original_name: file.header.original_name,
            description: file.header.description,
        },
        payload: file.payload,
        certificate_chain: file
            .certificate_chain
            .into_iter()
            .map(|c| WasmCertificate {
                version: c.version,
                serial: c.serial,
                subject_id: c.subject_id,
                subject_name: c.subject_name,
                public_key: c.public_key,
                issuer_id: c.issuer_id,
                issued_at: c.issued_at,
                is_ca: c.is_ca,
                signature: c.signature,
            })
            .collect(),
        signature: file.signature,
        magic_range,
        version_range,
        flags_range,
        header_range,
        payload_range,
        cert_chain_range,
        signature_range,
    };

    serde_wasm_bindgen::to_value(&parsed)
        .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
}

/// Verify an Aletheia file
/// trusted_root_keys should be a JS Array of Uint8Array
#[wasm_bindgen]
pub fn verify_aletheia_file(data: &[u8], trusted_root_keys: JsValue) -> Result<JsValue, JsValue> {
    let file = from_bytes(data).map_err(|e| JsValue::from_str(&format!("Parse error: {}", e)))?;

    // Convert JsValue to Vec<Vec<u8>>
    let trusted_roots: Vec<Vec<u8>> = serde_wasm_bindgen::from_value(trusted_root_keys)
        .map_err(|e| JsValue::from_str(&format!("Invalid trusted roots format: {}", e)))?;

    let result = verify(&file, &trusted_roots)
        .map_err(|e| JsValue::from_str(&format!("Verification error: {}", e)))?;

    let wasm_result = WasmVerificationResult {
        valid: result.valid,
        creator_id: result.creator_id,
        creator_name: result.creator_name,
        signed_at: result.signed_at,
        description: result.description,
    };

    serde_wasm_bindgen::to_value(&wasm_result)
        .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
}

/// Decompress payload if compressed
#[wasm_bindgen]
pub fn decompress_payload(payload: &[u8], is_compressed: bool) -> Result<Vec<u8>, JsValue> {
    if !is_compressed {
        return Ok(payload.to_vec());
    }

    #[cfg(feature = "compression")]
    {
        lz4_flex::decompress_size_prepended(payload)
            .map_err(|e| JsValue::from_str(&format!("Decompression error: {}", e)))
    }

    #[cfg(not(feature = "compression"))]
    {
        Err(JsValue::from_str("Compression support not enabled"))
    }
}

/// Parse a CBOR-encoded certificate and return its details
/// Used for validating and displaying CA certificate information
#[wasm_bindgen]
pub fn parse_certificate(cbor_bytes: &[u8]) -> Result<JsValue, JsValue> {
    let cert: Certificate = ciborium::from_reader(cbor_bytes)
        .map_err(|e| JsValue::from_str(&format!("Certificate parse error: {}", e)))?;

    let wasm_cert = WasmCertificate {
        version: cert.version,
        serial: cert.serial,
        subject_id: cert.subject_id,
        subject_name: cert.subject_name,
        public_key: cert.public_key,
        issuer_id: cert.issuer_id,
        issued_at: cert.issued_at,
        is_ca: cert.is_ca,
        signature: cert.signature,
    };

    serde_wasm_bindgen::to_value(&wasm_cert)
        .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
}

/// Sign a file using CA credentials (all-in-one function)
///
/// This function:
/// 1. Generates an ephemeral keypair for this specific file
/// 2. Issues a certificate for the ephemeral key using the CA
/// 3. Signs the file with the ephemeral key
/// 4. Returns the complete .alx file bytes
///
/// # Arguments
/// * `payload` - The file content to sign
/// * `ca_private_key` - CA private key (32 bytes)
/// * `ca_cert_cbor` - CA certificate as CBOR bytes
/// * `creator_id` - Identity string for the signer (e.g., email)
/// * `content_type` - Optional MIME type
/// * `original_name` - Optional original filename
/// * `description` - Optional description
/// * `compress` - Whether to enable compression
#[wasm_bindgen]
pub fn sign_file_with_ca(
    payload: &[u8],
    ca_private_key: &[u8],
    ca_cert_cbor: &[u8],
    creator_id: &str,
    content_type: Option<String>,
    original_name: Option<String>,
    description: Option<String>,
    compress: bool,
) -> Result<Vec<u8>, JsValue> {
    // Get current timestamp from JavaScript
    let timestamp_ms = js_sys::Date::now();
    let timestamp = (timestamp_ms / 1000.0) as i64;

    // Parse CA certificate from CBOR
    let ca_cert: Certificate = ciborium::from_reader(ca_cert_cbor)
        .map_err(|e| JsValue::from_str(&format!("Failed to parse CA certificate: {}", e)))?;

    // Create CA from key and certificate
    let ca = CertificateAuthority::from_key_and_cert(ca_private_key, ca_cert.clone())
        .map_err(|e| JsValue::from_str(&format!("Failed to create CA: {}", e)))?;

    // Generate ephemeral keypair for this file
    let ephemeral_key = SigningKeyPair::generate();

    // Issue certificate for ephemeral key
    let ephemeral_cert = ca
        .issue_certificate_with_timestamp(
            creator_id,
            creator_id, // Use creator_id as name too for simplicity
            &ephemeral_key.public_key(),
            false, // Not a CA
            timestamp,
        )
        .map_err(|e| JsValue::from_str(&format!("Failed to issue certificate: {}", e)))?;

    // Build certificate chain: [ephemeral_cert, ca_cert]
    let cert_chain = vec![ephemeral_cert, ca_cert];

    // Create signer
    let signer = Signer::new(ephemeral_key, cert_chain)
        .map_err(|e| JsValue::from_str(&format!("Failed to create signer: {}", e)))?;

    // Optionally enable compression
    #[cfg(feature = "compression")]
    let signer = if compress {
        signer.with_compression()
    } else {
        signer
    };

    #[cfg(not(feature = "compression"))]
    let _ = compress; // Suppress unused warning

    // Build header
    let mut header = Header::new_with_timestamp(creator_id, timestamp);
    if let Some(ct) = content_type {
        header = header.with_content_type(ct);
    }
    if let Some(name) = original_name {
        header = header.with_original_name(name);
    }
    if let Some(desc) = description {
        header = header.with_description(desc);
    }

    // Sign the file
    let file = signer
        .sign(payload, header)
        .map_err(|e| JsValue::from_str(&format!("Failed to sign file: {}", e)))?;

    // Serialize to bytes
    let bytes = to_bytes(&file)
        .map_err(|e| JsValue::from_str(&format!("Failed to serialize file: {}", e)))?;

    Ok(bytes)
}
