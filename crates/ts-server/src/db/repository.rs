use rusqlite::params;
use serde::Serialize;
use super::Database;

#[derive(Debug, Clone, Serialize)]
pub struct SessionRecord {
    pub id: String,
    pub filename: String,
    pub start_time: String,
    pub end_time: Option<String>,
    pub duration_ms: Option<f64>,
    pub total_packets: u64,
    pub bitrate_bps: f64,
    pub p1_errors: u64,
    pub p2_errors: u64,
    pub p3_errors: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct PidSnapshotRecord {
    pub pid: u16,
    pub label: String,
    pub stream_type: Option<u8>,
    pub packets: u64,
    pub cc_errors: u64,
    pub bitrate_bps: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ErrorRecord {
    pub id: i64,
    pub timestamp_ms: f64,
    pub error_type: String,
    pub priority: String,
    pub pid: Option<u16>,
    pub detail: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BitrateRecord {
    pub timestamp_ms: f64,
    pub total_bps: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct HistoryStats {
    pub total_sessions: u64,
    pub total_errors: u64,
    pub total_packets: u64,
}

impl Database {
    pub fn create_session(&self, id: &str, filename: &str) -> Result<(), rusqlite::Error> {
        let conn = self.conn();
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO sessions (id, filename, start_time) VALUES (?1, ?2, ?3)",
            params![id, filename, now],
        )?;
        Ok(())
    }

    pub fn finish_session(
        &self, id: &str, total_packets: u64, bitrate_bps: f64,
        duration_ms: Option<f64>, p1: u64, p2: u64, p3: u64,
    ) -> Result<(), rusqlite::Error> {
        let conn = self.conn();
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "UPDATE sessions SET end_time=?1, total_packets=?2, bitrate_bps=?3, duration_ms=?4, p1_errors=?5, p2_errors=?6, p3_errors=?7 WHERE id=?8",
            params![now, total_packets as i64, bitrate_bps, duration_ms, p1 as i64, p2 as i64, p3 as i64, id],
        )?;
        Ok(())
    }

    pub fn save_pid_snapshot(
        &self, session_id: &str, pid: u16, label: &str, stream_type: Option<u8>,
        packets: u64, cc_errors: u64, bitrate_bps: f64,
    ) -> Result<(), rusqlite::Error> {
        let conn = self.conn();
        conn.execute(
            "INSERT INTO pid_snapshots (session_id, pid, label, stream_type, packets, cc_errors, bitrate_bps) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![session_id, pid as i32, label, stream_type.map(|s| s as i32), packets as i64, cc_errors as i64, bitrate_bps],
        )?;
        Ok(())
    }

    pub fn save_error(
        &self, session_id: &str, timestamp_ms: f64, error_type: &str,
        priority: &str, pid: Option<u16>, detail: Option<&str>,
    ) -> Result<(), rusqlite::Error> {
        let conn = self.conn();
        conn.execute(
            "INSERT INTO errors (session_id, timestamp_ms, error_type, priority, pid, detail) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![session_id, timestamp_ms, error_type, priority, pid.map(|p| p as i32), detail],
        )?;
        Ok(())
    }

    pub fn save_bitrate_sample(
        &self, session_id: &str, timestamp_ms: f64, total_bps: f64,
    ) -> Result<(), rusqlite::Error> {
        let conn = self.conn();
        conn.execute(
            "INSERT INTO bitrate_history (session_id, timestamp_ms, total_bps) VALUES (?1, ?2, ?3)",
            params![session_id, timestamp_ms, total_bps],
        )?;
        Ok(())
    }

    pub fn list_sessions(&self, limit: usize, offset: usize) -> Result<Vec<SessionRecord>, rusqlite::Error> {
        let conn = self.conn();
        let mut stmt = conn.prepare(
            "SELECT id, filename, start_time, end_time, duration_ms, total_packets, bitrate_bps, p1_errors, p2_errors, p3_errors FROM sessions ORDER BY start_time DESC LIMIT ?1 OFFSET ?2"
        )?;
        let rows = stmt.query_map(params![limit as i64, offset as i64], |row| {
            Ok(SessionRecord {
                id: row.get(0)?,
                filename: row.get(1)?,
                start_time: row.get(2)?,
                end_time: row.get(3)?,
                duration_ms: row.get(4)?,
                total_packets: row.get::<_, i64>(5)? as u64,
                bitrate_bps: row.get(6)?,
                p1_errors: row.get::<_, i64>(7)? as u64,
                p2_errors: row.get::<_, i64>(8)? as u64,
                p3_errors: row.get::<_, i64>(9)? as u64,
            })
        })?;
        rows.collect()
    }

    pub fn get_session(&self, id: &str) -> Result<Option<SessionRecord>, rusqlite::Error> {
        let conn = self.conn();
        let mut stmt = conn.prepare(
            "SELECT id, filename, start_time, end_time, duration_ms, total_packets, bitrate_bps, p1_errors, p2_errors, p3_errors FROM sessions WHERE id=?1"
        )?;
        let mut rows = stmt.query_map(params![id], |row| {
            Ok(SessionRecord {
                id: row.get(0)?,
                filename: row.get(1)?,
                start_time: row.get(2)?,
                end_time: row.get(3)?,
                duration_ms: row.get(4)?,
                total_packets: row.get::<_, i64>(5)? as u64,
                bitrate_bps: row.get(6)?,
                p1_errors: row.get::<_, i64>(7)? as u64,
                p2_errors: row.get::<_, i64>(8)? as u64,
                p3_errors: row.get::<_, i64>(9)? as u64,
            })
        })?;
        Ok(rows.next().transpose()?)
    }

    pub fn get_session_pids(&self, session_id: &str) -> Result<Vec<PidSnapshotRecord>, rusqlite::Error> {
        let conn = self.conn();
        let mut stmt = conn.prepare(
            "SELECT pid, label, stream_type, packets, cc_errors, bitrate_bps FROM pid_snapshots WHERE session_id=?1 ORDER BY pid"
        )?;
        let rows = stmt.query_map(params![session_id], |row| {
            Ok(PidSnapshotRecord {
                pid: row.get::<_, i32>(0)? as u16,
                label: row.get(1)?,
                stream_type: row.get::<_, Option<i32>>(2)?.map(|v| v as u8),
                packets: row.get::<_, i64>(3)? as u64,
                cc_errors: row.get::<_, i64>(4)? as u64,
                bitrate_bps: row.get(5)?,
            })
        })?;
        rows.collect()
    }

    pub fn get_session_errors(&self, session_id: &str, limit: usize) -> Result<Vec<ErrorRecord>, rusqlite::Error> {
        let conn = self.conn();
        let mut stmt = conn.prepare(
            "SELECT id, timestamp_ms, error_type, priority, pid, detail FROM errors WHERE session_id=?1 ORDER BY id DESC LIMIT ?2"
        )?;
        let rows = stmt.query_map(params![session_id, limit as i64], |row| {
            Ok(ErrorRecord {
                id: row.get(0)?,
                timestamp_ms: row.get(1)?,
                error_type: row.get(2)?,
                priority: row.get(3)?,
                pid: row.get::<_, Option<i32>>(4)?.map(|v| v as u16),
                detail: row.get(5)?,
            })
        })?;
        rows.collect()
    }

    pub fn get_session_bitrate(&self, session_id: &str) -> Result<Vec<BitrateRecord>, rusqlite::Error> {
        let conn = self.conn();
        let mut stmt = conn.prepare(
            "SELECT timestamp_ms, total_bps FROM bitrate_history WHERE session_id=?1 ORDER BY timestamp_ms"
        )?;
        let rows = stmt.query_map(params![session_id], |row| {
            Ok(BitrateRecord {
                timestamp_ms: row.get(0)?,
                total_bps: row.get(1)?,
            })
        })?;
        rows.collect()
    }

    pub fn delete_session(&self, id: &str) -> Result<bool, rusqlite::Error> {
        let conn = self.conn();
        let affected = conn.execute("DELETE FROM sessions WHERE id=?1", params![id])?;
        Ok(affected > 0)
    }

    pub fn stats(&self) -> Result<HistoryStats, rusqlite::Error> {
        let conn = self.conn();
        let total_sessions: u64 = conn.query_row(
            "SELECT COUNT(*) FROM sessions", [], |row| row.get::<_, i64>(0)
        )? as u64;
        let total_errors: u64 = conn.query_row(
            "SELECT COUNT(*) FROM errors", [], |row| row.get::<_, i64>(0)
        )? as u64;
        let total_packets: u64 = conn.query_row(
            "SELECT COALESCE(SUM(total_packets), 0) FROM sessions", [], |row| row.get::<_, i64>(0)
        )? as u64;
        Ok(HistoryStats { total_sessions, total_errors, total_packets })
    }

    pub fn cleanup_old(&self, retention_days: i64) -> Result<usize, rusqlite::Error> {
        let conn = self.conn();
        let cutoff = chrono::Utc::now() - chrono::Duration::days(retention_days);
        let affected = conn.execute(
            "DELETE FROM sessions WHERE start_time < ?1",
            params![cutoff.to_rfc3339()],
        )?;
        Ok(affected)
    }
}
