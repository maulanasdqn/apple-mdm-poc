CREATE TABLE IF NOT EXISTS devices (
    udid             TEXT PRIMARY KEY NOT NULL,
    push_token       TEXT,
    push_magic       TEXT,
    topic            TEXT,
    unlock_token     TEXT,
    enrollment_state TEXT NOT NULL DEFAULT 'authenticated',
    created_at       TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at       TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS command_queue (
    command_uuid TEXT PRIMARY KEY NOT NULL,
    udid         TEXT NOT NULL,
    request_type TEXT NOT NULL,
    command_json TEXT NOT NULL,
    status       TEXT NOT NULL DEFAULT 'pending',
    created_at   TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at   TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_cq_udid_status ON command_queue (udid, status);
