# Aletheia Agent

This document explains the role of the Aletheia project, the components that act as the "agent" performing signing and verification, and how to work with them across Rust, WASM, and the web viewer.

## Purpose
- Provide cryptographic proof that content was created or approved by a trusted human identity.
- Package content, metadata, certificates, and a signature into a compact `.alx` envelope.
- Allow offline, permanent verification with no expiry on signatures.

## Components at a Glance
- Core Rust library in [src](src) exports types and APIs for signing, verification, and file I/O; entry surface in [src/lib.rs](src/lib.rs).
- CLI (feature `cli`) in [src/bin/cli.rs](src/bin/cli.rs) offers commands to init a CA, issue certs, sign, verify, and inspect files.
- WASM bindings in [src/wasm.rs](src/wasm.rs) expose the same logic to the browser.
- Browser viewer in [aletheia-viewer](aletheia-viewer) consumes the WASM package to visualize `.alx` files safely.
- Formal format reference in [SPECIFICATION.md](SPECIFICATION.md); quick narrative in [README.md](README.md).

## Core Responsibilities (the "agent" duties)
- **Identity & Trust**: issue Ed25519-backed certificates via the CA module and validate chains (creator → intermediates → root).
- **Signing**: build deterministic signature input over magic bytes, version, flags, header, payload, and certificate chain, then sign with the creator key.
- **Verification**: parse `.alx`, validate structure, check certificate chain against trusted roots, and verify the Ed25519 signature.
- **Packaging**: emit a binary envelope with CBOR-encoded header and cert chain; optionally LZ4-compress payload when the `compression` feature is enabled.

## Key Data Structures
- `AletheiaFile`: in-memory representation of an `.alx` file (version, flags, header, payload, cert chain, signature).
- `Header`: CBOR-serializable metadata (content type, creator ID, timestamps, descriptions, optional custom map).
- `Certificate`: CBOR-encoded identity attestations, signed by issuers; chains end in a trusted root CA.
- `Flags`: bitfield describing payload compression and future extensions.

## Critical Invariants
- Certificate chain order: creator first, root last; root must be trusted and self-signed; creator cert must not be CA; intermediates must be CA.
- Signature input: every byte of the file except the final 64-byte signature; any change in header/payload/certs invalidates the signature.
- Compression flag must match payload encoding; decompression is gated behind the `compression` feature.
- Header `creator_id` must match the subject of the first certificate.

## Typical Flows
1. **Initialize a root CA**: generate key pair and self-signed root certificate.
2. **Issue a user certificate**: CA signs user public key and identity metadata.
3. **Sign content**: build header, optionally compress payload, compute signature, emit `.alx` file.
4. **Verify content**: load `.alx`, validate structure, verify chain against trusted roots, verify signature, then (optionally) render payload.

## Developer Workflow
- **Build/Install CLI**: `cargo install --path .` or `cargo build --release` (uses features `std` and `compression` by default).
- **Feature flags**: `std` (default), `compression` (default), `cli`, `wasm`. Combine as needed; for WASM builds disable `std` and enable `wasm`.
- **WASM build**: from repo root, `wasm-pack build --target web --out-dir aletheia-viewer/src/lib/wasm-pkg` (see [aletheia-viewer/README.md](aletheia-viewer/README.md)).
- **Web viewer**: in `aletheia-viewer`, run `npm install`, `npm run dev`, or `npm run build`; the app enforces "no payload preview before verification" for safety.
- **Tests**: Rust unit tests via `cargo test`; viewer tests via `npm test` inside `aletheia-viewer`.

## Security Posture
- Ed25519 signatures; CBOR encoding; optional LZ4 compression.
- No expiry on signatures; rely on trust roots and (optional) revocation processes external to the format.
- Viewer deliberately withholds payload rendering until verification succeeds to avoid executing malicious content.

## Extending the Agent
- New metadata: add optional fields to `Header` and update schema consumers; maintain CBOR compatibility.
- Additional flags: reserve unused bits in `Flags` and document semantics.
- Alternative UIs: reuse WASM bindings for other frontends while keeping the Rust core as the single source of truth.

## Quick References
- File format layout and field sizes: [SPECIFICATION.md](SPECIFICATION.md).
- Core API usage samples: [README.md](README.md) and module docs in [src/lib.rs](src/lib.rs).
- Viewer behavior and UX rules: [aletheia-viewer/README.md](aletheia-viewer/README.md).

## PKI Portal Implementation Choices
- Stack: Rust (Actix Web) + Postgres + sqlx for async DB access; TLS everywhere.
- Auth: OAuth2/OIDC; JWT validation middleware with scoped roles (admin/operator/auditor/api-client).
- Key management: HSM/KMS-backed keys for roots and intermediates; signing endpoints call KMS, never export private keys.
- API shape: aligned to [docs/pki-portal-openapi.yaml](docs/pki-portal-openapi.yaml) with handlers for roots, intermediates, issuance, revocation, bundles, policy, audit.
- Observability: tracing-based structured logs, request IDs, metrics endpoint, health checks.
- Deployment: containerized service; run DB migrations on startup; separate stage/prod KMS keys and trust bundles.
