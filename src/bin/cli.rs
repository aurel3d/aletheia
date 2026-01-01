use aletheia::{
    ca::{CertificateAuthority, SigningKeyPair},
    file::{read_from_file, write_to_file},
    signer::Signer,
    verifier::{verify, VerificationResult},
    Certificate, Header,
};
use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "aletheia")]
#[command(author, version, about = "Cryptographic proof of human-created content authenticity")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new Certificate Authority
    #[command(name = "ca-init")]
    CaInit {
        /// CA identifier (e.g., email or organization name)
        #[arg(short, long)]
        id: String,

        /// Human-readable CA name
        #[arg(short, long)]
        name: String,

        /// Output directory for CA files
        #[arg(short, long, default_value = ".")]
        output: PathBuf,
    },

    /// Issue a certificate to a user
    #[command(name = "cert-issue")]
    CertIssue {
        /// CA private key file
        #[arg(long)]
        ca_key: PathBuf,

        /// CA certificate file
        #[arg(long)]
        ca_cert: PathBuf,

        /// Subject identifier (e.g., email)
        #[arg(short, long)]
        id: String,

        /// Subject human-readable name
        #[arg(short, long)]
        name: String,

        /// Output directory for user files
        #[arg(short, long, default_value = ".")]
        output: PathBuf,

        /// Issue a CA certificate (can sign other certificates)
        #[arg(long, default_value = "false")]
        is_ca: bool,
    },

    /// Generate a new key pair
    #[command(name = "keygen")]
    KeyGen {
        /// Output directory for key files
        #[arg(short, long, default_value = ".")]
        output: PathBuf,

        /// Prefix for output files
        #[arg(short, long, default_value = "key")]
        prefix: String,
    },

    /// Sign a file
    Sign {
        /// File to sign
        #[arg(short, long)]
        input: PathBuf,

        /// Output .alx file (defaults to input + .alx)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Signer's private key file
        #[arg(long)]
        key: PathBuf,

        /// Signer's certificate file
        #[arg(long)]
        cert: PathBuf,

        /// CA certificate file (root of trust)
        #[arg(long)]
        ca_cert: PathBuf,

        /// Content type (MIME type)
        #[arg(long)]
        content_type: Option<String>,

        /// Description of the content
        #[arg(long)]
        description: Option<String>,

        /// Enable compression
        #[arg(long, default_value = "false")]
        compress: bool,
    },

    /// Verify a signed .alx file
    Verify {
        /// The .alx file to verify
        file: PathBuf,

        /// Trusted CA certificate file(s)
        #[arg(long, required = true)]
        trust: Vec<PathBuf>,

        /// Output the payload to a file
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Show detailed information
        #[arg(short, long, default_value = "false")]
        verbose: bool,
    },

    /// Show information about an .alx file without verification
    Info {
        /// The .alx file to inspect
        file: PathBuf,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::CaInit { id, name, output } => cmd_ca_init(&id, &name, &output),
        Commands::CertIssue {
            ca_key,
            ca_cert,
            id,
            name,
            output,
            is_ca,
        } => cmd_cert_issue(&ca_key, &ca_cert, &id, &name, &output, is_ca),
        Commands::KeyGen { output, prefix } => cmd_keygen(&output, &prefix),
        Commands::Sign {
            input,
            output,
            key,
            cert,
            ca_cert,
            content_type,
            description,
            compress,
        } => cmd_sign(
            &input,
            output.as_deref(),
            &key,
            &cert,
            &ca_cert,
            content_type.as_deref(),
            description.as_deref(),
            compress,
        ),
        Commands::Verify {
            file,
            trust,
            output,
            verbose,
        } => cmd_verify(&file, &trust, output.as_deref(), verbose),
        Commands::Info { file } => cmd_info(&file),
    }
}

fn cmd_ca_init(id: &str, name: &str, output: &PathBuf) -> Result<()> {
    std::fs::create_dir_all(output)?;

    let ca = CertificateAuthority::new_root(id, name);

    // Save private key
    let key_path = output.join("ca.key");
    let key_hex = hex::encode(ca.private_key_bytes());
    std::fs::write(&key_path, &key_hex)?;
    println!("CA private key saved to: {}", key_path.display());

    // Save certificate
    let cert_path = output.join("ca.cert");
    let mut cert_bytes = Vec::new();
    ciborium::into_writer(&ca.certificate, &mut cert_bytes)?;
    let cert_b64 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &cert_bytes);
    std::fs::write(&cert_path, &cert_b64)?;
    println!("CA certificate saved to: {}", cert_path.display());

    println!("\nCA initialized successfully!");
    println!("  ID:   {}", id);
    println!("  Name: {}", name);
    println!("\nIMPORTANT: Keep ca.key secure! Anyone with this key can issue certificates.");

    Ok(())
}

