use axum::{
    extract::{Multipart, State},
    http::StatusCode,
    Json,
};
use serde::Serialize;
use std::sync::Arc;
use crate::state::AppState;
use crate::ingest::file::analyze_bytes;

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
    let result = analyze_bytes(&file_data, &filename);

    {
        let mut analyzer = state.analyzer.write().await;
        *analyzer = result;

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
