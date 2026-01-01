# Contributing to Aletheia

Thank you for your interest in contributing to Aletheia! This document outlines our development workflow and guidelines.

## Branch Strategy

We use a **trunk-based development** workflow with the following branch types:

### Main Branch

- `main` - Production-ready code
- Protected branch
- All changes must go through pull requests
- CI must pass before merging

### Feature Branches

Format: `feature/<description>`

Examples:
- `feature/add-revocation-support`
- `feature/batch-signing`
- `feature/certificate-expiry`

Use for:
- New features
- Enhancements
- Non-breaking changes

### Fix Branches

Format: `fix/<description>`

Examples:
- `fix/signature-validation-bug`
- `fix/memory-leak`
- `fix/wasm-compatibility`

Use for:
- Bug fixes
- Security fixes
- Patches

### Release Branches

Format: `release/v<version>`

Examples:
- `release/v0.2.0`
- `release/v1.0.0`

Use for:
- Preparing releases
- Version bumps
- Release notes
- Last-minute fixes

## Pull Request Workflow

### 1. Create a Branch

```bash
# Feature
git checkout -b feature/my-feature

# Fix
git checkout -b fix/my-fix

# Release
git checkout -b release/v0.2.0
```

### 2. Make Changes

Follow our coding standards:
- Run `cargo fmt` before committing
- Ensure `cargo clippy` passes
- Add tests for new functionality
- Update documentation

### 3. Run CI Locally

```bash
# Format check
cargo fmt --all -- --check

# Clippy
cargo clippy --all-features -- -D warnings

# Tests
cargo test --all-features

# WASM build
cargo build --target wasm32-unknown-unknown --no-default-features --features wasm,compression
```

### 4. Commit Changes

Use conventional commits format:

```bash
git commit -m "feat: add batch signing support"
git commit -m "fix: resolve signature validation bug"
git commit -m "docs: update README with examples"
git commit -m "test: add tests for certificate chain"
```

Types:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation
- `test`: Tests
- `refactor`: Code refactoring
- `perf`: Performance improvement
- `chore`: Maintenance

### 5. Push and Create PR

```bash
git push origin feature/my-feature
```

Then create a pull request on GitHub:
- Use a clear, descriptive title
- Fill out the PR template
- Link related issues
- Request reviews from maintainers

### 6. CI Checks

The following checks must pass:

✅ **Check** - Cargo check passes
✅ **Test Suite** - All tests pass on Linux, Windows, macOS
✅ **Rustfmt** - Code is formatted correctly
✅ **Clippy** - No linting warnings
✅ **WebAssembly** - Builds for WASM target
✅ **CLI Build** - CLI builds on all platforms
✅ **Security Audit** - No known vulnerabilities
✅ **Documentation** - Docs build successfully

### 7. Review Process

- At least one approval required
- All CI checks must pass
- No merge conflicts
- Squash and merge to main

## Development Setup

### Prerequisites

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Add WASM target
rustup target add wasm32-unknown-unknown

# Install tools
cargo install cargo-tarpaulin  # Coverage
cargo install wasm-pack        # WASM
```

### Build and Test

```bash
# Full build
cargo build --all-features

# Run tests
cargo test

# Run specific test
cargo test test_name

# Build CLI
cargo build --bin aletheia --features cli

# Build for WASM
cargo build --target wasm32-unknown-unknown --no-default-features --features wasm
```

### Code Style

We follow standard Rust conventions:

```rust
// Use explicit types when helpful
let payload: Vec<u8> = data.to_vec();

// Document public APIs
/// Sign data and create an Aletheia file
pub fn sign(&self, data: &[u8]) -> Result<AletheiaFile> {
    // ...
}

// Keep functions focused and small
// Prefer functional style when clear
// Use descriptive variable names
```

### Testing

- Write unit tests for all new functionality
- Add integration tests for features
- Test edge cases and error conditions
- Ensure tests work on all platforms

Example:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sign_and_verify() {
        let ca = CertificateAuthority::new_root("test", "Test CA");
        // ...
    }

    #[cfg(feature = "compression")]
    #[test]
    fn test_compression() {
        // ...
    }
}
```

## Release Process

### Version Numbering

We follow [Semantic Versioning](https://semver.org/):

- `MAJOR.MINOR.PATCH`
- `1.0.0` - Breaking changes
- `0.2.0` - New features
- `0.1.1` - Bug fixes

### Creating a Release

1. Create release branch:
   ```bash
   git checkout -b release/v0.2.0
   ```

2. Update version in `Cargo.toml`

3. Update `CHANGELOG.md` (if exists)

4. Create PR to main

5. After merge, create GitHub release:
   - Tag: `v0.2.0`
   - Title: `v0.2.0 - Release Name`
   - Include changelog

6. Publish to crates.io:
   ```bash
   cargo publish
   ```

## Questions?

- Open an issue for bugs or feature requests
- Start a discussion for questions
- Join our community chat (if available)

## Code of Conduct

Be respectful, inclusive, and professional. We're all here to build something great together.

---

Thank you for contributing to Aletheia! 🎉
