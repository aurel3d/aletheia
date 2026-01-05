-- Certificate revocation registry
CREATE TABLE IF NOT EXISTS revocations (
    serial TEXT PRIMARY KEY REFERENCES certificates(serial) ON DELETE CASCADE,
    reason TEXT,
    revoked_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_revocations_revoked_at ON revocations (revoked_at DESC);
