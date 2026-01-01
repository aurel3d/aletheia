# Aletheia File Format Specification

**Version:** 1.0
**Extension:** `.alx`
**MIME Type:** `application/x-aletheia`

## Purpose

Aletheia provides cryptographic proof that data was **created or approved by a trusted human identity**. In an era of AI-generated content, Aletheia enables:
- Verification that content originates from a specific person
- Permanent, non-expiring attestation
- Tamper-evident packaging

## Core Principles

1. **Permanence**: Signed files never expire. A valid signature remains valid forever.
2. **Human Attestation**: Certificates are issued only to verified human identities.
3. **Immutability**: Any modification invalidates the signature.

## File Structure

All multi-byte integers are stored in **little-endian** format.

```
Offset  Size        Field
─────────────────────────────────────────────────────────
0x00    8 bytes     Magic bytes ("ALETHEIA")
0x08    1 byte      Version major
0x09    1 byte      Version minor
0x0A    2 bytes     Flags
0x0C    4 bytes     Header length (H)
0x10    H bytes     Header (CBOR encoded)
0x10+H  8 bytes     Payload length (P)
0x18+H  P bytes     Payload (raw or compressed bytes)
...     4 bytes     Certificate chain length (C)
...     C bytes     Certificate chain
...     64 bytes    Signature (Ed25519)
─────────────────────────────────────────────────────────
```

## Magic Bytes

```
Hex:    41 4C 45 54 48 45 49 41
ASCII:  A  L  E  T  H  E  I  A
```

## Version

- **Major** (1 byte): Incremented for breaking changes
- **Minor** (1 byte): Incremented for backward-compatible additions

Current version: `1.0`

## Flags

2-byte bitfield:

| Bit | Name              | Description                          |
|-----|-------------------|--------------------------------------|
| 0   | COMPRESSED        | Payload is compressed (zstd)         |
| 1-15| Reserved          | Must be 0                            |

## Header (CBOR)

The header is a CBOR-encoded map containing metadata:

| Field              | Type     | Required | Description                        |
|--------------------|----------|----------|------------------------------------|
| `content_type`     | string   | No       | MIME type of payload               |
| `signed_at`        | integer  | Yes      | Unix timestamp when signed         |
| `creator_id`       | string   | Yes      | Unique identifier of the signer    |
| `original_name`    | string   | No       | Original filename if applicable    |
| `description`      | string   | No       | Human-readable description         |
| `custom`           | map      | No       | Application-specific metadata      |

Example (CBOR diagnostic notation):
```
{
  "content_type": "image/png",
  "signed_at": 1704067200,
  "creator_id": "alice@example.com",
  "original_name": "artwork.png",
  "description": "Original artwork - human created"
}
```

## Payload

The payload contains the actual data being signed. It can be:
- **Raw bytes**: When COMPRESSED flag is 0
- **Zstd-compressed**: When COMPRESSED flag is 1

The payload is data-type agnostic. The `content_type` header field indicates how to interpret the bytes.

## Certificate Chain

The certificate chain establishes trust from the signing key back to the Certificate Authority (CA).

### Certificate Format (CBOR)

Each certificate is CBOR-encoded:

| Field           | Type       | Description                              |
|-----------------|------------|------------------------------------------|
| `version`       | integer    | Certificate format version (1)           |
| `serial`        | bytes      | Unique certificate serial number         |
| `subject_id`    | string     | Identity of the certificate holder       |
| `subject_name`  | string     | Human-readable name                      |
| `public_key`    | bytes      | Ed25519 public key (32 bytes)            |
| `issuer_id`     | string     | Identity of the issuing CA               |
| `issued_at`     | integer    | Unix timestamp of issuance               |
| `is_ca`         | boolean    | True if this certificate can issue others|
| `signature`     | bytes      | Issuer's signature over certificate      |

**Note**: Certificates do NOT expire. Once issued, they remain valid indefinitely.

### Chain Structure

The chain is stored as a CBOR array of certificates:
```
[creator_cert, intermediate_cert, ..., root_cert]
```

- First certificate: Creator's certificate (used for signing)
- Last certificate: Root CA certificate (self-signed)
- Each certificate is signed by the next one in the chain

## Signature

The signature is computed using **Ed25519** over the following data:

```
signature_input = magic_bytes || version || flags || header_length ||
                  header || payload_length || payload ||
                  cert_chain_length || cert_chain
```

The signature is exactly **64 bytes**.

## Verification Process

1. **Parse** the file structure
2. **Validate** magic bytes and version compatibility
3. **Extract** the certificate chain
4. **Verify chain**: Each certificate is signed by the next, root is trusted
5. **Check revocation**: Verify against revocation list (optional)
6. **Verify signature**: Using creator's public key from first certificate
7. **Decompress** payload if COMPRESSED flag is set

If all steps pass, the file is **authentic** - it was signed by the claimed human identity and has not been modified.

## Revocation (Optional)

While certificates don't expire, the CA may maintain a revocation list for compromised keys. Revocation is:
- Published by the CA
- Contains serial numbers of revoked certificates
- Optional to check (depends on application requirements)

## Security Considerations

- Ed25519 provides 128-bit security level
- The CA must verify human identity before issuing certificates
- Private keys must be securely stored by certificate holders
- The CA's root key must be protected with extreme care
- Revocation checking is recommended if key compromise is a concern

## Use Cases

1. **Artists**: Sign original artwork to prove human creation
2. **Journalists**: Sign articles to prove authorship
3. **Researchers**: Sign papers and data to prove authenticity
4. **Legal**: Sign documents with permanent attribution
5. **Software**: Sign releases to prove trusted origin

## Example File (Hex Dump)

```
41 4C 45 54 48 45 49 41  # Magic: "ALETHEIA"
01 00                    # Version: 1.0
00 00                    # Flags: none
2A 00 00 00              # Header length: 42 bytes
[42 bytes of CBOR]       # Header
00 10 00 00 00 00 00 00  # Payload length: 4096 bytes
[4096 bytes]             # Payload
80 01 00 00              # Cert chain length: 384 bytes
[384 bytes of CBOR]      # Certificate chain
[64 bytes]               # Ed25519 signature
```

## MIME Type Registration

- **Type name**: application
- **Subtype name**: x-aletheia
- **File extension**: .alx
