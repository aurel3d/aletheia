# Aletheia

Cryptographic proof of human-created content authenticity.

In an era of AI-generated content, Aletheia provides a way to verify that data was **created or approved by a trusted human identity**. It uses digital signatures and a certificate authority model to ensure:

- **Authentication** - Proves who created/signed the data
- **Integrity** - Detects any modification after signing
- **Permanence** - Signatures never expire

## Installation

```bash
cargo install --path .
```

Or build from source:

```bash
cargo build --release
```

## Quick Start

### 1. Initialize a Certificate Authority

```bash
aletheia ca-init --id "ca@example.com" --name "Example CA" --output ./ca
```

This creates:
- `ca/ca.key` - CA private key (keep this secure!)
- `ca/ca.cert` - CA certificate (distribute to verifiers)

### 2. Issue a Certificate to a Human

```bash
aletheia cert-issue \
  --ca-key ./ca/ca.key \
  --ca-cert ./ca/ca.cert \
  --id "alice@example.com" \
  --name "Alice Smith" \
  --output ./alice
```

### 3. Sign Content

```bash
aletheia sign \
  --input artwork.png \
  --key ./alice/alice_example_com.key \
  --cert ./alice/alice_example_com.cert \
  --ca-cert ./ca/ca.cert \
  --description "Original digital artwork"
```

Creates `artwork.png.alx` - the signed Aletheia file.

### 4. Verify Authenticity

```bash
aletheia verify artwork.png.alx --trust ./ca/ca.cert --verbose
```

Output:
```
VERIFIED
  Creator: Alice Smith (alice@example.com)
  Signed:  2024-01-15 10:30:00 UTC
  Description: Original digital artwork

  This content was signed by a verified human identity.
  The signature is valid and the certificate chain is trusted.
```

### 5. Extract Original Content

```bash
aletheia verify artwork.png.alx --trust ./ca/ca.cert --output extracted.png
```

## CLI Commands

| Command | Description |
|---------|-------------|
| `ca-init` | Initialize a new Certificate Authority |
| `cert-issue` | Issue a certificate to a user |
| `keygen` | Generate a new key pair |
| `sign` | Sign a file (creates .alx) |
| `verify` | Verify a signed .alx file |
| `info` | Show information about an .alx file |

Run `aletheia <command> --help` for detailed options.

## Library Usage

```rust
use aletheia::{
    ca::{CertificateAuthority, SigningKeyPair},
    signer::Signer,
    verifier::verify,
    Header,
};

// Create CA and issue certificate
let ca = CertificateAuthority::new_root("ca@example.com", "Root CA");
let user_keys = SigningKeyPair::generate();
let user_cert = ca.issue_certificate(
    "alice@example.com",
    "Alice",
    &user_keys.public_key(),
    false,
).unwrap();

// Sign content
let chain = vec![user_cert, ca.certificate.clone()];
let signer = Signer::new(user_keys, chain).unwrap();
let header = Header::new("alice@example.com")
    .with_description("My content");
let signed = signer.sign(b"Hello, World!", header).unwrap();

// Verify
let trusted_roots = vec![ca.public_key()];
let result = verify(&signed, &trusted_roots).unwrap();
println!("Signed by: {}", result.creator_name);
```

## File Format

Aletheia files (`.alx`) use a binary format:

```
ALETHEIA (magic)  | 8 bytes
Version           | 2 bytes (major.minor)
Flags             | 2 bytes (compression, etc.)
Header length     | 4 bytes
Header            | CBOR-encoded metadata
Payload length    | 8 bytes
Payload           | Raw or compressed data
Cert chain length | 4 bytes
Certificate chain | CBOR-encoded certificates
Signature         | 64 bytes (Ed25519)
```

See [SPECIFICATION.md](SPECIFICATION.md) for full details.

## Security

- **Ed25519** signatures (128-bit security level)
- **CBOR** encoding for compact binary representation
- **Zstd** compression (optional)
- Certificate chain validation
- No expiration - signatures are permanent

## Use Cases

- **Artists**: Sign original artwork to prove human creation
- **Journalists**: Sign articles to prove authorship
- **Researchers**: Sign papers and datasets
- **Legal**: Sign documents with permanent attribution
- **Software**: Sign releases to prove trusted origin

## License

MIT
