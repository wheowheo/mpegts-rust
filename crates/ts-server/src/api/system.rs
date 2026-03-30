use axum::{extract::State, Json};
use std::sync::Arc;
use ts_analyzer::capacity;
use crate::state::AppState;

#[derive(serde::Serialize)]
pub struct SystemResponse {
    pub system: ts_analyzer::system_stats::SystemSnapshot,
    pub capacity: capacity::CapacityEstimate,
}

pub async fn get_system_stats(
    State(state): State<Arc<AppState>>,
) -> Json<SystemResponse> {
    let mut collector = state.system_stats.write().await;
    let sys = collector.snapshot();

    let output = state.output_session.read().await;
    let output_status = output.status.read().await;

    let (bitrate, stream_count) = if output_status.running {
        (
            output_status.config.as_ref().map(|c| c.bitrate_bps).unwrap_or(0),
            1u32,
        )
    } else {
        (0, 0)
    };

    let cap = capacity::estimate_capacity(
        &sys,
        bitrate,
        stream_count,
        1_000_000_000, // 1 Gbps default
    );

    Json(SystemResponse {
        system: sys,
        capacity: cap,
    })
}
