-- Intermediate CA certificate registry
CREATE TABLE IF NOT EXISTS intermediates (
    id UUID PRIMARY KEY,
    issuer_id UUID NULL REFERENCES roots(id) ON DELETE SET NULL,
    name TEXT NOT NULL,
    fingerprint TEXT NOT NULL,
    status TEXT NOT NULL CHECK (status IN ('active', 'revoked')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_intermediates_status ON intermediates (status);
CREATE INDEX IF NOT EXISTS idx_intermediates_issuer ON intermediates (issuer_id);
