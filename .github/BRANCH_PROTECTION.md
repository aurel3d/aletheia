# Branch Protection Rules

This document describes the branch protection rules that should be configured for the `main` branch.

## Configuring Branch Protection

Go to: `Settings` → `Branches` → `Add branch protection rule`

### Branch Name Pattern

```
main
```

## Required Settings

### ✅ Require a pull request before merging

- [x] **Require approvals: 1**
  - At least one approval from a maintainer is required

- [x] **Dismiss stale pull request approvals when new commits are pushed**
  - Ensures reviewers see the latest changes

- [x] **Require review from Code Owners** (optional)
  - If CODEOWNERS file is configured

### ✅ Require status checks to pass before merging

- [x] **Require branches to be up to date before merging**
  - Prevents merge conflicts

**Required status checks:**
- ✅ `Check`
- ✅ `Test Suite (ubuntu-latest)`
- ✅ `Test Suite (windows-latest)`
- ✅ `Test Suite (macos-latest)`
- ✅ `Rustfmt`
- ✅ `Clippy`
- ✅ `WebAssembly`
- ✅ `CLI Build (ubuntu-latest)`
- ✅ `CLI Build (windows-latest)`
- ✅ `CLI Build (macos-latest)`
- ✅ `Security Audit`
- ✅ `Documentation`

### ✅ Require conversation resolution before merging

- [x] All conversations must be resolved

### ✅ Require signed commits (recommended)

- [x] Requires commits to be signed with GPG

### ✅ Require linear history

- [x] Prevent merge commits
- Use "Squash and merge" or "Rebase and merge"

### ✅ Include administrators

- [x] Enforce rules for administrators too

### ❌ Allow force pushes

- [ ] **Disabled** - Protects history

### ❌ Allow deletions

- [ ] **Disabled** - Prevents accidental deletion

## Merge Methods

Configure allowed merge methods in: `Settings` → `General` → `Pull Requests`

Recommended settings:
- [ ] Allow merge commits (disabled)
- [x] **Allow squash merging** (enabled, default)
- [x] Allow rebase merging (enabled)

## Auto-merge

- [x] Enable auto-merge when all checks pass (optional)

## Recommended: CODEOWNERS File

Create `.github/CODEOWNERS`:

```
# Default owners for everything
*       @maintainer-username

# Specific ownership
/src/   @core-team
/docs/  @docs-team
*.md    @docs-team
```

## CI/CD Status Badge

Add to README.md:

```markdown
[![CI](https://github.com/aurel3d/aletheia/workflows/CI/badge.svg)](https://github.com/aurel3d/aletheia/actions)
```

## Additional Security

### Dependabot

Enable in: `Settings` → `Security` → `Dependabot`

- [x] Dependabot alerts
- [x] Dependabot security updates
- [x] Dependabot version updates

### Code Scanning

Enable in: `Settings` → `Security` → `Code scanning`

- [x] CodeQL analysis
- [x] Rust security audit

## Summary

With these settings:
- ✅ All PRs require review
- ✅ CI must pass before merge
- ✅ Code must be formatted and linted
- ✅ Tests must pass on all platforms
- ✅ WASM compatibility verified
- ✅ Security vulnerabilities detected
- ✅ Documentation builds successfully
- ✅ History is protected

This ensures high code quality and prevents breaking changes from reaching `main`.
