# Aletheia PKI Portal

Actix Web + Postgres skeleton for the PKI portal that issues and publishes trust artifacts for Aletheia.

## Status
- Health endpoint implemented.
- All API surfaces are stubbed per [docs/pki-portal-openapi.yaml](../docs/pki-portal-openapi.yaml) and return 501/placeholder data.
- Persistence, auth, KMS integration, and real handlers are not wired yet.

## Run (dev)
```bash
cd pki-portal
DATABASE_URL=postgres://user:pass@localhost:5432/pki \
DB_MAX_CONNECTIONS=5 \
BIND_ADDR=0.0.0.0:8080 \
cargo run
```

Apply migrations before first run (sqlx CLI or manual):
```bash
psql "$DATABASE_URL" -f migrations/20260105_create_roots.sql
psql "$DATABASE_URL" -f migrations/20260105_create_intermediates.sql
```

## Next steps
- Add migrations and model layer (sqlx) for roots, intermediates, certificates, revocations, audit.
- Add OAuth2/OIDC middleware and role-based scopes.
- Implement handlers for roots/intermediates/cert issuance, trust bundles, and revocations.
- Add structured error types and request validation.
- Integrate tracing/metrics exporters.
