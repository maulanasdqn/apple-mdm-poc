CREATE TABLE IF NOT EXISTS devices (
    udid           TEXT PRIMARY KEY NOT NULL,
    push_token     TEXT,
    push_magic     TEXT,
    topic          TEXT,
    unlock_token   TEXT,
    enrollment_state TEXT NOT NULL DEFAULT 'authenticated',
    created_at     TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at     TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS command_queue (
    command_uuid   TEXT PRIMARY KEY NOT NULL,
    udid           TEXT NOT NULL REFERENCES devices(udid) ON DELETE CASCADE,
    request_type   TEXT NOT NULL,
    payload_plist  BLOB NOT NULL,
    status         TEXT NOT NULL DEFAULT 'pending',
    created_at     TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at     TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_command_queue_udid_status
    ON command_queue (udid, status);

CREATE TABLE IF NOT EXISTS command_results (
    id             INTEGER PRIMARY KEY AUTOINCREMENT,
    command_uuid   TEXT NOT NULL,
    udid           TEXT NOT NULL,
    status         TEXT NOT NULL,
    response_plist BLOB,
    created_at     TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_command_results_uuid
    ON command_results (command_uuid);
