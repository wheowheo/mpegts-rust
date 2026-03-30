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
    let mut file_data = Vec::new();
    let mut filename = String::from("unknown");

    while let Ok(Some(field)) = multipart.next_field().await {
        if field.name() == Some("file") {
            filename = field
                .file_name()
                .unwrap_or("unknown")
                .to_string();
            file_data = field
                .bytes()
                .await
                .map_err(|_| StatusCode::BAD_REQUEST)?
                .to_vec();
            break;
        }
    }

    if file_data.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let ws_tx = state.ws_tx.clone();

    // 분석 실행
    {
        let mut analyzer = state.analyzer.write().await;
        *analyzer = ts_analyzer::StreamAnalyzer::new();
        analyzer.set_filename(&filename);

        let mut offset = 0;

        // sync byte 탐색
        while offset < file_data.len() {
            if file_data[offset] == 0x47 {
                break;
            }
            offset += 1;
        }

        while offset + TS_PACKET_SIZE <= file_data.len() {
            analyzer.feed_packet(&file_data[offset..offset + TS_PACKET_SIZE]);
            offset += TS_PACKET_SIZE;

            // 일정 주기마다 WebSocket으로 실시간 데이터 전송
            if analyzer.total_packets() % 10000 == 0 {
                if let Ok(json) = serde_json::to_string(&serde_json::json!({
                    "total_bitrate_bps": analyzer.bitrate_stats.latest_total_bitrate(),
                    "pcr_jitter_ms": analyzer.pcr_jitter.average_jitter_ms(),
                    "total_packets": analyzer.total_packets(),
                })) {
                    let _ = ws_tx.send(json);
                }
            }
        }

        // 최종 데이터 전송
        let info = analyzer.stream_info();
        if let Ok(json) = serde_json::to_string(&serde_json::json!({
            "total_bitrate_bps": info.total_bitrate_bps,
            "pids": analyzer.pid_map.pids.values().collect::<Vec<_>>(),
            "complete": true,
        })) {
            let _ = ws_tx.send(json);
        }

        Ok(Json(AnalyzeResponse {
            status: "complete".into(),
            total_packets: analyzer.total_packets(),
            filename,
        }))
    }
}
