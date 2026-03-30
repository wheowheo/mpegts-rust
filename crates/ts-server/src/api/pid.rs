use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::Serialize;
use std::sync::Arc;
use ts_analyzer::pid_detail::PidDetailData;
use crate::state::AppState;

#[derive(Serialize)]
pub struct PidResponse {
    pub pid: u16,
    pub label: String,
    pub stream_type: Option<u8>,
    pub packet_count: u64,
    pub cc_errors: u64,
    pub bitrate_bps: f64,
    pub has_pcr: bool,
    pub scrambled: bool,
    pub percentage: f64,
}

pub async fn get_pid_map(
    State(state): State<Arc<AppState>>,
) -> Json<Vec<PidResponse>> {
    let analyzer = state.analyzer.read().await;
    let total = analyzer.pid_map.total_packets.max(1) as f64;

    let mut pids: Vec<PidResponse> = analyzer
        .pid_map
        .pids
        .values()
        .map(|info| PidResponse {
            pid: info.pid,
            label: info.label.clone(),
            stream_type: info.stream_type,
            packet_count: info.packet_count,
            cc_errors: info.cc_errors,
            bitrate_bps: info.bitrate_bps,
            has_pcr: info.has_pcr,
            scrambled: info.scrambled,
            percentage: (info.packet_count as f64 / total) * 100.0,
        })
        .collect();

    pids.sort_by_key(|p| p.pid);
    Json(pids)
}

pub async fn get_pid_detail(
    State(state): State<Arc<AppState>>,
    Path(pid): Path<u16>,
) -> Result<Json<PidResponse>, StatusCode> {
    let analyzer = state.analyzer.read().await;
    let total = analyzer.pid_map.total_packets.max(1) as f64;

    analyzer
        .pid_map
        .pids
        .get(&pid)
        .map(|info| {
            Json(PidResponse {
                pid: info.pid,
                label: info.label.clone(),
                stream_type: info.stream_type,
                packet_count: info.packet_count,
                cc_errors: info.cc_errors,
                bitrate_bps: info.bitrate_bps,
                has_pcr: info.has_pcr,
                scrambled: info.scrambled,
                percentage: (info.packet_count as f64 / total) * 100.0,
            })
        })
        .ok_or(StatusCode::NOT_FOUND)
}

pub async fn get_pid_full_detail(
    State(state): State<Arc<AppState>>,
    Path(pid): Path<u16>,
) -> Result<Json<PidDetailData>, StatusCode> {
    let analyzer = state.analyzer.read().await;
    let total = analyzer.pid_map.total_packets.max(1) as f64;

    let info = analyzer.pid_map.pids.get(&pid).ok_or(StatusCode::NOT_FOUND)?;
    let detail = analyzer.pid_details.get(&pid).ok_or(StatusCode::NOT_FOUND)?;

    let percentage = (info.packet_count as f64 / total) * 100.0;
    Ok(Json(detail.build_detail(info.bitrate_bps, percentage, &info.label, info.stream_type)))
}
