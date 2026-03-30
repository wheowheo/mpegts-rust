use axum::{extract::State, Json};
use std::sync::Arc;
use crate::state::AppState;
use ts_analyzer::stream_info::StreamInfo;

pub async fn get_stream_info(
    State(state): State<Arc<AppState>>,
) -> Json<StreamInfo> {
    let analyzer = state.analyzer.read().await;
    Json(analyzer.stream_info())
}
