# PKI Portal Plan

Purpose: track implementation steps for the web portal that manages trusted root certificates and issuance workflows for Aletheia.

## Current Step
- Trust bundle metadata (migration, handlers, tests) — done. Next: flesh out bundle content/signature pipeline later.

## Working Plan
- Document trust-bundle and revocation updates in OpenAPI/README.
- Add audit/policy scaffolding and auth placeholders.
- Later: implement signed bundle generation/storage and signature verification.

## Milestones
- [x] Define REST resources and endpoints (issuance, revocation, bundles, policy, audit).
- [x] Draft OpenAPI document (JSON/YAML) for the API.
- [x] Design data models and persistence layout (roots, intermediates, certs, revocations, bundles, audit log).
- [x] Specify security model (authN/Z, roles, HSM/KMS usage, rate limits, dual control).
- [x] Plan trust-bundle and revocation list publication formats (signed JSON + fingerprints/ETag).
- [x] Outline operational workflows (rotation, revocation, approvals, backups).
- [x] Map client integration: Aletheia CLI/viewer flags for bundle/revocation URLs and pinning.

## Security Model (authN/Z, HSM, hardening)
- AuthN: OAuth2 client credentials for automation; OIDC + MFA for operators; short-lived access tokens; refresh token disabled for service accounts.
- AuthZ roles: `pki.admin` (roots, policy), `pki.operator` (issue/revoke, bundles), `pki.read` (read-only), `pki.auditor` (audit feed). Enforce scope checks per route; default deny.
- Key protection: roots/intermediates keys reside in HSM/KMS; signing happens via KMS APIs; no key export; enforce key usage and rotation policies.
- Transport and headers: TLS-only, HSTS, secure cookies for OIDC sessions, CSRF protection for browser flows.
- Rate limiting: per-client and global rate limits on mutation endpoints; login throttling.
- Dual control: require two-operator approval for root rotation and destructive policy changes (future workflow queue).
- Audit: every mutate endpoint emits append-only audit events with actor, scope, request hash, and result.

## Publication Formats (bundles, revocations)
- Trust bundle JSON: `{ version, issued_at, roots: [{id,fingerprint,cert_pem}], intermediates: [{id,fingerprint,cert_pem}], signature, signer_fingerprint }`; signed with detached signature; served with ETag for cache validation.
- Revocation list JSON: `{ version, issued_at, entries: [{serial, reason, revoked_at}], signature, signer_fingerprint }`; also served with ETag.
- Signatures: detached (e.g., .sig) using HSM-backed key; include signer_fingerprint for pinning; provide SHA-256 checksum header.
- Content negotiation: `application/json` primary; consider `application/cbor` later.

## Operational Workflows
- Root rotation: stage new root (disabled), publish upcoming version, dual approval, then activate and mark old root retired; update trust bundle version.
- Intermediate issuance: admin/operator creates under parent with path_len constraints; auto-add to bundle upon activation.
- Certificate issuance: requires pinned issuer; validates subject fields; records status active.
- Revocation: operator/admin submits serial+reason; status becomes revoked; revocation list version increments and is republished.
- Backups: DB backups daily with PITR; bundle and revocation artifacts stored in versioned object storage with immutability where possible.

## Client Integration (CLI & Viewer)
- CLI flags: `--trust-bundle-url`, `--revocation-url`, `--bundle-fingerprint`, `--revocation-cache-ttl`, `--offline`, `--cache-dir`.
- Behaviors: honor ETag/If-None-Match; verify detached signatures with pinned fingerprint; distinguish network vs signature vs untrusted root errors via exit codes.
- Viewer: settings for portal base URL, bundle/revocation URLs; cache in IndexedDB with version+fingerprint; offline indicator; manual refresh and clear cache; show signer fingerprint and revocation status in UI.

## API Draft (OpenAPI sketch)
- Base URL: `/api/v1`; auth via OAuth2 client credentials (automation) and OIDC sessions + MFA (operators). Roles: admin, operator, auditor, api-client.