fn cmd_cert_issue(
    ca_key_path: &PathBuf,
    ca_cert_path: &PathBuf,
    subject_id: &str,
    subject_name: &str,
    output: &PathBuf,
    is_ca: bool,
) -> Result<()> {
    // Load CA
    let ca_key_hex = std::fs::read_to_string(ca_key_path)
        .context("Failed to read CA key file")?;
    let ca_key_bytes = hex::decode(ca_key_hex.trim())
        .context("Invalid CA key format")?;

    let ca_cert = load_certificate(ca_cert_path)?;
    let ca = CertificateAuthority::from_key_and_cert(&ca_key_bytes, ca_cert)
        .context("Failed to load CA")?;

    // Generate user key pair
    let user_keys = SigningKeyPair::generate();

    // Issue certificate
    let user_cert = ca
        .issue_certificate(subject_id, subject_name, &user_keys.public_key(), is_ca)
        .context("Failed to issue certificate")?;

    std::fs::create_dir_all(output)?;

    // Save user private key
    let key_path = output.join(format!("{}.key", sanitize_filename(subject_id)));
    let key_hex = hex::encode(user_keys.private_key_bytes());
    std::fs::write(&key_path, &key_hex)?;
    println!("Private key saved to: {}", key_path.display());

    // Save user certificate
    let cert_path = output.join(format!("{}.cert", sanitize_filename(subject_id)));
    save_certificate(&user_cert, &cert_path)?;
    println!("Certificate saved to: {}", cert_path.display());

    println!("\nCertificate issued successfully!");
    println!("  Subject ID:   {}", subject_id);
    println!("  Subject Name: {}", subject_name);
    println!("  Is CA:        {}", is_ca);
    println!("  Issuer:       {}", ca.certificate.subject_id);

    Ok(())
}

fn cmd_keygen(output: &PathBuf, prefix: &str) -> Result<()> {
    std::fs::create_dir_all(output)?;

    let keys = SigningKeyPair::generate();

    // Save private key
    let key_path = output.join(format!("{}.key", prefix));
    let key_hex = hex::encode(keys.private_key_bytes());
    std::fs::write(&key_path, &key_hex)?;
    println!("Private key saved to: {}", key_path.display());

    // Save public key
    let pub_path = output.join(format!("{}.pub", prefix));
    let pub_hex = hex::encode(keys.public_key());
    std::fs::write(&pub_path, &pub_hex)?;
    println!("Public key saved to: {}", pub_path.display());

    println!("\nKey pair generated successfully!");

    Ok(())
}

fn cmd_sign(
    input: &PathBuf,
    output: Option<&std::path::Path>,
    key_path: &PathBuf,
    cert_path: &PathBuf,
    ca_cert_path: &PathBuf,
    content_type: Option<&str>,
    description: Option<&str>,
    compress: bool,
) -> Result<()> {
    // Load signing key
    let key_hex = std::fs::read_to_string(key_path)
        .context("Failed to read private key file")?;
    let key_bytes = hex::decode(key_hex.trim())
        .context("Invalid key format")?;
    let signing_key = SigningKeyPair::from_bytes(&key_bytes)
        .context("Failed to load signing key")?;

    // Load certificates
    let user_cert = load_certificate(cert_path)?;
    let ca_cert = load_certificate(ca_cert_path)?;

    // Build certificate chain
    let chain = vec![user_cert.clone(), ca_cert];

    // Create signer
    let mut signer = Signer::new(signing_key, chain)
        .context("Failed to create signer")?;
    if compress {
        signer = signer.with_compression();
    }

    // Read input file
    let payload = std::fs::read(input)
        .context("Failed to read input file")?;

    // Build header
    let mut header = Header::new(&user_cert.subject_id);
    if let Some(ct) = content_type {
        header = header.with_content_type(ct);
    }
    if let Some(desc) = description {
        header = header.with_description(desc);
    }
    if let Some(name) = input.file_name().and_then(|n| n.to_str()) {
        header = header.with_original_name(name);
    }

    // Sign
    let signed_file = signer.sign(&payload, header)
        .context("Failed to sign file")?;

    // Determine output path
    let output_path = output
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| {
            let mut p = input.clone();
            let new_name = format!(
                "{}.alx",
                p.file_name().unwrap_or_default().to_string_lossy()
            );
            p.set_file_name(new_name);
            p
        });

    // Write output
    write_to_file(&signed_file, &output_path)
        .context("Failed to write output file")?;

    println!("Signed file created: {}", output_path.display());
    println!("  Creator:     {} ({})", user_cert.subject_name, user_cert.subject_id);
    println!("  Compressed:  {}", compress);
    println!("  Payload:     {} bytes", payload.len());

    Ok(())
}

