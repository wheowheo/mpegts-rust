use axum::{
    extract::{Multipart, State},
    http::StatusCode,
    Json,
};
use serde::Serialize;
use std::sync::Arc;
use ts_core::packet::TS_PACKET_SIZE;
use crate::state::AppState;

#[derive(Serialize)]
pub struct AnalyzeResponse {
    pub status: String,
    pub total_packets: u64,
    pub filename: String,
}

pub async fn start_analysis(
    State(state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> Result<Json<AnalyzeResponse>, StatusCode> {
    let mut filename = String::from("unknown");
    let ws_tx = state.ws_tx.clone();

    // 새 analyzer 준비
    let mut analyzer = ts_analyzer::StreamAnalyzer::new();
    let mut remainder = Vec::with_capacity(TS_PACKET_SIZE);
    let mut synced = false;

    while let Ok(Some(field)) = multipart.next_field().await {
        if field.name() != Some("file") {
            continue;
        }

        filename = field
            .file_name()
            .unwrap_or("unknown")
            .to_string();
        analyzer.set_filename(&filename);

        // 청크 단위로 스트리밍 처리
        let mut stream = field;
        while let Ok(Some(chunk)) = stream.chunk().await {
            remainder.extend_from_slice(&chunk);

            // 첫 sync byte 탐색 (최초 1회)
            if !synced {
                if let Some(pos) = remainder.iter().position(|&b| b == 0x47) {
                    remainder.drain(..pos);
                    synced = true;
                } else {
                    remainder.clear();
                    continue;
                }
            }

            // 188바이트 단위로 패킷 파싱
            let complete_packets = remainder.len() / TS_PACKET_SIZE;
            for i in 0..complete_packets {
                let offset = i * TS_PACKET_SIZE;
                analyzer.feed_packet(&remainder[offset..offset + TS_PACKET_SIZE]);
            }
            let consumed = complete_packets * TS_PACKET_SIZE;
            remainder.drain(..consumed);

            // 주기적으로 WebSocket push
            if analyzer.total_packets() % 50000 == 0 && analyzer.total_packets() > 0 {
                if let Ok(json) = serde_json::to_string(&serde_json::json!({
                    "total_packets": analyzer.total_packets(),
                    "total_bitrate_bps": analyzer.bitrate_stats.latest_total_bitrate(),
                })) {
                    let _ = ws_tx.send(json);
                }
            }
        }
        break;
    }

    if analyzer.total_packets() == 0 {
        return Err(StatusCode::BAD_REQUEST);
    }

    analyzer.sync_pid_bitrates();

    // save session to DB
    let session_id = uuid::Uuid::new_v4().to_string();
    if let Err(e) = state.db.create_session(&session_id, &filename) {
        tracing::warn!("failed to create session: {}", e);
    }

    let total_packets = analyzer.total_packets();
    let tr_summary = analyzer.tr101290.summary();
    let bitrate = analyzer.bitrate_calc.bitrate_bps().unwrap_or(0.0);
    let duration_ms = analyzer.duration_ms();

    // batch save in single transaction
    {
        let conn = state.db.conn();
        let tx = conn.unchecked_transaction().ok();

        for info in analyzer.pid_map.pids.values() {
            conn.execute(
                "INSERT INTO pid_snapshots (session_id, pid, label, stream_type, packets, cc_errors, bitrate_bps) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                rusqlite::params![&session_id, info.pid as i32, &info.label, info.stream_type.map(|s| s as i32), info.packet_count as i64, info.cc_errors as i64, info.bitrate_bps],
            ).ok();
        }

        for err in tr_summary.errors.iter().take(1000) {
            let prio = match err.priority {
                ts_analyzer::tr101290::Priority::P1 => "P1",
                ts_analyzer::tr101290::Priority::P2 => "P2",
                ts_analyzer::tr101290::Priority::P3 => "P3",
            };
            conn.execute(
                "INSERT INTO errors (session_id, timestamp_ms, error_type, priority, pid, detail) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                rusqlite::params![&session_id, err.timestamp_ms, &err.error_type, prio, err.pid.map(|p| p as i32), Some(&err.description)],
            ).ok();
        }

        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "UPDATE sessions SET end_time=?1, total_packets=?2, bitrate_bps=?3, duration_ms=?4, p1_errors=?5, p2_errors=?6, p3_errors=?7 WHERE id=?8",
            rusqlite::params![now, total_packets as i64, bitrate, duration_ms, tr_summary.p1_count as i64, tr_summary.p2_count as i64, tr_summary.p3_count as i64, &session_id],
        ).ok();

        if let Some(tx) = tx { tx.commit().ok(); }
    }

    {
        let mut sid = state.current_session_id.write().await;
        *sid = Some(session_id);
    }

    // build JSON before acquiring lock
    let info = analyzer.stream_info();
    let ws_json = serde_json::to_string(&serde_json::json!({
        "total_bitrate_bps": info.total_bitrate_bps,
        "complete": true,
    })).ok();

    {
        let mut stored = state.analyzer.write().await;
        *stored = analyzer;
    }

    if let Some(json) = ws_json {
        let _ = ws_tx.send(json);
    }

    Ok(Json(AnalyzeResponse {
        status: "complete".into(),
        total_packets,
        filename,
    }))
}
