# Aletheia File Viewer

A browser-only web application for visualizing and cryptographically verifying Aletheia (.alx) files.

## 🔒 Security-First Design

**CRITICAL RULE: NO PAYLOAD PREVIEW BEFORE VERIFICATION**

The application enforces a strict security policy:
- Payload content is **NEVER** rendered until cryptographic verification succeeds
- This prevents malicious content from exploiting rendering vulnerabilities
- The Payload tab is completely disabled until verification passes

## Features

### Core Functionality
- ✅ **100% Browser-Only** - No backend required, runs entirely in WASM + JavaScript
- ✅ **Drag & Drop** - Intuitive file loading with visual feedback
- ✅ **Trusted Roots** - Import one or more CA certificates for verification
- ✅ **Complete Verification** - Multi-step verification with clear status reporting
- ✅ **Interactive Hex Viewer** - Clickable hex dump with byte range highlighting
- ✅ **Certificate Chain Display** - Visual representation of trust chain
- ✅ **Safe Payload Preview** - Only enabled after successful verification

### User Interface

#### Split View Layout
- **Left Panel**: File structure tree showing:
  - Magic bytes (ALETHEIA)
  - Version (major.minor)
  - Flags (compression, etc.)
  - Header (CBOR-encoded metadata)
  - Payload (actual content)
  - Certificate Chain (trust chain)
  - Signature (Ed25519, 64 bytes)

- **Right Panel**: Tabbed detail view:
  - **Hex View**: Interactive hex dump with byte highlighting
  - **Header**: Pretty-printed CBOR metadata
  - **Certificates**: Vertical chain visualization with copy buttons
  - **Payload**: Safe content preview (images, text, or download)

#### Verification Panel
Always visible at the top, showing:
1. **Large Status Banner**: VERIFIED ✓ or NOT VERIFIED ✗
2. **Step-by-step Checklist**:
   - ✓ Parse file structure
   - ✓ Decode CBOR (header + certificates)
   - ✓ Verify creator ID matches certificate
   - ✓ Verify certificate chain signatures
   - ✓ Verify root is trusted
   - ✓ Verify file signature
3. **Clear Error Messages**: Specific failure reasons when verification fails

## Tech Stack

- **Build Tool**: Vite 7
- **Framework**: Vue 3 (Composition API)
- **Language**: TypeScript
- **Components**: Single File Components (SFC) with `<script setup lang="ts">`
- **Styling**: Tailwind CSS 4
- **Cryptography**: Rust + WASM (Ed25519 via `ed25519-dalek`)
- **Encoding**: CBOR via Rust (`ciborium`)
- **Compression**: LZ4 via Rust (`lz4_flex`)
- **Testing**: Vitest with happy-dom

## Architecture

### WASM Integration
The application uses a Rust library compiled to WebAssembly for:
- Binary parsing with exact byte range tracking
- CBOR encoding/decoding
- Ed25519 signature verification
- Certificate chain validation
- LZ4 decompression

This ensures:
- ✅ Zero code duplication (same verification logic as CLI)
- ✅ Memory-safe parsing (Rust prevents buffer overflows)
- ✅ Native performance for cryptographic operations
- ✅ Consistent behavior across platforms

### Critical Correctness Guarantees

#### Signature Input Extraction
The `signature_input` is **EXACTLY** all bytes of the .alx file **EXCEPT** the final 64 bytes.

```typescript
const fileData = new Uint8Array(1000)  // Example file
const signatureInput = fileData.slice(0, fileData.length - 64)  // Bytes 0-935
const signature = fileData.slice(fileData.length - 64)          // Bytes 936-999
```

This is **critical** for Ed25519 verification correctness.

#### Certificate Chain Verification
1. **Chain Order**: Creator → Intermediate(s) → Root CA
2. **Signature Verification**: Each cert must be signed by the next cert's public key
3. **Root Trust**: Final cert must be self-signed AND match a trusted root
4. **CA Validation**:
   - Creator cert must NOT be a CA (is_ca: false)
   - Intermediate/root certs MUST be CAs (is_ca: true)

#### Byte Range Tracking
All major fields track their byte ranges for hex highlighting:
- `magicRange`: [0, 8)
- `versionRange`: [8, 10)
- `flagsRange`: [10, 12)
- `headerRange`: [12, 12+4+header_len)
- `payloadRange`: [offset, offset+payload_len)
- `certChainRange`: [offset, offset+chain_len)
- `signatureRange`: [file_len-64, file_len)

Ranges are:
- ✅ Non-overlapping
- ✅ Sequential (no gaps)
- ✅ Verified by unit tests

## Building

### Prerequisites
```bash
# Install Rust and wasm-pack
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
cargo install wasm-pack

# Install Node.js 18+ and npm
```

### Build WASM Module
```bash
# From repository root
wasm-pack build --target web --out-dir aletheia-viewer/src/lib/wasm-pkg
```

This generates:
- `aletheia.js` - JavaScript bindings
- `aletheia.d.ts` - TypeScript definitions
- `aletheia_bg.wasm` - WebAssembly binary (~234KB)

### Build Web App
```bash
cd aletheia-viewer

# Install dependencies
npm install

# Development server
npm run dev

# Production build
npm run build

# Run tests
npm test
```

