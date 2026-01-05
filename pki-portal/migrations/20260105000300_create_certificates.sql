-- End-entity certificate registry
CREATE TABLE IF NOT EXISTS certificates (
    serial TEXT PRIMARY KEY,
    issuer_id UUID NULL REFERENCES intermediates(id) ON DELETE SET NULL,
    subject_id TEXT NOT NULL,
    subject_name TEXT NOT NULL,
    is_ca BOOLEAN NOT NULL DEFAULT false,
    public_key BYTEA NOT NULL,
    status TEXT NOT NULL CHECK (status IN ('active', 'revoked')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_certificates_created_at ON certificates (created_at DESC);
CREATE INDEX IF NOT EXISTS idx_certificates_issuer ON certificates (issuer_id);
CREATE INDEX IF NOT EXISTS idx_certificates_subject_id ON certificates (subject_id);
