CREATE TABLE IF NOT EXISTS images (
    id TEXT PRIMARY KEY,
    prompt TEXT NOT NULL,
    negative_prompt TEXT,
    model TEXT NOT NULL,
    size TEXT NOT NULL,
    steps INTEGER NOT NULL,
    cfg_scale REAL NOT NULL,
    seed INTEGER,
    src TEXT NOT NULL,
    generation_time_secs REAL,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_images_created_at ON images(created_at DESC);