## Usage

### 1. Open the Application
Navigate to http://localhost:5173 (dev) or serve the `dist/` folder (production).

### 2. Load Trusted Root(s)
Click **"Load Trusted Root(s)"** and select one or more CA certificate files:
- Accepted formats: `.cert` files (CBOR-encoded certificates)
- Multiple roots supported for different trust hierarchies
- Indicator shows: "✓ N trusted root(s) loaded"

### 3. Open .alx File
Click **"Open .alx File"** or drag & drop an `.alx` file onto the button.

### 4. View Verification Results
The verification panel shows:
- **VERIFIED**: All checks passed, payload is safe to view
- **NOT VERIFIED**: One or more checks failed, payload preview disabled

### 5. Explore the File
- **File Structure**: Click nodes to highlight corresponding hex bytes
- **Hex View**: Click bytes to select the containing structure
- **Header**: View metadata (creator, timestamp, content type, etc.)
- **Certificates**: Examine the trust chain, copy public keys
- **Payload**: Only available after verification (images, text, download)

## Testing

The project includes comprehensive unit tests:

```bash
npm test
```

### Test Coverage
- ✅ Utility functions (hex conversion, formatting, content type detection)
- ✅ Byte range validation (non-overlapping, sequential)
- ✅ Signature input extraction (exact byte slicing)
- ✅ Certificate chain logic (ordering, CA validation)
- ✅ Edge cases (empty files, truncated files, large files)

**Note**: WASM module tests require special configuration. The Rust code has its own test suite:
```bash
cargo test
```

## Performance Considerations

### Large Files
The application handles large files (~50MB) efficiently:

1. **Hex View**: Virtual scrolling renders only 50 visible lines
2. **Payload**: Decompression on-demand via WASM
3. **Text Preview**: 1MB size limit to prevent browser freezing
4. **Images**: Rendered via Blob URLs (browser-native decoding)

### Memory Usage
- WASM parser uses subarray/slices (no unnecessary copying)
- Raw file bytes kept in memory once for hex view
- Parsed structure shares references where possible

## File Format Reference

### Structure
```
┌─────────────────┬──────┐
│ Magic Bytes     │  8 B │  "ALETHEIA"
├─────────────────┼──────┤
│ Version         │  2 B │  major, minor
├─────────────────┼──────┤
│ Flags           │  2 B │  compression, etc.
├─────────────────┼──────┤
│ Header Length   │  4 B │  u32 LE
├─────────────────┼──────┤
│ Header (CBOR)   │  var │  metadata
├─────────────────┼──────┤
│ Payload Length  │  8 B │  u64 LE
├─────────────────┼──────┤
│ Payload         │  var │  actual content
├─────────────────┼──────┤
│ Cert Chain Len  │  4 B │  u32 LE
├─────────────────┼──────┤
│ Cert Chain      │  var │  CBOR array
├─────────────────┼──────┤
│ Signature       │ 64 B │  Ed25519
└─────────────────┴──────┘
```

### Header CBOR
Required fields:
- `creator_id`: string (matches first cert's subject_id)
- `signed_at`: i64 (Unix timestamp)

Optional fields:
- `content_type`: string (MIME type)
- `original_name`: string (source filename)
- `description`: string (content description)
- `custom`: map (arbitrary metadata)

### Certificate CBOR
Each certificate contains:
- `version`: u8
- `serial`: bytes (unique identifier)
- `subject_id`: string (e.g., email)
- `subject_name`: string (human-readable)
- `public_key`: bytes[32] (Ed25519)
- `issuer_id`: string (who signed this)
- `issued_at`: i64 (Unix timestamp)
- `is_ca`: bool (can sign other certs)
- `signature`: bytes[64] (Ed25519)

## Security Notes

### Threat Model
This application is designed to prevent:
- ✅ **Malicious Payload Rendering**: Payload hidden until verification
- ✅ **Certificate Chain Attacks**: Full chain validation required
- ✅ **TOCTOU Attacks**: Single verification pass, no re-checks
- ✅ **Buffer Overflows**: Memory-safe Rust parser

### Trusted Roots Management
- Load trusted roots from **separate, known-good files**
- Verify root fingerprints out-of-band (e.g., via secure channel)
- Root certificates are **never** extracted from .alx files

### Limitations
- ⚠️ Browser sandbox: No filesystem access (by design)
- ⚠️ Timestamp trust: Relies on signer's clock
- ⚠️ Revocation: No CRL/OCSP support (intentional for offline use)

## Browser Compatibility

- Chrome/Edge 90+
- Firefox 88+
- Safari 15+

Requirements:
- WebAssembly support
- JavaScript ES2020
- SubtleCrypto API (for SHA-256 fingerprints)

## License

See repository LICENSE file.

## Contributing

This application prioritizes correctness and security over features. Changes must:
1. Maintain the "no preview before verify" guarantee
2. Include tests for critical paths
3. Preserve byte-exact parsing semantics
4. Not weaken cryptographic verification

---

**Remember**: Never preview untrusted payload content before cryptographic verification succeeds. This is not just a feature—it's the entire point of Aletheia.