```yaml
openapi: 3.0.3
info:
	title: Aletheia PKI Portal API
	version: 0.1.0
servers:
	- url: https://pki.example.com/api/v1
security:
	- oauth2: [pki.read, pki.write]
paths:
	/roots:
		get:
			summary: List roots
			responses: {"200": {"description": "Roots"}}
		post:
			summary: Create root (HSM-backed)
			security: [{oauth2: [pki.admin]}]
	/roots/{id}:
		get:
			summary: Get root cert and metadata
	/roots/{id}/rotate:
		post:
			summary: Rotate root (staged)
			security: [{oauth2: [pki.admin]}]
	/intermediates:
		get:
			summary: List intermediates
		post:
			summary: Create intermediate under parent
			security: [{oauth2: [pki.admin, pki.operator]}]
	/intermediates/{id}:
		get:
			summary: Get intermediate
	/certificates:
		post:
			summary: Issue end-entity cert
			requestBody:
				content:
					application/json:
						schema: { $ref: '#/components/schemas/CertificateRequest' }
			responses:
				"201":
					description: Issued certificate
					content:
						application/json: { schema: { $ref: '#/components/schemas/Certificate' } }
						application/cbor: { schema: { type: string, format: byte } }
	/certificates/{serial}:
		get:
			summary: Get certificate and status
	/revocations:
		get:
			summary: Signed revocation list (JSON + detached signature)
		post:
			summary: Revoke certificate
			security: [{oauth2: [pki.operator, pki.admin]}]
	/trust-bundles/latest:
		get:
			summary: Latest signed trust bundle (roots [+ intermediates])
	/trust-bundles/{version}:
		get:
			summary: Fetch specific bundle
	/policy:
		get:
			summary: Issuance constraints
		put:
			summary: Update constraints
			security: [{oauth2: [pki.admin]}]
	/audit/logs:
		get:
			summary: Append-only audit feed (paginated)
			security: [{oauth2: [pki.auditor, pki.admin]}]
components:
	securitySchemes:
		oauth2:
			type: oauth2
			flows:
				clientCredentials:
					tokenUrl: https://pki.example.com/oauth/token
					scopes:
						pki.read: Read PKI data
						pki.write: Issue/revoke
						pki.admin: Manage roots/intermediates/policy
						pki.auditor: Read audit logs
	schemas:
		CertificateRequest:
			type: object
			required: [subject_id, subject_name, public_key, is_ca]
			properties:
				subject_id: { type: string }
				subject_name: { type: string }
				public_key: { type: string, format: byte }
				is_ca: { type: boolean }
		Certificate:
			type: object
			properties:
				serial: { type: string }
				issuer: { type: string }
				subject_id: { type: string }
				subject_name: { type: string }
				is_ca: { type: boolean }
				public_key: { type: string, format: byte }
				issued_at: { type: integer, format: int64 }
				status: { type: string, enum: [active, revoked] }
```

## Notes
- Keys for roots/intermediates must stay in HSM/KMS; no export.
- All signed artifacts (bundles, revocation lists) should include detached signatures and signer fingerprints for pinning.

## Validation / Linting
- Run OpenAPI lint: `npm exec @stoplight/spectral lint docs/pki-portal-openapi.yaml`.
- Add 401/403 responses where auth is required.
- Ensure scopes used in security blocks match `components.securitySchemes.oauth2.flows.clientCredentials.scopes`.
- Consider adding request/response examples for trust bundles, revocations, and issuance.

## Client Integration (CLI + Viewer)
- CLI flags to add:
	- `--trust-bundle-url <url>`: fetch signed bundle (ETag-aware); accept local file path too.
	- `--revocation-url <url>`: fetch signed revocation list; allow `--revocation-cache-ttl`.
	- `--bundle-fingerprint <hex>`: pin expected signer fingerprint; fail on mismatch.
	- `--offline` and `--cache-dir <path>`: rely on cached bundle/revocations when offline.
- CLI behaviors:
	- Use ETag/If-None-Match to avoid refetch; store bundle version and signer fingerprint.
	- Validate detached signature of bundle/revocations with pinned fingerprint; reject otherwise.
	- Exit codes: distinguish network failure vs. signature failure vs. untrusted root.
- Viewer (web):
	- Settings panel to configure portal base URL, trust-bundle URL, revocation URL.
	- Cache bundle/revocations in IndexedDB with version + fingerprint; honor ETag.
	- Pinning UI: show signer fingerprint, allow user confirmation on first trust (TOFU option).
	- Offline mode indicator; allow manual refresh; clear cache button.
	- Surface revocation status in verification banner and cert chain view.
- WASM bindings: expose functions to load trust bundle and revocation list from bytes/URLs and to return validation errors separately (network vs signature vs format).
- Format expectations:
	- `trust-bundle.json`: `{ version, issued_at, roots: [{id,fingerprint,cert_pem}], intermediates?: [...], signature, signer_fingerprint }`
	- `revocations.json`: `{ version, issued_at, entries: [{serial, reason, revoked_at}], signature, signer_fingerprint }`
