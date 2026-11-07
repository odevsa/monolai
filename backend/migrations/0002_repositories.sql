CREATE TABLE IF NOT EXISTS repositories (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    repo_type TEXT NOT NULL,
    base_url TEXT NOT NULL,
    api_token TEXT,
    enabled INTEGER NOT NULL DEFAULT 0,
    is_builtin INTEGER NOT NULL DEFAULT 0,
    config_json TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE INDEX IF NOT EXISTS idx_repositories_created_at ON repositories(created_at DESC);

INSERT OR IGNORE INTO repositories (id, name, repo_type, base_url, enabled, is_builtin, config_json)
VALUES 
    ('huggingface', 'Hugging Face Hub', 'huggingface', 'https://huggingface.co', 0, 1, '{}');