fn cmd_verify(
    file: &PathBuf,
    trust_paths: &[PathBuf],
    output: Option<&std::path::Path>,
    verbose: bool,
) -> Result<()> {
    // Load trusted roots
    let mut trusted_roots = Vec::new();
    for path in trust_paths {
        let cert = load_certificate(path)
            .with_context(|| format!("Failed to load trusted cert: {}", path.display()))?;
        trusted_roots.push(cert.public_key);
    }

    // Load the .alx file
    let alx_file = read_from_file(file)
        .context("Failed to read .alx file")?;

    // Verify
    match verify(&alx_file, &trusted_roots) {
        Ok(result) => {
            print_verification_success(&result, verbose);

            // Extract payload if requested
            if let Some(out_path) = output {
                let payload = alx_file.get_payload()
                    .context("Failed to decompress payload")?;
                std::fs::write(out_path, &payload)
                    .context("Failed to write output file")?;
                println!("\nPayload extracted to: {}", out_path.display());
            }

            Ok(())
        }
        Err(e) => {
            println!("VERIFICATION FAILED");
            println!("  Error: {}", e);
            bail!("Verification failed: {}", e);
        }
    }
}

fn cmd_info(file: &PathBuf) -> Result<()> {
    let alx_file = read_from_file(file)
        .context("Failed to read .alx file")?;

    println!("Aletheia File Information");
    println!("=========================");
    println!("File:          {}", file.display());
    println!("Version:       {}.{}", alx_file.version_major, alx_file.version_minor);
    println!("Compressed:    {}", alx_file.flags.is_compressed());
    println!();
    println!("Header:");
    println!("  Creator ID:  {}", alx_file.header.creator_id);
    println!("  Signed at:   {}", format_timestamp(alx_file.header.signed_at));
    if let Some(ct) = &alx_file.header.content_type {
        println!("  Content-Type: {}", ct);
    }
    if let Some(name) = &alx_file.header.original_name {
        println!("  Original name: {}", name);
    }
    if let Some(desc) = &alx_file.header.description {
        println!("  Description: {}", desc);
    }
    println!();
    println!("Payload:       {} bytes", alx_file.payload.len());
    if alx_file.flags.is_compressed() {
        if let Ok(decompressed) = alx_file.get_payload() {
            println!("  (decompressed: {} bytes)", decompressed.len());
        }
    }
    println!();
    println!("Certificate Chain ({} certificates):", alx_file.certificate_chain.len());
    for (i, cert) in alx_file.certificate_chain.iter().enumerate() {
        let role = if i == 0 { "Creator" } else if cert.is_ca { "CA" } else { "Intermediate" };
        println!("  [{}] {} - {} ({})", i, role, cert.subject_name, cert.subject_id);
        println!("      Issued by: {}", cert.issuer_id);
        println!("      Issued at: {}", format_timestamp(cert.issued_at));
    }

    Ok(())
}

// Helper functions

fn load_certificate(path: &PathBuf) -> Result<Certificate> {
    let content = std::fs::read_to_string(path)
        .context("Failed to read certificate file")?;
    let bytes = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, content.trim())
        .context("Invalid certificate format (not base64)")?;
    let cert: Certificate = ciborium::from_reader(&bytes[..])
        .context("Invalid certificate format (not valid CBOR)")?;
    Ok(cert)
}

fn save_certificate(cert: &Certificate, path: &PathBuf) -> Result<()> {
    let mut bytes = Vec::new();
    ciborium::into_writer(cert, &mut bytes)?;
    let b64 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &bytes);
    std::fs::write(path, &b64)?;
    Ok(())
}

fn sanitize_filename(s: &str) -> String {
    s.chars()
        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
        .collect()
}

fn format_timestamp(ts: i64) -> String {
    chrono::DateTime::from_timestamp(ts, 0)
        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S UTC").to_string())
        .unwrap_or_else(|| ts.to_string())
}

fn print_verification_success(result: &VerificationResult, verbose: bool) {
    println!("VERIFIED");
    println!("  Creator: {} ({})", result.creator_name, result.creator_id);
    println!("  Signed:  {}", format_timestamp(result.signed_at));
    if let Some(desc) = &result.description {
        println!("  Description: {}", desc);
    }
    if verbose {
        println!("\n  This content was signed by a verified human identity.");
        println!("  The signature is valid and the certificate chain is trusted.");
    }
}
