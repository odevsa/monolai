CREATE TABLE IF NOT EXISTS audios (
    id TEXT PRIMARY KEY,
    audio_type TEXT NOT NULL, -- 'tts' | 'asr'
    model TEXT NOT NULL,
    prompt TEXT,
    audio_url TEXT,
    voice TEXT,
    speed REAL,
    format TEXT,
    duration_secs REAL,
    transcript TEXT,
    filename TEXT,
    language TEXT,
    word_count INTEGER,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_audios_created_at ON audios(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_audios_type ON audios(audio_type);
