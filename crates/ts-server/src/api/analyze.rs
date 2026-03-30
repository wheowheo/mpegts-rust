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

    // 최종 결과 전송
    let total_packets = analyzer.total_packets();
    {
        let mut stored = state.analyzer.write().await;
        *stored = analyzer;

        let info = stored.stream_info();
        if let Ok(json) = serde_json::to_string(&serde_json::json!({
            "total_bitrate_bps": info.total_bitrate_bps,
            "pids": stored.pid_map.pids.values().collect::<Vec<_>>(),
            "complete": true,
        })) {
            let _ = ws_tx.send(json);
        }
    }

    Ok(Json(AnalyzeResponse {
        status: "complete".into(),
        total_packets,
        filename,
    }))
}
