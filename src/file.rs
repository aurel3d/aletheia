use crate::{AletheiaError, AletheiaFile, Certificate, Flags, Header, Result, MAGIC_BYTES};
use std::io::{Read, Write};

/// Write an Aletheia file to a writer
pub fn write<W: Write>(file: &AletheiaFile, mut writer: W) -> Result<()> {
    // Magic bytes
    writer.write_all(MAGIC_BYTES)?;

    // Version
    writer.write_all(&[file.version_major, file.version_minor])?;

    // Flags
    writer.write_all(&file.flags.to_bytes())?;

    // Header (CBOR)
    let mut header_bytes = Vec::new();
    ciborium::into_writer(&file.header, &mut header_bytes)
        .map_err(|e| AletheiaError::CborEncode(e.to_string()))?;

    writer.write_all(&(header_bytes.len() as u32).to_le_bytes())?;
    writer.write_all(&header_bytes)?;

    // Payload
    writer.write_all(&(file.payload.len() as u64).to_le_bytes())?;
    writer.write_all(&file.payload)?;

    // Certificate chain (CBOR)
    let mut cert_chain_bytes = Vec::new();
    ciborium::into_writer(&file.certificate_chain, &mut cert_chain_bytes)
        .map_err(|e| AletheiaError::CborEncode(e.to_string()))?;

    writer.write_all(&(cert_chain_bytes.len() as u32).to_le_bytes())?;
    writer.write_all(&cert_chain_bytes)?;

    // Signature
    writer.write_all(&file.signature)?;

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
    // Magic bytes
    let mut magic = [0u8; 8];
    reader.read_exact(&mut magic)?;
    if &magic != MAGIC_BYTES {
        return Err(AletheiaError::InvalidMagic);
    }

    // Version
    let mut version = [0u8; 2];
    reader.read_exact(&mut version)?;
    let version_major = version[0];
    let version_minor = version[1];

    // Check version compatibility
    if version_major != 1 {
        return Err(AletheiaError::UnsupportedVersion {
            major: version_major,
            minor: version_minor,
        });
    }

    // Flags
    let mut flags_bytes = [0u8; 2];
    reader.read_exact(&mut flags_bytes)?;
    let flags = Flags::from_bytes(flags_bytes);

    // Header length
    let mut header_len_bytes = [0u8; 4];
    reader.read_exact(&mut header_len_bytes)?;
    let header_len = u32::from_le_bytes(header_len_bytes) as usize;

    // Header
    let mut header_bytes = vec![0u8; header_len];
    reader.read_exact(&mut header_bytes)?;
    let header: Header = ciborium::from_reader(&header_bytes[..])
        .map_err(|e| AletheiaError::CborDecode(e.to_string()))?;

    // Payload length
    let mut payload_len_bytes = [0u8; 8];
    reader.read_exact(&mut payload_len_bytes)?;
    let payload_len = u64::from_le_bytes(payload_len_bytes) as usize;

    // Payload
    let mut payload = vec![0u8; payload_len];
    reader.read_exact(&mut payload)?;

    // Certificate chain length
    let mut cert_len_bytes = [0u8; 4];
    reader.read_exact(&mut cert_len_bytes)?;
    let cert_len = u32::from_le_bytes(cert_len_bytes) as usize;

    // Certificate chain
    let mut cert_chain_bytes = vec![0u8; cert_len];
    reader.read_exact(&mut cert_chain_bytes)?;
    let certificate_chain: Vec<Certificate> = ciborium::from_reader(&cert_chain_bytes[..])
        .map_err(|e| AletheiaError::CborDecode(e.to_string()))?;

    // Signature
    let mut signature = vec![0u8; 64];
    reader.read_exact(&mut signature)?;

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
    fn test_write_and_read() {
        let original = create_test_file();

        // Write to buffer
        let mut buffer = Vec::new();
        write(&original, &mut buffer).unwrap();

        // Read back
        let loaded = read(&buffer[..]).unwrap();

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
        let result = read(&data[..]);
        assert!(matches!(result, Err(AletheiaError::InvalidMagic)));
    }
}
