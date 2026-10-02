CREATE TABLE IF NOT EXISTS models (
    id TEXT PRIMARY KEY,
    runtime TEXT NOT NULL,
    flags TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
