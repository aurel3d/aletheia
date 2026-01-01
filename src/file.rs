//! File I/O operations for Aletheia files.
//!
//! This module provides functions for reading and writing `.alx` files.
//! For WASM/no_std environments, use `to_bytes` and `from_bytes` instead
//! of the file-based functions.

extern crate alloc;

use alloc::string::ToString;
use alloc::vec::Vec;
use crate::{AletheiaError, AletheiaFile, Certificate, Flags, Header, Result, MAGIC_BYTES};

/// Serialize an Aletheia file to bytes
pub fn to_bytes(file: &AletheiaFile) -> Result<Vec<u8>> {
    let mut buffer = Vec::new();

    // Magic bytes
    buffer.extend_from_slice(MAGIC_BYTES);

    // Version
    buffer.push(file.version_major);
    buffer.push(file.version_minor);

    // Flags
    buffer.extend_from_slice(&file.flags.to_bytes());

    // Header (CBOR)
    let mut header_bytes = Vec::new();
    ciborium::into_writer(&file.header, &mut header_bytes)
        .map_err(|e| AletheiaError::CborEncode(e.to_string()))?;

    buffer.extend_from_slice(&(header_bytes.len() as u32).to_le_bytes());
    buffer.extend_from_slice(&header_bytes);

    // Payload
    buffer.extend_from_slice(&(file.payload.len() as u64).to_le_bytes());
    buffer.extend_from_slice(&file.payload);

    // Certificate chain (CBOR)
    let mut cert_chain_bytes = Vec::new();
    ciborium::into_writer(&file.certificate_chain, &mut cert_chain_bytes)
        .map_err(|e| AletheiaError::CborEncode(e.to_string()))?;

    buffer.extend_from_slice(&(cert_chain_bytes.len() as u32).to_le_bytes());
    buffer.extend_from_slice(&cert_chain_bytes);

    // Signature
    buffer.extend_from_slice(&file.signature);

    Ok(buffer)
}

/// Deserialize an Aletheia file from bytes
pub fn from_bytes(data: &[u8]) -> Result<AletheiaFile> {
    let mut cursor = 0;

    // Helper to read bytes
    let read_bytes = |cursor: &mut usize, len: usize| -> Result<&[u8]> {
        if *cursor + len > data.len() {
            return Err(AletheiaError::UnexpectedEof);
        }
        let result = &data[*cursor..*cursor + len];
        *cursor += len;
        Ok(result)
    };

    // Magic bytes
    let magic = read_bytes(&mut cursor, 8)?;
    if magic != MAGIC_BYTES {
        return Err(AletheiaError::InvalidMagic);
    }

    // Version
    let version = read_bytes(&mut cursor, 2)?;
    let version_major = version[0];
    let version_minor = version[1];

    if version_major != 1 {
        return Err(AletheiaError::UnsupportedVersion {
            major: version_major,
            minor: version_minor,
        });
    }

    // Flags
    let flags_bytes: [u8; 2] = read_bytes(&mut cursor, 2)?.try_into().unwrap();
    let flags = Flags::from_bytes(flags_bytes);

    // Header length
    let header_len_bytes: [u8; 4] = read_bytes(&mut cursor, 4)?.try_into().unwrap();
    let header_len = u32::from_le_bytes(header_len_bytes) as usize;

    // Header
    let header_bytes = read_bytes(&mut cursor, header_len)?;
    let header: Header = ciborium::from_reader(header_bytes)
        .map_err(|e| AletheiaError::CborDecode(e.to_string()))?;

    // Payload length
    let payload_len_bytes: [u8; 8] = read_bytes(&mut cursor, 8)?.try_into().unwrap();
    let payload_len = u64::from_le_bytes(payload_len_bytes) as usize;

    // Payload
    let payload = read_bytes(&mut cursor, payload_len)?.to_vec();

    // Certificate chain length
    let cert_len_bytes: [u8; 4] = read_bytes(&mut cursor, 4)?.try_into().unwrap();
    let cert_len = u32::from_le_bytes(cert_len_bytes) as usize;

    // Certificate chain
    let cert_chain_bytes = read_bytes(&mut cursor, cert_len)?;
    let certificate_chain: Vec<Certificate> = ciborium::from_reader(cert_chain_bytes)
        .map_err(|e| AletheiaError::CborDecode(e.to_string()))?;

    // Signature
    let signature = read_bytes(&mut cursor, 64)?.to_vec();

    Ok(AletheiaFile {
        version_major,
        version_minor,
        flags,
        header,
        payload,
        certificate_chain,
        signature,
    })
}

