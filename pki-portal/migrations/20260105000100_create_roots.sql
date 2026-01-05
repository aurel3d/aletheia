-- Root CA certificate registry
CREATE TABLE IF NOT EXISTS roots (
    id UUID PRIMARY KEY,
    name TEXT NOT NULL,
    fingerprint TEXT NOT NULL,
    status TEXT NOT NULL CHECK (status IN ('active', 'revoked')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_roots_status ON roots (status);
