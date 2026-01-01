//! # Aletheia
//!
//! Cryptographic proof of human-created content authenticity.
//!
//! Aletheia provides a signed envelope format (`.alx`) that proves:
//! - **Who** created or approved the content (authentication)
//! - **That** the content hasn't been modified (integrity)
//! - Content originates from a verified human identity (attestation)
//!
//! ## Quick Start
//!
//! ### Setting up a Certificate Authority
//!
//! ```rust
//! use aletheia::ca::CertificateAuthority;
//!
//! // Create a root CA (do this once, protect the keys!)
//! let ca = CertificateAuthority::new_root(
//!     "root@example.com",
//!     "Example Root CA"
//! );
//! ```
//!
//! ### Issuing a Certificate to a User
//!
//! ```rust
//! use aletheia::ca::{CertificateAuthority, SigningKeyPair};
//!
//! let ca = CertificateAuthority::new_root("root@example.com", "Root CA");
//!
//! // User generates their key pair
//! let user_keys = SigningKeyPair::generate();
//!
//! // CA issues a certificate binding the user's identity to their key
//! let user_cert = ca.issue_certificate(
//!     "alice@example.com",
//!     "Alice Smith",
//!     &user_keys.public_key(),
//!     false, // not a CA
//! ).unwrap();
//! ```
//!
//! ### Signing Content
//!
//! ```rust
//! use aletheia::{ca::{CertificateAuthority, SigningKeyPair}, signer::Signer, Header};
//!
//! let ca = CertificateAuthority::new_root("root@example.com", "Root CA");
//! let user_keys = SigningKeyPair::generate();
//! let user_cert = ca.issue_certificate(
//!     "alice@example.com", "Alice", &user_keys.public_key(), false
//! ).unwrap();
//!
//! // Build certificate chain: [user_cert, root_cert]
//! let chain = vec![user_cert, ca.certificate.clone()];
//!
//! // Create signer
//! let signer = Signer::new(user_keys, chain).unwrap();
//!
//! // Sign any data
//! let content = b"This is my original artwork";
//! let header = Header::new("alice@example.com")
//!     .with_content_type("image/png")
//!     .with_description("Original digital art");
//!
//! let signed_file = signer.sign(content, header).unwrap();
//!
//! // Save to .alx file
//! aletheia::file::write_to_file(&signed_file, "artwork.alx").unwrap();
//! ```
//!
//! ### Verifying Content
//!
//! ```rust,no_run
//! use aletheia::{file::read_from_file, verifier::verify};
//!
//! // Load the .alx file
//! let file = read_from_file("artwork.alx").unwrap();
//!
//! // Verify against trusted root CAs
//! let trusted_roots = vec![/* root CA public keys */];
//! let result = verify(&file, &trusted_roots).unwrap();
//!
//! println!("Created by: {} ({})", result.creator_name, result.creator_id);
//! println!("Signed at: {}", result.signed_at);
//! ```

mod error;
mod types;

pub mod ca;
pub mod certificate;
pub mod file;
pub mod signer;
pub mod verifier;

#[cfg(target_arch = "wasm32")]
pub mod wasm;

pub use error::{AletheiaError, Result};
pub use types::{
    AletheiaFile, Certificate, Flags, Header, MAGIC_BYTES, VERSION_MAJOR, VERSION_MINOR,
};
