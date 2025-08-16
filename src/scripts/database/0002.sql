  CREATE TABLE system_automations (
    id uuid PRIMARY KEY,
    kind NUMBER,
    target VARCHAR(32),
    schedule CHAR(6),
    param VARCHAR(32) NOT NULL,
    is_active BOOL NOT NULL DEFAULT TRUE,
    last_triggered TIMESTAMPTZ
);

CREATE INDEX idx_active_automations ON system_automations(kind) WHERE is_active;
