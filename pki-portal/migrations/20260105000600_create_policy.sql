-- Global policy singleton
CREATE TABLE IF NOT EXISTS policy (
    id INT PRIMARY KEY CHECK (id = 1),
    subject_id_pattern TEXT,
    allow_ca_issue BOOLEAN NOT NULL DEFAULT false,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_policy_id ON policy (id);
