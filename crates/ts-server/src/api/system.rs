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

    let manager = state.output_manager.read().await;
    let sessions = manager.list_all().await;
    let stream_count = sessions.iter().filter(|s| s.running).count() as u32;
    let avg_bitrate = if stream_count > 0 {
        sessions.iter()
            .filter(|s| s.running)
            .filter_map(|s| s.config.as_ref().map(|c| c.bitrate_bps))
            .sum::<u64>() / stream_count as u64
    } else {
        0
    };

    let cap = capacity::estimate_capacity(
        &sys,
        avg_bitrate,
        stream_count,
        1_000_000_000,
    );

    Json(SystemResponse {
        system: sys,
        capacity: cap,
    })
}
