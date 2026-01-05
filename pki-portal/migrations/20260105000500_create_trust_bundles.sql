-- Trust bundle metadata and signed payloads
CREATE TABLE IF NOT EXISTS trust_bundles (
    version TEXT PRIMARY KEY,
    issued_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    url TEXT NOT NULL,
    signer_fingerprint TEXT NOT NULL,
    status TEXT NOT NULL CHECK (status IN ('active', 'deprecated')),
    payload JSONB NOT NULL DEFAULT '{}'::jsonb,
    signature TEXT NOT NULL DEFAULT ''
);

CREATE INDEX IF NOT EXISTS idx_trust_bundles_issued_at ON trust_bundles (issued_at DESC);
