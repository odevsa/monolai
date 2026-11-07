CREATE TABLE IF NOT EXISTS models (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    filename TEXT NOT NULL,
    relative_path TEXT NOT NULL,
    size_bytes INTEGER NOT NULL DEFAULT 0,
    format TEXT NOT NULL,
    modality TEXT NOT NULL,
    architecture TEXT,
    source_repo TEXT,
    is_auxiliary INTEGER NOT NULL DEFAULT 0,
    auxiliary_type TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_models_format ON models(format);
CREATE INDEX IF NOT EXISTS idx_models_modality ON models(modality);
CREATE INDEX IF NOT EXISTS idx_models_created_at ON models(created_at DESC);

CREATE TABLE IF NOT EXISTS model_presets (
    id TEXT PRIMARY KEY,
    model_id TEXT NOT NULL,
    name TEXT NOT NULL,
    runtime_id TEXT NOT NULL,
    flags TEXT NOT NULL DEFAULT '{}',
    is_default INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (model_id) REFERENCES models(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_model_presets_model_id ON model_presets(model_id);
CREATE INDEX IF NOT EXISTS idx_model_presets_runtime_id ON model_presets(runtime_id);
CREATE INDEX IF NOT EXISTS idx_model_presets_created_at ON model_presets(created_at DESC);

CREATE TABLE IF NOT EXISTS downloads (
    id TEXT PRIMARY KEY,
    model_name TEXT NOT NULL,
    url TEXT NOT NULL,
    file_path TEXT NOT NULL,
    total_bytes INTEGER NOT NULL DEFAULT 0,
    downloaded_bytes INTEGER NOT NULL DEFAULT 0,
    status TEXT NOT NULL DEFAULT 'queued',
    error_message TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_downloads_status ON downloads(status);
CREATE INDEX IF NOT EXISTS idx_downloads_created_at ON downloads(created_at DESC);
