pub const SCHEMA_V1: &str = r#"
CREATE TABLE IF NOT EXISTS sessions (
    id TEXT PRIMARY KEY,
    filename TEXT NOT NULL,
    start_time TEXT NOT NULL,
    end_time TEXT,
    duration_ms REAL,
    total_packets INTEGER NOT NULL DEFAULT 0,
    bitrate_bps REAL NOT NULL DEFAULT 0.0,
    p1_errors INTEGER NOT NULL DEFAULT 0,
    p2_errors INTEGER NOT NULL DEFAULT 0,
    p3_errors INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS pid_snapshots (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id TEXT NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
    pid INTEGER NOT NULL,
    label TEXT NOT NULL,
    stream_type INTEGER,
    packets INTEGER NOT NULL DEFAULT 0,
    cc_errors INTEGER NOT NULL DEFAULT 0,
    bitrate_bps REAL NOT NULL DEFAULT 0.0
);

CREATE TABLE IF NOT EXISTS errors (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id TEXT NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
    timestamp_ms REAL NOT NULL,
    error_type TEXT NOT NULL,
    priority TEXT NOT NULL,
    pid INTEGER,
    detail TEXT
);

CREATE TABLE IF NOT EXISTS bitrate_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id TEXT NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
    timestamp_ms REAL NOT NULL,
    total_bps REAL NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_pid_snapshots_session ON pid_snapshots(session_id);
CREATE INDEX IF NOT EXISTS idx_errors_session ON errors(session_id);
CREATE INDEX IF NOT EXISTS idx_errors_priority ON errors(priority);
CREATE INDEX IF NOT EXISTS idx_bitrate_history_session ON bitrate_history(session_id);
CREATE INDEX IF NOT EXISTS idx_sessions_start_time ON sessions(start_time DESC);
CREATE INDEX IF NOT EXISTS idx_errors_timestamp ON errors(timestamp_ms);
"#;