// std-only file I/O functions
#[cfg(feature = "std")]
mod std_io {
    use super::*;
    use std::io::{Read, Write};

    /// Write an Aletheia file to a writer
    pub fn write<W: Write>(file: &AletheiaFile, mut writer: W) -> Result<()> {
        let bytes = to_bytes(file)?;
        writer.write_all(&bytes)?;
        Ok(())
    }

    /// Write an Aletheia file to a path
    pub fn write_to_file(file: &AletheiaFile, path: impl AsRef<std::path::Path>) -> Result<()> {
        let f = std::fs::File::create(path)?;
        let writer = std::io::BufWriter::new(f);
        write(file, writer)
    }

    /// Read an Aletheia file from a reader
    pub fn read<R: Read>(mut reader: R) -> Result<AletheiaFile> {
        let mut buffer = Vec::new();
        reader.read_to_end(&mut buffer)?;
        from_bytes(&buffer)
    }

    /// Read an Aletheia file from a path
    pub fn read_from_file(path: impl AsRef<std::path::Path>) -> Result<AletheiaFile> {
        let f = std::fs::File::open(path)?;
        let reader = std::io::BufReader::new(f);
        read(reader)
    }

    /// Check if a file appears to be an Aletheia file by checking magic bytes
    pub fn is_aletheia_file(path: impl AsRef<std::path::Path>) -> Result<bool> {
        let mut f = std::fs::File::open(path)?;
        let mut magic = [0u8; 8];
        match f.read_exact(&mut magic) {
            Ok(_) => Ok(&magic == MAGIC_BYTES),
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => Ok(false),
            Err(e) => Err(e.into()),
        }
    }
}

#[cfg(feature = "std")]
pub use std_io::*;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ca::{CertificateAuthority, SigningKeyPair},
        signer::Signer,
    };

    fn create_test_file() -> AletheiaFile {
        let ca = CertificateAuthority::new_root("root@example.com", "Root CA");
        let user_keys = SigningKeyPair::generate();

        let user_cert = ca
            .issue_certificate("alice@example.com", "Alice", &user_keys.public_key(), false)
            .unwrap();

        let chain = vec![user_cert, ca.certificate.clone()];
        let signer = Signer::new(user_keys, chain).unwrap();

        let payload = b"Test content for file I/O";
        let header = Header::new("alice@example.com")
            .with_content_type("text/plain")
            .with_description("Test file");

        signer.sign(payload, header).unwrap()
    }

    #[test]
    fn test_to_bytes_and_from_bytes() {
        let original = create_test_file();

        // Serialize
        let bytes = to_bytes(&original).unwrap();

        // Deserialize
        let loaded = from_bytes(&bytes).unwrap();

        assert_eq!(loaded.version_major, original.version_major);
        assert_eq!(loaded.version_minor, original.version_minor);
        assert_eq!(loaded.flags.is_compressed(), original.flags.is_compressed());
        assert_eq!(loaded.payload, original.payload);
        assert_eq!(loaded.signature, original.signature);
        assert_eq!(loaded.header.creator_id, original.header.creator_id);
        assert_eq!(
            loaded.certificate_chain.len(),
            original.certificate_chain.len()
        );
    }

    #[cfg(feature = "std")]
    #[test]
    fn test_file_roundtrip() {
        let original = create_test_file();
        let temp_dir = tempfile::tempdir().unwrap();
        let path = temp_dir.path().join("test.alx");

        // Write to file
        write_to_file(&original, &path).unwrap();

        // Verify it's an Aletheia file
        assert!(is_aletheia_file(&path).unwrap());

        // Read back
        let loaded = read_from_file(&path).unwrap();
        assert_eq!(loaded.payload, original.payload);
    }

    #[test]
    fn test_invalid_magic() {
        let data = b"NOTVALID12345678";
        let result = from_bytes(data);
        assert!(matches!(result, Err(AletheiaError::InvalidMagic)));
    }
}
