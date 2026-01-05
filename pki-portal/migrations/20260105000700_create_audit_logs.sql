-- Audit events
CREATE TABLE IF NOT EXISTS audit_logs (
    id UUID PRIMARY KEY,
    event_type TEXT NOT NULL,
    actor TEXT NULL,
    scope TEXT NULL,
    payload JSONB NULL,
    occurred_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_audit_logs_occurred_at ON audit_logs (occurred_at DESC);
